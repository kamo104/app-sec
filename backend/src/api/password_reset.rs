use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::{debug, error};

use crate::db::{DBHandle, generate_verification_token, hash_token};
use crate::email::EmailSender;
use api_types::{SuccessCode, ErrorCode, SuccessResponse, ErrorResponse, PasswordResetRequest, PasswordResetCompleteRequest};
use super::utils::{internal_error, validation_error, error_response, success_response, BASE_URL_DEV, BASE_URL_PROD, PASSWORD_RESET_TOKEN_DURATION_HOURS};

#[utoipa::path(
    post,
    path = "/api/request-password-reset",
    request_body = PasswordResetRequest,
    responses(
        (status = 200, description = "Password reset requested (always returns success for security)", body = SuccessResponse)
    ),
    tag = "auth"
)]
pub async fn request_password_reset(
    State(db): State<Arc<DBHandle>>,
    Json(payload): Json<PasswordResetRequest>,
) -> impl IntoResponse {
    debug!("Received password reset request for '{}'", payload.email);

    let user = match db.user_login_table.get_by_username(&payload.email).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            return success_response(SuccessCode::PasswordResetRequested).into_response();
        }
        Err(e) => {
            error!("Database error checking user: {:?}", e);
            return internal_error().into_response();
        }
    };

    let token = generate_verification_token();
    let token_hash = match hash_token(&token) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash reset token: {:?}", e);
            return internal_error().into_response();
        }
    };

    let expires_at = OffsetDateTime::now_utc() + time::Duration::hours(PASSWORD_RESET_TOKEN_DURATION_HOURS);

    if let Err(e) = db.password_reset_tokens_table.insert(user.user_id, &token_hash, expires_at).await {
        error!("Failed to store reset token: {:?}", e);
        return internal_error().into_response();
    }

    if let Err(e) = db.user_login_table.set_password_reset_flag(user.user_id, true).await {
        error!("Failed to set password reset flag: {:?}", e);
        return internal_error().into_response();
    }

    let base_url = if db.is_dev {
        BASE_URL_DEV
    } else {
        BASE_URL_PROD
    };
    let reset_link = format!("{}/reset-password?token={}", base_url, token);
    let email_sender = EmailSender::new_mailhog();

    if let Err(e) = email_sender.send_password_reset_email(&user.email, &reset_link).await {
        error!("Failed to send reset email: {:?}", e);
        return internal_error().into_response();
    }

    success_response(SuccessCode::PasswordResetRequested).into_response()
}

#[utoipa::path(
    post,
    path = "/api/complete-password-reset",
    request_body = PasswordResetCompleteRequest,
    responses(
        (status = 200, description = "Password reset completed", body = SuccessResponse),
        (status = 400, description = "Invalid token or validation error", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn complete_password_reset(
    State(db): State<Arc<DBHandle>>,
    Json(payload): Json<PasswordResetCompleteRequest>,
) -> impl IntoResponse {
    debug!("Received password reset completion request");

    let token_hash = match hash_token(&payload.token) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash reset token: {:?}", e);
            return internal_error().into_response();
        }
    };

    let reset_record = match db.password_reset_tokens_table.get_by_token_hash(&token_hash).await {
        Ok(record) => record,
        Err(sqlx::Error::RowNotFound) => {
            return error_response(StatusCode::BAD_REQUEST, ErrorCode::InvalidToken).into_response();
        }
        Err(e) => {
            error!("Database error looking up reset token: {:?}", e);
            return internal_error().into_response();
        }
    };

    if OffsetDateTime::now_utc() > reset_record.expires_at {
        return error_response(StatusCode::BAD_REQUEST, ErrorCode::InvalidToken).into_response();
    }

    let password_result = field_validator::validate_password(&payload.new_password);
    if !password_result.is_valid() {
        return validation_error(vec![password_result]).into_response();
    }

    match db.user_login_table.set_password_by_user_id(reset_record.user_id, &payload.new_password).await {
        Ok(_) => {
            let _ = db.user_login_table.set_password_reset_flag(reset_record.user_id, false).await;
            let _ = db.password_reset_tokens_table.delete_by_user_id(reset_record.user_id).await;

            success_response(SuccessCode::PasswordResetCompleted).into_response()
        }
        Err(e) => {
            error!("Failed to update password: {:?}", e);
            internal_error().into_response()
        }
    }
}
