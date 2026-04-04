# Refactor: Deepen RecentProjects Module — Separate I/O, Logic, and Time Formatting

> **Quick Summary**: Split the monolithic `recent_projects.rs` (482 lines) into three focused modules: a `time_utils.rs` utility (backed by the `time` crate), a pure in-memory `recent_projects_list.rs` for business logic, and a slimmed `recent_projects.rs` as a persistence adapter. Preserve all public API surface and behavioral contracts.
>
> **Deliverables**:
> - `time_utils.rs` — standalone time formatting module using `time` crate
> - `recent_projects_list.rs` — pure in-memory business logic (no filesystem, no clock)
> - Refactored `recent_projects.rs` — thin persistence adapter delegating to list + I/O
> - All 11 original test scenarios covered (or replaced with equivalents)
> - `time` crate added to workspace dependencies
> - Zero changes to `app.rs` consumer, one import-line change in `dashboard.rs`
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES — 3 waves
> **Critical Path**: T1 → T3 → T5 → T7 → T8 → F1-F4

---

## Context

### Original Request
> GitHub Issue #2: The `RecentProjects` module is the largest file in the Fabrica codebase (~482 lines) and mixes five distinct concerns: data types, file I/O, business logic, time formatting, and tests. Split into three modules with clear boundaries.

### Interview Summary
**Key Discussions**:
- File organization: user wants idiomatic Rust → flat `.rs` files under `src/` (matches existing pattern)
- Test scope: include test restructuring to match new module boundaries
- Dependency strategy: separate phase for `time` crate investigation and addition
- Phase granularity: 5-phase breakdown approved as-is

**Research Findings**:
- `dashboard.rs` accesses `pub projects: Vec<RecentProject>` directly at 8+ sites — must preserve field access
- `add()`, `remove()`, `prune()` all auto-save — behavioral contract consumers depend on
- `format_relative_time` is `pub` (effectively `pub(crate)` since module is `pub(crate)`)
- `time` crate NOT currently a dependency — needs workspace-level addition
- Current timestamp format outputs `Z` suffix, not `+00:00` — must use custom format description to preserve
- Current output strings are compact: `"just now"`, `"5m ago"`, `"2h ago"`, `"3d ago"`, `"long ago"`

### Metis Review
**Identified Gaps** (all addressed):
- `Z` vs `+00:00` format: Use custom `format_description!` for output, `Rfc3339` for parsing
- `pub projects` field access: Preserve as-is, do NOT add accessors
- `last_opened` type: Keep as `String`, do NOT change to `OffsetDateTime`
- Future timestamps: Handle `elapsed.is_negative()` → return `"just now"`
- `"unknown"` fallback: Preserve for unparseable timestamps
- Test mapping: Explicit old→new mapping included in plan
- `time` crate features: Exactly `["parsing", "formatting", "macros"]` — no more

---

## Work Objectives

### Core Objective
Refactor `recent_projects.rs` into three focused modules with clear separation of concerns: time formatting (standalone utility), business logic (pure in-memory), and persistence (thin I/O adapter).

### Concrete Deliverables
- `crates/fabrica/src/time_utils.rs` — time formatting functions
- `crates/fabrica/src/recent_projects_list.rs` — pure business logic struct
- Refactored `crates/fabrica/src/recent_projects.rs` — slimmed to persistence adapter
- Updated `crates/fabrica/src/main.rs` — new module declarations
- Updated `crates/fabrica/src/dashboard.rs` — import line change only
- `time` crate in workspace dependencies

### Definition of Done
- [ ] `cargo test --package fabrica` — all tests pass
- [ ] `cargo clippy --package fabrica -- -D warnings` — zero warnings
- [ ] Zero `SystemTime`/`UNIX_EPOCH` references remain in `crates/fabrica/src/`
- [ ] All 11 original test scenarios covered (verified by explicit mapping)
- [ ] `app.rs` has ZERO changes (diff is clean)
- [ ] `dashboard.rs` has only import-line change

### Must Have
- Identical public API surface for `RecentProjects` (`load`, `save`, `add`, `remove`, `prune`)
- Identical public signature for `format_relative_time(&str) -> String`
- Identical output strings: `"just now"`, `"Xm ago"`, `"Xh ago"`, `"Xd ago"`, `"long ago"`, `"unknown"`
- Auto-save on every mutation (`add`, `remove`, `prune` all persist after execution)
- Atomic write pattern (temp file + rename) preserved
- `Z` suffix in generated timestamps (NOT `+00:00`)
- JSON file format backward-compatible (old files with `Z` load correctly)
- `pub projects: Vec<RecentProject>` field stays publicly accessible
- `MAX_RECENT_PROJECTS = 5` as named constant
- Pure list accepts timestamp as parameter (decoupled from clock)
- Pure list accepts predicate for prune (decoupled from filesystem)

### Must NOT Have (Guardrails)
- NO accessor methods for `projects` field — dashboard accesses it directly
- NO change to `last_opened` type from `String` to `OffsetDateTime`
- NO change to `"5m ago"` style to `"5 minutes ago"` style
- NO changes to `app.rs` — its API surface is identical
- NO changes to `dashboard.rs` beyond the import line
- NO new `time` crate features beyond `["parsing", "formatting", "macros"]`
- NO change to JSON file structure (`{"recent_projects": [...]}`)
- NO change to 5-project truncation limit
- NO removal of `"unknown"` fallback for unparseable timestamps
- NO `time` crate `local-offset` or `serde` features
- NO error handling improvements (keep graceful `eprintln!` + degradation)
- NO refactoring of GPUI entity wrapper or dashboard rendering

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: YES (`cargo test` with `#[cfg(test)]` modules)
- **Automated tests**: Tests-after (restructure existing tests + add new pure tests)
- **Framework**: Rust built-in test framework (`cargo test`)
- **Pattern**: Tests in `#[cfg(test)] mod tests` within each module file

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Compilation**: Use Bash — `cargo check`, `cargo test`, `cargo clippy`
- **Logic verification**: Use Bash — run specific test filters, assert output
- **File verification**: Use Grep — verify no residual references to deleted code
- **API surface**: Use Grep — verify import paths updated correctly

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — dependency + time module):
├── Task 1: Add `time` crate to workspace dependencies [quick]
├── Task 2: Create `time_utils.rs` module [deep]
└── Task 3: Delete time functions from `recent_projects.rs` + update `main.rs` [quick]

Wave 2 (After Wave 1 — pure logic extraction):
├── Task 4: Create `recent_projects_list.rs` pure in-memory module [deep]
├── Task 5: Write pure logic tests for `RecentProjectsList` [unspecified-high]
└── Task 6: Update `dashboard.rs` import for `format_relative_time` [quick]

Wave 3 (After Wave 2 — adapter + test migration):
├── Task 7: Refactor `recent_projects.rs` to thin persistence adapter [deep]
├── Task 8: Migrate and restructure persistence tests [unspecified-high]
└── Task 9: Final test audit + cleanup [unspecified-high]

Wave FINAL (After ALL tasks — 4 parallel reviews):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real QA — run all tests + verify API contracts (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: T1 → T3 → T4 → T7 → T8 → T9 → F1-F4
Parallel Speedup: ~40% faster than sequential
Max Concurrent: 3 (Waves 1 & 2)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| T1 | — | T2, T3 | 1 |
| T2 | T1 | T3, T6 | 1 |
| T3 | T1, T2 | T4, T7 | 1 |
| T4 | T3 | T5, T7 | 2 |
| T5 | T4 | T7, T9 | 2 |
| T6 | T2 | T9 | 2 |
| T7 | T4, T5 | T8 | 3 |
| T8 | T7 | T9 | 3 |
| T9 | T5, T6, T8 | F1-F4 | 3 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1 → `quick`, T2 → `deep`, T3 → `quick`
- **Wave 2**: 3 tasks — T4 → `deep`, T5 → `unspecified-high`, T6 → `quick`
- **Wave 3**: 3 tasks — T7 → `deep`, T8 → `unspecified-high`, T9 → `unspecified-high`
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 1. Add `time` crate to workspace dependencies

  **What to do**:
  - Add `time = { version = "0.3", features = ["parsing", "formatting", "macros"] }` to `[workspace.dependencies]` in root `Cargo.toml`
  - Add `time.workspace = true` to `crates/fabrica/Cargo.toml` under `[dependencies]`
  - Follow the exact pattern used by `serde` and `serde_json` in the workspace
  - Run `cargo check` to verify compilation
  - Run `cargo tree -i time` to verify dependency resolution

  **Must NOT do**:
  - Do NOT add features beyond `["parsing", "formatting", "macros"]` (no `local-offset`, no `serde`)
  - Do NOT add to individual crate Cargo.toml without workspace pattern

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple dependency addition following existing workspace pattern
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - `gpui-*`: Not needed — no GPUI code involved

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 1 start (first task)
  - **Blocks**: T2, T3
  - **Blocked By**: None (can start immediately)

  **References**:

  **Pattern References** (existing code to follow):
  - `Cargo.toml` (root) — Workspace dependency declaration pattern: `serde = { version = "1.0.228", features = ["derive"] }` under `[workspace.dependencies]`
  - `crates/fabrica/Cargo.toml` — Workspace reference pattern: `serde.workspace = true` under `[dependencies]`

  **Why Each Reference Matters**:
  - Root `Cargo.toml`: Shows the exact TOML format for workspace deps with features
  - `crates/fabrica/Cargo.toml`: Shows how to reference workspace deps from a crate

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Dependency compiles correctly
    Tool: Bash
    Preconditions: time crate added to both Cargo.toml files
    Steps:
      1. Run `cargo check --package fabrica`
      2. Assert exit code 0
    Expected Result: Compilation succeeds with zero errors
    Failure Indicators: Exit code non-zero, "could not find" or "feature not found" errors
    Evidence: .sisyphus/evidence/task-1-dep-compiles.txt

  Scenario: Correct features are enabled
    Tool: Bash
    Preconditions: time crate added
    Steps:
      1. Run `cargo tree -p time -f "{f}"`
      2. Assert output contains "parsing", "formatting", "macros"
      3. Assert output does NOT contain "local-offset" or "serde"
    Expected Result: Only the three specified features are enabled
    Failure Indicators: Missing features or unexpected features present
    Evidence: .sisyphus/evidence/task-1-features.txt
  ```

  **Commit**: YES
  - Message: `chore(deps): add time crate for ISO 8601 parsing and formatting`
  - Files: `Cargo.toml`, `crates/fabrica/Cargo.toml`
  - Pre-commit: `cargo check --package fabrica`

- [x] 2. Create `time_utils.rs` module with `time` crate

  **What to do**:
  - Create `crates/fabrica/src/time_utils.rs` with three public functions:
    - `now_iso() -> String` — returns current UTC timestamp as ISO 8601 with `Z` suffix
      - Use `OffsetDateTime::now_utc()` + custom `format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z")`
      - NOT `Rfc3339` format (which outputs `+00:00` instead of `Z`)
    - `format_relative_time(iso_str: &str) -> String` — converts ISO string to human-readable relative string
      - Use `OffsetDateTime::parse(iso_str, &Rfc3339)` for parsing (accepts both `Z` and `+00:00`)
      - On parse error: return `"unknown"` (preserve current behavior)
      - Handle negative durations (future timestamps): return `"just now"`
      - Preserve exact output thresholds: <60s → "just now", <3600s → "Xm ago", <86400s → "Xh ago", <2592000s → "Xd ago", else → "long ago"
    - `parse_iso(iso_str: &str) -> Option<OffsetDateTime>` — internal helper for parsing (pub(crate))
  - Write `#[cfg(test)] mod tests` within `time_utils.rs`:
    - `test_format_relative_time` with hardcoded timestamps (NOT relative to "now")
      - Test: `"2025-01-15T12:00:00Z"` evaluated at a known later time → verify bucket
    - `test_format_relative_time_malformed` — garbage input → `"unknown"`
    - `test_format_relative_time_empty` — empty string → `"unknown"`
    - `test_now_iso_format` — verify output matches regex `\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z`
    - `test_parse_iso_valid` — `"2025-01-15T12:00:00Z"` parses successfully
    - `test_parse_iso_with_offset` — `"2025-01-15T12:00:00+05:30"` parses (behavioral expansion)
    - `test_format_relative_time_future` — future timestamp → `"just now"`
  - Declare `pub(crate) mod time_utils;` in `main.rs`

  **Must NOT do**:
  - Do NOT use `Rfc3339` for output formatting (outputs `+00:00` not `Z`)
  - Do NOT change output format strings (keep `"5m ago"` not `"5 minutes ago"`)
  - Do NOT remove `"unknown"` fallback
  - Do NOT add `local-offset` feature usage

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Requires understanding time crate API, careful format handling, and rewriting time logic
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - `gpui-*`: Not needed — no GPUI code, pure Rust logic

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T1)
  - **Parallel Group**: Wave 1 (after T1)
  - **Blocks**: T3, T6
  - **Blocked By**: T1

  **References**:

  **Pattern References** (existing code to follow):
  - `crates/fabrica/src/recent_projects.rs:115-216` — Current time functions to understand exact behavior: `current_iso_timestamp()`, `format_relative_time()`, `parse_iso_to_epoch()`, `days_to_ymd()`, `is_leap()`. These are being REPLACED.
  - `crates/fabrica/src/recent_projects.rs:165-187` — The `format_relative_time()` implementation: threshold constants (60, 3600, 86400, 2592000), output strings ("just now", "Xm ago", "Xh ago", "Xd ago", "long ago"), "unknown" fallback

  **API/Type References** (contracts to implement against):
  - `crates/fabrica/src/dashboard.rs:7` — Current import: `use crate::recent_projects::format_relative_time;` — must produce same function signature
  - `crates/fabrica/src/dashboard.rs:113` — Usage: `format_relative_time(&project.last_opened)` — accepts `&str`, returns `String`
  - `crates/fabrica/src/dashboard.rs:236` — Another usage site for verification

  **Test References** (testing patterns to follow):
  - `crates/fabrica/src/recent_projects.rs:464-481` — Current `test_format_relative_time` test pattern — understand what's being tested, rewrite with hardcoded timestamps instead of `epoch_to_iso` helper

  **External References**:
  - `time` crate docs: `OffsetDateTime`, `format_description!` macro, `Rfc3339` well-known format
  - The `format_description!` macro provides compile-time format verification

  **Why Each Reference Matters**:
  - Lines 115-216: The code being replaced — must understand exact behavior to replicate
  - Lines 165-187: The threshold values and output strings — these are product decisions that must be preserved exactly
  - Dashboard usage: The consumer contract — function signature must match
  - Current test: The test being rewritten — understand coverage before replacing

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Time module compiles and tests pass
    Tool: Bash
    Preconditions: time_utils.rs created, main.rs updated
    Steps:
      1. Run `cargo test --package fabrica -- time_utils`
      2. Assert all tests pass
      3. Assert at least 7 tests ran (format_relative_time, malformed, empty, now_iso_format, parse_valid, parse_offset, future)
    Expected Result: All time_utils tests pass
    Failure Indicators: Any test failure, compilation error
    Evidence: .sisyphus/evidence/task-2-time-tests.txt

  Scenario: format_relative_time preserves output format
    Tool: Bash
    Preconditions: time_utils.rs created
    Steps:
      1. Grep `time_utils.rs` for the exact strings: "just now", "m ago", "h ago", "d ago", "long ago", "unknown"
      2. Assert ALL six strings are present in the implementation
    Expected Result: All six output strings found in source
    Failure Indicators: Any output string missing
    Evidence: .sisyphus/evidence/task-2-output-strings.txt

  Scenario: now_iso outputs Z suffix not +00:00
    Tool: Bash
    Preconditions: time_utils.rs created with test
    Steps:
      1. Run `cargo test --package fabrica -- test_now_iso_format`
      2. Assert test passes (test verifies regex matches Z suffix)
    Expected Result: now_iso() output ends with 'Z', not '+00:00'
    Failure Indicators: Test fails, or output contains '+00:00'
    Evidence: .sisyphus/evidence/task-2-z-suffix.txt
  ```

  **Commit**: NO (groups with T3)

- [x] 3. Delete time functions from `recent_projects.rs` and update module declarations

  **What to do**:
  - Remove from `recent_projects.rs`:
    - `use std::time::{SystemTime, UNIX_EPOCH};` import
    - `current_iso_timestamp()` function (lines ~115-132)
    - `days_to_ymd()` function (lines ~134-159)
    - `is_leap()` function (lines ~161-163)
    - `parse_iso_to_epoch()` function (lines ~189-216)
    - The `epoch_to_iso()` test helper (lines ~218-230)
  - Add to `recent_projects.rs`:
    - `use crate::time_utils::{now_iso, format_relative_time};` (for adapter use — `add()` calls `now_iso`)
    - NOTE: `format_relative_time` is NOT re-exported from here. Dashboard imports it from `time_utils` directly after T6.
  - Verify `main.rs` has both module declarations:
    - `pub(crate) mod time_utils;`
    - `pub(crate) mod recent_projects;` (existing)
  - Run `cargo test --package fabrica` — the `format_relative_time` test in `recent_projects.rs` should be removed (moved to `time_utils.rs`). Remaining tests should still compile and pass (they don't call time functions directly, they construct `RecentProject` with string timestamps).

  **Must NOT do**:
  - Do NOT delete `format_relative_time` from the module yet — it's still used by dashboard.rs. Remove only the time-internal functions.
  - Actually: `format_relative_time` IS being moved to `time_utils.rs` in T2. In `recent_projects.rs`, delete it entirely. Dashboard import is updated in T6.
  - Do NOT touch any business logic (add/remove/prune) — that's T4/T7
  - Do NOT touch any tests except removing `epoch_to_iso` helper and `test_format_relative_time`

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Surgical deletion of functions that have been replicated in time_utils.rs
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - All gpui skills: Not needed — deleting plain Rust functions

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T1 and T2)
  - **Parallel Group**: Wave 1 (after T1+T2)
  - **Blocks**: T4, T7
  - **Blocked By**: T1, T2

  **References**:

  **Pattern References** (existing code to follow):
  - `crates/fabrica/src/recent_projects.rs:1-6` — Current imports — understand which to remove (`std::time`)
  - `crates/fabrica/src/recent_projects.rs:87-102` — `add()` method calls `current_iso_timestamp()` — after deletion, add `use crate::time_utils::now_iso;` and replace call

  **API/Type References**:
  - `crates/fabrica/src/main.rs:5` — Current `pub(crate) mod recent_projects;` — add `pub(crate) mod time_utils;` alongside

  **Why Each Reference Matters**:
  - Import section: Know exactly what to remove
  - `add()` method: The ONE call site for `current_iso_timestamp()` — must update to use `now_iso()`
  - main.rs: Where module declarations live

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: No SystemTime/UNIX_EPOCH references remain
    Tool: Bash
    Preconditions: Time functions deleted from recent_projects.rs
    Steps:
      1. Run `grep -rn "SystemTime\|UNIX_EPOCH" crates/fabrica/src/recent_projects.rs`
      2. Assert exit code 1 (no matches found)
    Expected Result: Zero references to std::time types
    Failure Indicators: Any grep match
    Evidence: .sisyphus/evidence/task-3-no-std-time.txt

  Scenario: Deleted functions are gone
    Tool: Bash
    Preconditions: Deletions complete
    Steps:
      1. Run `grep -n "fn days_to_ymd\|fn is_leap\|fn parse_iso_to_epoch\|fn epoch_to_iso\|fn current_iso_timestamp" crates/fabrica/src/recent_projects.rs`
      2. Assert exit code 1 (no matches)
    Expected Result: All deleted functions removed
    Failure Indicators: Any grep match
    Evidence: .sisyphus/evidence/task-3-deleted-fns.txt

  Scenario: Remaining tests still pass
    Tool: Bash
    Preconditions: Deletions complete, epoch_to_iso and test_format_relative_time removed
    Steps:
      1. Run `cargo test --package fabrica 2>&1`
      2. Assert all tests pass (should be 10 remaining tests in recent_projects, 7+ in time_utils)
    Expected Result: Zero test failures, compilation succeeds
    Failure Indicators: Compilation error or test failure
    Evidence: .sisyphus/evidence/task-3-tests-pass.txt
  ```

  **Commit**: YES (groups with T2)
  - Message: `refactor(recent-projects): extract time_utils module`
  - Files: `crates/fabrica/src/time_utils.rs`, `crates/fabrica/src/recent_projects.rs`, `crates/fabrica/src/main.rs`
  - Pre-commit: `cargo test --package fabrica`

- [ ] 4. Create `recent_projects_list.rs` — pure in-memory business logic module

  **What to do**:
  - Create `crates/fabrica/src/recent_projects_list.rs` with:
    - Shared types: `RecentProject` struct (moved from `recent_projects.rs`), `RecentProjectsFile` struct (if needed for serde)
    - Constant: `pub(crate) const MAX_RECENT_PROJECTS: usize = 5;`
    - `RecentProjectsList` struct with `projects: Vec<RecentProject>` field
    - Methods:
      - `add(&mut self, path: PathBuf, timestamp: String)` — dedup by path (remove existing), prepend new entry at index 0, truncate to `MAX_RECENT_PROJECTS`. NO filesystem validation, NO auto-save.
      - `remove(&mut self, path: &Path)` — filter out matching path. NO auto-save.
      - `prune(&mut self, should_retain: impl Fn(&Path) -> bool)` — retain only entries where predicate returns true for their path. NO auto-save.
      - `new() -> Self` / `Default` impl — empty list
    - `pub projects: Vec<RecentProject>` field stays public (dashboard accesses directly)
  - Declare `pub(crate) mod recent_projects_list;` in `main.rs`
  - The `RecentProject` type must have `pub path: PathBuf` and `pub last_opened: String` with `Serialize, Deserialize` derives
  - Update `recent_projects.rs` to import `RecentProject` and `RecentProjectsList` from the new module instead of defining them locally
  - Verify compilation: `cargo check --package fabrica`

  **Must NOT do**:
  - Do NOT add path existence validation in the list module (that's the adapter's job)
  - Do NOT call `save()` from the list module
  - Do NOT generate timestamps in the list module
  - Do NOT make `projects` field private or add accessor methods
  - Do NOT change `last_opened` from `String` to `OffsetDateTime`
  - Do NOT add the predicate to `add()` — only `prune()` gets a predicate

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Core architectural extraction — must carefully separate pure logic from I/O, preserve behavioral contracts
  - **Skills**: [`gpui-entity`]
    - `gpui-entity`: RecentProjects is a GPUI Entity — need to understand how the entity wrapper interacts with the extracted list
  - **Skills Evaluated but Omitted**:
    - `gpui-components`: Not building UI components

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T3 for clean recent_projects.rs)
  - **Parallel Group**: Wave 2 (after Wave 1 complete)
  - **Blocks**: T5, T7
  - **Blocked By**: T3

  **References**:

  **Pattern References** (existing code to follow):
  - `crates/fabrica/src/recent_projects.rs:7-16` — `RecentProject` and `RecentProjectsFile` structs — exact field names, derive macros, visibility
  - `crates/fabrica/src/recent_projects.rs:87-112` — The business logic being extracted: `add()` (dedup + prepend + truncate), `remove()` (filter), `prune()` (filter by existence)
  - `crates/fabrica/src/recent_projects.rs:97` — Magic number `5` for truncation — extract as `MAX_RECENT_PROJECTS`

  **API/Type References**:
  - `crates/fabrica/src/dashboard.rs:36` — `projects.projects.get(i)` — direct field access pattern. The extracted list MUST keep `pub projects: Vec<RecentProject>`.
  - `crates/fabrica/src/app.rs:85-87` — Calls `entity.update(cx, |rp, _| rp.add(...))` — the adapter's `add()` must keep same signature, but internally delegates to list

  **Why Each Reference Matters**:
  - Lines 7-16: Exact struct definitions to move — serde derives, field visibility
  - Lines 87-112: The logic to extract — dedup algorithm, truncation, remove filter
  - Dashboard field access: Confirms `projects` must stay public
  - app.rs entity usage: Confirms adapter must keep same mutation signatures

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: List module compiles independently
    Tool: Bash
    Preconditions: recent_projects_list.rs created, main.rs updated
    Steps:
      1. Run `cargo check --package fabrica`
      2. Assert exit code 0
    Expected Result: Clean compilation
    Failure Indicators: Compilation errors, unresolved imports
    Evidence: .sisyphus/evidence/task-4-list-compiles.txt

  Scenario: RecentProject type moved correctly
    Tool: Bash
    Preconditions: Types moved to new module
    Steps:
      1. Run `grep -n "struct RecentProject" crates/fabrica/src/recent_projects_list.rs`
      2. Assert match found
      3. Run `grep -n "struct RecentProject" crates/fabrica/src/recent_projects.rs`
      4. Assert exit code 1 (not found — moved out)
    Expected Result: Type defined in list module only, not duplicated
    Failure Indicators: Type in both files, or not in either
    Evidence: .sisyphus/evidence/task-4-type-moved.txt

  Scenario: MAX_RECENT_PROJECTS constant exists
    Tool: Bash
    Preconditions: List module created
    Steps:
      1. Run `grep -n "MAX_RECENT_PROJECTS" crates/fabrica/src/recent_projects_list.rs`
      2. Assert match found with value 5
    Expected Result: Named constant replaces magic number
    Failure Indicators: No match, or value != 5
    Evidence: .sisyphus/evidence/task-4-constant.txt
  ```

  **Commit**: NO (groups with T5)

- [ ] 5. Write pure logic tests for `RecentProjectsList`

  **What to do**:
  - Add `#[cfg(test)] mod tests` to `recent_projects_list.rs` with pure tests (NO filesystem, NO temp dirs, NO I/O):
    - `test_add_new_project` — add a path, verify it appears at index 0
    - `test_add_dedup_moves_to_top` — add path A, add path B, add path A again → A is at index 0 with updated timestamp
    - `test_add_truncation_at_five` — add 6 entries → only 5 remain, oldest dropped
    - `test_add_truncation_exact_five` — add exactly 5 → all remain
    - `test_remove_existing` — remove a path that exists → it's gone
    - `test_remove_nonexistent` — remove a path that doesn't exist → no change
    - `test_prune_with_predicate_retain_all` — predicate always returns true → no change
    - `test_prune_with_predicate_remove_all` — predicate always returns false → empty list
    - `test_prune_with_predicate_selective` — predicate returns false for specific path → only that removed
    - `test_empty_list_operations` — remove and prune on empty list → no panic
  - All tests construct `RecentProjectsList` directly and call methods with explicit parameters
  - Timestamps are hardcoded strings like `"2025-01-15T12:00:00Z"`
  - Run `cargo test --package fabrica -- recent_projects_list` to verify

  **Must NOT do**:
  - Do NOT use `temp_dir()` in any test
  - Do NOT create real filesystem paths
  - Do NOT call `save()` in any test
  - Do NOT test path validation (that's adapter's responsibility)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Many test cases to write, each testing specific behavioral edge cases
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - `tdd`: We're writing tests after the module (T4 already created the code)
    - `gpui-test`: Not testing GPUI components, testing pure Rust logic

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T4)
  - **Parallel Group**: Wave 2 (with T6 which is independent)
  - **Blocks**: T7, T9
  - **Blocked By**: T4

  **References**:

  **Test References** (testing patterns to follow):
  - `crates/fabrica/src/recent_projects.rs:332-348` — `test_add_new_project` — understand what's being tested
  - `crates/fabrica/src/recent_projects.rs:351-378` — `test_add_existing_project_dedup` — dedup behavior
  - `crates/fabrica/src/recent_projects.rs:381-404` — `test_truncation_at_five` — truncation boundary
  - `crates/fabrica/src/recent_projects.rs:407-431` — `test_remove_project` — remove behavior
  - `crates/fabrica/src/recent_projects.rs:434-461` — `test_prune_nonexistent_paths` — prune behavior

  **Why Each Reference Matters**:
  - Each test shows the exact behavior to replicate in pure form — same assertions, minus filesystem setup

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Pure tests pass without filesystem
    Tool: Bash
    Preconditions: Tests written in recent_projects_list.rs
    Steps:
      1. Run `cargo test --package fabrica -- recent_projects_list`
      2. Assert at least 10 tests ran
      3. Assert 0 failures
    Expected Result: All pure logic tests pass
    Failure Indicators: Any test failure
    Evidence: .sisyphus/evidence/task-5-pure-tests.txt

  Scenario: No temp_dir usage in pure tests
    Tool: Bash
    Preconditions: Tests written
    Steps:
      1. Run `grep -n "temp_dir\|TempDir\|tempdir" crates/fabrica/src/recent_projects_list.rs`
      2. Assert exit code 1 (no matches)
    Expected Result: Zero filesystem dependencies in pure test module
    Failure Indicators: Any grep match
    Evidence: .sisyphus/evidence/task-5-no-fs.txt
  ```

  **Commit**: YES (groups with T4)
  - Message: `refactor(recent-projects): extract pure in-memory RecentProjectsList`
  - Files: `crates/fabrica/src/recent_projects_list.rs`, `crates/fabrica/src/main.rs`, `crates/fabrica/src/recent_projects.rs` (type relocations)
  - Pre-commit: `cargo test --package fabrica -- recent_projects_list`

- [x] 6. Update `dashboard.rs` import for `format_relative_time`

  **What to do**:
  - Change `dashboard.rs` line 7 from:
    ```rust
    use crate::recent_projects::{RecentProjects, format_relative_time};
    ```
    to:
    ```rust
    use crate::recent_projects::RecentProjects;
    use crate::time_utils::format_relative_time;
    ```
  - Or equivalently, split into two use statements or use nested paths:
    ```rust
    use crate::recent_projects::RecentProjects;
    use crate::time_utils::format_relative_time;
    ```
  - Verify `cargo check --package fabrica` compiles
  - Verify `cargo test --package fabrica` passes

  **Must NOT do**:
  - Do NOT change any other line in dashboard.rs
  - Do NOT change `RecentProjects` import source (stays from `recent_projects`)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Single import line change
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - All: Too trivial for any skill

  **Parallelization**:
  - **Can Run In Parallel**: YES (independent of T4/T5)
  - **Parallel Group**: Wave 2 (with T4/T5)
  - **Blocks**: T9
  - **Blocked By**: T2 (time_utils must exist)

  **References**:

  **Pattern References**:
  - `crates/fabrica/src/dashboard.rs:7` — Current import line to modify

  **Why Each Reference Matters**:
  - Line 7: The exact line to change — verify no other imports from recent_projects need updating

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Dashboard compiles with new import
    Tool: Bash
    Preconditions: Import updated in dashboard.rs
    Steps:
      1. Run `cargo check --package fabrica`
      2. Assert exit code 0
    Expected Result: Clean compilation
    Failure Indicators: Unresolved import error
    Evidence: .sisyphus/evidence/task-6-dashboard-import.txt

  Scenario: Only import line changed in dashboard
    Tool: Bash
    Preconditions: Change made
    Steps:
      1. Run `git diff crates/fabrica/src/dashboard.rs`
      2. Assert diff shows only the import line change (lines starting with -use / +use)
      3. Assert no other lines in diff
    Expected Result: Exactly 2-3 lines changed (old import removed, new imports added)
    Failure Indicators: More than import lines in diff
    Evidence: .sisyphus/evidence/task-6-diff.txt
  ```

  **Commit**: YES
  - Message: `refactor(dashboard): update format_relative_time import to time_utils`
  - Files: `crates/fabrica/src/dashboard.rs`
  - Pre-commit: `cargo check --package fabrica`

- [ ] 7. Refactor `recent_projects.rs` to thin persistence adapter

  **What to do**:
  - Rewrite `RecentProjects` struct to be a thin coordination layer:
    ```rust
    pub(crate) struct RecentProjects {
        list: RecentProjectsList,
        file_path: PathBuf,
    }
    ```
  - Keep `pub projects: Vec<RecentProject>` accessible — either via `Deref` to `RecentProjectsList`, or by keeping `projects` as a direct field that the list mutates, or by delegating `projects()` accessor. **Simplest approach**: keep `projects` as a direct field on `RecentProjects` that mirrors the list, OR restructure so dashboard accesses `entity.read(cx).projects` through a Deref to the inner list. Choose the approach that requires zero dashboard changes.
  - Implement methods as thin wrappers:
    - `load(path: impl Into<PathBuf>) -> Self` — reads JSON, populates list
    - `save(&self)` — serializes list, atomic write (temp file + rename) — preserve exact pattern
    - `add(&mut self, path: impl Into<PathBuf>)` — validate path exists + is dir, call `now_iso()` for timestamp, delegate to `self.list.add(path, timestamp)`, call `self.save()`
    - `remove(&mut self, path: &Path)` — delegate to `self.list.remove(path)`, call `self.save()`
    - `prune(&mut self)` — call `self.list.prune(|p| p.exists() && p.is_dir())`, call `self.save()`
  - Remove all inline business logic — dedup, truncation, ordering all delegated to list
  - Import `RecentProjectsList`, `RecentProject`, `MAX_RECENT_PROJECTS` from `crate::recent_projects_list`
  - Import `now_iso` from `crate::time_utils`
  - Keep `Default` impl returning empty projects + empty file_path
  - Verify `cargo check --package fabrica` compiles
  - Verify `cargo test --package fabrica` passes (existing tests that remain should still work)

  **CRITICAL DESIGN DECISION for `projects` field access**:
  Dashboard accesses `recent_projects.read(cx).projects` directly. After refactoring, `RecentProjects` must expose `pub projects: Vec<RecentProject>`. Two approaches:
  - **Option A (Recommended)**: Keep `pub projects: Vec<RecentProject>` directly on `RecentProjects`. The adapter syncs this field after each list mutation. Simple, zero dashboard changes.
  - **Option B**: Implement `Deref<Target = RecentProjectsList>` on `RecentProjects`. Dashboard's `.projects` calls would go through Deref to the list's `projects` field. Elegant but may interact with GPUI Entity in unexpected ways.

  Use Option A unless it causes issues with GPUI Entity's borrow checker.

  **Must NOT do**:
  - Do NOT add any business logic to the adapter (dedup, truncation, ordering all stay in list)
  - Do NOT change auto-save behavior
  - Do NOT change atomic write pattern
  - Do NOT change `load()` error handling (graceful degradation on missing/corrupted)
  - Do NOT change the JSON file format
  - Do NOT touch dashboard.rs or app.rs

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Core architectural change — adapter pattern requires careful delegation design
  - **Skills**: [`gpui-entity`]
    - `gpui-entity`: Need to understand how Entity<RecentProjects> interacts with the refactored struct — borrow rules, read/update patterns
  - **Skills Evaluated but Omitted**:
    - `gpui-context`: Not using App/Window context directly

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T4/T5)
  - **Parallel Group**: Wave 3
  - **Blocks**: T8
  - **Blocked By**: T4, T5

  **References**:

  **Pattern References**:
  - `crates/fabrica/src/recent_projects.rs:41-85` — Current `load()` and `save()` implementations — preserve exact error handling and atomic write pattern
  - `crates/fabrica/src/recent_projects.rs:70-85` — Atomic write: temp file with `.json.tmp` extension + `fs::rename` — preserve this exactly
  - `crates/fabrica/src/recent_projects.rs:87-102` — Current `add()` — understand path validation + dedup + timestamp + truncate + save flow

  **API/Type References**:
  - `crates/fabrica/src/app.rs:37-42` — Construction: `cx.new(|_| RecentProjects::load(path))` — must keep same constructor
  - `crates/fabrica/src/app.rs:85-87` — Usage: `entity.update(cx, |rp, _| rp.add(...))` — must keep same mutation signatures
  - `crates/fabrica/src/app.rs:48-49` — Usage: `entity.update(cx, |rp, _| rp.remove(...))` and `rp.prune()`
  - `crates/fabrica/src/dashboard.rs:36,49,76,123,216,236,271` — All sites accessing `.projects` field directly

  **External References**:
  - `time` crate: `now_iso()` from `crate::time_utils` (created in T2)

  **Why Each Reference Matters**:
  - load/save: Exact patterns to preserve — error handling, atomic write
  - app.rs usage: The adapter's public API contract — cannot change signatures
  - dashboard field access: Determines how `projects` field must be exposed

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Adapter compiles and delegates correctly
    Tool: Bash
    Preconditions: recent_projects.rs refactored
    Steps:
      1. Run `cargo check --package fabrica`
      2. Assert exit code 0
    Expected Result: Clean compilation
    Failure Indicators: Type mismatches, missing imports, borrow errors
    Evidence: .sisyphus/evidence/task-7-adapter-compiles.txt

  Scenario: No business logic remains in adapter
    Tool: Bash
    Preconditions: Refactoring complete
    Steps:
      1. Run `grep -n "retain\|truncate\|insert(0" crates/fabrica/src/recent_projects.rs`
      2. Assert NO matches outside of load/save (load may use retain for deserialization)
    Expected Result: Dedup/truncation logic removed from adapter
    Failure Indicators: Any list manipulation logic in adapter
    Evidence: .sisyphus/evidence/task-7-no-logic.txt

  Scenario: Auto-save preserved on all mutations
    Tool: Bash
    Preconditions: Adapter refactored
    Steps:
      1. Read `recent_projects.rs`
      2. Verify `add()`, `remove()`, `prune()` all end with `self.save()` call
    Expected Result: All three mutation methods auto-save
    Failure Indicators: Any mutation method missing save call
    Evidence: .sisyphus/evidence/task-7-auto-save.txt
  ```

  **Commit**: NO (groups with T8)

- [ ] 8. Migrate and restructure persistence tests

  **What to do**:
  - Update the test module in `recent_projects.rs` to test the adapter:
    - Keep/adapt existing persistence tests:
      - `test_load_valid_json` — loads JSON with valid format, verifies deserialization
      - `test_load_missing_file` — missing file returns empty list (graceful degradation)
      - `test_load_corrupted_json` — corrupted JSON returns empty list with warning
      - `test_save_load_roundtrip` — save then load preserves data
      - `test_empty_list_save_load` — empty list round-trips correctly
    - Add/verify integration tests:
      - `test_add_persists_to_disk` — add() then reload from disk → entry present
      - `test_remove_persists_to_disk` — remove() then reload → entry absent
      - `test_prune_persists_to_disk` — prune() then reload → only existing dirs remain
      - `test_add_rejects_nonexistent_path` — add() with non-existent path → no change, no file write
    - Verify old JSON files with `Z` suffix load correctly (backward compatibility)
  - Update test helpers:
    - Remove `default_with_path()` if no longer needed (or update for new struct)
    - Tests create temp dirs + write JSON files as before (persistence tests NEED filesystem)
  - Remove any remaining references to deleted functions (`epoch_to_iso`, etc.)
  - Run `cargo test --package fabrica` — verify all tests pass

  **Must NOT do**:
  - Do NOT remove tests that verify load/save behavior
  - Do NOT add tests for business logic that's already tested in list module
  - Do NOT change JSON file format in test fixtures

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Multiple test cases to migrate, verify, and augment — careful work
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - `gpui-test`: Not testing GPUI components
    - `tdd`: Writing tests after implementation

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T7)
  - **Parallel Group**: Wave 3 (after T7)
  - **Blocks**: T9
  - **Blocked By**: T7

  **References**:

  **Test References** (testing patterns to follow):
  - `crates/fabrica/src/recent_projects.rs:238-256` — `test_load_valid_json` — test structure to preserve
  - `crates/fabrica/src/recent_projects.rs:259-263` — `test_load_missing_file` — graceful degradation test
  - `crates/fabrica/src/recent_projects.rs:266-279` — `test_load_corrupted_json` — error handling test
  - `crates/fabrica/src/recent_projects.rs:282-311` — `test_save_load_roundtrip` — round-trip test pattern
  - `crates/fabrica/src/recent_projects.rs:314-329` — `test_empty_list_save_load` — empty state test

  **Why Each Reference Matters**:
  - Each test shows the exact pattern to replicate with the refactored adapter — same assertions, updated construction

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: All persistence tests pass
    Tool: Bash
    Preconditions: Tests migrated to adapter
    Steps:
      1. Run `cargo test --package fabrica`
      2. Assert 0 failures
      3. Count total tests across all modules — assert >= 12
    Expected Result: All tests pass, adequate coverage
    Failure Indicators: Any test failure
    Evidence: .sisyphus/evidence/task-8-persistence-tests.txt

  Scenario: Old JSON with Z suffix loads correctly
    Tool: Bash
    Preconditions: backward compatibility test exists
    Steps:
      1. Run `cargo test --package fabrica -- test_load_valid_json`
      2. Assert test passes (this test loads JSON with "Z" suffix timestamps)
    Expected Result: Old file format still loads
    Failure Indicators: Test failure
    Evidence: .sisyphus/evidence/task-8-backward-compat.txt

  Scenario: No references to deleted helpers
    Tool: Bash
    Preconditions: Migration complete
    Steps:
      1. Run `grep -rn "epoch_to_iso\|default_with_path" crates/fabrica/src/recent_projects.rs`
      2. Assert exit code 1 (no matches) OR assert only in test module if still used
    Expected Result: Deleted helpers removed
    Failure Indicators: References to deleted functions
    Evidence: .sisyphus/evidence/task-8-no-old-helpers.txt
  ```

  **Commit**: YES (groups with T7)
  - Message: `refactor(recent-projects): slim down to persistence adapter with migrated tests`
  - Files: `crates/fabrica/src/recent_projects.rs`
  - Pre-commit: `cargo test --package fabrica`

- [ ] 9. Final test audit + cleanup

  **What to do**:
  - Run `cargo test --package fabrica` and capture full output — record test count
  - Verify explicit test mapping from original 11 to new tests:

    | Original Test | New Location | Status |
    |---|---|---|
    | `test_load_valid_json` | `recent_projects.rs` (adapter) | Preserved |
    | `test_load_missing_file` | `recent_projects.rs` (adapter) | Preserved |
    | `test_load_corrupted_json` | `recent_projects.rs` (adapter) | Preserved |
    | `test_save_load_roundtrip` | `recent_projects.rs` (adapter) | Preserved |
    | `test_empty_list_save_load` | `recent_projects.rs` (adapter) | Preserved |
    | `test_add_new_project` | `recent_projects_list.rs` (pure) + adapter integration | Split |
    | `test_add_existing_project_dedup` | `recent_projects_list.rs` (pure) | Migrated |
    | `test_truncation_at_five` | `recent_projects_list.rs` (pure) | Migrated |
    | `test_remove_project` | `recent_projects_list.rs` (pure) + adapter integration | Split |
    | `test_prune_nonexistent_paths` | `recent_projects_list.rs` (pure) + adapter integration | Split |
    | `test_format_relative_time` | `time_utils.rs` | Migrated + rewritten |

  - Verify all 11 scenarios are covered (some are split into pure + adapter versions → more tests)
  - Run `cargo clippy --package fabrica -- -D warnings` — zero warnings
  - Grep for any remaining `std::time::` references in `crates/fabrica/src/` — should be zero
  - Grep for any remaining `days_to_ymd`, `is_leap`, `parse_iso_to_epoch` — should be zero
  - Verify `app.rs` has zero diff: `git diff crates/fabrica/src/app.rs` — empty
  - Verify `dashboard.rs` has only import change: `git diff crates/fabrica/src/dashboard.rs` — only import lines
  - Record final line counts for each module:
    - `time_utils.rs` (expected ~50-80 lines)
    - `recent_projects_list.rs` (expected ~100-130 lines with tests)
    - `recent_projects.rs` (expected ~100-150 lines with tests)

  **Must NOT do**:
  - Do NOT add new features or improvements beyond what's specified
  - Do NOT refactor beyond the plan scope

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Systematic verification across multiple files — careful audit work
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - All: This is verification work, not implementation

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T5, T6, T8)
  - **Parallel Group**: Wave 3 (final task)
  - **Blocks**: F1-F4 (Final Verification)
  - **Blocked By**: T5, T6, T8

  **References**:

  **Pattern References**:
  - All three new/restructured modules: `time_utils.rs`, `recent_projects_list.rs`, `recent_projects.rs`
  - Consumers: `app.rs`, `dashboard.rs`

  **Why Each Reference Matters**:
  - Need to verify the complete picture — all modules, all consumers, all tests

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Full test suite passes with adequate coverage
    Tool: Bash
    Preconditions: All refactoring complete
    Steps:
      1. Run `cargo test --package fabrica 2>&1`
      2. Assert 0 failures
      3. Assert test count >= 12 (covering all 11 original scenarios plus new edge cases)
    Expected Result: All tests pass, no regressions
    Failure Indicators: Any test failure
    Evidence: .sisyphus/evidence/task-9-full-tests.txt

  Scenario: Zero clippy warnings
    Tool: Bash
    Preconditions: All code complete
    Steps:
      1. Run `cargo clippy --package fabrica -- -D warnings 2>&1`
      2. Assert exit code 0
    Expected Result: Zero warnings
    Failure Indicators: Any clippy warning
    Evidence: .sisyphus/evidence/task-9-clippy.txt

  Scenario: No residual deleted code
    Tool: Bash
    Preconditions: Cleanup complete
    Steps:
      1. Run `grep -rn "days_to_ymd\|is_leap\|parse_iso_to_epoch\|epoch_to_iso\|SystemTime\|UNIX_EPOCH" crates/fabrica/src/`
      2. Assert exit code 1 (no matches anywhere in src/)
    Expected Result: All deleted code fully removed
    Failure Indicators: Any grep match
    Evidence: .sisyphus/evidence/task-9-no-residual.txt

  Scenario: app.rs has zero diff
    Tool: Bash
    Preconditions: All changes committed
    Steps:
      1. Run `git diff main -- crates/fabrica/src/app.rs`
      2. Assert empty output
    Expected Result: app.rs completely unchanged
    Failure Indicators: Any diff output
    Evidence: .sisyphus/evidence/task-9-app-diff.txt

  Scenario: dashboard.rs has only import change
    Tool: Bash
    Preconditions: All changes committed
    Steps:
      1. Run `git diff main -- crates/fabrica/src/dashboard.rs`
      2. Assert diff contains only lines starting with `-use` and `+use` (import changes)
      3. Assert no other lines changed
    Expected Result: Only import line modified
    Failure Indicators: Non-import lines in diff
    Evidence: .sisyphus/evidence/task-9-dashboard-diff.txt
  ```

  **Commit**: YES
  - Message: `test(recent-projects): audit and verify all test scenarios covered`
  - Files: Any remaining cleanup
  - Pre-commit: `cargo test --package fabrica && cargo clippy --package fabrica -- -D warnings`

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in `.sisyphus/evidence/`. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --package fabrica -- -D warnings` + `cargo test --package fabrica`. Review all changed files for: `as any`/transmute, empty catches, `println!` in prod, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real QA** — `unspecified-high`
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration: load old JSON file with `Z` timestamps, call all public methods, verify round-trip. Test edge cases: empty state, malformed timestamp, future timestamp. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (`git log --oneline`, `git diff main`). Verify 1:1 — everything in spec was built, nothing beyond spec was built. Check "Must NOT do" compliance. Detect cross-task contamination: Task N touching Task M's files. Flag unaccounted changes. Verify `app.rs` has ZERO diff. Verify `dashboard.rs` diff is only import line.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **T1**: `chore(deps): add time crate for ISO 8601 parsing and formatting` — `Cargo.toml`, `crates/fabrica/Cargo.toml`
- **T2+T3**: `refactor(recent-projects): extract time_utils module` — `time_utils.rs`, `recent_projects.rs`, `main.rs`
- **T4+T5**: `refactor(recent-projects): extract pure in-memory RecentProjectsList` — `recent_projects_list.rs`, `main.rs`
- **T6**: `refactor(dashboard): update format_relative_time import` — `dashboard.rs`
- **T7+T8**: `refactor(recent-projects): slim down to persistence adapter` — `recent_projects.rs`
- **T9**: `test(recent-projects): audit and verify all test scenarios covered` — test files

---

## Success Criteria

### Verification Commands
```bash
cargo test --package fabrica           # Expected: all tests pass, 0 failures
cargo clippy --package fabrica -- -D warnings  # Expected: 0 warnings
grep -r "SystemTime\|UNIX_EPOCH" crates/fabrica/src/  # Expected: 0 matches
git diff crates/fabrica/src/app.rs     # Expected: empty (zero changes)
git diff crates/fabrica/src/dashboard.rs  # Expected: only import line change
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass (≥12 tests covering ≥11 original scenarios)
- [ ] `time_utils.rs` exists with `format_relative_time` and `current_iso_timestamp`
- [ ] `recent_projects_list.rs` exists with pure in-memory `RecentProjectsList`
- [ ] `recent_projects.rs` is a thin adapter delegating to list + I/O
- [ ] Zero `SystemTime`/`UNIX_EPOCH` references in src/
- [ ] `app.rs` has zero diff
