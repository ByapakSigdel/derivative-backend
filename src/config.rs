//! Configuration management for the Derivative backend.
//!
//! Loads configuration from environment variables with sensible defaults.

use std::env;
use std::time::Duration;

use once_cell::sync::Lazy;

/// Global configuration instance
pub static CONFIG: Lazy<Config> = Lazy::new(Config::from_env);

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection URL
    pub database_url: String,

    /// Database connection pool settings
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub database_connect_timeout: Duration,
    pub database_idle_timeout: Duration,

    /// JWT configuration
    pub jwt_secret: String,
    pub jwt_access_expiry: i64,
    pub jwt_refresh_expiry: i64,

    /// Server configuration
    pub server_host: String,
    pub server_port: u16,

    /// File upload configuration
    pub upload_dir: String,
    pub max_upload_size: usize,
    pub max_avatar_size: usize,
    pub max_asset_size: usize,

    /// CORS configuration
    pub cors_origin: String,

    /// Logging level
    pub rust_log: String,

    /// Rate limiting
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),

            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("DATABASE_MAX_CONNECTIONS must be a number"),

            database_min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .expect("DATABASE_MIN_CONNECTIONS must be a number"),

            database_connect_timeout: Duration::from_secs(
                env::var("DATABASE_CONNECT_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .expect("DATABASE_CONNECT_TIMEOUT must be a number"),
            ),

            database_idle_timeout: Duration::from_secs(
                env::var("DATABASE_IDLE_TIMEOUT")
                    .unwrap_or_else(|_| "600".to_string())
                    .parse()
                    .expect("DATABASE_IDLE_TIMEOUT must be a number"),
            ),

            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),

            // 7 days. The frontend doesn't have a refresh-on-401 flow yet,
            // so a short access token logs users out mid-session. Keep this
            // long until refresh is wired up.
            jwt_access_expiry: env::var("JWT_ACCESS_EXPIRY")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()
                .expect("JWT_ACCESS_EXPIRY must be a number"),

            // 30 days — refresh has to outlive access by a healthy margin.
            jwt_refresh_expiry: env::var("JWT_REFRESH_EXPIRY")
                .unwrap_or_else(|_| "2592000".to_string())
                .parse()
                .expect("JWT_REFRESH_EXPIRY must be a number"),

            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),

            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .expect("SERVER_PORT must be a number"),

            upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()),

            max_upload_size: env::var("MAX_UPLOAD_SIZE")
                .unwrap_or_else(|_| "10485760".to_string())
                .parse()
                .expect("MAX_UPLOAD_SIZE must be a number"),

            max_avatar_size: env::var("MAX_AVATAR_SIZE")
                .unwrap_or_else(|_| "5242880".to_string())
                .parse()
                .expect("MAX_AVATAR_SIZE must be a number"),

            max_asset_size: env::var("MAX_ASSET_SIZE")
                .unwrap_or_else(|_| "10485760".to_string())
                .parse()
                .expect("MAX_ASSET_SIZE must be a number"),

            cors_origin: env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),

            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),

            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .expect("RATE_LIMIT_REQUESTS must be a number"),

            rate_limit_window_secs: env::var("RATE_LIMIT_WINDOW_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .expect("RATE_LIMIT_WINDOW_SECS must be a number"),
        }
    }

    /// Get the server address string
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    /// Get the avatars upload directory
    pub fn avatars_dir(&self) -> String {
        format!("{}/avatars", self.upload_dir)
    }

    /// Get the project assets upload directory
    pub fn project_assets_dir(&self) -> String {
        format!("{}/project_assets", self.upload_dir)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}
