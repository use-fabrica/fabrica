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

## 2026-04-04 T5: Pure Tests for RecentProjectsList
- 10 tests added: add_new, dedup_moves_to_top, truncation_at_five, truncation_exact_five, remove_existing, remove_nonexistent, prune_retain_all, prune_remove_all, prune_selective, empty_list_operations
- `add()` inserts at position 0, so items added in order a,b,c result in list [c,b,a] — test assertions must account for reversed insertion order
- After adding 6 items (p0..p5), truncation drops the oldest (first-added), leaving [p5,p4,p3,p2,p1] — last is p1 not p4
- Rust closure lifetime issue: passing `&mut |_| true` directly to `prune(&mut impl FnMut)` fails with "implementation of FnMut is not general enough". Fix: bind closure to a typed variable first: `let mut pred = |_p: &Path| true; list.prune(&mut pred);`
- Alternative fix: annotate the closure parameter type inline: `list.prune(&mut |p: &Path| ...)`
- No filesystem I/O in tests — all paths are hardcoded strings, no `temp_dir()` usage
- All 27 package tests pass (10 new + 17 existing)

## 2026-04-04 T7: Adapter Delegation Pattern
- Rust `Deref` does NOT enable field access — only method calls. Cannot use Deref to expose `list.projects` as `rp.projects`
- Used `std::mem::take` pattern: transfer Vec to temp list, delegate, take back — O(1) for small Vecs (max 5 entries)
- Adapter keeps `pub projects: Vec<RecentProject>` directly on struct for zero dashboard changes
- Import needed: `use crate::recent_projects_list::{RecentProject, RecentProjectsList};`

## 2026-04-04 T8: Test Migration Strategy
- Removed `test_add_existing_project_dedup` and `test_truncation_at_five` from adapter (pure logic migrated to list module)
- Added 4 integration tests: `test_add_persists_to_disk`, `test_remove_persists_to_disk`, `test_prune_persists_to_disk`, `test_add_rejects_nonexistent_path`
- `test_add_rejects_nonexistent_path` verifies both: projects stays empty AND no file created on disk
- Total: 29 tests (12 adapter + 10 list + 7 time_utils)

## 2026-04-04 T9: Clippy Cleanup
- `#[expect(dead_code)]` warns in test builds when lint IS triggered — use `#[allow(dead_code)]` instead
- `std::io::Error::new(ErrorKind::Other, e)` → `std::io::Error::other(e)` (clippy io_other_error lint)
- `impl Default` with empty Vec → `#[derive(Default)]` (clippy derivable_impls lint)
- `let _ = expr_returning_unit()` → just call expr (clippy let_unit_value lint)
- Final: 29 tests, 0 clippy warnings, 0 LSP diagnostics
