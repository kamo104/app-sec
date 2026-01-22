use anyhow::Result;
use serde::Deserialize;
use std::net::Ipv4Addr;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub session: SessionConfig,
    pub token: TokenConfig,
    pub cleanup: CleanupConfig,
    pub mail: MailConfig,
    pub urls: UrlsConfig,
    pub tls: TlsConfig,
    pub security: SecurityConfig,
    pub admin: AdminConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdminConfig {
    /// Default admin username (created on first startup if no users exist)
    pub username: String,
    /// Default admin email
    pub email: String,
    /// Default admin password
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub prod_path: String,
    pub dev_path: String,
    pub keyring_service_name: String,
    pub keyring_username: String,
    pub db_key_env_var_name: String,
    pub db_key_length: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub bind_addr: Ipv4Addr,
    pub port: u16,
    pub dev_mode: bool,
    pub max_body_size: usize,
    pub uploads_folder: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SessionConfig {
    pub duration_days: i64,
    pub token_bytes: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TokenConfig {
    pub email_verification_duration_hours: i64,
    pub password_reset_duration_hours: i64,
    pub confirmation_bytes: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CleanupConfig {
    pub interval_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub from_email: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UrlsConfig {
    pub base_url_dev: String,
    pub base_url_prod: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TlsConfig {
    /// Enable TLS (HTTPS) - should be true in production
    pub enabled: bool,
    /// Path to the certificate file (PEM format)
    pub cert_path: String,
    /// Path to the private key file (PEM format)
    pub key_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    /// Enable HSTS (HTTP Strict Transport Security)
    pub hsts_enabled: bool,
    /// HSTS max-age in seconds (default: 1 year = 31536000)
    pub hsts_max_age_seconds: u64,
    /// Include subdomains in HSTS
    pub hsts_include_subdomains: bool,
    /// Enable HSTS preload
    pub hsts_preload: bool,
    /// Allowed origins for CORS (comma-separated, empty = allow same origin only in prod)
    pub cors_allowed_origins: String,
}

pub fn load_config() -> Result<Config> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .add_source(config::Environment::with_prefix("APPSEC").separator("__"))
        .build()?;

    let config: Config = settings.try_deserialize()?;
    Ok(config)
}
