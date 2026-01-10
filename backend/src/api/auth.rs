use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tower_cookies::Cookies;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::{debug, error};

use crate::db::{DBHandle, hash_token};
use api_types::{AuthErrorResponse, AuthError, AuthCheckResponse, AuthCheckSuccess, AuthRefreshResponse, AuthRefreshSuccess};
use super::auth_extractor::AuthenticatedUser;
use super::utils::{create_session_cookie, SESSION_DURATION_DAYS};

#[utoipa::path(
    get,
    path = "/api/auth/check",
    responses(
        (status = 200, description = "Session is valid", body = AuthCheckResponse),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse)
    ),
    tag = "auth"
)]
pub async fn auth_check(auth: AuthenticatedUser) -> impl IntoResponse {
    (StatusCode::OK, Json(AuthCheckResponse {
        success: AuthCheckSuccess::Ok,
        username: auth.user.username,
        email: auth.user.email,
        session_expires_at: auth.session.session_expiry.unix_timestamp(),
        session_created_at: auth.session.session_created_at.unix_timestamp(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    responses(
        (status = 200, description = "Session refreshed successfully", body = AuthRefreshResponse),
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

    let token = match cookies.get("session_token") {
        Some(t) => t,
        None => {
            error!("No session token found in refresh request");
            return (StatusCode::UNAUTHORIZED, Json(AuthErrorResponse::default())).into_response();
        }
    };

    let session_hash = match hash_token(token.value()) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash session token: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthErrorResponse { error: AuthError::Internal })).into_response();
        }
    };

    let new_expiry = OffsetDateTime::now_utc() + time::Duration::days(SESSION_DURATION_DAYS);

    match db.user_sessions_table.update_expiry(&session_hash, new_expiry).await {
        Ok(_) => {
            debug!("Session refreshed successfully for user: {}", auth.user.username);

            let cookie = create_session_cookie(token.value().to_string(), Some(new_expiry), db.is_dev);
            cookies.add(cookie);

            (StatusCode::OK, Json(AuthRefreshResponse {
                success: AuthRefreshSuccess::SessionRefreshed,
                username: auth.user.username,
                email: auth.user.email,
                session_expires_at: new_expiry.unix_timestamp(),
                session_created_at: auth.session.session_created_at.unix_timestamp(),
            })).into_response()
        }
        Err(e) => {
            error!("Failed to refresh session: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthErrorResponse { error: AuthError::Internal })).into_response()
        }
    }
}
