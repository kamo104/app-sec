use anyhow::Result;
use const_format::formatcp;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::{Executor, FromRow, Row};

use super::user_login::UserLoginTable;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct UserData {
    pub user_id: i64,
    pub counter: i64,
}

#[derive(Debug)]
pub struct UserDataTable {
    conn_pool: SqlitePool,
}

impl UserDataTable {
    pub const TABLE_NAME: &str = "user_data";
    const TABLE_CREATION: &str = formatcp!(
        "CREATE TABLE IF NOT EXISTS {} (\
            user_id INTEGER PRIMARY KEY NOT NULL CONSTRAINT fk_user_data REFERENCES {} (user_id) ON DELETE CASCADE ON UPDATE CASCADE, \
            counter INTEGER NOT NULL DEFAULT 0\
        )",
        UserDataTable::TABLE_NAME,
        UserLoginTable::TABLE_NAME,
    );

    pub(super) fn new(conn_pool: SqlitePool) -> Self {
        Self { conn_pool }
    }

    pub(super) async fn create_table(&self) -> Result<()> {
        self.conn_pool
            .execute(UserDataTable::TABLE_CREATION)
            .await?;
        Ok(())
    }

    pub async fn insert(&self, user_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "INSERT INTO {} (user_id, counter) VALUES ($1, 0)",
            UserDataTable::TABLE_NAME
        ))
        .bind(user_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn get_by_user_id(&self, user_id: i64) -> std::result::Result<UserData, sqlx::Error> {
        sqlx::query_as::<_, UserData>(formatcp!(
            "SELECT * FROM {} WHERE user_id = $1",
            UserDataTable::TABLE_NAME
        ))
        .bind(user_id)
        .fetch_one(&self.conn_pool)
        .await
    }

    pub async fn update_counter(&self, user_id: i64, new_counter: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET counter = $1 WHERE user_id = $2",
            UserDataTable::TABLE_NAME
        ))
        .bind(new_counter)
        .bind(user_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn get_counter(&self, user_id: i64) -> Result<i64> {
        let row = sqlx::query(formatcp!(
            "SELECT counter FROM {} WHERE user_id = $1",
            UserDataTable::TABLE_NAME
        ))
        .bind(user_id)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(row.get("counter"))
    }
}
