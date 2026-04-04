# Final QA Report — recent-projects-refactor

**Date:** 2026-04-04
**Branch:** refactor/dashboard-event-decoupling
**Base:** dev
**Verdict: APPROVE**

---

## Phase 1: Clean State Verification

| Check | Result |
|-------|--------|
| `cargo clean` | PASS — 14,791 files removed |
| `cargo check --package fabrica` | PASS — exit 0, clean compile |
| `cargo test --package fabrica` | PASS — 29/29 tests, 0 failures |
| `cargo clippy --package fabrica -- -D warnings` | PASS — 0 warnings, exit 0 |

**Evidence:** `full-test-suite.txt`, `clippy.txt`, `cargo-check.txt`

---

## Phase 2: QA Scenarios from Plan (T1–T9)

### T1: time Dependency Features
| Scenario | Result | Detail |
|----------|--------|--------|
| Dependency compiles | PASS | `cargo check` exit 0 |
| Correct features | PASS | `alloc,default,formatting,macros,parsing,std` — parsing, formatting, macros present; local-offset and serde absent |

### T2: time_utils Module
| Scenario | Result | Detail |
|----------|--------|--------|
| Tests pass | PASS | 7/7 tests pass |
| Output strings present | PASS | "just now", "m ago", "h ago", "d ago", "long ago", "unknown" — all 6 found |
| Z suffix output | PASS | `test_now_iso_format` passes |

### T3: Old Code Removal
| Scenario | Result | Detail |
|----------|--------|--------|
| No SystemTime/UNIX_EPOCH | PASS | grep exits 1 — no matches in recent_projects.rs |
| Deleted functions gone | PASS | `days_to_ymd`, `is_leap`, `parse_iso_to_epoch`, `epoch_to_iso`, `current_iso_timestamp` — none found |
| All tests pass | PASS | 29/29 |

### T4: List Module Structure
| Scenario | Result | Detail |
|----------|--------|--------|
| Module compiles | PASS | `cargo check` exit 0 |
| RecentProject in list module | PASS | `struct RecentProject` found in recent_projects_list.rs only |
| MAX_RECENT_PROJECTS | PASS | `pub(crate) const MAX_RECENT_PROJECTS: usize = 5;` found |

### T5: Pure List Tests
| Scenario | Result | Detail |
|----------|--------|--------|
| Pure tests pass | PASS | 10/10 tests pass |
| No filesystem in pure tests | PASS | No temp_dir/TempDir/tempdir references |

### T6: Dashboard Integration
| Scenario | Result | Detail |
|----------|--------|--------|
| Dashboard compiles | PASS | `cargo check` exit 0 |
| Only import lines changed | PASS | diff shows only 2 import lines changed (removed `format_relative_time` import, added `time_utils` import) |

### T7: Thin Adapter Pattern
| Scenario | Result | Detail |
|----------|--------|--------|
| Adapter compiles | PASS | `cargo check` exit 0 |
| No business logic | PASS | No `retain`/`truncate`/`insert(0` in adapter — delegates to `RecentProjectsList` |
| Auto-save preserved | PASS | `add()`, `remove()`, `prune()` all call `self.save()` |

### T8: Persistence Tests
| Scenario | Result | Detail |
|----------|--------|--------|
| All persistence tests pass | PASS | 12/12 adapter tests pass |
| Old JSON with Z loads | PASS | `test_load_valid_json` passes (loads `2025-01-01T00:00:00Z` format) |
| No old epoch_to_iso | PASS | `epoch_to_iso` not found |
| default_with_path exists | NOTE | Present as legitimate private constructor for adapter (used in tests and load). Not the old helper — it's a thin factory for dependency injection of file paths. |

### T9: Final Verification
| Scenario | Result | Detail |
|----------|--------|--------|
| Full test suite passes | PASS | 29/29 tests |
| Zero clippy warnings | PASS | exit 0 |
| No residual deleted code | PASS | No `days_to_ymd`, `is_leap`, `parse_iso_to_epoch`, `epoch_to_iso`, `SystemTime`, `UNIX_EPOCH` in `crates/fabrica/src/` |
| app.rs zero diff | PASS | `git diff dev -- app.rs` — empty |
| dashboard.rs import-only diff | PASS | Only 2 import lines changed |

---

## Phase 3: Integration Tests (Cross-Task)

| Test | Result | Description |
|------|--------|-------------|
| `test_load_valid_json` | PASS | Old JSON with Z suffix loads correctly |
| `test_save_load_roundtrip` | PASS | Save → Load → data preserved (3 projects) |
| `test_add_persists_to_disk` | PASS | Add → reload → data persisted |
| `test_remove_persists_to_disk` | PASS | Remove → reload → removed correctly |
| `test_prune_persists_to_disk` | PASS | Prune → reload → pruned correctly |
| `test_add_rejects_nonexistent_path` | PASS | Nonexistent path rejected, no file created |

**Integration: 6/6 PASS**

---

## Phase 4: Edge Cases

| Test | Result | Description |
|------|--------|-------------|
| `test_load_missing_file` | PASS | Missing file → empty list (graceful degradation) |
| `test_load_corrupted_json` | PASS | Corrupted JSON → empty list (graceful degradation) |
| `test_format_relative_time_malformed` | PASS | "not-a-timestamp" → "unknown" |
| `test_format_relative_time_future` | PASS | Future timestamp → "just now" |
| `test_format_relative_time_empty` | PASS | Empty string → "unknown" |
| `test_add_dedup_moves_to_top` | PASS | Same path twice → appears once at top |
| `test_add_truncation_at_five` | PASS | 6 projects → only 5 remain |
| `test_empty_list_operations` | PASS | Remove/prune on empty list → no panic |

**Edge Cases: 8/8 PASS**

---

## Summary

```
Scenarios [26/26 pass] | Integration [6/6] | Edge Cases [8 tested] | VERDICT: APPROVE
```

### Test Distribution
- `time_utils`: 7 tests
- `recent_projects_list`: 10 tests (pure, no filesystem)
- `recent_projects`: 12 tests (adapter with filesystem)
- **Total: 29 tests, 0 failures**

### Notes
1. Base branch is `dev` (not `main`)
2. `default_with_path` exists in adapter — this is a legitimate private constructor for dependency injection, not an old helper
3. All adapter methods (`add`, `remove`, `prune`) delegate to `RecentProjectsList` and call `self.save()` after mutation
4. `app.rs` has zero diff from dev
5. `dashboard.rs` has exactly 2 import-line changes
6. No residual deleted code found anywhere in `crates/fabrica/src/`
