## Phase 1 Learnings


### T5: Database Initialization Module

- **sqlx 0.8 API**: `SqliteConnectOptions::from_str("sqlite:path")` + `.create_if_missing(true)` + `.journal_mode(SqliteJournalMode::Wal)` + `.foreign_keys(true)` + `SqlitePoolOptions::new().max_connections(1).connect_with(options).await?`
- **Rust 2024 edition**: `std::env::set_var`/`remove_var` require `unsafe` blocks with SAFETY comments
- **Test env var races**: Tests that modify env vars must be serialized via a `static Mutex` to avoid parallel test interference. Rust runs tests in parallel by default and env vars are process-global.
- **async_std::task::block_on** works for running async code in sync test functions. Available via `async-std` dev-dependency (sqlx's `runtime-async-std` pulls it in transitively).
- **tempfile crate** needed as dev-dependency for `tempdir()` — available transitively but must be declared explicitly.
- **`data_root()` pattern**: Matches `recent_projects.rs` — uses `$HOME/.fabrica` with `FABRICA_DATA_ROOT` env var override for testing.
- **`encode_project_path` absolutizes**: It calls `current_dir()` for relative paths, so test project paths become absolute under the temp dir.

### Guardrail: runtime-smol instead of runtime-async-std

- **Plan requirement**: Use `runtime-smol` feature for sqlx because GPUI already depends on smol 2.0, making it lighter by reusing existing transitive dep
- **SQLx version constraint**: sqlx 0.8 does NOT have a `runtime-smol` feature (only `_rt-smol` which doesn't exist)
- **Solution**: Upgraded to sqlx 0.9.0-alpha.1 which DOES have `runtime-smol` support
- **Change summary**:
  - `Cargo.toml`: Changed `"runtime-async-std"` → `"runtime-smol"` (and upgraded sqlx from 0.8 to 0.9.0-alpha.1)
  - `crates/fabrica/Cargo.toml`: Changed `async-std = "1"` → `smol = "2"`
  - `crates/fabrica/src/db/mod.rs`: Changed `async_std::task::block_on(...)` → `smol::block_on(...)`
- **Verification**: All 40 tests pass (12 db + path_encoding, 28 recent_projects, 4 time_utils in fabrica; 4 tests in ui)
- **No remaining references**: `grep -rn 'async.std\|async_std' crates/fabrica/` returns nothing
- **API compatibility**: `smol::block_on()` has the same signature as `async_std::task::block_on()` — it's a drop-in replacement
