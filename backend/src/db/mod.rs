use anyhow::Result;
use api_types::UserRole;
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

mod comments;
mod email_verification_tokens;
mod password_reset_tokens;
mod posts;
mod ratings;
mod user_data;
mod user_login;
mod user_sessions;

pub use comments::{Comment, CommentsTable};
pub use email_verification_tokens::EmailVerificationTokensTable;
pub use password_reset_tokens::PasswordResetTokensTable;
pub use posts::{Post, PostsTable};
pub use ratings::{RATING_DOWNVOTE, RATING_UPVOTE, Rating, RatingsTable};
pub use user_data::UserDataTable;
pub use user_login::{UserLogin, UserLoginTable};
pub use user_sessions::{UserSession, UserSessionsTable};

use crate::config::Config;

const CONFIRMATION_TOKEN_BYTES: usize = 32;

pub fn generate_verification_token() -> String {
    let mut token_bytes = [0u8; CONFIRMATION_TOKEN_BYTES];
    OsRng.try_fill_bytes(&mut token_bytes).unwrap();
    hex::encode(token_bytes)
}

pub fn generate_session_token(token_bytes: usize) -> String {
    let mut token_bytes_vec = vec![0u8; token_bytes];
    OsRng.try_fill_bytes(&mut token_bytes_vec).unwrap();
    hex::encode(token_bytes_vec)
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
    pub posts_table: PostsTable,
    pub comments_table: CommentsTable,
    pub ratings_table: RatingsTable,
    /// Whether TLS is enabled (used for cookie Secure flag)
    pub tls_enabled: bool,
}

impl DBHandle {
    async fn create_tables(&self) -> Result<()> {
        self.user_login_table.create_table().await?;
        self.user_data_table.create_table().await?;
        self.user_sessions_table.create_table().await?;
        self.email_verification_tokens_table.create_table().await?;
        self.password_reset_tokens_table.create_table().await?;
        self.posts_table.create_table().await?;
        self.comments_table.create_table().await?;
        self.ratings_table.create_table().await?;
        Ok(())
    }

    pub async fn open(config: &Config) -> Result<Arc<Self>> {
        let database_path = &config.database.path;
        let tls_enabled = config.tls.enabled;

        let opts = if config.database.encrypt {
            // Get encryption key from config or keyring
            let secret_key = Self::get_encryption_key(config).await?;

            SqliteConnectOptions::from_str(database_path)
                .unwrap()
                .pragma("key", secret_key)
                .pragma("foreign_keys", "ON")
                .create_if_missing(true)
        } else {
            // No encryption
            SqliteConnectOptions::from_str(database_path)
                .unwrap()
                .pragma("foreign_keys", "ON")
                .create_if_missing(true)
        };

        let conn_pool = SqlitePool::connect_with(opts).await?;
        let db_handle = Self {
            user_login_table: UserLoginTable::new(conn_pool.clone()),
            user_sessions_table: UserSessionsTable::new(conn_pool.clone()),
            email_verification_tokens_table: EmailVerificationTokensTable::new(conn_pool.clone()),
            password_reset_tokens_table: PasswordResetTokensTable::new(conn_pool.clone()),
            user_data_table: UserDataTable::new(conn_pool.clone()),
            posts_table: PostsTable::new(conn_pool.clone()),
            comments_table: CommentsTable::new(conn_pool.clone()),
            ratings_table: RatingsTable::new(conn_pool.clone()),
            tls_enabled,
        };

        db_handle.create_tables().await?;

        // Create default admin user if database is empty
        db_handle.create_default_admin_if_empty(config).await?;

        Ok(Arc::new(db_handle))
    }

    /// Get the database encryption key from config or system keyring.
    /// The key can be provided via:
    /// 1. config.toml: database.key = "..."
    /// 2. Environment variable: APPSEC__DATABASE__KEY=...
    /// 3. System keyring (fallback, auto-generates if not present)
    async fn get_encryption_key(config: &Config) -> Result<String> {
        let db_key_length = config.database.db_key_length;

        // Check if key is provided in config (includes env var override via config crate)
        if let Some(ref key) = config.database.key {
            if key.len() == db_key_length * 2 {
                // Key should be hex-encoded (64 chars for 32 bytes)
                return Ok(format!("\"x'{}'\"", key.to_uppercase()));
            } else if !key.is_empty() {
                return Err(anyhow::anyhow!(
                    "Invalid database key length: expected {} hex chars, got {}",
                    db_key_length * 2,
                    key.len()
                ));
            }
        }

        // Fall back to system keyring
        let keyring_service_name = config.database.keyring_service_name.clone();
        let keyring_username = config.database.keyring_username.clone();

        tokio::task::spawn_blocking(move || -> Result<String> {
            let keyring_entry = Entry::new(&keyring_service_name, &keyring_username)?;
            let keyring_secret = keyring_entry.get_secret()?;
            let secret = if keyring_secret.len() != db_key_length {
                let mut key = vec![0u8; db_key_length];
                OsRng.try_fill_bytes(&mut key).unwrap();
                keyring_entry.set_secret(&key)?;
                key
            } else {
                keyring_secret
            };
            let mut key_str = String::with_capacity(db_key_length * 2);
            for byte in secret {
                write!(key_str, "{:02X}", byte)?;
            }
            Ok(format!("\"x'{}'\"", key_str))
        })
        .await?
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

    pub fn start_cleanup_task(self: Arc<Self>, config: Config) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let interval_duration = std::time::Duration::from_secs(config.cleanup.interval_seconds);
            tracing::info!(
                "Starting cleanup task with interval of {} seconds",
                config.cleanup.interval_seconds
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

    /// Creates the default admin user if no users exist in the database.
    /// The admin credentials are loaded from the configuration file.
    async fn create_default_admin_if_empty(&self, config: &Config) -> Result<()> {
        let is_empty = self.user_login_table.is_empty().await?;

        if !is_empty {
            tracing::debug!("Database has existing users, skipping default admin creation");
            return Ok(());
        }

        tracing::info!("No users in database, creating default admin user");

        // Validate admin credentials from config
        let username_valid = field_validator::validate_username(&config.admin.username);
        let email_valid = field_validator::validate_email(&config.admin.email);
        let password_valid = field_validator::validate_password(&config.admin.password);

        if !username_valid.is_valid() {
            return Err(anyhow::anyhow!(
                "Invalid admin username in config: {:?}",
                username_valid
            ));
        }
        if !email_valid.is_valid() {
            return Err(anyhow::anyhow!(
                "Invalid admin email in config: {:?}",
                email_valid
            ));
        }
        if !password_valid.is_valid() {
            return Err(anyhow::anyhow!(
                "Invalid admin password in config: {:?}",
                password_valid
            ));
        }

        let hashed_password = hash_password(&config.admin.password)?;

        let admin_user = UserLogin {
            user_id: 0,
            username: config.admin.username.clone(),
            email: config.admin.email.clone(),
            password: Some(hashed_password),
            email_verified: true, // Admin email is pre-verified
            email_verified_at: Some(sqlx::types::time::OffsetDateTime::now_utc()),
            password_reset: false,
            role: UserRole::Admin,
            deleted_at: None,
        };

        let user_id = self.user_login_table.new_user(&admin_user).await?;
        tracing::info!(
            "Created default admin user '{}' with id {}",
            config.admin.username,
            user_id
        );

        // Create user_data record for admin
        self.user_data_table.insert(user_id).await?;

        Ok(())
    }
}
