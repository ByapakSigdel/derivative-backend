//! Database connection pool management.

use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

use crate::config::CONFIG;
use crate::errors::{AppError, AppResult};

/// Type alias for the database connection pool
pub type DbPool = PgPool;

/// Create a new database connection pool
pub async fn create_pool() -> AppResult<DbPool> {
    info!("Creating database connection pool...");
    
    let pool = PgPoolOptions::new()
        .max_connections(CONFIG.database_max_connections)
        .min_connections(CONFIG.database_min_connections)
        .acquire_timeout(CONFIG.database_connect_timeout)
        .idle_timeout(Some(CONFIG.database_idle_timeout))
        .connect(&CONFIG.database_url)
        .await
        .map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to create database pool: {}", e))
        })?;
    
    info!(
        "Database pool created with max_connections={}, min_connections={}",
        CONFIG.database_max_connections, CONFIG.database_min_connections
    );
    
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &DbPool) -> AppResult<()> {
    info!("Running database migrations...");
    
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to run migrations: {}", e)))?;
    
    info!("Database migrations completed successfully");
    
    Ok(())
}

/// Check database health
pub async fn check_health(pool: &DbPool) -> AppResult<bool> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    Ok(true)
}

/// Get pool statistics
#[derive(Debug, serde::Serialize)]
pub struct PoolStats {
    pub size: u32,
    pub idle: u32,
    pub active: u32,
}

pub fn get_pool_stats(pool: &DbPool) -> PoolStats {
    PoolStats {
        size: pool.size(),
        idle: pool.num_idle() as u32,
        active: (pool.size() - pool.num_idle() as u32),
    }
}
