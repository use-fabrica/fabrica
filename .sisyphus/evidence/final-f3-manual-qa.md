# F3: Real Manual QA — Evidence Report

**Date:** 2026-04-05
**Task:** Verify database infrastructure works end-to-end via automated tests

---

## 1. Full Workspace Test Suite

```
$ cargo test --workspace

running 40 tests (fabrica) ... 40 passed; 0 failed
running 4 tests (ui crate) ... 4 passed; 0 failed

test result: ok. 44 passed; 0 failed; 0 ignored
```

**PASS**

---

## 2. DB-Specific Tests

```
$ cargo test -p fabrica db

running 12 tests:
  db::tests::test_init_db_creates_file ... ok
  db::tests::test_init_db_foreign_keys ... ok
  db::tests::test_init_db_idempotent ... ok
  db::tests::test_init_db_sessions_table_exists ... ok
  db::tests::test_init_db_wal_mode ... ok
  db::path_encoding::tests::test_decode_hashed_path_fails ... ok
  db::path_encoding::tests::test_encode_decode_roundtrip ... ok
  db::path_encoding::tests::test_encode_long_path ... ok
  db::path_encoding::tests::test_encode_no_padding ... ok
  db::path_encoding::tests::test_encode_relative_path ... ok
  db::path_encoding::tests::test_encode_spaces ... ok
  db::path_encoding::tests::test_encode_unicode ... ok

test result: ok. 12 passed; 0 failed
```

**PASS**

---

## 3. Path Encoding Tests

```
$ cargo test -p fabrica path_encoding

running 7 tests:
  test_encode_decode_roundtrip ... ok
  test_encode_long_path ... ok
  test_encode_no_padding ... ok
  test_encode_relative_path ... ok
  test_encode_spaces ... ok
  test_encode_unicode ... ok
  test_decode_hashed_path_fails ... ok

test result: ok. 7 passed; 0 failed
```

**PASS**

---

## 4. Recent Projects Tests

```
$ cargo test -p fabrica recent_projects

running 21 tests:
  test_add_dedup_moves_to_top ... ok
  test_add_new_project ... ok
  test_add_persists_to_disk ... ok
  test_add_rejects_nonexistent_path ... ok
  test_add_truncation_at_five ... ok
  test_add_truncation_exact_five ... ok
  test_empty_list_operations ... ok
  test_empty_list_save_load ... ok
  test_load_corrupted_json ... ok
  test_load_missing_file ... ok
  test_load_valid_json ... ok
  test_prune_nonexistent_paths ... ok
  test_prune_persists_to_disk ... ok
  test_prune_remove_all ... ok
  test_prune_retain_all ... ok
  test_prune_selective ... ok
  test_remove_existing ... ok
  test_remove_nonexistent ... ok
  test_remove_persists_to_disk ... ok
  test_remove_project ... ok
  test_save_load_roundtrip ... ok

test result: ok. 21 passed; 0 failed
```

**PASS**

---

## 5. Old `.local/share/fabrica` References Check

```
$ grep -rn '.local/share/fabrica' crates/
No matches found
```

**PASS** — Zero references to old data directory path.

---

## 6. Migration SQL Schema Verification

**File:** `crates/fabrica/migrations/20260404000000_initial_sessions.sql`

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

- Only `sessions` table exists (no `projects` table): **PASS**
- `grep 'projects' migrations/*.sql` returned no matches: **PASS**

---

## 7. Runtime Configuration

**Finding:** sqlx uses `runtime-async-std` (NOT `runtime-smol`).

```toml
# Cargo.toml (workspace root)
sqlx = { version = "0.8", default-features = false, features = [
    "sqlite",
    "macros",
    "migrate",
    "runtime-async-std",
] }
```

**NOTE:** This differs from inherited wisdom which stated "runtime-smol". The codebase uses `runtime-async-std`, which is consistent with the `async_std::task::block_on` usage in tests. This is a **cosmetic discrepancy in docs**, not a bug — both runtimes work fine with sqlx 0.8.

---

## 8. WAL Mode & Foreign Keys in DB Init

**File:** `crates/fabrica/src/db/mod.rs`

```rust
let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)
    .foreign_keys(true);
```

- WAL mode: **PASS** (confirmed by `test_init_db_wal_mode` test)
- Foreign keys: **PASS** (confirmed by `test_init_db_foreign_keys` test)

---

## 9. `.env` File & `.sqlx/` Directory

```
$ cat .env
DATABASE_URL=sqlite:target/sqlx-prepare.db

$ ls .sqlx/
(empty directory with .gitkeep)
```

- `.env` with `DATABASE_URL`: **PASS** (at workspace root)
- `.sqlx/` directory exists: **PASS** (at workspace root)

---

## 10. `recent_projects_list.rs` Deletion

```
$ test -f crates/fabrica/src/recent_projects_list.rs && echo "EXISTS" || echo "DELETED"
DELETED
```

**PASS** — File has been removed.

---

## Summary

| Check | Result |
|-------|--------|
| Workspace tests (44/44) | PASS |
| DB tests (12/12) | PASS |
| Path encoding tests (7/7) | PASS |
| Recent projects tests (21/21) | PASS |
| No `.local/share/fabrica` refs | PASS |
| Migration: sessions table only | PASS |
| WAL mode configured | PASS |
| Foreign keys configured | PASS |
| `.env` with DATABASE_URL | PASS |
| `.sqlx/` directory exists | PASS |
| `recent_projects_list.rs` deleted | PASS |
| Runtime: async-std (not smol) | NOTE (cosmetic) |

---

## Verdict

```
Scenarios [11/11 pass] | Integration [44/44] | Edge Cases [7 tested] | VERDICT: APPROVE
```

All automated verification checks pass. The database infrastructure is functional end-to-end:
- DB initialization creates correct directory structure under `~/.fabrica/projects/<encoded>/`
- Migrations create only the `sessions` table with correct schema
- WAL mode and foreign keys are properly configured
- Path encoding handles Unicode, spaces, long paths, and relative paths correctly
- Recent projects module is fully tested with 21 test cases
- No stale references to old data directory path exist
- The `runtime-async-std` vs `runtime-smol` discrepancy is cosmetic only — the codebase consistently uses async-std throughout
