use axum::http::StatusCode;
use axum_extra::protobuf::Protobuf;
use tower_cookies::Cookie;
use sqlx::types::time::OffsetDateTime;

use proto_types::v1::{ApiResponse, ResponseCode, SuccessCode, ErrorCode, ApiData, ValidationErrorData, api_data, api_response};

pub const SESSION_DURATION_DAYS: i64 = 7;
pub const EMAIL_VERIFICATION_TOKEN_DURATION_HOURS: i64 = 2;
pub const PASSWORD_RESET_TOKEN_DURATION_HOURS: i64 = 1;

pub const BASE_URL_DEV: &str = "http://localhost:4000";
pub const BASE_URL_PROD: &str = "https://example.com";

pub fn success_response(success_code: SuccessCode, data: Option<ApiData>) -> (StatusCode, Protobuf<ApiResponse>) {
    (
        StatusCode::OK,
        Protobuf(ApiResponse {
            code: ResponseCode::Success.into(),
            detail: Some(api_response::Detail::Success(success_code.into())),
            data,
        }),
    )
}

pub fn error_response(status: StatusCode, error_code: ErrorCode, data: Option<ApiData>) -> (StatusCode, Protobuf<ApiResponse>) {
    (
        status,
        Protobuf(ApiResponse {
            code: ResponseCode::Error.into(),
            detail: Some(api_response::Detail::Error(error_code.into())),
            data,
        }),
    )
}

pub fn auth_error() -> (StatusCode, Protobuf<ApiResponse>) {
    error_response(StatusCode::UNAUTHORIZED, ErrorCode::InvalidCredentials, None)
}

pub fn internal_error() -> (StatusCode, Protobuf<ApiResponse>) {
    error_response(StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::Internal, None)
}

pub fn validation_error(field_errors: Vec<proto_types::v1::ValidationFieldError>) -> (StatusCode, Protobuf<ApiResponse>) {
    error_response(
        StatusCode::BAD_REQUEST,
        ErrorCode::Validation,
        Some(ApiData {
            data: Some(api_data::Data::ValidationError(ValidationErrorData {
                field_errors,
            })),
        }),
    )
}

pub fn create_session_cookie(value: String, expires_at: Option<OffsetDateTime>, is_dev: bool) -> Cookie<'static> {
    let mut cookie = Cookie::new("session_token", value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_secure(!is_dev);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    if let Some(expiry) = expires_at {
        cookie.set_expires(Some(expiry.into()));
    }
    cookie
}
