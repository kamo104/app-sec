use anyhow::Result;
use core::fmt::Write;
use keyring::Entry;
use rand_core::{OsRng, TryRngCore};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;
use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};

mod user_login;
mod user_sessions;
mod email_verification_tokens;
mod password_reset_tokens;
mod user_data;

pub use user_login::{UserLogin, UserLoginTable};
pub use user_sessions::{UserSession, UserSessionsTable};
pub use email_verification_tokens::EmailVerificationTokensTable;
pub use password_reset_tokens::PasswordResetTokensTable;
pub use user_data::UserDataTable;

const KEYRING_SERVICE_NAME: &str = "APPSEC_DB_KEY";
const KEYRING_USERNAME: &str = "APPSEC";
const KEYRING_DB_KEY_LEN: usize = 32;

const DEV_DATABASE_PATH: &str = "data_dev.db";

const CONFIRMATION_TOKEN_BYTES: usize = 32;

pub const SESSION_TOKEN_BYTES: usize = 32;

pub const CLEANUP_INTERVAL_SECONDS: u64 = 3600;

pub fn generate_verification_token() -> String {
    let mut token_bytes = [0u8; CONFIRMATION_TOKEN_BYTES];
    OsRng.try_fill_bytes(&mut token_bytes).unwrap();
    hex::encode(token_bytes)
}

pub fn generate_session_token() -> String {
    let mut token_bytes = [0u8; SESSION_TOKEN_BYTES];
    OsRng.try_fill_bytes(&mut token_bytes).unwrap();
    hex::encode(token_bytes)
}

pub fn generate_session_id() -> String {
    let mut id_bytes = [0u8; 8];
    OsRng.try_fill_bytes(&mut id_bytes).unwrap();
    hex::encode(id_bytes)
}

pub fn hash_token(token: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

#[derive(Debug)]
pub struct DBHandle {
    pub user_login_table: UserLoginTable,
    pub user_sessions_table: UserSessionsTable,
    pub email_verification_tokens_table: EmailVerificationTokensTable,
    pub password_reset_tokens_table: PasswordResetTokensTable,
    pub user_data_table: UserDataTable,
    pub is_dev: bool,
}

impl DBHandle {
    async fn create_tables(&self) -> Result<()> {
        self.user_login_table.create_table().await?;
        self.user_data_table.create_table().await?;
        self.user_sessions_table.create_table().await?;
        self.email_verification_tokens_table.create_table().await?;
        self.password_reset_tokens_table.create_table().await?;
        Ok(())
    }

    pub async fn open(database_path: &str) -> Result<Arc<Self>> {
        DBHandle::open_with_opts(database_path, None, false).await
    }

    pub async fn open_dev() -> Result<Arc<Self>> {
        let opts = SqliteConnectOptions::from_str(DEV_DATABASE_PATH)
            .unwrap()
            .pragma("foreign_keys", "ON")
            .create_if_missing(true);

        DBHandle::open_with_opts(DEV_DATABASE_PATH, Some(opts), true).await
    }

    pub async fn open_with_opts(
        database_path: &str,
        opts_override: Option<SqliteConnectOptions>,
        is_dev: bool,
    ) -> Result<Arc<Self>> {
        let mut secret_key = String::with_capacity(KEYRING_DB_KEY_LEN);
        if opts_override.is_none() {
            let keyring_entry = Entry::new(KEYRING_SERVICE_NAME, KEYRING_USERNAME)?;
            let keyring_secret = keyring_entry.get_secret()?;
            let secret = if keyring_secret.len() != KEYRING_DB_KEY_LEN {
                let mut key = vec![0u8; KEYRING_DB_KEY_LEN];
                OsRng.try_fill_bytes(&mut key).unwrap();
                keyring_entry.set_secret(&key)?;
                key
            } else {
                keyring_secret
            };
            for byte in secret {
                write!(secret_key, "{:02X}", byte)?;
            }
            secret_key = format!("\"x'{}'\"", secret_key);
        }
        let opts = match opts_override {
            Some(opts) => opts,
            None => SqliteConnectOptions::from_str(database_path)
                .unwrap()
                .pragma("key", secret_key.to_owned())
                .pragma("foreign_keys", "ON")
                .create_if_missing(true),
        };

        let conn_pool = SqlitePool::connect_with(opts).await?;
        let db_handle = Self {
            user_login_table: UserLoginTable::new(conn_pool.clone()),
            user_sessions_table: UserSessionsTable::new(conn_pool.clone()),
            email_verification_tokens_table: EmailVerificationTokensTable::new(conn_pool.clone()),
            password_reset_tokens_table: PasswordResetTokensTable::new(conn_pool.clone()),
            user_data_table: UserDataTable::new(conn_pool.clone()),
            is_dev,
        };

        db_handle.create_tables().await?;
        Ok(Arc::new(db_handle))
    }

    pub async fn cleanup_expired_data(&self) -> Result<()> {
        if let Err(e) = self.user_sessions_table.cleanup_expired_sessions().await {
            tracing::error!("Failed to cleanup expired sessions: {:?}", e);
        }

        if let Err(e) = self
            .email_verification_tokens_table
            .cleanup_expired_tokens()
            .await
        {
            tracing::error!(
                "Failed to cleanup expired email verification tokens: {:?}",
                e
            );
        }

        if let Err(e) = self
            .password_reset_tokens_table
            .cleanup_expired_tokens()
            .await
        {
            tracing::error!("Failed to cleanup expired password reset tokens: {:?}", e);
        }

        tracing::info!("Completed cleanup of expired sessions and tokens");
        Ok(())
    }

    pub fn start_cleanup_task(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let interval_duration = std::time::Duration::from_secs(CLEANUP_INTERVAL_SECONDS);
            tracing::info!(
                "Starting cleanup task with interval of {} seconds",
                CLEANUP_INTERVAL_SECONDS
            );

            loop {
                tokio::time::sleep(interval_duration).await;

                tracing::debug!("Running scheduled cleanup of expired data");
                if let Err(e) = self.cleanup_expired_data().await {
                    tracing::error!("Cleanup task failed: {:?}", e);
                }
            }
        })
    }
}
