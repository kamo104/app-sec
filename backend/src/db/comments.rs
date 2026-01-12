use anyhow::Result;
use const_format::formatcp;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use sqlx::{Executor, FromRow, SqlitePool};

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    pub comment_id: i64,
    pub post_id: i64,
    pub user_id: i64,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct CommentsTable {
    conn_pool: SqlitePool,
}

impl CommentsTable {
    pub const TABLE_NAME: &str = "comments";
    const TABLE_CREATION: &str = formatcp!(
        "CREATE TABLE IF NOT EXISTS {} (\
            comment_id INTEGER PRIMARY KEY AUTOINCREMENT, \
            post_id INTEGER NOT NULL REFERENCES posts(post_id) ON DELETE CASCADE, \
            user_id INTEGER NOT NULL REFERENCES user_login(user_id) ON DELETE CASCADE, \
            content TEXT NOT NULL, \
            created_at INTEGER NOT NULL, \
            deleted_at INTEGER\
        )",
        CommentsTable::TABLE_NAME,
    );

    pub(super) fn new(conn_pool: SqlitePool) -> Self {
        Self { conn_pool }
    }

    pub(super) async fn create_table(&self) -> Result<()> {
        self.conn_pool
            .execute(CommentsTable::TABLE_CREATION)
            .await?;
        Ok(())
    }

    pub async fn insert(&self, comment: &Comment) -> Result<i64> {
        let result = sqlx::query_scalar(formatcp!(
            "INSERT INTO {} (post_id, user_id, content, created_at) VALUES ($1, $2, $3, $4) RETURNING comment_id",
            CommentsTable::TABLE_NAME
        ))
        .bind(comment.post_id)
        .bind(comment.user_id)
        .bind(&comment.content)
        .bind(comment.created_at)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(result)
    }

    pub async fn get_by_id(&self, comment_id: i64) -> std::result::Result<Comment, sqlx::Error> {
        sqlx::query_as::<_, Comment>(formatcp!(
            "SELECT * FROM {} WHERE comment_id = $1",
            CommentsTable::TABLE_NAME
        ))
        .bind(comment_id)
        .fetch_one(&self.conn_pool)
        .await
    }

    pub async fn get_visible_by_post_id(
        &self,
        post_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Comment>> {
        let comments = sqlx::query_as::<_, Comment>(formatcp!(
            "SELECT * FROM {} WHERE post_id = $1 AND deleted_at IS NULL ORDER BY created_at ASC LIMIT $2 OFFSET $3",
            CommentsTable::TABLE_NAME
        ))
        .bind(post_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.conn_pool)
        .await?;
        Ok(comments)
    }

    pub async fn soft_delete(&self, comment_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET deleted_at = $1 WHERE comment_id = $2",
            CommentsTable::TABLE_NAME
        ))
        .bind(OffsetDateTime::now_utc())
        .bind(comment_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn restore(&self, comment_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET deleted_at = NULL WHERE comment_id = $1",
            CommentsTable::TABLE_NAME
        ))
        .bind(comment_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn hard_delete(&self, comment_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE comment_id = $1",
            CommentsTable::TABLE_NAME
        ))
        .bind(comment_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn is_owner(&self, comment_id: i64, user_id: i64) -> Result<bool> {
        let comment = self.get_by_id(comment_id).await?;
        Ok(comment.user_id == user_id)
    }

    pub async fn count_by_post_id(&self, post_id: i64) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(formatcp!(
            "SELECT COUNT(*) FROM {} WHERE post_id = $1 AND deleted_at IS NULL",
            CommentsTable::TABLE_NAME
        ))
        .bind(post_id)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(count)
    }
}
