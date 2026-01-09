use axum::{
    extract::State,
    response::IntoResponse,
};
use tower_cookies::Cookies;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;

use crate::db::{DBHandle, hash_token};
use proto_types::v1::SuccessCode;
use super::utils::{create_session_cookie, success_response};

pub async fn logout_user(
    State(db): State<Arc<DBHandle>>,
    cookies: Cookies,
) -> impl IntoResponse {
    if let Some(token) = cookies.get("session_token") {
        if let Ok(hash) = hash_token(token.value()) {
            let _ = db.user_sessions_table.delete_by_hash(&hash).await;
        }
    }

    let cookie = create_session_cookie(String::new(), Some(OffsetDateTime::UNIX_EPOCH), db.is_dev);

    cookies.add(cookie);

    success_response(SuccessCode::SuccessLoggedOut, None)
}
