use anyhow::Result;
use const_format::formatcp;
use sqlx::types::time::OffsetDateTime;
use sqlx::{Executor, FromRow, SqlitePool};

use super::user_login::UserLoginTable;

#[derive(FromRow, Clone, Debug)]
#[allow(dead_code)]
pub struct EmailVerificationToken {
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(Debug)]
#[allow(dead_code)]
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

    pub(super) fn new(conn_pool: SqlitePool) -> Self {
        Self { conn_pool }
    }

    pub(super) async fn create_table(&self) -> Result<()> {
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

    #[allow(dead_code)]
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
