use anyhow::Result;
use const_format::formatcp;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use sqlx::{Executor, FromRow, SqlitePool};

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct Post {
    pub post_id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub image_path: String,
    pub created_at: OffsetDateTime,
    pub updated_at: Option<OffsetDateTime>,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct PostsTable {
    conn_pool: SqlitePool,
}

impl PostsTable {
    pub const TABLE_NAME: &str = "posts";
    const TABLE_CREATION: &str = formatcp!(
        "CREATE TABLE IF NOT EXISTS {} (\
            post_id INTEGER PRIMARY KEY AUTOINCREMENT, \
            user_id INTEGER NOT NULL REFERENCES user_login(user_id) ON DELETE CASCADE, \
            title TEXT NOT NULL, \
            description TEXT, \
            image_path TEXT NOT NULL, \
            created_at INTEGER NOT NULL, \
            updated_at INTEGER, \
            deleted_at INTEGER\
        )",
        PostsTable::TABLE_NAME,
    );

    pub(super) fn new(conn_pool: SqlitePool) -> Self {
        Self { conn_pool }
    }

    pub(super) async fn create_table(&self) -> Result<()> {
        self.conn_pool.execute(PostsTable::TABLE_CREATION).await?;
        Ok(())
    }

    pub async fn insert(&self, post: &Post) -> Result<i64> {
        let result = sqlx::query_scalar(formatcp!(
            "INSERT INTO {} (\
                user_id, title, description, image_path, created_at\
            ) VALUES ($1, $2, $3, $4, $5) RETURNING post_id",
            PostsTable::TABLE_NAME
        ))
        .bind(post.user_id)
        .bind(&post.title)
        .bind(&post.description)
        .bind(&post.image_path)
        .bind(post.created_at)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(result)
    }

    pub async fn get_by_id(&self, post_id: i64) -> std::result::Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>(formatcp!(
            "SELECT * FROM {} WHERE post_id = $1",
            PostsTable::TABLE_NAME
        ))
        .bind(post_id)
        .fetch_one(&self.conn_pool)
        .await
    }

    pub async fn get_visible_by_id(&self, post_id: i64) -> std::result::Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>(formatcp!(
            "SELECT * FROM {} WHERE post_id = $1 AND deleted_at IS NULL",
            PostsTable::TABLE_NAME
        ))
        .bind(post_id)
        .fetch_one(&self.conn_pool)
        .await
    }

    pub async fn get_all_visible(&self, limit: i64, offset: i64) -> Result<Vec<Post>> {
        let posts = sqlx::query_as::<_, Post>(formatcp!(
            "SELECT * FROM {} WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            PostsTable::TABLE_NAME
        ))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.conn_pool)
        .await?;
        Ok(posts)
    }

    pub async fn get_by_user_id(&self, user_id: i64, limit: i64, offset: i64) -> Result<Vec<Post>> {
        let posts = sqlx::query_as::<_, Post>(formatcp!(
            "SELECT * FROM {} WHERE user_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            PostsTable::TABLE_NAME
        ))
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.conn_pool)
        .await?;
        Ok(posts)
    }

    pub async fn search(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<Post>> {
        let search_pattern = format!("%{}%", query);
        let posts = sqlx::query_as::<_, Post>(formatcp!(
            "SELECT * FROM {} WHERE deleted_at IS NULL AND (title LIKE $1 OR description LIKE $1) ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            PostsTable::TABLE_NAME
        ))
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.conn_pool)
        .await?;
        Ok(posts)
    }

    pub async fn update(&self, post_id: i64, title: &str, description: Option<&str>) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET title = $1, description = $2, updated_at = $3 WHERE post_id = $4",
            PostsTable::TABLE_NAME
        ))
        .bind(title)
        .bind(description)
        .bind(OffsetDateTime::now_utc())
        .bind(post_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn soft_delete(&self, post_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET deleted_at = $1 WHERE post_id = $2",
            PostsTable::TABLE_NAME
        ))
        .bind(OffsetDateTime::now_utc())
        .bind(post_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn restore(&self, post_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "UPDATE {} SET deleted_at = NULL WHERE post_id = $1",
            PostsTable::TABLE_NAME
        ))
        .bind(post_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn get_all_deleted(&self, limit: i64, offset: i64) -> Result<Vec<Post>> {
        let posts = sqlx::query_as::<_, Post>(formatcp!(
            "SELECT * FROM {} WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC LIMIT $1 OFFSET $2",
            PostsTable::TABLE_NAME
        ))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.conn_pool)
        .await?;
        Ok(posts)
    }

    pub async fn hard_delete(&self, post_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE post_id = $1",
            PostsTable::TABLE_NAME
        ))
        .bind(post_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    pub async fn is_owner(&self, post_id: i64, user_id: i64) -> Result<bool> {
        let post = self.get_by_id(post_id).await?;
        Ok(post.user_id == user_id)
    }

    pub async fn count_visible(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(formatcp!(
            "SELECT COUNT(*) FROM {} WHERE deleted_at IS NULL",
            PostsTable::TABLE_NAME
        ))
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(count)
    }
}
