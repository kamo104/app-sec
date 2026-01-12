use anyhow::Result;
use const_format::formatcp;
use sqlx::types::time::OffsetDateTime;
use sqlx::{Executor, FromRow, SqlitePool};

use super::user_login::UserLoginTable;

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

    pub(super) fn new(conn_pool: SqlitePool) -> Self {
        Self { conn_pool }
    }

    pub(super) async fn create_table(&self) -> Result<()> {
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
