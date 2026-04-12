# Zed's Custom Async Runtime — Deep Research

> Zed doesn't use tokio as its primary runtime. It uses **the OS itself** as the async executor, with thin wrappers around native platform APIs. This is what makes their approach so interesting.

---

## The Big Idea

Most Rust async apps use tokio or smol as their runtime. Zed doesn't. Instead:

| Layer | What Zed does | What most Rust apps do |
|-------|--------------|----------------------|
| **Task type** | Uses `async_task::Runnable` directly | Use tokio's `JoinHandle` or smol's `Task` |
| **Main thread scheduling** | macOS GCD (`dispatch_async_f`) or Linux event loop | tokio's runtime or smol's reactor |
| **Background thread pool** | `smol::ThreadPool` (lightweight) | tokio's multi-threaded runtime |
| **Event loop** | Platform-native (GCD on macOS, epoll on Linux) | tokio or async-std event loop |
| **Tokio usage** | Minimal — only `gpui_tokio` with 2 worker threads for WASM extensions | Everything runs on tokio |

The core insight: **the OS already has a perfectly good async scheduler. Why add another one?**

---

## Architecture Overview

### Two Executors

GPUI has exactly two executors:

```
┌─────────────────────────────────────────────────┐
│                    App                          │
│                                                 │
│  ┌──────────────────┐  ┌─────────────────────┐ │
│  │ ForegroundExecutor│  │ BackgroundExecutor   │ │
│  │ (main thread only)│  │ (thread pool)        │ │
│  │                   │  │                      │ │
│  │ UI mutations      │  │ I/O, LSP, git, etc.  │ │
│  │ Entity updates    │  │ Heavy computation    │ │
│  │ Event handling    │  │ Network requests     │ │
│  │ Rendering         │  │ File system          │ │
│  └──────────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────┘
```

- **ForegroundExecutor** — runs tasks on the main thread. Required for any UI or entity mutation. This is the "holy" thread.
- **BackgroundExecutor** — multi-threaded pool for CPU-intensive or I/O work. Uses `smol::ThreadPool` under the hood.

### The Platform Dispatcher (the secret sauce)

Both executors delegate to a `PlatformDispatcher` — a trait with different implementations per platform:

| Platform | Implementation | How it schedules |
|----------|---------------|-----------------|
| **macOS** | `MacDispatcher` | Grand Central Dispatch (`dispatch_async_f`, `dispatch_async_f_on_main_queue`) |
| **Linux** | `LinuxDispatcher` | Wayland/X11 event loop integration |
| **Windows** | `WindowsDispatcher` | Windows message loop |
| **Tests** | `TestDispatcher` | Deterministic, seed-controlled execution order |

This is how Zed achieves "native feel" — work is scheduled through the OS's own priority-aware scheduler, not a userspace runtime.

---

## Key Source Files

Read these in order. Each is a piece of the puzzle.

### 1. Executor definitions
**`crates/gpui/src/executor.rs`**
- `ForegroundExecutor` and `BackgroundExecutor` structs
- `Task<T>` — the future handle (wraps `async_task::Task`)
- `Priority` enum (Low, Medium, Realtime)
- `.spawn()`, `.spawn_with_priority()`, `.scoped()` methods
- How `async_task::Builder` creates `(Runnable, Task)` pairs

**What to look for:** The `spawn` method creates a `Runnable` via `async_task::Builder::new()`, then dispatches it via the platform dispatcher. That's it. No runtime, no reactor — just "here's a closure, run it on a thread."

### 2. Platform dispatcher trait
**`crates/gpui/src/platform.rs`**
- `PlatformDispatcher` trait — the abstract interface
- `dispatch(runnable, priority)` — schedule on background
- `dispatch_on_main_thread(runnable, priority)` — schedule on main thread
- `dispatch_after(duration, runnable)` — delayed scheduling
- `RunnableVariant` — wraps `async_task::Runnable` with metadata

### 3. macOS implementation (GCD)
**`crates/gpui/src/platform/mac/dispatcher.rs`**
- The actual `dispatch_async_f` calls into macOS GCD
- How `Runnable` is turned into a raw pointer, sent to GCD, and invoked
- Priority mapping: GPUI `Priority` → GCD QoS classes

**Why this matters:** This is the ~50 lines of code that replace an entire async runtime. GCD handles thread pooling, priority inversion prevention, and cooperative scheduling. For free.

### 4. Test dispatcher (deterministic testing)
**`crates/gpui/src/platform/test/dispatcher.rs`**
- `TestDispatcher` — deterministic, seed-based task ordering
- `run_until_parked()` — runs all pending tasks until quiescence
- `tick()` — single step execution
- Uses the `scheduler` crate for controlled randomness

**Why this matters:** This is what makes GPUI testable. You can write async tests that are 100% deterministic — no flaky tests from scheduling order.

### 5. Async contexts (crossing .await boundaries)
**`crates/gpui/src/app/async_context.rs`**
- `AsyncApp` — holds `Weak<AppCell>`, safe to hold across `.await`
- `AsyncWindowContext` — same, but also knows which window you're in
- `cx.update(|cx| { ... })` — the gateway to mutate App state from async code
- `cx.spawn()` — spawn a foreground task from async context

**Why this matters:** Rust's borrow checker doesn't let you hold `&mut App` across `.await` points. `AsyncApp` solves this by holding a weak reference and re-borrowing on each `.update()` call.

---

## How It All Connects

Here's the flow when you write typical GPUI async code:

```rust
// User code:
cx.spawn(|this, mut cx| async move {
    // 1. This closure returns a Future
    // 2. cx.spawn() wraps it in async_task::Builder
    // 3. Creates (Runnable, Task) pair
    // 4. Dispatches Runnable via PlatformDispatcher

    let result = cx.background_executor()
        .spawn(async {
            // 5. This goes to BackgroundExecutor
            // 6. Which uses smol::ThreadPool
            // 7. Runs on a background thread
            heavy_computation()
        })
        .await;
        // 8. When done, the waker fires
        // 9. GPUI re-schedules the continuation on the main thread
        // 10. via dispatch_on_main_thread()

    this.update(cx, |this, cx| {
        // 11. Now we're back on the main thread
        // 12. Can safely mutate entity state
        this.set_result(result);
        cx.notify();
    });
});
```

### The waker trick

When a background task completes, how does the main thread know to wake up?

1. `async_task` creates a `Runnable` with a custom `Waker`
2. When the future inside `spawn()` completes, the waker is triggered
3. The waker calls `dispatch_on_main_thread()` with the continuation
4. GCD (or Linux equivalent) wakes the main thread
5. Main thread runs the continuation

**No tokio reactor. No epoll loop. Just the OS scheduler.**

---

## The Effect System (Deferred Execution)

GPUI doesn't execute side effects immediately. From `app.rs`:

```
App::update() {
    pending_updates += 1
    run your closure
    if this is the outermost update:
        flush_effects()   // ← this is where things actually happen
    pending_updates -= 1
}

flush_effects() {
    loop {
        release dropped entities
        dispatch queued notifications
        dispatch queued events to subscribers
        if nothing new was queued:
            schedule window redraw
            break
    }
}
```

**Why:** This gives Zed **run-to-completion semantics**. When you call `entity.notify()`, no observers fire immediately. All effects are queued and flushed after the outermost update. This prevents reentrancy bugs and makes the system predictable.

This is similar to how React batches state updates — but at the framework level, not just for rendering.

---

## Dependencies (What Zed Actually Uses)

From `crates/gpui/Cargo.toml`:

| Crate | Purpose |
|-------|---------|
| `async-task` | The `Task<T>` type. Creates `(Runnable, Task)` pairs. This is the only async primitive. |
| `smol` | Used only for `ThreadPool` (background executor). Not used as a runtime. |
| `futures` | Utility combinators (`FutureExt`, channels like `oneshot`, `mpsc`) |
| `parking_lot` | Synchronization primitives (faster than std) |
| No `tokio` | GPUI itself doesn't depend on tokio at all |

The `gpui_tokio` crate exists as a bridge — it runs a tiny tokio runtime (2 worker threads) only for WASM extensions that need it. It's not part of the core async system.

---

## What Makes This Approach Good

### 1. **Zero overhead scheduling**
No tokio reactor, no task stealing, no work-stealing queue. Just `dispatch_async_f` — the OS does the scheduling.

### 2. **Native priority support**
GCD has quality-of-service classes (user-interactive, user-initiated, utility, background). GPUI maps its `Priority` enum directly to these. The OS scheduler understands priorities natively.

### 3. **Deterministic testing**
The `TestDispatcher` replaces the OS scheduler with a seed-controlled deterministic one. Every async test is reproducible. No flaky tests.

### 4. **Main thread is sacred**
The architecture enforces that UI mutations happen on the main thread. `ForegroundExecutor` and `BackgroundExecutor` are separate types. You can't accidentally mutate UI state from a background thread.

### 5. **Minimal dependencies**
One `async_task` crate for the Task type. One `smol` for the thread pool. That's it. No runtime to update, no version conflicts, no feature flag hell.

---

## Trade-offs

### What you lose by not using tokio

- **Ecosystem compatibility** — most Rust async libraries assume tokio. You need bridges (`gpui_tokio`) for anything that requires tokio.
- **Async I/O** — tokio provides `AsyncRead`, `AsyncWrite`, `TcpStream`, etc. Zed handles I/O differently (often through the platform or dedicated threads).
- **Community patterns** — every tokio tutorial, every StackOverflow answer assumes tokio. You're off the beaten path.

### Why Zed accepts these trade-offs

- **Performance** — GCD is faster than any userspace scheduler for UI workloads because it has kernel awareness.
- **Responsiveness** — the OS scheduler knows about display vsync, input priorities, and thermal management. tokio doesn't.
- **Control** — Zed's team controls the entire async stack. No surprises from runtime updates.

---

## Resources

| Resource | Link | What it covers |
|----------|------|---------------|
| **Zed Decoded: Async Rust** (blog post) | https://zed.dev/blog/zed-decoded-async-rust | The primary resource. Walks through `cx.spawn()`, executors, GCD integration. Has a 1-hour companion video. |
| **Ownership and Data Flow in GPUI** (blog post) | https://zed.dev/blog/gpui-ownership | How `App` owns all state, entity handles, the effect queue (`flush_effects`), run-to-completion semantics. |
| **Zed Architecture doc** | https://mintlify.com/zed-industries/zed/contributing/architecture | Official architecture overview. Lists core crates, entity model, rendering pipeline. |
| **GPUI Framework (DeepWiki)** | https://deepwiki.com/zed-industries/zed/2-core-architecture | AI-generated but thorough. Good for browsing the architecture tree. |
| **App Context and Entity System (DeepWiki)** | https://deepwiki.com/zed-industries/zed/2.1-app-context-and-entity-system | Deep dive into `App`, entities, `AsyncApp`, executors, reactive subscriptions. |
| **`executor.rs` source** | https://github.com/zed-industries/zed/blob/main/crates/gpui/src/executor.rs | The actual executor implementation. ~400 lines. Read this. |
| **`platform.rs` source** | https://github.com/zed-industries/zed/blob/main/crates/gpui/src/platform.rs | The `PlatformDispatcher` trait and `RunnableVariant`. |
| **`async_context.rs` source** | https://github.com/zed-industries/zed/blob/main/crates/gpui/src/app/async_context.rs | `AsyncApp` and `AsyncWindowContext` — how to cross `.await` safely. |
| **`gpui-tokio-bridge` crate** | https://docs.rs/gpui-tokio-bridge/latest/gpui_tokio_bridge/ | The bridge for when you MUST use tokio. Only 2 worker threads. Shows the boundary clearly. |
| **`async_task` crate** | https://docs.rs/async-task/latest/async_task/ | The single dependency that makes it all work. Understand `Runnable` and `Task`. |

---

## Reading Order

```
1. Blog: "Zed Decoded: Async Rust"          ← Start here (30 min read + 1hr video)
   ↓
2. Blog: "Ownership and Data Flow in GPUI"   ← How state + effects work (20 min)
   ↓
3. Source: executor.rs                       ← The implementation (~400 lines)
   ↓
4. Source: platform.rs                       ← The PlatformDispatcher trait
   ↓
5. Source: async_context.rs                  ← Crossing .await boundaries
   ↓
6. Source: test/dispatcher.rs                ← Deterministic testing
   ↓
7. async_task crate docs                    ← Understand the primitive
```

---

## Applicability to Your Project

For your coding agent harness, you don't need to replicate Zed's approach exactly. But the key takeaways are:

1. **You don't need tokio for everything.** Your GUI event loop can be the "runtime" — schedule async work, wake the main thread when results are ready.
2. **Separate foreground/background executors.** UI mutations on main thread, everything else on a thread pool. This prevents jank.
3. **Deferred effects.** Batch notifications and events. Don't fire them immediately during mutations. Flush after the outermost update.
4. **`async_task` as the primitive.** It's just `(Runnable, Task)` — no runtime, no reactor. You provide the scheduling.

Since your stack uses wgpu + winit (not GPUI), your "foreground executor" will be the winit event loop. You can use `async_task` + a thread pool (`smol` or `rayon`) for background work, and schedule continuations back onto the winit loop via `EventLoopProxy`.
