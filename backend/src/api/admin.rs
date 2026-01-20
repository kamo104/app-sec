use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::error;

use super::auth_extractor::AdminUser;
use crate::db::DBHandle;
use api_types::{
    DeletedPostResponse, DeletedPostsListResponse, PaginationQuery, PostError, PostErrorResponse,
    UpdateUserRoleRequest, UserInfoResponse, UserListResponse,
};

#[utoipa::path(
    get,
    path = "/api/admin/users",
    responses(
        (status = 200, description = "List of users", body = UserListResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not authorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin"
)]
pub async fn list_users(State(db): State<Arc<DBHandle>>, _admin: AdminUser) -> impl IntoResponse {
    match db.user_login_table.get_all_users().await {
        Ok(users) => {
            let user_responses: Vec<UserInfoResponse> = users
                .into_iter()
                .map(|u| UserInfoResponse {
                    user_id: u.user_id,
                    username: u.username,
                    email: u.email,
                    role: u.role,
                    email_verified: u.email_verified,
                    is_deleted: u.deleted_at.is_some(),
                })
                .collect();

            (
                StatusCode::OK,
                Json(UserListResponse {
                    users: user_responses,
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to list users: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/admin/users/{user_id}/role",
    params(
        ("user_id" = i64, Path, description = "User ID")
    ),
    request_body = UpdateUserRoleRequest,
    responses(
        (status = 200, description = "Role updated"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not authorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin"
)]
pub async fn update_user_role(
    State(db): State<Arc<DBHandle>>,
    admin: AdminUser,
    Path(user_id): Path<i64>,
    Json(payload): Json<UpdateUserRoleRequest>,
) -> impl IntoResponse {
    // Prevent admin from changing their own role
    if user_id == admin.user.user_id {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Verify user exists and is not deleted
    if db.user_login_table.get_by_user_id(user_id).await.is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    match db.user_login_table.set_role(user_id, payload.role).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            error!("Failed to update user role: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/admin/users/{user_id}",
    params(
        ("user_id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not authorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin"
)]
pub async fn delete_user(
    State(db): State<Arc<DBHandle>>,
    admin: AdminUser,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    // Prevent admin from deleting themselves
    if user_id == admin.user.user_id {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Verify user exists and is not already deleted
    if db.user_login_table.get_by_user_id(user_id).await.is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    match db.user_login_table.delete_by_user_id(user_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            error!("Failed to delete user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/admin/posts/deleted",
    params(
        ("limit" = Option<i64>, Query, description = "Number of posts to return (default 20)"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination (default 0)")
    ),
    responses(
        (status = 200, description = "List of deleted posts", body = DeletedPostsListResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not authorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin"
)]
pub async fn list_deleted_posts(
    State(db): State<Arc<DBHandle>>,
    _admin: AdminUser,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    match db
        .posts_table
        .get_all_deleted(pagination.limit, pagination.offset)
        .await
    {
        Ok(posts) => {
            let mut responses = Vec::with_capacity(posts.len());

            for post in &posts {
                match db
                    .user_login_table
                    .get_by_user_id_include_deleted(post.user_id)
                    .await
                {
                    Ok(user) => {
                        if let Some(deleted_at) = post.deleted_at {
                            let is_user_deleted = user.deleted_at.is_some();

                            responses.push(DeletedPostResponse {
                                post_id: post.post_id,
                                user_id: post.user_id,
                                username: user.username,
                                is_user_deleted,
                                title: post.title.clone(),
                                deleted_at: deleted_at.unix_timestamp(),
                            });
                        }
                    }
                    Err(e) => {
                        error!("Failed to get user for post: {:?}", e);
                        continue;
                    }
                }
            }

            (
                StatusCode::OK,
                Json(DeletedPostsListResponse {
                    posts: responses,
                    total: posts.len() as i64,
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to list deleted posts: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/admin/posts/{post_id}/restore",
    params(
        ("post_id" = i64, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Post restored"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not authorized"),
        (status = 404, description = "Post not found", body = PostErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin"
)]
pub async fn restore_post(
    State(db): State<Arc<DBHandle>>,
    _admin: AdminUser,
    Path(post_id): Path<i64>,
) -> impl IntoResponse {
    // Verify post exists (including deleted ones)
    if db.posts_table.get_by_id(post_id).await.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(PostErrorResponse {
                error: PostError::NotFound,
                validation: None,
            }),
        )
            .into_response();
    }

    match db.posts_table.restore(post_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            error!("Failed to restore post: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
