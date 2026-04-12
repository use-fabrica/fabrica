# Coding Agent Harness — Scaffold Phases

> Stack: **wgpu** (GPU rendering) + **glyphon** (text) + **taffy** (layout) + **jsonrpsee** (JSON-RPC over socket)
> Async: **Custom runtime** (Zed-style) for GUI, **tokio** only for JSON-RPC
> Goal: Desktop GUI that communicates with an agent backend via JSON-RPC socket, so a React Native mobile app can connect later.

---

## Phase 1 — Cargo Workspace & Project Structure

**Goal:** Set up a multi-crate monorepo that separates concerns cleanly and isolates async runtimes.

### Suggested layout

```
agent-harness/
├── Cargo.toml              # Workspace root (virtual manifest)
├── Cargo.lock
├── crates/
│   ├── runtime/            # Custom async runtime (async_task + smol + EventLoopProxy)
│   ├── gui/                # wgpu + glyphon + taffy frontend (uses crate::runtime)
│   ├── agent/              # Agent core (LLM interaction, tool execution)
│   ├── rpc/                # JSON-RPC server/client — the ONLY crate that touches tokio
│   └── shared/             # Shared types, protocol definitions
```

### Key dependency flow

```
shared ← (no deps, pure types)
  ↑
runtime ← async_task, smol, futures (NO tokio)
  ↑
gui ← runtime, wgpu, glyphon, taffy, winit (NO tokio)
  ↑
agent ← runtime, shared (NO tokio for core logic)
  ↑
rpc ← jsonrpsee, tokio (tokio ISOLATED here)
```

**Rule: `gui` and `runtime` never depend on tokio.** Tokio lives only in `rpc/` for jsonrpsee. The GUI talks to the RPC layer through channel boundaries (`futures::channel::mpsc` or `std::sync::mpsc`).

### Resources

| Resource | Link | Why |
|----------|------|-----|
| **The Cargo Book — Workspaces** | https://doc.rust-lang.org/stable/cargo/reference/workspaces.html | Official reference for workspace configuration. Read this first. |
| **The Cargo Book — Package Layout** | https://doc.rust-lang.org/stable/cargo/guide/project-layout.html | Conventions for file placement inside each crate. |
| **Managing Complex Rust Workspaces** (elijah samson) | https://towardsdev.com/managing-complex-rust-workspaces-f44ee8c72709 | Excellent practical guide. Advocates flat `crates/*` layout (like rust-analyzer). Key insight: keep root as virtual manifest, don't nest hierarchically. |
| **Cargo Workspace Best Practices** (Reintech) | https://reintech.io/blog/cargo-workspace-best-practices-large-rust-projects | Production patterns: separation by function, build perf, CI setup. |
| **rust-analyzer repo structure** | https://github.com/rust-lang/rust-analyzer/tree/master/crates | Real-world reference: 36 crates under `crates/`, flat layout. Study this. |
| **cargo xtask pattern** | https://github.com/matklad/cargo-xtask | Write project automation in Rust instead of Makefiles. |

### What to learn

- How `[workspace] members = ["crates/*"]` works in root `Cargo.toml`
- Why virtual manifest (no `[package]` in root) is cleaner
- How crates depend on each other via `path = "../shared"` dependencies
- **Dependency isolation** — how to keep tokio out of crates that don't need it
- The `cargo test --workspace`, `cargo clippy --workspace` workflow

---

## Phase 2 — Custom Async Runtime

**Goal:** Build a thin async runtime in `crates/runtime/` inspired by Zed's approach. No tokio. Just `async_task` for the Task type, `smol` for background thread pool, and `winit::EventLoopProxy` as the foreground dispatcher.

This is the **foundation** — every other phase builds on it.

### Architecture

```
┌────────────────────────────────────────────────────────┐
│                  Your Async Runtime                     │
│                                                        │
│  ┌────────────────────┐   ┌─────────────────────────┐ │
│  │ ForegroundExecutor  │   │ BackgroundExecutor       │ │
│  │ (main thread only)  │   │ (thread pool)            │ │
│  │                     │   │                          │ │
│  │ Backed by winit's   │   │ Backed by smol::         │ │
│  │ EventLoopProxy      │   │ ThreadPool               │ │
│  │                     │   │                          │ │
│  │ UI mutations        │   │ I/O, computation         │ │
│  │ Entity updates      │   │ Agent work               │ │
│  │ Rendering triggers  │   │ Network (via RPC bridge) │ │
│  └────────────────────┘   └─────────────────────────┘ │
│                                                        │
│  Both use async_task::Task<T> as the primitive         │
│  Both use async_task::Runnable for scheduling          │
└────────────────────────────────────────────────────────┘
```

### Key components to build

1. **`Task<T>`** — wraps `async_task::Task<T>`. Dropping cancels. `.detach()` lets it run freely.
2. **`ForegroundExecutor`** — holds `EventLoopProxy`. `spawn()` creates `(Runnable, Task)` via `async_task`, sends `Runnable` to main thread via `proxy.send_event()`.
3. **`BackgroundExecutor`** — holds `smol::ThreadPool`. `spawn()` creates `(Runnable, Task)`, pushes `Runnable` into the pool. Waker re-schedules continuation on foreground via `EventLoopProxy`.
4. **`Priority`** — enum (Low, Medium, High). Maps to how urgently things get scheduled.

### The waker trick (how background → foreground works)

```
Background task completes
    → async_task Waker fires
    → Waker calls EventLoopProxy::send_event(Runnable)
    → winit event loop wakes up on main thread
    → You poll the Runnable
    → The async continuation resumes on the main thread
```

This is exactly what Zed does, but instead of GCD's `dispatch_async_f`, you use winit's `EventLoopProxy::send_event()`.

### Resources

| Resource | Link | Why |
|----------|------|-----|
| **`research/ZED_ASYNC_RUNTIME.md`** | (local file in this repo) | Your detailed research on Zed's approach. Architecture, source file map, reading order. Read this first. |
| **Zed Decoded: Async Rust** (blog) | https://zed.dev/blog/zed-decoded-async-rust | The primary resource. Walks through `cx.spawn()`, executors, how Zed avoids tokio. 1hr companion video included. |
| **Zed's `executor.rs` source** | https://github.com/zed-industries/zed/blob/main/crates/gpui/src/executor.rs | ~400 lines. The production implementation. Study `spawn()`, `spawn_with_priority()`, how `async_task::Builder` creates `(Runnable, Task)`. |
| **`async_task` crate docs** | https://docs.rs/async-task/latest/async_task/ | The ONLY async primitive you need. Understand `Runnable`, `Task`, `FallibleTask`, and `Builder`. Read all of it — it's small. |
| **`smol` crate docs** | https://docs.rs/smol/latest/smol/ | You only need `smol::ThreadPool` from this. Lightweight, no runtime baggage. |
| **winit `EventLoopProxy` docs** | https://docs.rs/winit/latest/winit/event_loop/struct.EventLoopProxy.html | Thread-safe handle to wake the event loop. This IS your foreground dispatcher. |
| **Zed's `async_context.rs`** | https://github.com/zed-industries/zed/blob/main/crates/gpui/src/app/async_context.rs | How Zed crosses `.await` boundaries safely. The `AsyncApp` pattern — hold weak ref, re-borrow on `.update()`. |
| **Zed's test dispatcher** | https://github.com/zed-industries/zed/blob/main/crates/gpui/src/platform/test/dispatcher.rs | Deterministic async testing. `run_until_parked()`, seed-controlled ordering. Worth studying for later. |

### What to learn

- How `async_task::Builder::new().spawn(local_fn)` creates `(Runnable, Task<T>)`
- How `Runnable::run()` polls the future once — you call this when the event loop picks it up
- How to make a `Waker` that sends `Runnable` via `EventLoopProxy` (so background completions resume on main thread)
- `smol::ThreadPool::spawn_async()` for background work
- Why this is simpler than tokio: no reactor, no runtime, no feature flags — just scheduling

---

## Phase 3 — wgpu Window & Render Loop

**Goal:** Get a window on screen with a working wgpu render loop. Wire the render loop into your custom runtime's foreground executor.

### Resources

| Resource | Link | Why |
|----------|------|-----|
| **Learn Wgpu (sotrh)** — THE tutorial | https://sotrh.github.io/learn-wgpu/ | The de facto wgpu tutorial. Tutorials 1-3 (window, surface, pipeline). This is where you'll spend the most time. |
| **wgpu repo examples** | https://github.com/gfx-rs/wgpu/tree/master/examples/src | Official examples: `hello_window`, `hello_triangle`, `hello_compute`. Read the source after finishing sotrh's tutorial. |
| **wgpu API docs** | https://docs.rs/wgpu/latest/wgpu/ | Reference for `Instance`, `Adapter`, `Device`, `Queue`, `Surface`, `RenderPipeline`. |
| **winit crate** | https://docs.rs/winit/latest/winit/ | Window creation and event loop. Your foreground executor IS this event loop. |
| **WGSL spec** | https://www.w3.org/TR/WGSL/ | The shader language. You'll write `.wgsl` files for your render pipeline. |

### What to learn

- **Instance → Adapter → Device + Queue** — the initialization chain
- **Surface** — the thing you draw to (connected to your winit window)
- **Render pipeline** — vertex shader → fragment shader pipeline
- **Event loop** — `ApplicationHandler` trait. This is where you also dispatch foreground Runnables from your runtime
- **Resize handling** — reconfiguring the surface on window resize

### How the event loop integrates with your runtime

```
impl ApplicationHandler for App {
    fn user_event(event: UserEvent, elwt: &EventLoopThreadTarget) {
        match event {
            // Your custom runtime sends Runnables here via EventLoopProxy
            UserEvent::Runnable(runnable) => runnable.run(),
        }
    }

    fn window_event(event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                // render frame
            }
            // handle resize, input, etc.
        }
    }
}
```

`EventLoopProxy::send_event(Runnable)` → `user_event()` → `runnable.run()`. That's the entire foreground dispatch mechanism.

### Key concept: The render loop

```
loop {
    handle events (resize, input)
    dispatch foreground runnables (from EventLoopProxy)
    request_redraw()
    on redraw:
        get frame from surface
        create encoder
        begin_render_pass (clear color)
        end_render_pass
        queue.submit(encoder.finish)
        frame.present()
}
```

---

## Phase 4 — Text Rendering with glyphon

**Goal:** Render text on screen using glyphon. Start with a hardcoded string, then evolve to multi-line text.

### Resources

| Resource | Link | Why |
|----------|------|-----|
| **glyphon repo** | https://github.com/grovesNL/glyphon | The crate itself. README shows basic usage. 702 stars, actively maintained. |
| **glyphon API docs** | https://docs.rs/glyphon/latest/glyphon/ | Key types: `TextRenderer`, `TextAtlas`, `TextArea`, `Viewport`, `FontSystem`. |
| **glyphon examples** | https://github.com/grovesNL/glyphon/tree/main/examples | Working examples of text rendering with wgpu. Study these carefully. |
| **cosmic-text (underlying text engine)** | https://github.com/pop-os/cosmic-text | glyphon uses cosmic-text under the hood for text shaping, layout, and rasterization. Understanding this helps when you need advanced text features (editing, selection, IME). |
| **cosmic-text docs** | https://docs.rs/cosmic-text/latest/cosmic_text/ | `Buffer`, `Editor`, `Cursor` — these are what you'll use for an editable text area later. |

### Key types to understand

```
FontSystem    → loads fonts, manages font database
TextAtlas     → GPU texture atlas for rasterized glyphs
TextRenderer  → renders text into your wgpu render pass
Viewport      → screen resolution for text rendering
TextArea      → a chunk of text + its position/bounds
Buffer        → (from cosmic-text) text buffer with layout
```

### What to learn

- How to create a `FontSystem` and `TextAtlas`
- How `TextRenderer::prepare()` takes text areas and bakes them into GPU data
- How `TextRenderer::render()` draws into your existing render pass (no extra render pass needed — this is the "middleware pattern")
- How `Viewport` connects to window size
- The relationship between glyphon and cosmic-text (glyphon = renderer, cosmic-text = text engine)

---

## Phase 5 — UI Layout with taffy

**Goal:** Use taffy to compute layout for your UI elements (panels, splits, scroll regions) and feed those computed positions into your wgpu renderer.

### Resources

| Resource | Link | Why |
|----------|------|-----|
| **taffy repo** | https://github.com/DioxusLabs/taffy | The crate. README has a complete usage example. 3K+ stars. |
| **taffy API docs** | https://docs.rs/taffy/latest/taffy/ | Key type: `TaffyTree`. Read the struct docs — they have inline examples. |
| **taffy examples** | https://github.com/DioxusLabs/taffy/tree/main/examples | `basic.rs`, `flexbox_gap.rs`, `grid_holy_grail.rs`, `measure.rs`. The `measure.rs` example shows text measurement integration — critical for your use case. |
| **Flexbox Froggy** | https://flexboxfroggy.com/ | Interactive game to learn flexbox. Taffy implements CSS flexbox faithfully, so web flexbox knowledge transfers directly. |
| **A Complete Guide to Flexbox** (CSS-Tricks) | https://css-tricks.com/snippets/css/a-guide-to-flexbox/ | Visual reference for all flexbox properties. Maps 1:1 to taffy's `Style` struct. |
| **CSS Grid Garden** | https://cssgridgarden.com/ | If you want to use grid layout instead of (or alongside) flexbox. |

### Key concept: Layout → Render bridge

```
1. Build a TaffyTree of UI nodes (each node = a panel/region)
2. Set Style on each node (flex_direction, size, padding, gap, etc.)
3. Call tree.compute_layout(root, available_size)
4. Read tree.layout(node).size and tree.layout(node).location for each node
5. Feed those rects into your wgpu renderer (for panels) and glyphon (for text bounds)
```

### What to learn

- `TaffyTree::new_leaf()` — create a node with style
- `TaffyTree::new_with_children()` — create a parent with children
- `Style` struct — it mirrors CSS flexbox properties exactly
- `compute_layout()` — runs the layout algorithm
- The `measure` example — shows how to integrate text measurement (so taffy knows how big text is for auto-sizing)

---

## Phase 6 — JSON-RPC Socket Communication

**Goal:** Set up a JSON-RPC server over TCP in `crates/rpc/` — the **only** crate that touches tokio. The GUI communicates with the RPC layer through channel boundaries, not shared futures.

### The async boundary

```
┌──────────────┐         channel          ┌──────────────┐
│   GUI crate  │  ←── mpsc/oneshot ──→   │   RPC crate  │
│  (runtime/)  │                          │   (tokio)    │
│              │                          │              │
│  Foreground  │    GUI sends request     │  jsonrpsee   │
│  Background  │    RPC sends response    │  Server      │
│  Executors   │    back via channel      │  tokio::spawn│
└──────────────┘                          └──────────────┘
       ↑ no tokio                              ↑ tokio lives here
```

The bridge: background executor spawns a `std::thread` that runs a tiny tokio runtime, which hosts the jsonrpsee server. Communication across the boundary uses `futures::channel::mpsc` or `std::sync::mpsc`. No tokio types leak into the GUI.

### Resources

| Resource | Link | Why |
|----------|------|-----|
| **jsonrpsee repo** | https://github.com/paritytech/jsonrpsee | The standard Rust JSON-RPC library. 830+ stars, used by Substrate, zkSync, NEAR. Built on async/await + tokio. |
| **jsonrpsee API docs** | https://docs.rs/jsonrpsee-server/latest/jsonrpsee_server/ | Server-side docs. `Server`, `RpcModule`, `ServerBuilder`. |
| **jsonrpsee examples** | https://github.com/paritytech/jsonrpsee/tree/master/examples/examples | `proc_macro.rs` (recommended), `ws_pubsub.rs`. |
| **jsonrpsee hello-world example** | https://github.com/jac18281828/hello-world-jsonrpsee | Minimal standalone example: server + client. |
| **tokio tutorial** | https://tokio.rs/tokio/tutorial | You need tokio basics — but ONLY for this crate. The rest of your app doesn't use it. |
| **Tokio API docs** | https://docs.rs/tokio/latest/tokio/ | For `TcpListener`, runtime setup. |
| **`futures::channel` docs** | https://docs.rs/futures/latest/futures/channel/index.html | `mpsc` and `oneshot` channels — your bridge between tokio land and runtime land. |
| **Zed's `gpui_tokio` bridge** | https://docs.rs/gpui-tokio-bridge/latest/gpui_tokio_bridge/ | Zed's approach to the same problem: tiny tokio runtime (2 threads) bridged to GPUI's executors. |

### Architecture decision: TCP vs Unix Socket

| Option | Pros | Cons |
|--------|------|------|
| **TCP** (`127.0.0.1:PORT`) | Works from any language (React Native, web, etc.). No socket file cleanup. | Slightly slower than Unix socket for local IPC. Port management. |
| **Unix Socket** (`/tmp/agent.sock`) | Faster local IPC. No port conflicts. | Not accessible from React Native (need a bridge/proxy). |
| **Both** | Unix socket for local GUI, TCP for remote/mobile. | More code to maintain. |

**Recommendation:** Start with TCP (`127.0.0.1`) — it works from everywhere including React Native. Add Unix socket later if you need the perf.

### Key concept: The proc-macro API

```rust
#[rpc(server, client, namespace = "agent")]
pub trait AgentApi {
    #[method(name = "execute")]
    async fn execute(&self, prompt: String) -> Result<String, Error>;

    #[subscription(name = "subscribeOutput" => "output", item = String)]
    fn subscribe_output(&self);
}
```

This generates both server trait and client proxy from one definition. The RPC crate implements the server. The GUI crate uses the client — but calls it through the channel bridge, not by directly awaiting tokio futures.

### What to learn

- tokio basics: `#[tokio::main]`, `runtime::Builder::new_multi_thread()` — scoped to `rpc/` only
- jsonrpsee `Server::builder().build(addr).await?`
- `RpcModule` — registering methods
- Proc-macro RPC definition (defines your entire API surface)
- Subscriptions for streaming agent output to the GUI
- **Channel bridging** — how to send requests from your custom runtime to tokio and get responses back

---

## Reference Projects to Study

Real-world Rust projects that use parts of your stack. Read their source, not their docs.

| Project | Stars | What to study | Link |
|---------|-------|---------------|------|
| **Zed** | 79K+ | Custom async runtime (`executor.rs`), wgpu rendering, taffy layout. Study `crates/gpui/src/` | https://github.com/zed-industries/zed |
| **Alacritty** | 58K+ | GPU-accelerated terminal rendering, wgpu-based text rendering at scale | https://github.com/alacritty/alacritty |
| **Lapce** | 35K+ | Code editor with custom GPU UI, uses similar rendering approach | https://github.com/lapce/lapce |
| **cosmic-text** | 1K+ | Text shaping/layout engine used by glyphon — study for editor features | https://github.com/pop-os/cosmic-text |
| **Dioxus** | 25K+ | Uses taffy for layout. Good reference for taffy integration patterns | https://github.com/DioxusLabs/dioxus |
| **Bevy** | 38K+ | Game engine using taffy for UI layout. See `bevy_ui/` crate | https://github.com/bevyengine/bevy |

---

## Learning Order

```
Phase 1 (workspace)        ← 1-2 hours
    ↓
Phase 2 (custom runtime)   ← 3-5 days (new territory, study Zed's executor.rs deeply)
    ↓
Phase 3 (wgpu window)      ← 2-4 days (biggest rendering learning curve)
    ↓
Phase 4 (glyphon text)     ← 1-2 days
    ↓
Phase 5 (taffy layout)     ← 1-2 days
    ↓
Phase 6 (JSON-RPC)         ← 2-3 days (tokio scoped to one crate)
```

**Parallelism:**
- Phases 2 and 6 can be done in parallel — runtime and RPC don't depend on each other
- Phases 4 and 5 can be partially parallel — text and layout are independent concepts
- Phases 3, 4, 5 all depend on Phase 2 (your runtime provides the event loop)

**Critical path:** Phase 1 → Phase 2 → Phase 3 → Phase 4+5 → integrate with Phase 6
