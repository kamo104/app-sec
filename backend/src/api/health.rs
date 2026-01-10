use axum::response::IntoResponse;

use api_types::{SuccessCode, SuccessResponse};
use super::utils::success_response;

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Health check successful", body = SuccessResponse)
    ),
    tag = "health"
)]
pub async fn health_check() -> impl IntoResponse {
    success_response(SuccessCode::Ok)
}
