use axum::response::IntoResponse;

use proto_types::v1::SuccessCode;
use super::utils::success_response;

pub async fn health_check() -> impl IntoResponse {
    success_response(SuccessCode::SuccessOk, None)
}
