use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::error;

use super::auth_extractor::AuthenticatedUser;
use crate::db::{Comment, DBHandle};
use api_types::{
    AuthErrorResponse, CommentError, CommentErrorResponse, CommentListResponse, CommentResponse,
    CreateCommentRequest, CreateCommentResponse, PaginationQuery, ValidationErrorData,
};

#[utoipa::path(
    get,
    path = "/api/posts/{post_id}/comments",
    params(
        ("post_id" = i64, Path, description = "Post ID"),
        ("limit" = Option<i64>, Query, description = "Number of comments to return (default 20)"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination (default 0)")
    ),
    responses(
        (status = 200, description = "List of comments", body = CommentListResponse),
        (status = 404, description = "Post not found", body = CommentErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "comments"
)]
pub async fn list_comments(
    State(db): State<Arc<DBHandle>>,
    Path(post_id): Path<i64>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    // Verify post exists
    if db.posts_table.get_visible_by_id(post_id).await.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(CommentErrorResponse {
                error: CommentError::PostNotFound,
                validation: None,
            }),
        )
            .into_response();
    }

    match db
        .comments_table
        .get_visible_by_post_id(post_id, pagination.limit, pagination.offset)
        .await
    {
        Ok(comments) => {
            let total = db
                .comments_table
                .count_by_post_id(post_id)
                .await
                .unwrap_or(0);
            let mut responses = Vec::with_capacity(comments.len());

            for comment in &comments {
                match db
                    .user_login_table
                    .get_by_user_id_include_deleted(comment.user_id)
                    .await
                {
                    Ok(user) => {
                        let is_user_deleted = user.deleted_at.is_some();

                        responses.push(CommentResponse {
                            comment_id: comment.comment_id,
                            post_id: comment.post_id,
                            user_id: comment.user_id,
                            username: user.username,
                            is_user_deleted,
                            content: comment.content.clone(),
                            created_at: comment.created_at.unix_timestamp(),
                        });
                    }
                    Err(e) => {
                        error!("Failed to get user for comment: {:?}", e);
                        continue;
                    }
                }
            }

            (
                StatusCode::OK,
                Json(CommentListResponse {
                    comments: responses,
                    total,
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to list comments: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/posts/{post_id}/comments",
    params(
        ("post_id" = i64, Path, description = "Post ID")
    ),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = CreateCommentResponse),
        (status = 400, description = "Validation error", body = CommentErrorResponse),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse),
        (status = 404, description = "Post not found", body = CommentErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "comments"
)]
pub async fn create_comment(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    Path(post_id): Path<i64>,
    Json(payload): Json<CreateCommentRequest>,
) -> impl IntoResponse {
    // Verify post exists
    if db.posts_table.get_visible_by_id(post_id).await.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(CommentErrorResponse {
                error: CommentError::PostNotFound,
                validation: None,
            }),
        )
            .into_response();
    }

    // Validate content
    let content_validation = field_validator::validate_comment_content(&payload.content);
    if !content_validation.is_valid() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CommentErrorResponse {
                error: CommentError::Validation,
                validation: Some(ValidationErrorData::from_errors(vec![content_validation])),
            }),
        )
            .into_response();
    }

    let comment = Comment {
        comment_id: 0, // Will be set by DB
        post_id,
        user_id: auth.user.user_id,
        content: payload.content,
        created_at: OffsetDateTime::now_utc(),
        deleted_at: None,
    };

    match db.comments_table.insert(&comment).await {
        Ok(comment_id) => (
            StatusCode::CREATED,
            Json(CreateCommentResponse { comment_id }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to create comment: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/comments/{comment_id}",
    params(
        ("comment_id" = i64, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Comment deleted"),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse),
        (status = 403, description = "Not authorized"),
        (status = 404, description = "Comment not found", body = CommentErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "comments"
)]
pub async fn delete_comment(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    Path(comment_id): Path<i64>,
) -> impl IntoResponse {
    // Get comment
    let comment = match db.comments_table.get_by_id(comment_id).await {
        Ok(c) => c,
        Err(sqlx::Error::RowNotFound) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CommentErrorResponse {
                    error: CommentError::NotFound,
                    validation: None,
                }),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to get comment: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Check if already deleted
    if comment.deleted_at.is_some() {
        return (
            StatusCode::NOT_FOUND,
            Json(CommentErrorResponse {
                error: CommentError::NotFound,
                validation: None,
            }),
        )
            .into_response();
    }

    // Check ownership (admins can also delete)
    if comment.user_id != auth.user.user_id && auth.user.role != api_types::UserRole::Admin {
        return StatusCode::FORBIDDEN.into_response();
    }

    match db.comments_table.soft_delete(comment_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            error!("Failed to delete comment: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
