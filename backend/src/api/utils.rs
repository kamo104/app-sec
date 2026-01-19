use sqlx::types::time::OffsetDateTime;
use tower_cookies::Cookie;

use crate::config::Config;

pub fn create_session_cookie(
    value: String,
    expires_at: Option<OffsetDateTime>,
    is_dev: bool,
) -> Cookie<'static> {
    let mut cookie = Cookie::new("session_token", value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_secure(!is_dev);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
    if let Some(expiry) = expires_at {
        cookie.set_expires(Some(expiry.into()));
    }
    cookie
}

pub fn get_base_url(config: &Config) -> String {
    if config.server.dev_mode {
        config.urls.base_url_dev.clone()
    } else {
        config.urls.base_url_prod.clone()
    }
}
