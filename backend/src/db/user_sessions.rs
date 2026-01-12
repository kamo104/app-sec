use anyhow::Result;
use const_format::formatcp;
use sqlx::types::time::OffsetDateTime;
use sqlx::{Executor, FromRow, SqlitePool};

use super::user_login::UserLoginTable;

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

    pub(super) fn new(conn_pool: SqlitePool) -> Self {
        Self { conn_pool }
    }

    pub(super) async fn create_table(&self) -> Result<()> {
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
