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

use crate::db::{DBHandle, UserSession, generate_session_id, generate_session_token, hash_token};
use api_types::{ErrorCode, ErrorResponse, LoginResponseData, LoginRequest};
use super::utils::{internal_error, validation_error, error_response, create_session_cookie, SESSION_DURATION_DAYS};

#[utoipa::path(
    post,
    path = "/api/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponseData),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Invalid credentials or email not verified", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn login_user(
    State(db): State<Arc<DBHandle>>,
    cookies: Cookies,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    debug!("Received login request - username: {}", payload.username);

    let username_result = field_validator::validate_username(&payload.username);
    if !username_result.is_valid() {
        debug!("Username validation failed for '{}': {:?}", payload.username, username_result.errors);
        return validation_error(vec![username_result]).into_response();
    }

    let user = match db.user_login_table.get_by_username(&payload.username).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            debug!("Login failed: user '{}' not found", payload.username);
            return error_response(StatusCode::UNAUTHORIZED, ErrorCode::InvalidCredentials).into_response();
        }
        Err(e) => {
            debug!(
                "Database error checking user '{}': {:?}",
                payload.username, e
            );
            return internal_error().into_response();
        }
    };

    if !user.email_verified {
        debug!(
            "Login failed: user '{}' has not verified their email",
            payload.username
        );
        return error_response(StatusCode::UNAUTHORIZED, ErrorCode::EmailNotVerified).into_response();
    }

    match db
        .user_login_table
        .is_password_correct(&payload.username, &payload.password)
        .await
    {
        Ok(true) => {
            debug!("Login successful for '{}'", payload.username);

            let session_token = generate_session_token();
            let session_hash = match hash_token(&session_token) {
                Ok(hash) => hash,
                Err(e) => {
                    error!("Failed to hash session token: {:?}", e);
                    return internal_error().into_response();
                }
            };

            let now = OffsetDateTime::now_utc();
            let expiry = now + time::Duration::days(SESSION_DURATION_DAYS);
            let session = UserSession {
                user_id: user.user_id,
                session_id: generate_session_id(),
                session_hash,
                session_expiry: expiry,
                session_created_at: now,
            };

            if let Err(e) = db.user_sessions_table.insert(&session).await {
                error!("Failed to store session: {:?}", e);
                return internal_error().into_response();
            }

            let cookie = create_session_cookie(session_token, Some(expiry), db.is_dev);

            cookies.add(cookie);

            let response_data = LoginResponseData {
                username: user.username,
                email: user.email,
                session_expires_at: expiry.unix_timestamp(),
                session_created_at: now.unix_timestamp(),
            };

            (StatusCode::OK, Json(response_data)).into_response()
        }
        Ok(false) => {
            debug!(
                "Login failed: incorrect password for '{}'",
                payload.username
            );
            error_response(StatusCode::UNAUTHORIZED, ErrorCode::InvalidCredentials).into_response()
        }
        Err(e) => {
            debug!(
                "Password verification error for '{}': {:?}",
                payload.username, e
            );
            error_response(StatusCode::UNAUTHORIZED, ErrorCode::Internal).into_response()
        }
    }
}
