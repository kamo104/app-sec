use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::error;

use super::auth_extractor::AuthenticatedUser;
use crate::db::{DBHandle, RATING_DOWNVOTE, RATING_UPVOTE, Rating};
use api_types::{
    AuthErrorResponse, RatePostRequest, RatingError, RatingErrorResponse, RatingResponse,
};

#[utoipa::path(
    post,
    path = "/api/posts/{post_id}/rate",
    params(
        ("post_id" = i64, Path, description = "Post ID")
    ),
    request_body = RatePostRequest,
    responses(
        (status = 200, description = "Rating updated", body = RatingResponse),
        (status = 400, description = "Invalid rating value", body = RatingErrorResponse),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse),
        (status = 404, description = "Post not found", body = RatingErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "ratings"
)]
pub async fn rate_post(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    Path(post_id): Path<i64>,
    Json(payload): Json<RatePostRequest>,
) -> impl IntoResponse {
    // Validate rating value
    if payload.value != RATING_UPVOTE && payload.value != RATING_DOWNVOTE {
        return (
            StatusCode::BAD_REQUEST,
            Json(RatingErrorResponse {
                error: RatingError::InvalidValue,
            }),
        )
            .into_response();
    }

    // Verify post exists
    if db.posts_table.get_visible_by_id(post_id).await.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(RatingErrorResponse {
                error: RatingError::PostNotFound,
            }),
        )
            .into_response();
    }

    let rating = Rating {
        rating_id: 0, // Will be set by DB
        post_id,
        user_id: auth.user.user_id,
        value: payload.value,
        created_at: OffsetDateTime::now_utc(),
    };

    match db.ratings_table.upsert(&rating).await {
        Ok(_) => {
            let score = db.ratings_table.get_score(post_id).await.unwrap_or(0);
            (
                StatusCode::OK,
                Json(RatingResponse {
                    score,
                    user_rating: Some(payload.value),
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to rate post: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/posts/{post_id}/rate",
    params(
        ("post_id" = i64, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Rating removed", body = RatingResponse),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse),
        (status = 404, description = "Post not found", body = RatingErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "ratings"
)]
pub async fn remove_rating(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    Path(post_id): Path<i64>,
) -> impl IntoResponse {
    // Verify post exists
    if db.posts_table.get_visible_by_id(post_id).await.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(RatingErrorResponse {
                error: RatingError::PostNotFound,
            }),
        )
            .into_response();
    }

    match db.ratings_table.delete(auth.user.user_id, post_id).await {
        Ok(_) => {
            let score = db.ratings_table.get_score(post_id).await.unwrap_or(0);
            (
                StatusCode::OK,
                Json(RatingResponse {
                    score,
                    user_rating: None,
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to remove rating: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
