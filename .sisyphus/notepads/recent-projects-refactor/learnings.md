# Learnings

## 2026-04-04 Session Start
- Codebase: GPUI-based Rust app (Fabrica), edition 2024, Rust 1.94+
- `recent_projects.rs` is 482 lines with mixed concerns
- Dashboard accesses `pub projects: Vec<RecentProject>` directly at 8+ sites — cannot make private
- All mutations (add/remove/prune) auto-save — behavioral contract
- `format_relative_time` is `pub` but effectively `pub(crate)` (module is `pub(crate)`)
- `time` crate NOT currently a dependency
- Workspace dep pattern: add to root Cargo.toml `[workspace.dependencies]`, reference with `x.workspace = true`

## 2026-04-04 time_utils.rs Created
- `time` crate v0.3 with features `parsing`, `formatting`, `macros` already in workspace deps
- `format_description!` produces `Z` suffix; `Rfc3339` produces `+00:00` — must use former for backward compat
- `Rfc3339` parsing accepts both `Z` and `+00:00` — use for input, `format_description!` for output
- `OffsetDateTime::now_utc() - parsed` gives `time::Duration`; use `.is_negative()` for future detection
- `elapsed.whole_seconds()` returns `i64` (signed) — no need for `abs()`
- Dead code warnings expected until `recent_projects.rs` is migrated to use `crate::time_utils::*`
- 7 tests all pass: relative time (past/future), malformed/empty input, ISO format validation, parse with Z and offset

## 2026-04-04 T4: recent_projects_list.rs Created
- `Vec::retain` passes `&T`, not `&Field` — so `prune()` must wrap the predicate: `self.projects.retain(|p| should_retain(&p.path))`
- The `mut` is needed on `should_retain` parameter in `prune()` to allow the closure to be called multiple times
- Moving `RecentProject` struct to new module: just move the definition, add `use crate::recent_projects_list::RecentProject;` in old module
- `RecentProjectsList` and `MAX_RECENT_PROJECTS` will show dead_code warnings until T7 wires them in — expected
- No changes to existing tests needed — they use `super::*` which still includes `RecentProject` via the re-import
