use tower_cookies::Cookie;
use sqlx::types::time::OffsetDateTime;
use konst::{primitive::parse_i64, unwrap_ctx};

// Configuration loaded from .env at compile time via build.rs
pub const SESSION_DURATION_DAYS: i64 = unwrap_ctx!(parse_i64(env!("SESSION_DURATION_DAYS")));
pub const EMAIL_VERIFICATION_TOKEN_DURATION_HOURS: i64 = unwrap_ctx!(parse_i64(env!("EMAIL_VERIFICATION_TOKEN_DURATION_HOURS")));
pub const PASSWORD_RESET_TOKEN_DURATION_HOURS: i64 = unwrap_ctx!(parse_i64(env!("PASSWORD_RESET_TOKEN_DURATION_HOURS")));

pub const BASE_URL_DEV: &str = env!("BASE_URL_DEV");
pub const BASE_URL_PROD: &str = env!("BASE_URL_PROD");

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
