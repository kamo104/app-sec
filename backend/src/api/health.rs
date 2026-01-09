use axum::{
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;

use proto_types::v1::{ApiResponse, ResponseCode};

pub async fn health_check() -> impl IntoResponse {
    let response = ApiResponse {
        code: ResponseCode::Success.into(),
        data: None,
    };
    (StatusCode::OK, Protobuf(response))
}
