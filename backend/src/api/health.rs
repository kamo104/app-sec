use axum::response::IntoResponse;
use axum::Json;

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Health check successful")
    ),
    tag = "health"
)]
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({}))
}
