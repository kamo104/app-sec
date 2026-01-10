use axum::response::IntoResponse;
use axum::Json;

use api_types::HealthResponse;

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Health check successful", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse::default())
}
