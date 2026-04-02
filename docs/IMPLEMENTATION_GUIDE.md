# Fabrica Implementation Guide

The step-by-step order to implement the pane system. Each phase builds on the previous one. Don't skip ahead — later phases depend on earlier ones being working and tested.

---

## Phase 1: Hello DockArea

> **Goal**: A window that renders a single placeholder panel inside a DockArea.

### Step 1.1: Create `panels/editor_panel.rs` — one placeholder panel

```
What: A minimal struct that implements Panel + Render + Focusable
Why:  You need at least one panel to put in the DockArea
```

The panel struct holds:
- `focus_handle: FocusHandle`

Implement:
- `Focusable` — returns `self.focus_handle.clone()`
- `Panel` — `panel_name()` returns `"EditorPanel"`
- `Render` — `div().size_full().child("Editor")` (literally just a label)

Verify: It compiles. Nothing renders yet — that's fine.

### Step 1.2: Create `app.rs` — the FabricaApp struct

```
What: The main app struct that owns a DockArea entity
Why:  Replaces the current demo Fabrica struct in main.rs
```

`FabricaApp` owns:
- `dock_area: Entity<DockArea>`
- `focus_handle: FocusHandle`

In `FabricaApp::new()`:
1. Create a `DockArea::new("main-dock", None, window, cx)`
2. Get `cx.entity().downgrade()` for the weak reference
3. Create the editor panel entity
4. `dock_area.set_center(DockItem::tab(editor_panel, &weak, window, cx), window, cx)`
5. `dock_area.set_locked(true, window, cx)` — no user splits

In `FabricaApp::render()`:
- `div().size_full().child(self.dock_area.clone())`

Verify: Swap `Fabrica` for `FabricaApp` in `main.rs`. Window shows "Editor" in a dock area.

### Step 1.3: Update `main.rs`

```
What: Wire FabricaApp as the root view instead of the demo Fabrica
Why:  The current main.rs renders a design system demo
```

Changes to `main.rs`:
- `use app::FabricaApp` instead of the inline `Fabrica` struct
- Remove the inline `Fabrica` struct and `impl Render for Fabrica`
- In the window closure: `cx.new(|cx| FabricaApp::new(window, cx))`

Verify: `cargo run` opens a window showing "Editor" in a locked dock area.

---

## Phase 2: All Placeholder Panels

> **Goal**: All 6 panels exist as placeholders, placed in the correct docks.

### Step 2.1: Create remaining panel files

```
What: 5 more files identical in structure to editor_panel.rs
Why:  You need them to populate the layout
```

Create these files in `panels/`:
- `file_tree_panel.rs` — renders `div().child("File Tree")`
- `chat_panel.rs` — renders `div().child("Chat")`
- `terminal_panel.rs` — renders `div().child("Terminal")`
- `review_panel.rs` — renders `div().child("Review")`
- `outline_panel.rs` — renders `div().child("Outline")`

Each one: same structure as editor_panel (FocusHandle + Panel + Render).

Update `panels/mod.rs` to re-export all of them.

Verify: All compile. No visual change yet.

### Step 2.2: Place all panels in DockArea

```
What: Update FabricaApp::new() to create all panels and place them
Why:  You want the full IDE layout visible
```

In `FabricaApp::new()`:
```
center:  DockItem::tabs([editor, chat])
left:    DockItem::tab(file_tree)           — size 280px
right:   DockItem::tabs([review, outline])  — size 300px
bottom:  DockItem::tab(terminal)            — size 200px
```

Verify: `cargo run` shows left dock (File Tree), center (Editor + Chat tabs), right dock (Review + Outline tabs), bottom dock (Terminal). All locked — no splits possible.

---

## Phase 3: Config-Driven Placement

> **Goal**: Panel positions come from `fabrica.json`, not hardcoded.

### Step 3.1: Create `config.rs`

```
What: FabricaConfig struct + load() function + default values
Why:  Users need to customize panel positions without touching code
```

Types:
- `DockPosition` enum: Left, Center, Right, Bottom
- `PanelConfig` struct: dock, size, visible
- `FabricaConfig` struct: layout.panels: HashMap<String, PanelConfig>
- `FabricaConfig::load()` — reads `~/.config/fabrica/fabrica.json`, falls back to `FabricaConfig::default()`
- `FabricaConfig::default()` — the hardcoded defaults (review on right, etc.)
- `panels_by_dock()` — groups panel names by dock position

Verify: `FabricaConfig::load()` returns defaults when no file exists. Unit test or print it.

### Step 3.2: Update `FabricaApp::new()` to read config

```
What: Replace hardcoded dock placement with config-driven placement
Why:  The whole point of Phase 3
```

- Load config in `FabricaApp::new()`
- Group panels by dock position using `config.panels_by_dock()`
- For each dock zone, create `DockItem::tabs(...)` from the configured panels
- Pass configured sizes to `set_left_dock` / `set_right_dock` / `set_bottom_dock`

Verify: Default config renders same layout as Phase 2. Create a test `fabrica.json` that moves review to left dock → restart → review is on the left.

---

## Phase 4: Navigation Keybindings

> **Goal**: Alt+H/J/K/L moves focus between dock zones. Alt+B toggles bottom dock.

### Step 4.1: Create `keybindings.rs`

```
What: Action definitions + key registrations
Why:  GPUI requires explicit action types and key bindings
```

Define actions:
```
actions!(fabrica, [
    FocusLeft,    // alt-h
    FocusRight,   // alt-l
    FocusUp,      // alt-k
    FocusDown,    // alt-j
    ToggleBottom, // alt-b
    ToggleLeft,   // alt-[ or whatever
    ToggleRight,  // alt-]
]);
```

`register(cx: &mut App)`:
- `cx.bind_keys([...])` for each action

Call `keybindings::register(cx)` in `main.rs` before window creation.

Verify: Compiles. No behavior yet — need handlers.

### Step 4.2: Wire navigation handlers in `FabricaApp::render()`

```
What: on_action handlers that focus dock zones
Why:  Actions are dispatched but nobody handles them yet
```

In `FabricaApp::render()`:
```
div()
    .key_context("FabricaApp")
    .track_focus(&self.focus_handle)
    .on_action(cx.listener(|this, _: &FocusLeft, window, cx| {
        this.focus_zone(Zone::Left, window, cx);
    }))
    // ... FocusRight → Zone::Right, FocusDown → Zone::Bottom, FocusUp → Zone::Center
    .on_action(cx.listener(|this, _: &ToggleBottom, window, cx| {
        this.dock_area.update(cx, |dock, cx| {
            dock.toggle_dock(DockPlacement::Bottom, window, cx);
        });
    }))
```

`focus_zone()` implementation — accesses `dock_area.left_dock()`, `.right_dock()`, `.bottom_dock()` entities and focuses their focus handles.

Verify: Alt+B toggles bottom dock. Alt+H/J/K/L moves focus highlight between zones.

---

## Phase 5: Dashboard

> **Goal**: When no project is open, show a welcome screen instead of the DockArea.

### Step 5.1: Create `dashboard.rs`

```
What: A standalone view (NOT a Panel) with "New Project" and "Open Project" buttons
Why:  First-run experience — users shouldn't see an empty IDE
```

`Dashboard` struct:
- `focus_handle: FocusHandle`
- `recent_projects: Vec<PathBuf>` (empty for now)

`Render`:
- Centered layout
- "New Project" button
- "Open Project" button
- Recent projects list (placeholder)

### Step 5.2: Add conditional render to `FabricaApp`

```
What: FabricaApp renders either Dashboard or DockArea based on state
Why:  Two distinct modes — welcome vs workspace
```

`FabricaApp` now owns:
- `dashboard: Entity<Dashboard>`
- `dock_area: Entity<DockArea>`
- `project: Option<ProjectState>` — None = dashboard mode

`render()`:
```
match &self.project {
    None    => div().size_full().child(self.dashboard.clone()),
    Some(_) => div().size_full().child(self.dock_area.clone()),
}
```

### Step 5.3: Implement project opening

```
What: "Open Project" button triggers state transition
Why:  Users need to get from dashboard to workspace
```

`Dashboard` button click → emits event or calls `FabricaApp::open_project(path)`:
- Sets `self.project = Some(ProjectState::new(path))`
- Creates DockArea with panels (same as Phase 2/3)
- `cx.notify()` — re-renders, now showing DockArea

Verify: App opens on dashboard. Click "Open Project" → switches to dock area layout. `fabrica ./some-project` via CLI skips dashboard.

---

## Phase 6: Project & Session Model

> **Goal**: Support multiple projects, each with multiple sessions and worktrees.

### Step 6.1: Create `project/` module with data types

```
What: Project, Session, Worktree structs (pure data, no UI)
Why:  Foundation for multi-project and session switching
```

Files:
- `project/project.rs` — `Project { path, worktrees, sessions, active_session_ix }`
- `project/session.rs` — `Session { id, worktree_id, layout_state: Option<DockAreaState> }`
- `project/worktree.rs` — `Worktree { path, branch }`

These are data containers. No rendering logic.

### Step 6.2: Session switching with layout persistence

```
What: Switch between sessions within a project, each restoring its layout
Why:  Like tmux sessions — different contexts, different layouts
```

On session switch:
1. Current session: `session.layout_state = dock_area.dump(cx)` — save layout
2. New session: `dock_area.load(new_session.layout_state, window, cx)` — restore layout
   - Or if no saved state: create default layout from config

Add keybinding: `Alt+1` through `Alt+9` to switch sessions.

Verify: Open 2 sessions, resize panels differently in each, switch between them — layout is preserved.

### Step 6.3: Project switching

```
What: Switch between projects (each project has its own session list)
Why:  Multi-project support
Why:  Multi-project support
```

`FabricaApp` now owns:
- `projects: Vec<Project>`
- `active_project_ix: usize`

On project switch: dump current session, load target project's active session.

Add keybinding: `Alt+P` for project switcher (or go back to dashboard).

Verify: Two projects open, switch between them — each restores its last session's layout.

---

## Phase 7: Worktree Support

> **Goal**: Within a project, switch between git worktrees.

### Step 7.1: Worktree listing and switching

```
What: List worktrees, switch active worktree for a session
Why:  Git worktrees let you have multiple branches checked out simultaneously
```

- Detect worktrees from `.git/worktrees/` directory
- Add `Alt+W` keybinding for worktree switcher
- Switching worktree updates the file tree and active branch context

This is git-specific and can be deferred. Phases 1-6 give you a fully functional pane system.

---

## Phase 8: Real Panel Content

> **Goal**: Replace placeholder labels with actual functionality.

This is where the real work begins — each panel gets its own implementation:

- `editor_panel.rs` → code editor (integration with a text editing crate)
- `terminal_panel.rs` → PTY integration (portable-pty or similar)
- `chat_panel.rs` → AI chat interface
- `file_tree_panel.rs` → file system browser
- `review_panel.rs` → code review / diff viewer
- `outline_panel.rs` → symbol tree

Each is a self-contained effort. The panel structure from Phase 2 stays the same — you're just filling in the `Render` implementation.

---

## Dependency Graph

```
Phase 1  ─→ Phase 2 ─→ Phase 3 ─→ Phase 4
   │                                      │
   └──→ Phase 5 ─→ Phase 6 ─→ Phase 7     │
              ↑                             │
              └─────────────────────────────┘
                    (Phase 4 keybindings used in all later phases)
```

- Phase 5 (dashboard) depends on Phase 1 only — can be done in parallel with 2-4
- Phase 6 (sessions) depends on Phase 3 (config) and Phase 5 (project open)
- Phase 7 (worktrees) depends on Phase 6 (sessions)
- Phase 8 (real panels) can start anytime after Phase 2

---

## File Structure Reference

```
crates/fabrica/src/
├── main.rs                    # thin entry point
├── app.rs                     # FabricaApp — orchestrator
├── dashboard.rs               # welcome screen (Phase 5)
├── config.rs                  # fabrica.json (Phase 3)
├── keybindings.rs             # actions + key bindings (Phase 4)
├── workspace.rs               # DockArea builder from session state (Phase 6)
│
├── project/                   # domain model (Phase 6)
│   ├── mod.rs
│   ├── project.rs
│   ├── session.rs
│   └── worktree.rs
│
└── panels/                    # Panel trait implementations (Phase 1-2)
    ├── mod.rs
    ├── editor_panel.rs
    ├── chat_panel.rs
    ├── file_tree_panel.rs
    ├── terminal_panel.rs
    ├── review_panel.rs
    └── outline_panel.rs
```
