use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use tauri::Manager;

use crate::error::{AppError, AppResult};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Initialise the SQLite connection pool and run pending migrations.
pub async fn init(app_handle: &tauri::AppHandle) -> AppResult<sqlx::SqlitePool> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Other(e.to_string()))?;

    tokio::fs::create_dir_all(&data_dir).await?;

    let db_path = data_dir.join("posto.db");

    let opts = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .synchronous(SqliteSynchronous::Normal);

    let pool = sqlx::SqlitePool::connect_with(opts).await?;

    MIGRATOR
        .run(&pool)
        .await
        .map_err(|e| AppError::Other(e.to_string()))?;

    Ok(pool)
}
