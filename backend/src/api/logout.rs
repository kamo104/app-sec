use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use tower_cookies::Cookies;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;

use crate::db::{DBHandle, hash_token};
use crate::generated::v1::{ApiResponse, ResponseCode};
use super::utils::create_session_cookie;

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

    let response = ApiResponse {
        code: ResponseCode::Success.into(),
        data: None,
    };

    (StatusCode::OK, Protobuf(response))
}
