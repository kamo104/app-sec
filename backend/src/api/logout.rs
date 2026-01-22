use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tower_cookies::Cookies;

use super::utils::create_session_cookie;
use crate::db::{DBHandle, hash_token};

// Note: utoipa proc macros require literal integers for status codes.
// 200 = OK
#[utoipa::path(
    post,
    path = "/api/logout",
    responses(
        (status = 200, description = "Logged out successfully")
    ),
    tag = "auth"
)]
pub async fn logout_user(State(db): State<Arc<DBHandle>>, cookies: Cookies) -> impl IntoResponse {
    if let Some(token) = cookies.get("session_token") {
        if let Ok(hash) = hash_token(token.value()) {
            let _ = db.user_sessions_table.delete_by_hash(&hash).await;
        }
    }

    let cookie = create_session_cookie(
        String::new(),
        Some(OffsetDateTime::UNIX_EPOCH),
        db.tls_enabled,
    );

    cookies.add(cookie);

    (StatusCode::OK, Json(serde_json::json!({})))
}
