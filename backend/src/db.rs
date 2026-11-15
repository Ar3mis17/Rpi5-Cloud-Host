//! Database connection and migrations

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use crate::config::get_config;
use crate::errors::AppResult;

pub type DbPool = PgPool;

pub async fn create_pool() -> AppResult<DbPool> {
    let config = get_config();
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database pool created and migrations applied");
    Ok(pool)
}

pub async fn health_check(pool: &DbPool) -> bool {
    sqlx::query("SELECT 1").execute(pool).await.is_ok()
}
