use axum::http::StatusCode;
use axum_extra::protobuf::Protobuf;
use tower_cookies::Cookie;
use sqlx::types::time::OffsetDateTime;

use crate::generated::v1::{ApiResponse, ResponseCode};

pub const SESSION_DURATION_DAYS: i64 = 7;
pub const EMAIL_VERIFICATION_TOKEN_DURATION_HOURS: i64 = 2;
pub const PASSWORD_RESET_TOKEN_DURATION_HOURS: i64 = 1;

pub const BASE_URL_DEV: &str = "http://localhost:4000";
pub const BASE_URL_PROD: &str = "https://example.com";

pub fn auth_error() -> (StatusCode, Protobuf<ApiResponse>) {
    (
        StatusCode::UNAUTHORIZED,
        Protobuf(ApiResponse {
            code: ResponseCode::ErrorInvalidCredentials.into(),
            data: None,
        }),
    )
}

pub fn internal_error() -> (StatusCode, Protobuf<ApiResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Protobuf(ApiResponse {
            code: ResponseCode::ErrorInternal.into(),
            data: None,
        }),
    )
}

pub fn create_session_cookie(value: String, expires_at: Option<OffsetDateTime>, is_dev: bool) -> Cookie<'static> {
    let mut cookie = Cookie::new("session_token", value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_secure(!is_dev);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    if let Some(expiry) = expires_at {
        cookie.set_expires(Some(expiry.into()));
    }
    cookie
}
