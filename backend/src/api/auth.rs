use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tower_cookies::Cookies;
use tracing::{debug, error};

use super::auth_extractor::AuthenticatedUser;
use super::utils::{SESSION_DURATION_DAYS, create_session_cookie};
use crate::db::{DBHandle, generate_session_token, hash_token};
use api_types::{AuthError, AuthErrorResponse, AuthSessionResponse};

// Note: utoipa proc macros require literal integers for status codes.
// 200 = OK, 401 = UNAUTHORIZED, 500 = INTERNAL_SERVER_ERROR
#[utoipa::path(
    get,
    path = "/api/auth/check",
    responses(
        (status = 200, description = "Session is valid", body = AuthSessionResponse),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse)
    ),
    tag = "auth"
)]
pub async fn auth_check(auth: AuthenticatedUser) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(AuthSessionResponse {
            username: auth.user.username,
            email: auth.user.email,
            role: auth.user.role,
            session_expires_at: auth.session.session_expiry.unix_timestamp(),
            session_created_at: auth.session.session_created_at.unix_timestamp(),
        }),
    )
}

// Note: utoipa proc macros require literal integers for status codes.
// 200 = OK, 401 = UNAUTHORIZED, 500 = INTERNAL_SERVER_ERROR
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    responses(
        (status = 200, description = "Session refreshed successfully", body = AuthSessionResponse),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse),
        (status = 500, description = "Internal server error", body = AuthErrorResponse)
    ),
    tag = "auth"
)]
pub async fn refresh_session(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    cookies: Cookies,
) -> impl IntoResponse {
    debug!("Refreshing session for user: {}", auth.user.username);

    let new_expiry = OffsetDateTime::now_utc() + time::Duration::days(SESSION_DURATION_DAYS);
    let token = generate_session_token();
    let token_hash = match hash_token(token.as_str()) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash token: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: AuthError::Internal,
                }),
            )
                .into_response();
        }
    };
    if let Err(e) = db
        .user_sessions_table
        .update_session(&auth.session.session_hash, new_expiry, token_hash.as_str())
        .await
    {
        error!("Failed to refresh session: {:?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthErrorResponse {
                error: AuthError::Internal,
            }),
        )
            .into_response();
    };

    debug!(
        "Session refreshed successfully for user: {}",
        auth.user.username
    );
    let cookie = create_session_cookie(token, Some(new_expiry), db.is_dev);
    cookies.add(cookie);
    (
        StatusCode::OK,
        Json(AuthSessionResponse {
            username: auth.user.username,
            email: auth.user.email,
            role: auth.user.role,
            session_expires_at: new_expiry.unix_timestamp(),
            session_created_at: auth.session.session_created_at.unix_timestamp(),
        }),
    )
        .into_response()
}
