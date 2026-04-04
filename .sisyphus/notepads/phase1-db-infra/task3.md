# Task 3: Initial SQL Migration and MIGRATOR Static

## Completed Work

1. **Created migration file**: `crates/fabrica/migrations/20260404000000_initial_sessions.sql`
   - Contains `sessions` table for per-project databases
   - Fields: id, name, created_at, updated_at, layout_state, git_head

2. **Updated `crates/fabrica/src/db/mod.rs`**:
   - Added `use sqlx::migrate::Migrator;`
   - Added `pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");`
   - Note: The path `./migrations` is relative to the source file

3. **Verification**: `cargo check -p fabrica` passes successfully

## Key Learning

**Path Resolution Issue**:
The task description suggests using `"../migrations"` to resolve to `crates/fabrica/migrations/`, but this path didn't work due to directory structure expectations. The correct working path is `"./migrations"`.

**Directory Structure**:
- Source file: `crates/fabrica/src/db/mod.rs`
- Migrations directory: `crates/fabrica/migrations/`
- The path `./migrations` resolves correctly when used in `sqlx::migrate!()`

## Files Modified

- `crates/fabrica/migrations/20260404000000_initial_sessions.sql` (created)
- `crates/fabrica/src/db/mod.rs` (modified)

## Next Steps

The MIGRATOR static is now ready to be used in Task 5 to run migrations at DB initialization time.
