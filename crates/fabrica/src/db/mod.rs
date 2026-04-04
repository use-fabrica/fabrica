pub mod path_encoding;

use sqlx::migrate::Migrator;

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Returns the data root directory for Fabrica (`~/.fabrica`).
///
/// Checks `FABRICA_DATA_ROOT` env var first (for testing), then falls back
/// to `$HOME/.fabrica`, or `.fabrica` if HOME is not set.
pub(crate) fn data_root() -> std::path::PathBuf {
    use std::path::PathBuf;

    if let Ok(custom) = std::env::var("FABRICA_DATA_ROOT") {
        return PathBuf::from(custom);
    }

    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".fabrica"))
        .unwrap_or_else(|_| PathBuf::from(".fabrica"))
}

/// Initializes a SQLite database pool for the given project path.
///
/// Creates the database directory under `~/.fabrica/projects/<encoded>/`,
/// connects with WAL mode and foreign keys enabled, runs migrations,
/// and returns a connection pool.
pub(crate) async fn init_db(project_path: &std::path::Path) -> anyhow::Result<sqlx::SqlitePool> {
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;

    let encoded = path_encoding::encode_project_path(project_path);
    let db_dir = data_root().join("projects").join(&encoded);
    std::fs::create_dir_all(&db_dir)?;

    let db_path = db_dir.join("fabrica.db");
    let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_data_root<F, Fut>(test: F)
    where
        F: FnOnce(std::path::PathBuf) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let data_root = temp.path().to_path_buf();
        // SAFETY: ENV_LOCK serializes env var access across test threads.
        unsafe { std::env::set_var("FABRICA_DATA_ROOT", &data_root) };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            async_std::task::block_on(test(data_root));
        }));
        // SAFETY: Same rationale as above.
        unsafe { std::env::remove_var("FABRICA_DATA_ROOT") };
        drop(temp);
        drop(_guard);
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    fn make_project(data_root: &std::path::Path, name: &str) -> std::path::PathBuf {
        let project = data_root.join(name);
        std::fs::create_dir_all(&project).unwrap();
        project
    }

    #[test]
    fn test_init_db_creates_file() {
        with_temp_data_root(|data_root| async move {
            let project = make_project(&data_root, "my-project");
            let pool = init_db(&project).await.unwrap();
            pool.close().await;

            let encoded = path_encoding::encode_project_path(&project);
            let db_dir = data_root.join("projects").join(&encoded);
            let db_path = db_dir.join("fabrica.db");
            assert!(db_dir.exists(), "DB directory should exist at {:?}", db_dir);
            assert!(db_path.exists(), "DB file should exist at {:?}", db_path);
        });
    }

    #[test]
    fn test_init_db_wal_mode() {
        with_temp_data_root(|data_root| async move {
            let project = make_project(&data_root, "wal-test");
            let pool = init_db(&project).await.unwrap();

            let row: (String,) = sqlx::query_as("PRAGMA journal_mode")
                .fetch_one(&pool)
                .await
                .unwrap();

            pool.close().await;
            assert_eq!(row.0, "wal", "journal_mode should be 'wal'");
        });
    }

    #[test]
    fn test_init_db_foreign_keys() {
        with_temp_data_root(|data_root| async move {
            let project = make_project(&data_root, "fk-test");
            let pool = init_db(&project).await.unwrap();

            let row: (i32,) = sqlx::query_as("PRAGMA foreign_keys")
                .fetch_one(&pool)
                .await
                .unwrap();

            pool.close().await;
            assert_eq!(row.0, 1, "foreign_keys should be 1");
        });
    }

    #[test]
    fn test_init_db_idempotent() {
        with_temp_data_root(|data_root| async move {
            let project = make_project(&data_root, "idempotent-test");

            let pool1 = init_db(&project).await.unwrap();
            pool1.close().await;

            let pool2 = init_db(&project).await.unwrap();
            pool2.close().await;
        });
    }

    #[test]
    fn test_init_db_sessions_table_exists() {
        with_temp_data_root(|data_root| async move {
            let project = make_project(&data_root, "sessions-test");
            let pool = init_db(&project).await.unwrap();

            let result = sqlx::query("SELECT COUNT(*) FROM sessions")
                .execute(&pool)
                .await;

            pool.close().await;
            assert!(result.is_ok(), "sessions table should be queryable");
        });
    }
}
