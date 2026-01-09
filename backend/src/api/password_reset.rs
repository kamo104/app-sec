use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::{debug, error};

use crate::db::{DBHandle, generate_verification_token, hash_token};
use crate::email::EmailSender;
use crate::generated::v1::{ApiData, ApiResponse, ResponseCode, ValidationErrorData, FieldType, api_data};
use super::utils::{internal_error, BASE_URL_DEV, BASE_URL_PROD, PASSWORD_RESET_TOKEN_DURATION_HOURS};

pub async fn request_password_reset(
    State(db): State<Arc<DBHandle>>,
    Protobuf(payload): Protobuf<crate::generated::v1::PasswordResetRequest>,
) -> impl IntoResponse {
    debug!("Received password reset request for '{}'", payload.email);

    let user = match db.user_login_table.get_by_username(&payload.email).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: None,
            };
            return (StatusCode::OK, Protobuf(response));
        }
        Err(e) => {
            error!("Database error checking user: {:?}", e);
            return internal_error();
        }
    };

    let token = generate_verification_token();
    let token_hash = match hash_token(&token) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash reset token: {:?}", e);
            return internal_error();
        }
    };

    let expires_at = OffsetDateTime::now_utc() + time::Duration::hours(PASSWORD_RESET_TOKEN_DURATION_HOURS);

    if let Err(e) = db.password_reset_tokens_table.insert(user.user_id, &token_hash, expires_at).await {
        error!("Failed to store reset token: {:?}", e);
        return internal_error();
    }

    if let Err(e) = db.user_login_table.set_password_reset_flag(user.user_id, true).await {
        error!("Failed to set password reset flag: {:?}", e);
        return internal_error();
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
        return internal_error();
    }

    let response = ApiResponse {
        code: ResponseCode::Success.into(),
        data: None,
    };
    (StatusCode::OK, Protobuf(response))
}

pub async fn complete_password_reset(
    State(db): State<Arc<DBHandle>>,
    Protobuf(payload): Protobuf<crate::generated::v1::PasswordResetCompleteRequest>,
) -> impl IntoResponse {
    debug!("Received password reset completion request");

    let token_hash = match hash_token(&payload.token) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash reset token: {:?}", e);
            return internal_error();
        }
    };

    let reset_record = match db.password_reset_tokens_table.get_by_token_hash(&token_hash).await {
        Ok(record) => record,
        Err(sqlx::Error::RowNotFound) => {
            let response = ApiResponse {
                code: ResponseCode::ErrorInvalidToken.into(),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Protobuf(response));
        }
        Err(e) => {
            error!("Database error looking up reset token: {:?}", e);
            return internal_error();
        }
    };

    if OffsetDateTime::now_utc() > reset_record.expires_at {
        let response = ApiResponse {
            code: ResponseCode::ErrorInvalidToken.into(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    let password_result = field_validator::validate_password(&payload.new_password);
    if !password_result.errors.is_empty() {
        let response = ApiResponse {
            code: ResponseCode::ErrorValidation.into(),
            data: Some(ApiData {
                data: Some(api_data::Data::ValidationError(ValidationErrorData {
                    field: FieldType::Password.into(),
                    errors: password_result.errors,
                })),
            }),
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    match db.user_login_table.set_password_by_user_id(reset_record.user_id, &payload.new_password).await {
        Ok(_) => {
            let _ = db.user_login_table.set_password_reset_flag(reset_record.user_id, false).await;
            let _ = db.password_reset_tokens_table.delete_by_user_id(reset_record.user_id).await;

            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: None,
            };
            (StatusCode::OK, Protobuf(response))
        }
        Err(e) => {
            error!("Failed to update password: {:?}", e);
            internal_error()
        }
    }
}
