use anyhow::Result;
use const_format::formatcp;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use sqlx::{Executor, FromRow, SqlitePool};

/// Rating value: +1 for upvote, -1 for downvote
pub const RATING_UPVOTE: i32 = 1;
pub const RATING_DOWNVOTE: i32 = -1;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct Rating {
    pub rating_id: i64,
    pub post_id: i64,
    pub user_id: i64,
    pub value: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct RatingsTable {
    conn_pool: SqlitePool,
}

impl RatingsTable {
    pub const TABLE_NAME: &str = "ratings";
    const TABLE_CREATION: &str = formatcp!(
        "CREATE TABLE IF NOT EXISTS {} (\
            rating_id INTEGER PRIMARY KEY AUTOINCREMENT, \
            post_id INTEGER NOT NULL REFERENCES posts(post_id) ON DELETE CASCADE, \
            user_id INTEGER NOT NULL REFERENCES user_login(user_id) ON DELETE CASCADE, \
            value INTEGER NOT NULL, \
            created_at INTEGER NOT NULL, \
            UNIQUE(post_id, user_id)\
        )",
        RatingsTable::TABLE_NAME,
    );

    pub(super) fn new(conn_pool: SqlitePool) -> Self {
        Self { conn_pool }
    }

    pub(super) async fn create_table(&self) -> Result<()> {
        self.conn_pool.execute(RatingsTable::TABLE_CREATION).await?;
        Ok(())
    }

    /// Insert or update a rating. Returns the rating_id.
    pub async fn upsert(&self, rating: &Rating) -> Result<i64> {
        let result = sqlx::query_scalar(formatcp!(
            "INSERT INTO {} (post_id, user_id, value, created_at) VALUES ($1, $2, $3, $4) \
            ON CONFLICT(post_id, user_id) DO UPDATE SET value = excluded.value \
            RETURNING rating_id",
            RatingsTable::TABLE_NAME
        ))
        .bind(rating.post_id)
        .bind(rating.user_id)
        .bind(rating.value)
        .bind(rating.created_at)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(result)
    }

    pub async fn get_by_user_and_post(
        &self,
        user_id: i64,
        post_id: i64,
    ) -> std::result::Result<Rating, sqlx::Error> {
        sqlx::query_as::<_, Rating>(formatcp!(
            "SELECT * FROM {} WHERE user_id = $1 AND post_id = $2",
            RatingsTable::TABLE_NAME
        ))
        .bind(user_id)
        .bind(post_id)
        .fetch_one(&self.conn_pool)
        .await
    }

    pub async fn delete(&self, user_id: i64, post_id: i64) -> Result<()> {
        sqlx::query(formatcp!(
            "DELETE FROM {} WHERE user_id = $1 AND post_id = $2",
            RatingsTable::TABLE_NAME
        ))
        .bind(user_id)
        .bind(post_id)
        .execute(&self.conn_pool)
        .await?;
        Ok(())
    }

    /// Get the net score for a post (sum of all rating values)
    pub async fn get_score(&self, post_id: i64) -> Result<i64> {
        let score: Option<i64> = sqlx::query_scalar(formatcp!(
            "SELECT SUM(value) FROM {} WHERE post_id = $1",
            RatingsTable::TABLE_NAME
        ))
        .bind(post_id)
        .fetch_one(&self.conn_pool)
        .await?;
        Ok(score.unwrap_or(0))
    }

    /// Get upvote and downvote counts separately
    pub async fn get_vote_counts(&self, post_id: i64) -> Result<(i64, i64)> {
        let upvotes: i64 = sqlx::query_scalar(formatcp!(
            "SELECT COUNT(*) FROM {} WHERE post_id = $1 AND value > 0",
            RatingsTable::TABLE_NAME
        ))
        .bind(post_id)
        .fetch_one(&self.conn_pool)
        .await?;

        let downvotes: i64 = sqlx::query_scalar(formatcp!(
            "SELECT COUNT(*) FROM {} WHERE post_id = $1 AND value < 0",
            RatingsTable::TABLE_NAME
        ))
        .bind(post_id)
        .fetch_one(&self.conn_pool)
        .await?;

        Ok((upvotes, downvotes))
    }

    /// Check if user has rated a post and what their rating is
    pub async fn get_user_rating(&self, user_id: i64, post_id: i64) -> Result<Option<i32>> {
        match self.get_by_user_and_post(user_id, post_id).await {
            Ok(rating) => Ok(Some(rating.value)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
