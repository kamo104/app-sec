use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tracing::error;

use crate::db::DBHandle;
use api_types::{CounterData, SetCounterRequest, AuthErrorResponse};
use super::auth_extractor::AuthenticatedUser;

// Note: utoipa proc macros require literal integers for status codes.
// 200 = OK, 401 = UNAUTHORIZED, 500 = INTERNAL_SERVER_ERROR
#[utoipa::path(
    get,
    path = "/api/counter/get",
    responses(
        (status = 200, description = "Counter value retrieved", body = CounterData),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "counter"
)]
pub async fn get_counter(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
) -> impl IntoResponse {
    match db.user_data_table.get_counter(auth.user.user_id).await {
        Ok(counter_value) => {
            (StatusCode::OK, Json(CounterData { value: counter_value })).into_response()
        }
        Err(e) => {
            error!("Failed to get counter: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// Note: utoipa proc macros require literal integers for status codes.
// 200 = OK, 401 = UNAUTHORIZED, 500 = INTERNAL_SERVER_ERROR
#[utoipa::path(
    post,
    path = "/api/counter/set",
    request_body = SetCounterRequest,
    responses(
        (status = 200, description = "Counter value updated", body = CounterData),
        (status = 401, description = "Not authenticated", body = AuthErrorResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "counter"
)]
pub async fn set_counter(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    Json(payload): Json<SetCounterRequest>,
) -> impl IntoResponse {
    match db
        .user_data_table
        .update_counter(auth.user.user_id, payload.value)
        .await
    {
        Ok(_) => {
            (StatusCode::OK, Json(CounterData { value: payload.value })).into_response()
        }
        Err(e) => {
            error!("Failed to update counter: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
