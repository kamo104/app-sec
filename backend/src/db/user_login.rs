use anyhow::{Result, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use const_format::formatcp;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Row};
use sqlx::types::time::OffsetDateTime;
use sqlx::{FromRow, SqlitePool};

use super::hash_password;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct UserLogin {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub email_verified: bool,
    pub email_verified_at: Option<OffsetDateTime>,
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
            password_reset INTEGER NOT NULL DEFAULT 0\
        )",
        UserLoginTable::TABLE_NAME,
    );

    pub(super) fn new(conn_pool: SqlitePool) -> Self {
        Self { conn_pool }
    }

    pub(super) async fn create_table(&self) -> Result<()> {
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
                password_reset\
            ) VALUES ($1, $2, $3, $4, $5, $6) \
                ON CONFLICT(username) DO \
                UPDATE SET email = excluded.email, password = excluded.password, \
                email_verified = excluded.email_verified, email_verified_at = excluded.email_verified_at, \
                password_reset = excluded.password_reset \
                RETURNING user_id",
            UserLoginTable::TABLE_NAME
        ))
        .bind(&row.username)
        .bind(&row.email)
        .bind(&row.password)
        .bind(row.email_verified)
        .bind(row.email_verified_at)
        .bind(row.password_reset)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(result.get("user_id"))
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
