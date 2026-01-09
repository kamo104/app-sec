use anyhow::{Result, anyhow};
use const_format::formatcp;
use core::fmt::Write;
use keyring::Entry;
use rand_core::{OsRng, TryRngCore};
use serde::{Deserialize, Serialize};
use sqlx::Executor;
use sqlx::Row;
use sqlx::types::time::OffsetDateTime;
use sqlx::{FromRow, SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;
use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};

/* --- KEYRING constants --- */
const KEYRING_SERVICE_NAME: &str = "APPSEC_DB_KEY";
const KEYRING_USERNAME: &str = "APPSEC";
const KEYRING_DB_KEY_LEN: usize = 32; // SQLCipher uses an AES key under the hood, so it's bound to 32B anyway
/* --- KEYRING constants --- */

/* --- DEVELOPMENT MODE constants --- */
/// Development database path - separate from production to avoid conflicts
const DEV_DATABASE_PATH: &str = "data_dev.db";
/* --- DEVELOPMENT MODE constants --- */

/* --- EMAIL VERIFICATION and PASSWORD FORGET constants --- */
const CONFIRMATION_TOKEN_BYTES: usize = 32; // 32 bytes = 64 hex characters
/* --- EMAIL VERIFICATION and PASSWORD FORGET constants --- */

/* --- SESSION constants --- */
pub const SESSION_TOKEN_BYTES: usize = 32;
/* --- SESSION constants --- */

/* --- CLEANUP constants --- */
/// Interval in seconds for running expired sessions and tokens cleanup
/// Default: 3600 seconds (1 hour)
pub const CLEANUP_INTERVAL_SECONDS: u64 = 3600;
/* --- CLEANUP constants --- */

/// Generate a cryptographically secure random token
pub fn generate_verification_token() -> String {
    let mut token_bytes = [0u8; CONFIRMATION_TOKEN_BYTES];
    OsRng.try_fill_bytes(&mut token_bytes).unwrap();
    hex::encode(token_bytes)
}

/// Generate a cryptographically secure random session token
pub fn generate_session_token() -> String {
    let mut token_bytes = [0u8; SESSION_TOKEN_BYTES];
    OsRng.try_fill_bytes(&mut token_bytes).unwrap();
    hex::encode(token_bytes)
}

/// Generate a unique session ID for display/admin purposes
pub fn generate_session_id() -> String {
    let mut id_bytes = [0u8; 8];
    OsRng.try_fill_bytes(&mut id_bytes).unwrap();
    hex::encode(id_bytes)
}

/// Hash a token using SHA256 for deterministic lookup
pub fn hash_token(token: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

/* --- USER_SESSIONS_TABLE --- */
/* This table is used to store user sessions */
#[derive(FromRow, Clone, Debug)]
pub struct UserSession {
    pub user_id: i64,
    pub session_id: String,
    pub session_hash: String,
    pub session_expiry: OffsetDateTime,
    pub session_created_at: OffsetDateTime,
}
#[derive(Debug)]
pub struct UserSessionsTable {
    conn_pool: SqlitePool,
}
impl UserSessionsTable {
    pub const TABLE_NAME: &str = "user_sessions";
    const TABLE_CREATION: &str = formatcp!(
        "CREATE TABLE IF NOT EXISTS {} (\
            user_id INTEGER NOT NULL CONSTRAINT uname REFERENCES {} (user_id) ON DELETE CASCADE ON UPDATE CASCADE, \
            session_id TEXT NOT NULL, \
            session_hash TEXT PRIMARY KEY NOT NULL, \
            session_expiry INTEGER NOT NULL, \
            session_created_at INTEGER NOT NULL \
        )",
        UserSessionsTable::TABLE_NAME,
        UserLoginTable::TABLE_NAME,
    );
    async fn create_table(&self) -> Result<()> {
        self.conn_pool
            .execute(UserSessionsTable::TABLE_CREATION)
            .await?;
        Ok(())
    }
    pub async fn insert(&self, row: &UserSession) -> Result<()> {
        sqlx::query(formatcp!(
            "INSERT INTO {} (\
                user_id, \
                session_id, \
                session_hash, \
                session_expiry, \
                session_created_at\
            ) VALUES ($1, $2, $3, $4, $5)",
            UserSessionsTable::TABLE_NAME
        ))
        .bind(row.user_id)
        .bind(&row.session_id)
        .bind(&row.session_hash)
        .bind(row.session_expiry)
        .bind(row.session_created_at)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
    pub async fn delete(&self, user_id: i64, session_hash: &str) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE user_id = $1 AND session_hash = $2",
            UserSessionsTable::TABLE_NAME
        ))
        .bind(user_id)
        .bind(session_hash)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
    pub async fn get_by_hash(
        &self,
        session_hash: &str,
    ) -> std::result::Result<UserSession, sqlx::Error> {
        sqlx::query_as::<_, UserSession>(formatcp!(
            "SELECT * FROM {} WHERE session_hash = $1",
            UserSessionsTable::TABLE_NAME
        ))
        .bind(session_hash)
        .fetch_one(&self.conn_pool)
        .await
    }
    pub async fn delete_by_hash(&self, session_hash: &str) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE session_hash = $1",
            UserSessionsTable::TABLE_NAME
        ))
        .bind(session_hash)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
    pub async fn update_expiry(
        &self,
        session_hash: &str,
        new_expiry: OffsetDateTime,
    ) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET session_expiry = $1 WHERE session_hash = $2",
            UserSessionsTable::TABLE_NAME
        ))
        .bind(new_expiry)
        .bind(session_hash)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
    pub async fn cleanup_expired_sessions(&self) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE session_expiry < $1",
            UserSessionsTable::TABLE_NAME
        ))
            .bind(OffsetDateTime::now_utc())
            .execute(&self.conn_pool)
            .await?;
        Ok(())
    }
}
/* --- USER_SESSIONS_TABLE --- */

/* --- EMAIL_VERIFICATION_TOKENS_TABLE --- */
/* This table is used to store email verification tokens */
#[derive(FromRow, Clone, Debug)]
#[allow(dead_code)] // Fields used for token lifecycle management
pub struct EmailVerificationToken {
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(Debug)]
#[allow(dead_code)] // Email verification is implemented but cleanup method is not yet scheduled
pub struct EmailVerificationTokensTable {
    conn_pool: SqlitePool,
}

impl EmailVerificationTokensTable {
    pub const TABLE_NAME: &str = "email_verification_tokens";
    const TABLE_CREATION: &str = formatcp!(
        "CREATE TABLE IF NOT EXISTS {} (\
            user_id INTEGER NOT NULL CONSTRAINT fk_user REFERENCES {} (user_id) ON DELETE CASCADE ON UPDATE CASCADE, \
            token_hash TEXT NOT NULL, \
            expires_at INTEGER NOT NULL, \
            created_at INTEGER NOT NULL, \
            CONSTRAINT pkey PRIMARY KEY (user_id)\
        )",
        EmailVerificationTokensTable::TABLE_NAME,
        UserLoginTable::TABLE_NAME,
    );

    async fn create_table(&self) -> Result<()> {
        self.conn_pool
            .execute(EmailVerificationTokensTable::TABLE_CREATION)
            .await?;
        Ok(())
    }

    pub async fn insert(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: OffsetDateTime,
    ) -> Result<()> {
        sqlx::query(formatcp!(
            "INSERT INTO {} (\
                user_id, \
                token_hash, \
                expires_at, \
                created_at\
            ) VALUES ($1, $2, $3, $4) \
                ON CONFLICT(user_id) DO \
                UPDATE SET token_hash = excluded.token_hash, expires_at = excluded.expires_at, created_at = excluded.created_at",
            EmailVerificationTokensTable::TABLE_NAME
        ))
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(OffsetDateTime::now_utc())
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn get_by_token_hash(
        &self,
        token_hash: &str,
    ) -> std::result::Result<EmailVerificationToken, sqlx::Error> {
        let out = sqlx::query_as::<_, EmailVerificationToken>(formatcp!(
            "SELECT * FROM {} WHERE token_hash = $1",
            EmailVerificationTokensTable::TABLE_NAME
        ))
        .bind(token_hash)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(out)
    }

    pub async fn delete_by_user_id(&self, user_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE user_id = $1",
            EmailVerificationTokensTable::TABLE_NAME
        ))
        .bind(user_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)] // Cleanup method for scheduled maintenance
    pub async fn cleanup_expired_tokens(&self) -> Result<()> {
        // Delete unverified user accounts whose email verification tokens have expired
        sqlx::query(formatcp!(
            "DELETE FROM {}
             WHERE user_id IN (
                 SELECT user_id FROM {}
                 WHERE expires_at < $1
             ) AND email_verified = false",
            UserLoginTable::TABLE_NAME,
            EmailVerificationTokensTable::TABLE_NAME
        ))
        .bind(OffsetDateTime::now_utc())
        .execute(&self.conn_pool)
        .await?;

        // Delete expired email verification tokens
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE expires_at < $1",
            EmailVerificationTokensTable::TABLE_NAME
        ))
            .bind(OffsetDateTime::now_utc())
            .execute(&self.conn_pool)
            .await?;
        Ok(())
    }
}
/* --- EMAIL_VERIFICATION_TOKENS_TABLE --- */

/* --- PASSWORD_RESET_TOKENS_TABLE --- */
/* This table is used to store password reset tokens */
#[derive(FromRow, Clone, Debug)]
pub struct PasswordResetToken {
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct PasswordResetTokensTable {
    conn_pool: SqlitePool,
}

impl PasswordResetTokensTable {
    pub const TABLE_NAME: &str = "password_reset_tokens";
    const TABLE_CREATION: &str = formatcp!(
        "CREATE TABLE IF NOT EXISTS {} (\
            user_id INTEGER NOT NULL CONSTRAINT fk_user_pr REFERENCES {} (user_id) ON DELETE CASCADE ON UPDATE CASCADE, \
            token_hash TEXT NOT NULL, \
            expires_at INTEGER NOT NULL, \
            created_at INTEGER NOT NULL, \
            CONSTRAINT pkey PRIMARY KEY (user_id)\
        )",
        PasswordResetTokensTable::TABLE_NAME,
        UserLoginTable::TABLE_NAME,
    );

    async fn create_table(&self) -> Result<()> {
        self.conn_pool
            .execute(PasswordResetTokensTable::TABLE_CREATION)
            .await?;
        Ok(())
    }

    pub async fn insert(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: OffsetDateTime,
    ) -> Result<()> {
        sqlx::query(formatcp!(
            "INSERT INTO {} (\
                user_id, \
                token_hash, \
                expires_at, \
                created_at\
            ) VALUES ($1, $2, $3, $4) \
                ON CONFLICT(user_id) DO \
                UPDATE SET token_hash = excluded.token_hash, expires_at = excluded.expires_at, created_at = excluded.created_at",
            PasswordResetTokensTable::TABLE_NAME
        ))
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(OffsetDateTime::now_utc())
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn get_by_token_hash(
        &self,
        token_hash: &str,
    ) -> std::result::Result<PasswordResetToken, sqlx::Error> {
        sqlx::query_as::<_, PasswordResetToken>(formatcp!(
            "SELECT * FROM {} WHERE token_hash = $1",
            PasswordResetTokensTable::TABLE_NAME
        ))
        .bind(token_hash)
        .fetch_one(&self.conn_pool)
        .await
    }

    pub async fn delete_by_user_id(&self, user_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE user_id = $1",
            PasswordResetTokensTable::TABLE_NAME
        ))
        .bind(user_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn cleanup_expired_tokens(&self) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE expires_at < $1",
            PasswordResetTokensTable::TABLE_NAME
        ))
            .bind(OffsetDateTime::now_utc())
            .execute(&self.conn_pool)
            .await?;
        Ok(())
    }
}
/* --- PASSWORD_RESET_TOKENS_TABLE --- */

/* --- USER_LOGIN_TABLE --- */
/* This table is used to store login information */
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct UserLogin {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub password: Option<String>, // hash in the PHC string format, NULL on password reset
    pub email_verified: bool,     // Whether the email has been verified
    pub email_verified_at: Option<OffsetDateTime>, // When the email was verified
    pub counter: i64,
    pub password_reset: bool,
}

#[derive(Debug)]
pub struct UserLoginTable {
    conn_pool: SqlitePool,
}
impl UserLoginTable {
    pub const TABLE_NAME: &str = "user_login";
    const TABLE_CREATION: &str = formatcp!(
        "CREATE TABLE IF NOT EXISTS {} (\
            user_id INTEGER PRIMARY KEY AUTOINCREMENT, \
            username TEXT UNIQUE NOT NULL, \
            email TEXT UNIQUE NOT NULL, \
            password TEXT, \
            email_verified INTEGER NOT NULL DEFAULT 0, \
            email_verified_at INTEGER, \
            counter INTEGER NOT NULL DEFAULT 0, \
            password_reset INTEGER NOT NULL DEFAULT 0\
        )",
        UserLoginTable::TABLE_NAME,
    );
    async fn create_table(&self) -> Result<()> {
        self.conn_pool
            .execute(UserLoginTable::TABLE_CREATION)
            .await?;
        Ok(())
    }
    pub async fn set(&self, row: &UserLogin) -> Result<i64> {
        let result = sqlx::query(formatcp!(
            "INSERT INTO {} (\
                username, \
                email, \
                password, \
                email_verified, \
                email_verified_at, \
                counter, \
                password_reset\
            ) VALUES ($1, $2, $3, $4, $5, $6, $7) \
                ON CONFLICT(username) DO \
                UPDATE SET email = excluded.email, password = excluded.password, \
                email_verified = excluded.email_verified, email_verified_at = excluded.email_verified_at, \
                counter = excluded.counter, password_reset = excluded.password_reset \
                RETURNING user_id",
            UserLoginTable::TABLE_NAME
        ))
        .bind(&row.username)
        .bind(&row.email)
        .bind(&row.password)
        .bind(row.email_verified)
        .bind(row.email_verified_at)
        .bind(row.counter)
        .bind(row.password_reset)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(result.get("user_id"))
    }
    #[allow(dead_code)] // Future feature for email updates
    pub async fn set_email(&self, username: &str, new_email: &str) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET (email) = ($1) WHERE username = $2",
            UserLoginTable::TABLE_NAME
        ))
        .bind(new_email)
        .bind(username)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
    #[allow(dead_code)] // Future feature for password reset
    pub async fn reset_password(&self, username: &str) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET (password) = (NULL) WHERE username = $1",
            UserLoginTable::TABLE_NAME
        ))
        .bind(username)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
    #[allow(dead_code)] // Future feature for username changes
    pub async fn change_username(&self, username: &str, new_username: &str) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET (username) = ($1) WHERE username = $2",
            UserLoginTable::TABLE_NAME
        ))
        .bind(new_username)
        .bind(username)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
    pub async fn get_by_username(
        &self,
        username: &str,
    ) -> std::result::Result<UserLogin, sqlx::Error> {
        let out = sqlx::query_as::<_, UserLogin>(formatcp!(
            "SELECT * FROM {} WHERE username = $1",
            UserLoginTable::TABLE_NAME
        ))
        .bind(username)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(out)
    }

    pub async fn get_by_email(&self, email: &str) -> std::result::Result<UserLogin, sqlx::Error> {
        let out = sqlx::query_as::<_, UserLogin>(formatcp!(
            "SELECT * FROM {} WHERE email = $1",
            UserLoginTable::TABLE_NAME
        ))
        .bind(email)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(out)
    }

    #[allow(dead_code)]
    pub async fn get_user_id_by_username(
        &self,
        username: &str,
    ) -> std::result::Result<i64, sqlx::Error> {
        let row = sqlx::query(formatcp!(
            "SELECT user_id FROM {} WHERE username = $1",
            UserLoginTable::TABLE_NAME
        ))
        .bind(username)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(row.get("user_id"))
    }

    pub async fn get_by_user_id(
        &self,
        user_id: i64,
    ) -> std::result::Result<UserLogin, sqlx::Error> {
        let out = sqlx::query_as::<_, UserLogin>(formatcp!(
            "SELECT * FROM {} WHERE user_id = $1",
            UserLoginTable::TABLE_NAME
        ))
        .bind(user_id)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(out)
    }
    pub async fn is_password_correct(&self, username: &str, password: &str) -> Result<bool> {
        let ul = sqlx::query_as::<_, UserLogin>(formatcp!(
            "SELECT * FROM {} WHERE username = $1",
            UserLoginTable::TABLE_NAME
        ))
        .bind(username)
        .fetch_one(&self.conn_pool)
        .await?;

        let hash = match ul.password {
            Some(hash) => hash,
            None => return Err(anyhow!("The password has been reset")),
        };
        let hash = PasswordHash::new(hash.as_str())?;
        return match Argon2::default().verify_password(password.as_bytes(), &hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        };
    }
    #[allow(dead_code)]
    pub async fn set_password(&self, username: &str, password: &str) -> Result<()> {
        let hash = hash_password(password)?;
        sqlx::query(formatcp!(
            "UPDATE {} SET (password) = ($1) WHERE username = $2",
            UserLoginTable::TABLE_NAME
        ))
        .bind(hash)
        .bind(username)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn set_password_by_user_id(&self, user_id: i64, password: &str) -> Result<()> {
        let hash = hash_password(password)?;
        sqlx::query(formatcp!(
            "UPDATE {} SET password = $1 WHERE user_id = $2",
            UserLoginTable::TABLE_NAME
        ))
        .bind(hash)
        .bind(user_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
    pub async fn is_username_free(&self, username: &str) -> Result<bool> {
        return match self.get_by_username(username).await {
            Ok(_r) => Ok(false),
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(true),
                _ => Err(anyhow!(Box::new(e))),
            },
        };
    }

    pub async fn is_email_free(&self, email: &str) -> Result<bool> {
        return match self.get_by_email(email).await {
            Ok(_r) => Ok(false),
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(true),
                _ => Err(anyhow!(Box::new(e))),
            },
        };
    }

    pub async fn new_user(&self, user_login: &UserLogin) -> Result<i64> {
        return match self.is_username_free(user_login.username.as_str()).await? {
            true => {
                let user_id = self.set(user_login).await?;
                Ok(user_id)
            }
            false => Err(anyhow!("username already taken")),
        };
    }
    pub async fn delete(&self, username: &str) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE username = $1",
            UserLoginTable::TABLE_NAME
        ))
        .bind(&username)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn mark_email_verified(&self, user_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET email_verified = 1, email_verified_at = $1 WHERE user_id = $2",
            UserLoginTable::TABLE_NAME
        ))
        .bind(OffsetDateTime::now_utc())
        .bind(user_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn update_counter(&self, user_id: i64, new_counter: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET counter = $1 WHERE user_id = $2",
            UserLoginTable::TABLE_NAME
        ))
        .bind(new_counter)
        .bind(user_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn set_password_reset_flag(&self, user_id: i64, flag: bool) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET password_reset = $1 WHERE user_id = $2",
            UserLoginTable::TABLE_NAME
        ))
        .bind(flag)
        .bind(user_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)] // Future feature for checking verification status
    pub async fn is_email_verified(&self, username: &str) -> Result<bool> {
        let row = sqlx::query(formatcp!(
            "SELECT email_verified FROM {} WHERE username = $1",
            UserLoginTable::TABLE_NAME
        ))
        .bind(username)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(row.get::<bool, _>("email_verified"))
    }
}
/* --- USER_LOGIN_TABLE --- */
#[derive(Debug)]
pub struct DBHandle {
    pub user_login_table: UserLoginTable,
    pub user_sessions_table: UserSessionsTable,
    pub email_verification_tokens_table: EmailVerificationTokensTable,
    pub password_reset_tokens_table: PasswordResetTokensTable,
    pub is_dev: bool,
}
impl DBHandle {
    async fn create_tables(&self) -> Result<()> {
        self.user_login_table.create_table().await?;
        self.user_sessions_table.create_table().await?;
        self.email_verification_tokens_table.create_table().await?;
        self.password_reset_tokens_table.create_table().await?;
        Ok(())
    }
    pub async fn open(database_path: &str) -> Result<Arc<Self>> {
        DBHandle::open_with_opts(database_path, None, false).await
    }
    /// Open database in development mode
    /// Uses a static key and separate database file to avoid keyring prompts during development
    pub async fn open_dev() -> Result<Arc<Self>> {
        // Use development database path and static key
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
            // let mut s = String::with_capacity(KEYRING_DB_KEY_LEN);
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
            // internaly, a connection pool is an Arc so it's ok to clone here
            // also: using a multi-threaded db access isn't recommended by sqlite but they are thread safe anyways
            user_login_table: UserLoginTable {
                conn_pool: conn_pool.clone(),
            },
            user_sessions_table: UserSessionsTable {
                conn_pool: conn_pool.clone(),
            },
            email_verification_tokens_table: EmailVerificationTokensTable {
                conn_pool: conn_pool.clone(),
            },
            password_reset_tokens_table: PasswordResetTokensTable {
                conn_pool: conn_pool.clone(),
            },
            is_dev,
        };

        db_handle.create_tables().await?;
        Ok(Arc::new(db_handle))
    }

    /// Run cleanup of all expired sessions and tokens
    /// This should be called periodically by the cleanup task
    pub async fn cleanup_expired_data(&self) -> Result<()> {
        // Clean up expired sessions
        if let Err(e) = self.user_sessions_table.cleanup_expired_sessions().await {
            tracing::error!("Failed to cleanup expired sessions: {:?}", e);
        }

        // Clean up expired email verification tokens
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

        // Clean up expired password reset tokens
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

    /// Start a background task that periodically cleans up expired sessions and tokens
    /// Returns a join handle that can be used to stop the task
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
