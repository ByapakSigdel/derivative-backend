//! Derivative Backend API
//! 
//! Production-ready Rust backend for the Derivative visual programming platform.

use std::net::SocketAddr;
use std::time::Duration;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    middleware as axum_middleware,
    routing::get,
    Router,
};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeDir,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;
mod websocket;

use config::CONFIG;
use db::{check_health, create_pool, get_pool_stats, DbPool};
use errors::AppResult;
use routes::{
    admin_routes, auth_routes, protected_auth_routes, community_routes, 
    project_routes, user_routes, ws_routes, collaboration_routes,
    admin_metrics_routes, metrics_routes
};
use utils::file_storage::ensure_upload_dirs;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| CONFIG.rust_log.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Starting Derivative Backend API v{}", env!("CARGO_PKG_VERSION"));
    
    // Create database connection pool
    let pool = create_pool().await?;
    
    // Run migrations (optional - can be disabled in production)
    #[cfg(feature = "run-migrations")]
    db::run_migrations(&pool).await?;
    
    // Ensure upload directories exist
    ensure_upload_dirs().await?;
    
    // Build CORS layer with multiple origins support
    let allowed_origins: Vec<HeaderValue> = CONFIG
        .cors_origin
        .split(',')
        .filter_map(|origin| origin.trim().parse().ok())
        .collect();
    
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
            header::ACCESS_CONTROL_REQUEST_METHOD,
            header::ACCESS_CONTROL_REQUEST_HEADERS,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));
    
    // Build the application router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Auth routes (no auth required for login/refresh)
        .nest("/api/auth", auth_routes())
        
        // Protected auth routes (auth required for me/logout)
        .nest(
            "/api/auth",
            protected_auth_routes().layer(axum_middleware::from_fn_with_state(
                pool.clone(),
                middleware::require_auth,
            )),
        )
        
        // Protected routes
        .nest(
            "/api/user-projects",
            project_routes()
                .merge(community_routes())
                .layer(axum_middleware::from_fn_with_state(
                    pool.clone(),
                    middleware::require_auth,
                )),
        )
        
        // Collaboration routes (protected)
        .nest(
            "/api",
            collaboration_routes().layer(axum_middleware::from_fn_with_state(
                pool.clone(),
                middleware::require_auth,
            )),
        )
        
        // Metrics routes (protected, for logging)
        .nest(
            "/api/metrics",
            metrics_routes().layer(axum_middleware::from_fn_with_state(
                pool.clone(),
                middleware::require_auth,
            )),
        )
        
        // User routes (protected)
        .nest(
            "/api/users",
            user_routes().layer(axum_middleware::from_fn_with_state(
                pool.clone(),
                middleware::require_auth,
            )),
        )
        
        // Admin routes (protected + admin only)
        .nest(
            "/api/admin",
            admin_routes()
                .layer(axum_middleware::from_fn(middleware::require_admin))
                .layer(axum_middleware::from_fn_with_state(
                    pool.clone(),
                    middleware::require_auth,
                )),
        )
        
        // Admin metrics routes (protected + admin only)
        .nest(
            "/api/admin/metrics",
            admin_metrics_routes()
                .layer(axum_middleware::from_fn(middleware::require_admin))
                .layer(axum_middleware::from_fn_with_state(
                    pool.clone(),
                    middleware::require_auth,
                )),
        )
        
        // WebSocket routes
        .nest("/ws", ws_routes())
        
        // File uploads serving
        .nest_service(
            "/api/uploads",
            ServeDir::new(&CONFIG.upload_dir),
        )
        
        // Apply global middleware
        .layer(
            ServiceBuilder::new()
                // Logging
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                // Request timeout
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                // Body size limit
                .layer(DefaultBodyLimit::max(CONFIG.max_upload_size))
                // CORS
                .layer(cors),
        )
        .with_state(pool.clone());
    
    // Parse server address
    let addr: SocketAddr = CONFIG.server_addr().parse()?;
    
    info!("Server listening on {}", addr);
    info!("CORS origin: {}", CONFIG.cors_origin);
    info!("Upload directory: {}", CONFIG.upload_dir);
    
    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Start server with graceful shutdown
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    
    info!("Server shutdown complete");
    
    Ok(())
}

/// Health check endpoint
async fn health_check(
    axum::extract::State(pool): axum::extract::State<DbPool>,
) -> AppResult<axum::Json<HealthResponse>> {
    let db_healthy = check_health(&pool).await.unwrap_or(false);
    let pool_stats = get_pool_stats(&pool);
    
    Ok(axum::Json(HealthResponse {
        status: if db_healthy { "healthy" } else { "unhealthy" },
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: DatabaseHealth {
            connected: db_healthy,
            pool_size: pool_stats.size,
            pool_idle: pool_stats.idle,
            pool_active: pool_stats.active,
        },
    }))
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
    version: String,
    database: DatabaseHealth,
}

#[derive(serde::Serialize)]
struct DatabaseHealth {
    connected: bool,
    pool_size: u32,
    pool_idle: u32,
    pool_active: u32,
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down...");
        }
        _ = terminate => {
            info!("Received SIGTERM, shutting down...");
        }
    }
}
