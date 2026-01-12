use axum::Json;
use axum::response::IntoResponse;

// Note: utoipa proc macros require literal integers for status codes.
// 200 = OK
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
