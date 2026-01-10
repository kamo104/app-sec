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
use api_types::{LoginErrorResponse, LoginError, LoginResponse, LoginRequest, ValidationErrorData};
use super::utils::{create_session_cookie, SESSION_DURATION_DAYS};

// Note: utoipa proc macros require literal integers for status codes.
// 200 = OK, 400 = BAD_REQUEST, 401 = UNAUTHORIZED, 500 = INTERNAL_SERVER_ERROR
#[utoipa::path(
    post,
    path = "/api/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Validation error", body = LoginErrorResponse),
        (status = 401, description = "Invalid credentials or email not verified", body = LoginErrorResponse),
        (status = 500, description = "Internal server error", body = LoginErrorResponse)
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
        return (
            StatusCode::BAD_REQUEST,
            Json(LoginErrorResponse {
                error: LoginError::Validation,
                validation: Some(ValidationErrorData::from_errors(vec![username_result])),
            })
        ).into_response();
    }

    let user = match db.user_login_table.get_by_username(&payload.username).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            debug!("Login failed: user '{}' not found", payload.username);
            return (
                StatusCode::UNAUTHORIZED,
                Json(LoginErrorResponse {
                    error: LoginError::InvalidCredentials,
                    validation: None,
                })
            ).into_response();
        }
        Err(e) => {
            debug!(
                "Database error checking user '{}': {:?}",
                payload.username, e
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginErrorResponse {
                    error: LoginError::Internal,
                    validation: None,
                })
            ).into_response();
        }
    };

    if !user.email_verified {
        debug!(
            "Login failed: user '{}' has not verified their email",
            payload.username
        );
        return (
            StatusCode::UNAUTHORIZED,
            Json(LoginErrorResponse {
                error: LoginError::EmailNotVerified,
                validation: None,
            })
        ).into_response();
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
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LoginErrorResponse {
                            error: LoginError::Internal,
                            validation: None,
                        })
                    ).into_response();
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
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LoginErrorResponse {
                        error: LoginError::Internal,
                        validation: None,
                    })
                ).into_response();
            }

            let cookie = create_session_cookie(session_token, Some(expiry), db.is_dev);

            cookies.add(cookie);

            let response_data = LoginResponse {
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
            (
                StatusCode::UNAUTHORIZED,
                Json(LoginErrorResponse {
                    error: LoginError::InvalidCredentials,
                    validation: None,
                })
            ).into_response()
        }
        Err(e) => {
            debug!(
                "Password verification error for '{}': {:?}",
                payload.username, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginErrorResponse {
                    error: LoginError::Internal,
                    validation: None,
                })
            ).into_response()
        }
    }
}
