use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use tower_cookies::Cookies;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::{debug, error};

use crate::db::{DBHandle, UserSession, generate_session_id, generate_session_token, hash_token};
use proto_types::v1::{ApiData, ApiResponse, LoginResponseData, ResponseCode, api_data};
use super::utils::{internal_error, validation_error, create_session_cookie, SESSION_DURATION_DAYS};

pub async fn login_user(
    State(db): State<Arc<DBHandle>>,
    cookies: Cookies,
    Protobuf(payload): Protobuf<proto_types::v1::LoginRequest>,
) -> impl IntoResponse {
    debug!("Received login request - username: {}", payload.username);

    let username_result = field_validator::validate_username(&payload.username);
    if !username_result.errors.is_empty() {
        debug!("Username validation failed for '{}': {:?}", payload.username, username_result.errors);
        return validation_error(vec![username_result]).into_response();
    }

    let user = match db.user_login_table.get_by_username(&payload.username).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            debug!("Login failed: user '{}' not found", payload.username);
            let response = ApiResponse {
                code: ResponseCode::ErrorInvalidCredentials.into(),
                data: None,
            };
            return (StatusCode::UNAUTHORIZED, Protobuf(response)).into_response();
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
        let response = ApiResponse {
            code: ResponseCode::ErrorEmailNotVerified.into(),
            data: None,
        };
        return (StatusCode::UNAUTHORIZED, Protobuf(response)).into_response();
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
            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: Some(ApiData {
                    data: Some(api_data::Data::LoginResponse(response_data)),
                }),
            };

            (StatusCode::OK, Protobuf(response)).into_response()
        }
        Ok(false) => {
            debug!(
                "Login failed: incorrect password for '{}'",
                payload.username
            );
            let response = ApiResponse {
                code: ResponseCode::ErrorInvalidCredentials.into(),
                data: None,
            };
            (StatusCode::UNAUTHORIZED, Protobuf(response)).into_response()
        }
        Err(e) => {
            debug!(
                "Password verification error for '{}': {:?}",
                payload.username, e
            );
            let response = ApiResponse {
                code: ResponseCode::ErrorInternal.into(),
                data: None,
            };
            (StatusCode::UNAUTHORIZED, Protobuf(response)).into_response()
        }
    }
}
