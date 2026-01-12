use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tower_cookies::Cookies;
use tracing::{debug, error};

use super::auth_extractor::AuthenticatedUser;
use crate::db::{DBHandle, Post, hash_token};
use api_types::{
    AuthErrorResponse, CreatePostMultipart, CreatePostResponse, PaginationQuery, PostError,
    PostErrorResponse, PostListResponse, PostResponse, SearchQuery, UpdatePostRequest,
    ValidationErrorData,
};

const UPLOADS_DIR: &str = "uploads";

/// Try to get user_id from session cookie without requiring authentication
async fn get_optional_user_id(db: &DBHandle, cookies: &Cookies) -> Option<i64> {
    let token = cookies.get("session_token")?;
    let hash = hash_token(token.value()).ok()?;
    let session = db.user_sessions_table.get_by_hash(&hash).await.ok()?;

    if session.session_expiry < OffsetDateTime::now_utc() {
        return None;
    }

    Some(session.user_id)
}

async fn build_post_response(
    db: &DBHandle,
    post: &Post,
    user_id: Option<i64>,
) -> Result<PostResponse, anyhow::Error> {
    let user = db.user_login_table.get_by_user_id(post.user_id).await?;
    let score = db.ratings_table.get_score(post.post_id).await?;
    let comment_count = db.comments_table.count_by_post_id(post.post_id).await?;
    let user_rating = match user_id {
        Some(uid) => db.ratings_table.get_user_rating(uid, post.post_id).await?,
        None => None,
    };

    Ok(PostResponse {
        post_id: post.post_id,
        user_id: post.user_id,
        username: user.username,
        title: post.title.clone(),
        description: post.description.clone(),
        image_url: format!("/api/posts/{}/image", post.post_id),
        score,
        comment_count,
        user_rating,
        created_at: post.created_at.unix_timestamp(),
        updated_at: post.updated_at.map(|t| t.unix_timestamp()),
    })
}

#[utoipa::path(
    get,
    path = "/api/posts",
    params(
        ("limit" = Option<i64>, Query, description = "Number of posts to return (default 20)"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination (default 0)")
    ),
    responses(
        (status = 200, description = "List of posts", body = PostListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "posts"
)]
pub async fn list_posts(
    State(db): State<Arc<DBHandle>>,
    cookies: Cookies,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    let user_id = get_optional_user_id(&db, &cookies).await;

    match db
        .posts_table
        .get_all_visible(pagination.limit, pagination.offset)
        .await
    {
        Ok(posts) => {
            let total = db.posts_table.count_visible().await.unwrap_or(0);
            let mut post_responses = Vec::with_capacity(posts.len());

            for post in &posts {
                match build_post_response(&db, post, user_id).await {
                    Ok(resp) => post_responses.push(resp),
                    Err(e) => {
                        error!("Failed to build post response: {:?}", e);
                        continue;
                    }
                }
            }

            (
                StatusCode::OK,
                Json(PostListResponse {
                    posts: post_responses,
                    total,
                    limit: pagination.limit,
                    offset: pagination.offset,
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to list posts: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/posts/search",
    params(
        ("q" = String, Query, description = "Search query"),
        ("limit" = Option<i64>, Query, description = "Number of posts to return (default 20)"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination (default 0)")
    ),
    responses(
        (status = 200, description = "Search results", body = PostListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "posts"
)]
pub async fn search_posts(
    State(db): State<Arc<DBHandle>>,
    cookies: Cookies,
    Query(search): Query<SearchQuery>,
) -> impl IntoResponse {
    let user_id = get_optional_user_id(&db, &cookies).await;

    match db
        .posts_table
        .search(&search.q, search.limit, search.offset)
        .await
    {
        Ok(posts) => {
            let mut post_responses = Vec::with_capacity(posts.len());

            for post in &posts {
                match build_post_response(&db, post, user_id).await {
                    Ok(resp) => post_responses.push(resp),
                    Err(e) => {
                        error!("Failed to build post response: {:?}", e);
                        continue;
                    }
                }
            }

            let total = post_responses.len() as i64;
            (
                StatusCode::OK,
                Json(PostListResponse {
                    posts: post_responses,
                    total,
                    limit: search.limit,
                    offset: search.offset,
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to search posts: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/posts/{post_id}",
    params(
        ("post_id" = i64, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Post details", body = PostResponse),
        (status = 404, description = "Post not found", body = PostErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "posts"
)]
pub async fn get_post(
    State(db): State<Arc<DBHandle>>,
    cookies: Cookies,
    Path(post_id): Path<i64>,
) -> impl IntoResponse {
    let user_id = get_optional_user_id(&db, &cookies).await;

    match db.posts_table.get_visible_by_id(post_id).await {
        Ok(post) => match build_post_response(&db, &post, user_id).await {
            Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
            Err(e) => {
                error!("Failed to build post response: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Err(sqlx::Error::RowNotFound) => (
            StatusCode::NOT_FOUND,
            Json(PostErrorResponse {
                error: PostError::NotFound,
                validation: None,
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to get post: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/posts",
    request_body(content_type = "multipart/form-data", content = inline(CreatePostMultipart)),
    responses(
        (status = 201, description = "Post created", body = CreatePostResponse),
        (status = 400, description = "Validation error or invalid image", body = PostErrorResponse),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "posts"
)]
pub async fn create_post(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Extract multipart data into CreatePostMultipart struct
    let mut post_data: Option<CreatePostMultipart> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "title" => {
                if let Ok(title) = field.text().await {
                    if post_data.is_none() {
                        post_data = Some(CreatePostMultipart {
                            title,
                            description: None,
                            image: Vec::new(),
                        });
                    } else if let Some(ref mut data) = post_data {
                        data.title = title;
                    }
                }
            }
            "description" => {
                if let Ok(description) = field.text().await {
                    if let Some(ref mut data) = post_data {
                        data.description = Some(description);
                    }
                }
            }
            "image" => {
                if let Ok(data) = field.bytes().await {
                    if let Some(ref mut post_data) = post_data {
                        post_data.image = data.to_vec();
                    }
                }
            }
            _ => {}
        }
    }

    // Validate the extracted data
    let post_data = match post_data {
        Some(data) => data,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PostErrorResponse {
                    error: PostError::Validation,
                    validation: Some(ValidationErrorData::from_errors(vec![
                        field_validator::validate_post_title(""),
                    ])),
                }),
            )
                .into_response();
        }
    };

    // Validate all fields using field-validator
    let mut validation_errors = Vec::new();

    // Validate title
    let title_validation = field_validator::validate_post_title(&post_data.title);
    if !title_validation.is_valid() {
        validation_errors.push(title_validation);
    }

    // Validate description if provided
    if let Some(ref desc) = post_data.description {
        let desc_validation = field_validator::validate_post_description(desc);
        if !desc_validation.is_valid() {
            validation_errors.push(desc_validation);
        }
    }

    // Return validation errors if any
    if !validation_errors.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(PostErrorResponse {
                error: PostError::Validation,
                validation: Some(ValidationErrorData::from_errors(validation_errors)),
            }),
        )
            .into_response();
    }

    // Validate image content and get extension from magic bytes
    if post_data.image.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(PostErrorResponse {
                error: PostError::InvalidImage,
                validation: None,
            }),
        )
            .into_response();
    }

    let image_ext = field_validator::validate_image_content_and_get_ext(&post_data.image);
    if image_ext.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(PostErrorResponse {
                error: PostError::InvalidImage,
                validation: None,
            }),
        )
            .into_response();
    }
    let image_ext = image_ext.unwrap();

    // Generate unique filename
    let filename = format!(
        "{}_{}.{}",
        auth.user.user_id,
        OffsetDateTime::now_utc().unix_timestamp_nanos(),
        image_ext
    );

    // Create uploads directory if it doesn't exist
    let uploads_path = std::path::Path::new(UPLOADS_DIR);
    if !uploads_path.exists() {
        if let Err(e) = std::fs::create_dir_all(uploads_path) {
            error!("Failed to create uploads directory: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    // Save image
    let file_path = uploads_path.join(&filename);
    if let Err(e) = std::fs::write(&file_path, &post_data.image) {
        error!("Failed to save image: {:?}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    debug!("Saved image to {:?}", file_path);

    let post = Post {
        post_id: 0, // Will be set by DB
        user_id: auth.user.user_id,
        title: post_data.title,
        description: post_data.description,
        image_path: filename,
        created_at: OffsetDateTime::now_utc(),
        updated_at: None,
        deleted_at: None,
    };

    match db.posts_table.insert(&post).await {
        Ok(post_id) => (StatusCode::CREATED, Json(CreatePostResponse { post_id })).into_response(),
        Err(e) => {
            error!("Failed to create post: {:?}", e);
            // Clean up uploaded file
            let _ = std::fs::remove_file(&file_path);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/posts/{post_id}",
    params(
        ("post_id" = i64, Path, description = "Post ID")
    ),
    request_body = UpdatePostRequest,
    responses(
        (status = 200, description = "Post updated"),
        (status = 400, description = "Validation error", body = PostErrorResponse),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse),
        (status = 403, description = "Not authorized"),
        (status = 404, description = "Post not found", body = PostErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "posts"
)]
pub async fn update_post(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    Path(post_id): Path<i64>,
    Json(payload): Json<UpdatePostRequest>,
) -> impl IntoResponse {
    // Check if post exists and user owns it
    let post = match db.posts_table.get_visible_by_id(post_id).await {
        Ok(p) => p,
        Err(sqlx::Error::RowNotFound) => {
            return (
                StatusCode::NOT_FOUND,
                Json(PostErrorResponse {
                    error: PostError::NotFound,
                    validation: None,
                }),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to get post: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Check ownership (admins can also edit)
    if post.user_id != auth.user.user_id && auth.user.role != api_types::UserRole::Admin {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Validate title
    let title_validation = field_validator::validate_post_title(&payload.title);
    if !title_validation.is_valid() {
        return (
            StatusCode::BAD_REQUEST,
            Json(PostErrorResponse {
                error: PostError::Validation,
                validation: Some(ValidationErrorData::from_errors(vec![title_validation])),
            }),
        )
            .into_response();
    }

    // Validate description if provided
    if let Some(ref desc) = payload.description {
        let desc_validation = field_validator::validate_post_description(desc);
        if !desc_validation.is_valid() {
            return (
                StatusCode::BAD_REQUEST,
                Json(PostErrorResponse {
                    error: PostError::Validation,
                    validation: Some(ValidationErrorData::from_errors(vec![desc_validation])),
                }),
            )
                .into_response();
        }
    }

    match db
        .posts_table
        .update(post_id, &payload.title, payload.description.as_deref())
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            error!("Failed to update post: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/posts/{post_id}",
    params(
        ("post_id" = i64, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Post deleted"),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse),
        (status = 403, description = "Not authorized"),
        (status = 404, description = "Post not found", body = PostErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "posts"
)]
pub async fn delete_post(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    Path(post_id): Path<i64>,
) -> impl IntoResponse {
    // Check if post exists
    let post = match db.posts_table.get_visible_by_id(post_id).await {
        Ok(p) => p,
        Err(sqlx::Error::RowNotFound) => {
            return (
                StatusCode::NOT_FOUND,
                Json(PostErrorResponse {
                    error: PostError::NotFound,
                    validation: None,
                }),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to get post: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Check ownership (admins can also delete)
    if post.user_id != auth.user.user_id && auth.user.role != api_types::UserRole::Admin {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Soft delete
    match db.posts_table.soft_delete(post_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            error!("Failed to delete post: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/posts/{post_id}/image",
    params(
        ("post_id" = i64, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Image data", content_type = "image/*"),
        (status = 404, description = "Post or image not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "posts"
)]
pub async fn get_post_image(State(db): State<Arc<DBHandle>>, Path(post_id): Path<i64>) -> Response {
    // Only serve images for non-deleted posts
    let post = match db.posts_table.get_visible_by_id(post_id).await {
        Ok(p) => p,
        Err(sqlx::Error::RowNotFound) => {
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(e) => {
            error!("Failed to get post for image: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let file_path = std::path::Path::new(UPLOADS_DIR).join(&post.image_path);

    match std::fs::read(&file_path) {
        Ok(data) => {
            let content_type = mime_guess::from_path(&post.image_path)
                .first_or_octet_stream()
                .to_string();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, "public, max-age=86400")
                .body(Body::from(data))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            error!("Failed to read image file {:?}: {:?}", file_path, e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
