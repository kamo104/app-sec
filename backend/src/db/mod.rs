use anyhow::{Result, anyhow};
use const_format::formatcp;
use core::fmt::Write;
use keyring::Entry;
use rand_core::{OsRng, TryRngCore};
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

pub type DBError = Box<dyn std::error::Error + Send + Sync>;
// pub type Result<T> = std::result::Result<T, DBError>;

/* --- KEYRING constants --- */
const KEYRING_SERVICE_NAME: &str = "FreeTrack_DB_KEY";
const KEYRING_USERNAME: &str = "freetrack";
const KEYRING_DB_KEY_LEN: usize = 32;
/* --- KEYRING constants --- */

/* --- USER_SESSIONS_TABLE --- */
/* This table is used to store user sessions */
#[derive(FromRow, Clone, Debug)]
pub struct UserSession {
    pub username: String,
    pub session_id: Vec<u8>,
    pub session_expiry: OffsetDateTime,
}
#[derive(Debug)]
pub struct UserSessionsTable {
    conn_pool: SqlitePool,
}
impl UserSessionsTable {
    pub const TABLE_NAME: &str = "user_sessions";
    const TABLE_CREATION: &str = formatcp!(
        "CREATE TABLE IF NOT EXISTS {} (\
            username TEXT NOT NULL CONSTRAINT uname REFERENCES {} (username) ON DELETE CASCADE ON UPDATE CASCADE, \
            session_id BLOB NOT NULL, \
            session_expiry INTEGER NOT NULL, \
            CONSTRAINT pkey PRIMARY KEY (username, session_id)\
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
                username, \
                session_id, \
                session_expiry\
            ) VALUES ($1, $2, $3)",
            UserSessionsTable::TABLE_NAME
        ))
        .bind(&row.username)
        .bind(&row.session_id)
        .bind(row.session_expiry)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
    pub async fn delete(&self, row: &UserSession) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE username = $1 AND session_id = $2",
            UserSessionsTable::TABLE_NAME
        ))
        .bind(&row.username)
        .bind(&row.session_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
    pub async fn cleanup_expired_sessions(&self) -> Result<()> {
        sqlx::query("DELETE FROM user_sessions WHERE session_expiry < $1")
            .bind(OffsetDateTime::now_utc())
            .execute(&self.conn_pool)
            .await?;
        Ok(())
    }
}
/* --- USER_SESSIONS_TABLE --- */

/* --- USER_LOGIN_TABLE --- */
/* This table is used to store login information */
#[derive(FromRow, Clone, Debug)]
pub struct UserLogin {
    pub username: String,
    pub email: String,
    pub password: Option<String>, // hash in the PHC string format, NULL on password reset
}

#[derive(Debug)]
pub struct UserLoginTable {
    conn_pool: SqlitePool,
}
impl UserLoginTable {
    pub const TABLE_NAME: &str = "user_login";
    const TABLE_CREATION: &str = formatcp!(
        "CREATE TABLE IF NOT EXISTS {} (\
            username TEXT PRIMARY KEY, \
            email TEXT NOT NULL, \
            password TEXT\
        )",
        UserLoginTable::TABLE_NAME,
    );
    async fn create_table(&self) -> Result<()> {
        self.conn_pool
            .execute(UserLoginTable::TABLE_CREATION)
            .await?;
        Ok(())
    }
    async fn set(&self, row: &UserLogin) -> Result<()> {
        sqlx::query(formatcp!(
            "INSERT INTO {} (\
                username, \
                email, \
                password\
            ) VALUES ($1, $2, $3) \
                ON CONFLICT(username) DO \
                UPDATE SET email = excluded.email, password = excluded.password",
            UserLoginTable::TABLE_NAME
        ))
        .bind(&row.username)
        .bind(&row.email)
        .bind(&row.password)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }
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
    pub async fn set_password(&self, username: &str, password: &str) -> Result<()> {
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
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
    pub async fn is_username_free(&self, username: &str) -> Result<bool> {
        return match self.get_by_username(username).await {
            Ok(_r) => Ok(false),
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(true),
                _ => Err(anyhow!(Box::new(e))),
            },
        };
    }
    pub async fn new_user(&self, user_login: &UserLogin) -> Result<()> {
        return match self.is_username_free(user_login.username.as_str()).await? {
            true => {
                self.set(user_login).await?;
                Ok(())
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
}
/* --- USER_LOGIN_TABLE --- */
#[derive(Debug)]
pub struct DBHandle {
    pub user_login_table: UserLoginTable,
    pub user_sessions_table: UserSessionsTable,
}
impl DBHandle {
    async fn create_tables(&self) -> Result<()> {
        self.user_login_table.create_table().await?;
        self.user_sessions_table.create_table().await?;
        Ok(())
    }
    pub async fn open(database_path: &str) -> Result<Arc<Self>> {
        DBHandle::open_with_opts(database_path, None).await
    }
    pub async fn open_with_opts(
        database_path: &str,
        opts_override: Option<SqliteConnectOptions>,
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
        };

        db_handle.create_tables().await?;
        Ok(Arc::new(db_handle))
    }
}
