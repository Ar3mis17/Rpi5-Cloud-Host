//! Configuration management

use dotenvy::dotenv;
use envy;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub jwt_secret: String,
    pub jwt_algorithm: String,
    pub access_token_expire_minutes: i64,
    pub refresh_token_expire_minutes: i64,
    pub upload_max_size_mb: i64,
    pub upload_chunk_size_mb: i64,
    pub max_concurrent_uploads: usize,
    pub default_disk_quota_gb: i64,
    pub storage_root: String,
    pub upload_temp_dir: String,
    pub rate_limit_requests_per_minute: u32,
    pub rate_limit_window_seconds: u64,
    pub cors_allow_origins: Vec<String>,
    pub session_cookie_secure: bool,
    pub session_cookie_httponly: bool,
    pub session_cookie_samesite: String,
    pub log_level: String,
    pub log_format: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv().ok();
        let config = envy::prefixed("AR3MIS_").from_env::<Config>()?;
        Ok(config)
    }

    pub fn database_url(&self) -> &str { &self.database_url }
    pub fn jwt_secret(&self) -> &[u8] { self.jwt_secret.as_bytes() }
    pub fn access_token_expire_secs(&self) -> i64 { self.access_token_expire_minutes * 60 }
    pub fn refresh_token_expire_secs(&self) -> i64 { self.refresh_token_expire_minutes * 60 }
    pub fn upload_max_size_bytes(&self) -> i64 { self.upload_max_size_mb * 1024 * 1024 }
    pub fn upload_chunk_size_bytes(&self) -> i64 { self.upload_chunk_size_mb * 1024 * 1024 }
    pub fn default_quota_bytes(&self) -> i64 { self.default_disk_quota_gb * 1024 * 1024 * 1024 }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| Config::from_env().expect("Failed to load config"))
}

pub fn init_config(config: Config) {
    CONFIG.set(config).expect("Config already initialized");
}
