# Fabrica

*Latin: workshop, forge.*

An opinionated coding agent with a desktop GUI.

Built in Rust with [GPUI](https://github.com/zed-industries/zed) — the same framework that powers [Zed](https://zed.dev).

## What

Fabrica is a coding agent with a dock-based GUI — editor, chat, terminal, file tree, review, and outline panels arranged in a configurable layout.

Navigate with vim-like keybindings (`Alt+H/J/K/L`), switch projects and sessions like tmux, and spawn a terminal to run neovim or helix when you need hands-on editing.

## Build & Run

### With Nix (recommended)

```sh
nix develop
cargo run
```

### With Cargo

Requires Rust 1.94+ and system dependencies (X11, Wayland, Vulkan, fontconfig, OpenSSL).

```sh
cargo run
```

## Tech Stack

- [GPUI](https://github.com/zed-industries/zed) — GPU-accelerated UI framework
- [gpui-component](https://github.com/longbridge/gpui-component) — Component library (DockArea, buttons, inputs, etc.)
- Rust Edition 2024

## License

[MIT](LICENSE)
