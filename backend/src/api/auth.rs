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

use crate::db::{DBHandle, hash_token};
use proto_types::v1::{ApiData, ApiResponse, LoginResponseData, ResponseCode, api_data};
use super::auth_extractor::AuthenticatedUser;
use super::utils::{auth_error, internal_error, create_session_cookie, SESSION_DURATION_DAYS};

pub async fn auth_check(auth: AuthenticatedUser) -> impl IntoResponse {
    let response = ApiResponse {
        code: ResponseCode::Success.into(),
        data: Some(ApiData {
            data: Some(api_data::Data::LoginResponse(LoginResponseData {
                username: auth.user.username,
                email: auth.user.email,
                session_expires_at: auth.session.session_expiry.unix_timestamp(),
                session_created_at: auth.session.session_created_at.unix_timestamp(),
            })),
        }),
    };
    (StatusCode::OK, Protobuf(response))
}

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
            return auth_error().into_response();
        }
    };

    let session_hash = match hash_token(token.value()) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash session token: {:?}", e);
            return internal_error().into_response();
        }
    };

    let new_expiry = OffsetDateTime::now_utc() + time::Duration::days(SESSION_DURATION_DAYS);

    match db.user_sessions_table.update_expiry(&session_hash, new_expiry).await {
        Ok(_) => {
            debug!("Session refreshed successfully for user: {}", auth.user.username);

            let cookie = create_session_cookie(token.value().to_string(), Some(new_expiry), db.is_dev);
            cookies.add(cookie);

            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: Some(ApiData {
                    data: Some(api_data::Data::LoginResponse(LoginResponseData {
                        username: auth.user.username,
                        email: auth.user.email,
                        session_expires_at: new_expiry.unix_timestamp(),
                        session_created_at: auth.session.session_created_at.unix_timestamp(),
                    })),
                }),
            };
            (StatusCode::OK, Protobuf(response)).into_response()
        }
        Err(e) => {
            error!("Failed to refresh session: {:?}", e);
            internal_error().into_response()
        }
    }
}
