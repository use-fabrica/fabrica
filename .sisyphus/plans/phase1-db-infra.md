# Plan: Phase 1 — DB Infrastructure + Project Storage

## TL;DR

> **Quick Summary**: Add per-project SQLite database infrastructure to Fabrica. Each opened project gets its own DB at `~/.fabrica/projects/<base64url-encoded-path>/fabrica.db` with a `sessions` table. Dashboard stays on JSON for recent projects. Refactor `recent_projects.rs` to use `~/.fabrica/` with a cleaner API.
> 
> **Deliverables**:
> - sqlx + base64 + smol deps added with SQLX_OFFLINE ready
> - Path encoding utility (Base64URL encode/decode)
> - Migration with `sessions` table
> - DB initialization module (pool, WAL, migrations)
> - Refactored `recent_projects.rs` (merged files, clean API, new data dir)
> - DB wired into `open_project()` via `cx.spawn()`
> - All references to `~/.local/share/fabrica/` removed
> 
> **Estimated Effort**: Medium
> **Parallel Execution**: YES — 2 waves + final verification
> **Critical Path**: Task 1 → Task 2 → Task 5 → Task 6 → Task 7 → Final

---

## Context

### Original Request
Phase 1 of the "Local DB and Session Model" plan — add the database layer end-to-end so that opening a project creates a SQLite database under `~/.fabrica/` with the schema applied.

### Interview Summary
**Key Discussions (9 questions resolved via grill-me skill)**:
- Q1: SQLX_OFFLINE from day one (`.sqlx/` committed to repo)
- Q2: Dashboard stays with JSON — no global DB, no `projects` table
- Q3: Refactor `recent_projects.rs` in place (not delete), move to `~/.fabrica/`
- Q4: Sessions schema has no `project_path` column — DB IS the project
- Q5: Clean break — no migration from old `~/.local/share/fabrica/`
- Q6: Phase 1 proves plumbing only — no visible behavior change
- Q7: Temporary `db_pool: Option<SqlitePool>` on Fabrica struct
- Q8: Aggressive cleanup of `recent_projects.rs` — merge two files, new API, kill drain pattern
- Q9: Base64URL encoding on `PathBuf.as_os_str().as_bytes()`, no symlink resolution

**Test strategy**: TDD — every task follows RED-GREEN-REFACTOR using the `tdd` skill.

### Metis Review
**Critical corrections applied**:
- `runtime-async-std` → `runtime-smol` — GPUI already depends on `smol 2.0`, `runtime-smol` is lighter and reuses existing transitive dep
- Migrations placed at `crates/fabrica/migrations/` not repo root — `migrate!()` macro path is relative to source file
- WAL mode set via `SqliteConnectOptions` (not connection URL) with `connect_with()`
- Pool options must stay at defaults (no `max_lifetime`, `idle_timeout`, `min_connections`) to avoid `rt::spawn` calls

**Identified Gaps (addressed)**:
- Symlink canonicalization: Use `std::env::current_dir()` + path joining (no `canonicalize()`)
- Path encoding length: Hash fallback for paths >200 encoded chars (ext4 255-byte limit)
- DB init failure: Log error and continue, don't block project open

---

## Work Objectives

### Core Objective
Add SQLite infrastructure so that every project gets its own database with the `sessions` schema. Prove the plumbing works: DB file appears, migrations apply, pool is usable. No visible behavior change to the user.

### Concrete Deliverables
- `crates/fabrica/src/db/mod.rs` — DB init module
- `crates/fabrica/src/db/path_encoding.rs` — encode/decode project paths
- `crates/fabrica/migrations/20260404000000_initial_sessions.sql` — sessions table
- Refactored `crates/fabrica/src/recent_projects.rs` — merged, clean API, `~/.fabrica/`
- Deleted `crates/fabrica/src/recent_projects_list.rs`
- Updated `crates/fabrica/src/app.rs` — pool field, async init, new data dir

### Definition of Done
- [ ] `cargo build` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace` passes
- [ ] Opening a project creates `~/.fabrica/projects/<encoded>/fabrica.db`
- [ ] DB has WAL mode, FK enabled, `sessions` table
- [ ] Reopening same project doesn't re-run migrations

### Must Have
- Per-project SQLite DB created on project open
- Path encoding round-trips losslessly (including unicode, spaces, non-UTF8)
- WAL mode + foreign keys enabled
- `sqlx::migrate!()` embedded migrations applied automatically
- `recent_projects.rs` refactored with clean API at `~/.fabrica/`
- `~/.local/share/fabrica/` no longer referenced anywhere
- All existing tests pass (adapted to new API)
- TDD workflow for all new code

### Must NOT Have (Guardrails)
- NO `query!()` / `query_as!()` macros — Phase 1 uses only runtime APIs
- NO `projects` table in any database
- NO dashboard behavior changes — dashboard reads JSON as today
- NO `layout.rs` modifications
- NO session CRUD — table exists but no data written in Phase 1
- NO UUID generation or git integration
- NO global/shared DB
- NO `max_lifetime`, `idle_timeout`, or `min_connections` on pool options
- NO symlink resolution during path canonicalization
- NO visible user behavior changes
- NO async-std dependency (use runtime-smol instead)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: YES (existing tests in `recent_projects.rs`)
- **Automated tests**: YES (TDD — RED-GREEN-REFACTOR for every task)
- **Framework**: `cargo test` (built-in Rust test framework)
- **TDD**: Each task writes failing test first, then implements, then refactors

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

### TDD Skill Usage
Every task that involves writing code MUST use the `tdd` skill:
1. Load the `tdd` skill for RED-GREEN-REFACTOR guidance
2. Write the failing test first
3. Implement the minimum to make it pass
4. Refactor and clean up

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — can start immediately):
└── Task 1: Add dependencies + SQLX_OFFLINE setup [quick]

Wave 2 (Core modules — after Task 1, MAX PARALLEL):
├── Task 2: Path encoding utility (depends: 1) [unspecified-low]
├── Task 3: Migration file — sessions table (depends: 1) [quick]
└── Task 4: Refactor recent_projects.rs (depends: 1) [unspecified-high]

Wave 3 (Integration — after Wave 2):
├── Task 5: DB initialization module (depends: 2, 3) [unspecified-high]
├── Task 6: Wire DB into Fabrica + update data dir (depends: 4, 5) [unspecified-high]
└── Task 7: Cleanup — verify old refs gone (depends: 6) [quick]

Wave FINAL (After ALL tasks — 4 parallel reviews, then user okay):
├── F1: Plan compliance audit (oracle)
├── F2: Code quality review (unspecified-high)
├── F3: Real manual QA (unspecified-high)
└── F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 2 → Task 5 → Task 6 → Task 7 → F1-F4 → user okay
Max Concurrent: 3 (Wave 2)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1    | —         | 2,3,4  | 1    |
| 2    | 1         | 5      | 2    |
| 3    | 1         | 5      | 2    |
| 4    | 1         | 6      | 2    |
| 5    | 2, 3      | 6      | 3    |
| 6    | 4, 5      | 7      | 3    |
| 7    | 6         | F1-F4  | 3    |

### Agent Dispatch Summary

- **Wave 1**: **1** — T1 → `quick`
- **Wave 2**: **3** — T2 → `unspecified-low`, T3 → `quick`, T4 → `unspecified-high`
- **Wave 3**: **3** — T5 → `unspecified-high`, T6 → `unspecified-high`, T7 → `quick`
- **FINAL**: **4** — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 1. Add Dependencies + SQLX_OFFLINE Setup

  **What to do**:
  - Add to `crates/fabrica/Cargo.toml`:
    ```toml
    sqlx = { version = "0.8", default-features = false, features = [
        "sqlite",        # Bundled SQLite driver
        "macros",        # query!() macros + offline mode support (for Phase 2)
        "migrate",       # Migrator, migrate!() macro
        "runtime-smol",  # async_io::Timer — works with GPUI's executor (GPUI depends on smol 2.0)
    ] }
    base64 = "0.22"
    ```
  - Add to `Cargo.toml` (workspace root):
    ```toml
    [profile.dev.package.sqlx-macros]
    opt-level = 3
    ```
  - Create `.env` at repo root: `DATABASE_URL=sqlite:target/sqlx-prepare.db`
  - Create empty `.sqlx/` directory and commit it (for SQLX_OFFLINE)
  - Run `cargo build` to verify deps resolve

  **Must NOT do**:
  - Do NOT use `runtime-async-std` or `runtime-tokio`
  - Do NOT add `async-std`, `uuid`, `git2` or other deps

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: [`tdd`] — build passing is the "test"

  **Parallelization**:
  - **Can Run In Parallel**: NO — all others depend on this
  - **Blocks**: Tasks 2, 3, 4
  - **Blocked By**: None

  **References**:
  - `Cargo.toml` (workspace root) — workspace member config
  - `crates/fabrica/Cargo.toml` — current deps (anyhow, gpui, serde, time)

  **Acceptance Criteria**:
  - [ ] `cargo build` succeeds
  - [ ] `.env` exists with DATABASE_URL
  - [ ] `.sqlx/` directory exists
  - [ ] `runtime-smol` feature in Cargo.toml (NOT `runtime-async-std`)

  **QA Scenarios:**

  ```
  Scenario: Build succeeds with new deps
    Tool: Bash
    Steps:
      1. Run `cargo check -p fabrica` — assert exit code 0
      2. Run `grep -q 'runtime-smol' crates/fabrica/Cargo.toml` — assert exit 0
      3. Run `grep -q 'base64' crates/fabrica/Cargo.toml` — assert exit 0
      4. Run `grep 'runtime-async-std' crates/fabrica/Cargo.toml` — assert exit 1
    Evidence: .sisyphus/evidence/task-1-build-check.txt
  ```

  **Commit**: `feat(deps): add sqlx, base64 to workspace dependencies`

- [x] 2. Path Encoding Utility

  **What to do**:
  - Create `crates/fabrica/src/db/mod.rs` (module declaration, re-exports)
  - Create `crates/fabrica/src/db/path_encoding.rs` with:
    - `encode_project_path(path: &Path) -> String` — Base64URL no-pad, hash fallback for >200 chars
    - `decode_project_path(encoded: &str) -> Result<PathBuf>` — reverse, errors on hashed paths
    - `absolutize(path: &Path) -> PathBuf` — make relative paths absolute WITHOUT resolving symlinks
  - Add `mod db;` to `crates/fabrica/src/main.rs`
  - **TDD**: Write all 7 tests FIRST, then implement

  **Tests (RED first)**:
  - `test_encode_decode_roundtrip`, `test_encode_unicode`, `test_encode_spaces`
  - `test_encode_long_path` (hash fallback), `test_decode_hashed_path_fails`
  - `test_encode_relative_path`, `test_encode_no_padding`

  **Must NOT do**:
  - Do NOT use `std::fs::canonicalize()` (resolves symlinks)
  - Do NOT use `String::from_utf8` (use `OsString::from_vec`)
  - Do NOT add crypto deps (use `std::hash::DefaultHasher`)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: [`tdd`] — RED-GREEN-REFACTOR

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3, 4)
  - **Blocks**: Task 5
  - **Blocked By**: Task 1

  **References**:
  - `crates/fabrica/src/main.rs` — add `mod db;` alongside existing module declarations
  - `crates/fabrica/src/recent_projects.rs:8-17` — visibility pattern (`pub(crate)`)

  **Acceptance Criteria**:
  - [ ] `cargo test -p fabrica path_encoding` — 7 tests pass
  - [ ] Encoded strings have no `/`, `=`, `+` chars
  - [ ] Round-trip is lossless for short paths
  - [ ] Long paths (>200 encoded chars) use `h`-prefixed hash

  **QA Scenarios:**

  ```
  Scenario: Path encoding round-trip
    Tool: Bash
    Steps:
      1. Run `cargo test -p fabrica test_encode_decode_roundtrip` — assert exit 0
    Evidence: .sisyphus/evidence/task-2-roundtrip.txt

  Scenario: Long path hash fallback
    Tool: Bash
    Steps:
      1. Run `cargo test -p fabrica test_encode_long_path` — assert exit 0
      2. Run `cargo test -p fabrica test_decode_hashed_path_fails` — assert exit 0
    Evidence: .sisyphus/evidence/task-2-long-path.txt
  ```

  **Commit**: `feat(db): add base64url path encoding for project DB paths`

- [x] 3. Migration File — Sessions Table

  **What to do**:
  - Create `crates/fabrica/migrations/20260404000000_initial_sessions.sql`:
    ```sql
    CREATE TABLE IF NOT EXISTS sessions (
        id           TEXT PRIMARY KEY NOT NULL,
        name         TEXT NOT NULL,
        created_at   TEXT NOT NULL,
        updated_at   TEXT NOT NULL,
        layout_state TEXT,
        git_head     TEXT
    );
    ```
  - Add to `crates/fabrica/src/db/mod.rs`:
    ```rust
    pub static MIGRATOR: Migrator = sqlx::migrate!("../migrations");
    ```

  **Must NOT do**:
  - Do NOT create a `projects` table
  - Do NOT add FK constraints or indexes
  - Do NOT place migrations at repo root

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: [] — declarative SQL, no testable logic

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 2, 4)
  - **Blocks**: Task 5
  - **Blocked By**: Task 1

  **References**:
  - `crates/fabrica/src/db/mod.rs` — created in Task 2, extend here
  - `sqlx::migrate!()` path is relative to source file: `"../migrations"` from `src/db/`

  **Acceptance Criteria**:
  - [ ] Migration file exists at `crates/fabrica/migrations/`
  - [ ] `MIGRATOR` static compiles
  - [ ] `cargo check -p fabrica` passes

  **QA Scenarios:**

  ```
  Scenario: Migration compiles, sessions table only
    Tool: Bash
    Steps:
      1. Run `cargo check -p fabrica` — assert exit 0
      2. Run `test -f crates/fabrica/migrations/20260404000000_initial_sessions.sql` — assert exit 0
      3. Run `grep -q 'projects' crates/fabrica/migrations/*.sql` — assert exit 1 (not found)
    Evidence: .sisyphus/evidence/task-3-migration.txt
  ```

  **Commit**: `feat(db): add initial migration with sessions table`

- [x] 4. Refactor `recent_projects.rs`

  **What to do**:
  - **Rewrite** `crates/fabrica/src/recent_projects.rs`:
    - Merge `RecentProjectsList` logic inline (kill drain pattern)
    - New API: `load()` (no args), `add(&Path)`, `remove(&Path)`, `prune()`, `list() -> &[RecentProject]`
    - Private `save()`, `data_dir()` returns `~/.fabrica`
    - `load_from()` behind `#[cfg(test)]`
  - **Delete** `crates/fabrica/src/recent_projects_list.rs`
  - Remove `mod recent_projects_list;` from `main.rs`
  - Update all `dashboard.rs` call sites: `.projects` → `.list()`
  - **TDD**: Adapt existing 39 tests to new API first, then implement

  **Must NOT do**:
  - Do NOT delete `recent_projects.rs` — refactor in place
  - Do NOT change dashboard behavior
  - Do NOT add SQLite to recent_projects
  - Do NOT make `save()` public

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`tdd`] — adapt tests first (RED), refactor (GREEN), clean up

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 2, 3)
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - `crates/fabrica/src/recent_projects.rs` — full 375-line impl to rewrite
  - `crates/fabrica/src/recent_projects_list.rs` — 159-line list logic to inline
  - `crates/fabrica/src/main.rs:9-10` — module declarations to update
  - `crates/fabrica/src/app.rs:33-41` — data dir construction to replace with `RecentProjects::load()`
  - `crates/fabrica/src/dashboard.rs:37,77,124` — `.projects` → `.list()` call sites

  **Acceptance Criteria**:
  - [ ] `cargo test -p fabrica recent_projects` — >=25 tests pass
  - [ ] `recent_projects_list.rs` deleted
  - [ ] `load()` takes no args, `save()` is private
  - [ ] Data dir is `~/.fabrica/`
  - [ ] Dashboard uses `.list()` not `.projects`
  - [ ] No `.local/share/fabrica` references remain

  **QA Scenarios:**

  ```
  Scenario: Refactored module passes all tests
    Tool: Bash
    Steps:
      1. Run `cargo test -p fabrica recent_projects` — assert exit 0, >=25 tests
      2. Run `test ! -f crates/fabrica/src/recent_projects_list.rs` — assert exit 0
    Evidence: .sisyphus/evidence/task-4-tests.txt

  Scenario: Dashboard uses new API, old path gone
    Tool: Bash
    Steps:
      1. Run `grep -n '\.list()' crates/fabrica/src/dashboard.rs` — assert matches found
      2. Run `grep -rn '.local/share/fabrica' crates/fabrica/src/` — assert exit 1
    Evidence: .sisyphus/evidence/task-4-api.txt
  ```

  **Commit**: `refactor(recent-projects): merge files, clean API, move to ~/.fabrica/`

- [x] 5. Database Initialization Module

  **What to do**:
  - Add to `crates/fabrica/src/db/mod.rs`:
    - `data_root() -> PathBuf` — returns `~/.fabrica` or `.fabrica` fallback
    - `init_db(project_path: &Path) -> Result<SqlitePool>` — async function:
      1. Encode project path → directory name
      2. Create `~/.fabrica/projects/<encoded>/` dir
      3. Connect with `SqliteConnectOptions` (WAL mode, FK enabled)
      4. Pool: `max_connections(1)`, defaults for everything else
      5. Run `MIGRATOR.run(&pool).await`
      6. Return pool
  - **TDD**: Write 5 tests FIRST

  **Tests (RED first)**:
  - `test_init_db_creates_file` — DB file at expected path
  - `test_init_db_wal_mode` — PRAGMA journal_mode = "wal"
  - `test_init_db_foreign_keys` — PRAGMA foreign_keys = 1
  - `test_init_db_idempotent` — call twice, second succeeds
  - `test_init_db_sessions_table_exists` — SELECT from sessions works

  **Must NOT do**:
  - Do NOT use `SqlitePool::connect()` — use `connect_with()`
  - Do NOT set `max_lifetime`, `idle_timeout`, `min_connections`
  - Do NOT use `query!()` macros
  - Do NOT write data to sessions table

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`tdd`] — RED-GREEN-REFACTOR

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on Tasks 2, 3
  - **Blocks**: Task 6
  - **Blocked By**: Tasks 2, 3

  **References**:
  - `crates/fabrica/src/db/mod.rs` — created in Task 2 (path_encoding) and Task 3 (MIGRATOR)
  - `crates/fabrica/src/db/path_encoding.rs` — `encode_project_path()` from Task 2
  - `crates/fabrica/migrations/` — migration files from Task 3

  **Acceptance Criteria**:
  - [ ] `cargo test -p fabrica db` — 5 tests pass
  - [ ] DB file created at `~/.fabrica/projects/<encoded>/fabrica.db`
  - [ ] WAL mode confirmed via PRAGMA
  - [ ] Migrations are idempotent

  **QA Scenarios:**

  ```
  Scenario: DB init creates valid database
    Tool: Bash
    Steps:
      1. Run `cargo test -p fabrica db` — assert exit 0, 5 tests pass
    Evidence: .sisyphus/evidence/task-5-db-tests.txt

  Scenario: Pool configuration is correct
    Tool: Bash
    Steps:
      1. Run `cargo test -p fabrica test_init_db_wal_mode` — assert exit 0
      2. Run `cargo test -p fabrica test_init_db_foreign_keys` — assert exit 0
    Evidence: .sisyphus/evidence/task-5-pool-config.txt
  ```

  **Commit**: `feat(db): add database initialization module`

- [x] 6. Wire DB into Fabrica + Update Data Dir

  **What to do**:
  - Add to `Fabrica` struct in `app.rs`: `db_pool: Option<SqlitePool>`
  - Change `open_project()` to use `cx.spawn()`:
    ```rust
    cx.spawn(async move |this, cx| {
        match init_db(&path).await {
            Ok(pool) => { this.update(cx, |fabrica, cx| {
                fabrica.db_pool = Some(pool);
                fabrica.project_open = true;
                cx.notify();
            }); }
            Err(e) => eprintln!("Warning: DB init failed: {}", e),
        }
    }).detach();
    ```
  - Update `close_project()` to drop pool: `self.db_pool = None`
  - Update data dir construction in `app.rs` to use `db::data_root()`
  - Ensure CLI path in `main.rs` also triggers DB init

  **Must NOT do**:
  - Do NOT block the UI on DB failure — log and continue
  - Do NOT change dashboard behavior
  - Do NOT use `query!()` macros
  - Do NOT modify `layout.rs`

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`tdd`] — integration test for DB file creation

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on Tasks 4, 5
  - **Blocks**: Task 7
  - **Blocked By**: Tasks 4, 5

  **References**:
  - `crates/fabrica/src/app.rs:81-90` — current `open_project()` to modify
  - `crates/fabrica/src/app.rs:92-97` — current `close_project()` to modify
  - `crates/fabrica/src/app.rs:58-68` — existing `cx.spawn()` pattern to follow
  - `crates/fabrica/src/main.rs:32-36` — CLI path opening

  **Acceptance Criteria**:
  - [ ] `db_pool: Option<SqlitePool>` on Fabrica struct
  - [ ] `open_project()` uses `cx.spawn()` for async DB init
  - [ ] `close_project()` drops pool
  - [ ] DB init failure logged, doesn't crash
  - [ ] `cargo build` passes

  **QA Scenarios:**

  ```
  Scenario: App builds with DB wiring
    Tool: Bash
    Steps:
      1. Run `cargo build` — assert exit 0
      2. Run `grep -q 'db_pool' crates/fabrica/src/app.rs` — assert exit 0
      3. Run `grep -q 'cx.spawn' crates/fabrica/src/app.rs` — assert exit 0
      4. Run `grep -q 'init_db' crates/fabrica/src/app.rs` — assert exit 0
    Evidence: .sisyphus/evidence/task-6-wiring.txt
  ```

  **Commit**: `feat(app): wire per-project DB init into open_project`

- [x] 7. Final Cleanup — Verify Old References Gone

  **What to do**:
  - Search entire codebase for `~/.local/share/fabrica` — remove any remaining refs
  - Run full verification: `cargo build && cargo test --workspace && cargo clippy --workspace`

  **Must NOT do**:
  - Do NOT add new functionality
  - Do NOT modify `layout.rs` or dashboard behavior

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on Task 6
  - **Blocks**: F1-F4
  - **Blocked By**: Task 6

  **References**:
  - All files modified in Tasks 1-6 — verify consistency

  **Acceptance Criteria**:
  - [ ] `grep -r '.local/share/fabrica' crates/` returns nothing
  - [ ] `cargo build` succeeds
  - [ ] `cargo test --workspace` passes
  - [ ] `cargo clippy --workspace` clean

  **QA Scenarios:**

  ```
  Scenario: Full build and test suite passes
    Tool: Bash
    Steps:
      1. Run `cargo build` — assert exit 0
      2. Run `cargo test --workspace` — assert exit 0, all tests pass
      3. Run `cargo clippy --workspace` — assert exit 0, no warnings
      4. Run `grep -r '.local/share/fabrica' crates/` — assert exit 1
    Evidence: .sisyphus/evidence/task-7-full-verification.txt
  ```

  **Commit**: `chore: remove old data directory references`

---

## Final Verification Wave

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in `.sisyphus/evidence/`. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo build` + `cargo clippy --workspace` + `cargo test --workspace`. Review all changed files for: `unwrap()` in non-test code without error handling, empty catches, `println!`/`eprintln!` in prod, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [x] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state (`rm -rf ~/.fabrica/`). Run `cargo run`. Open a project via dashboard. Verify DB file appears at `~/.fabrica/projects/<encoded>/fabrica.db`. Close project (Escape). Open same project again. Verify migrations don't re-run. Open different project — verify separate DB created. Save evidence to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (`git log/diff`). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **After Task 1**: `feat(deps): add sqlx, base64 to workspace dependencies` — `Cargo.toml`, `Cargo.lock`, `.env`, `.sqlx/`
- **After Task 2**: `feat(db): add base64url path encoding for project DB paths` — `crates/fabrica/src/db/`
- **After Task 3**: `feat(db): add initial migration with sessions table` — `crates/fabrica/migrations/`
- **After Task 4**: `refactor(recent-projects): merge files, clean API, move to ~/.fabrica/` — `crates/fabrica/src/recent_projects.rs`, delete `recent_projects_list.rs`
- **After Task 5**: `feat(db): add database initialization module` — `crates/fabrica/src/db/mod.rs`
- **After Task 6**: `feat(app): wire per-project DB init into open_project` — `crates/fabrica/src/app.rs`
- **After Task 7**: `chore: remove old data directory references` — various files
- Pre-commit for all: `cargo test --workspace`

---

## Success Criteria

### Verification Commands
```bash
cargo build                              # Expected: success
cargo test --workspace                   # Expected: all tests pass
cargo clippy --workspace                 # Expected: no warnings
ls ~/.fabrica/projects/                  # Expected: base64url-encoded directories
sqlite3 ~/.fabrica/projects/*/fabrica.db ".schema sessions"  # Expected: sessions table
sqlite3 ~/.fabrica/projects/*/fabrica.db "PRAGMA journal_mode"  # Expected: wal
sqlite3 ~/.fabrica/projects/*/fabrica.db "PRAGMA foreign_keys"  # Expected: 1
grep -r ".local/share/fabrica" crates/   # Expected: no matches
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass
- [ ] TDD workflow followed (tests written first for each task)
- [ ] No `query!()` macros used
- [ ] No `projects` table in any database
- [ ] Dashboard behavior unchanged
- [ ] `layout.rs` untouched
