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

pub fn load_config() -> Result<Config> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .add_source(config::Environment::with_prefix("APPSEC").separator("__"))
        .build()?;

    let config: Config = settings.try_deserialize()?;
    Ok(config)
}
