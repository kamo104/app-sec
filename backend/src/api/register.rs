use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::{debug, error};

use crate::db::{DBHandle, UserLogin, generate_verification_token, hash_token};
use crate::email::EmailSender;
use crate::generated::v1::{ApiData, ApiResponse, ResponseCode, ValidationErrorData, FieldType, api_data};
use super::utils::{internal_error, BASE_URL_DEV, BASE_URL_PROD, EMAIL_VERIFICATION_TOKEN_DURATION_HOURS};

pub async fn register_user(
    State(db): State<Arc<DBHandle>>,
    Protobuf(payload): Protobuf<crate::generated::v1::RegistrationRequest>,
) -> impl IntoResponse {
    debug!(
        "Received registration request - username: {}, email: {}",
        payload.username, payload.email
    );

    let username_result = field_validator::validate_username(&payload.username);
    let email_result = field_validator::validate_email(&payload.email);
    let password_result = field_validator::validate_password(&payload.password);

    if !username_result.errors.is_empty() {
        debug!("Username validation failed for '{}': {:?}", payload.username, username_result.errors);
        let response = ApiResponse {
            code: ResponseCode::ErrorValidation.into(),
            data: Some(ApiData {
                data: Some(api_data::Data::ValidationError(ValidationErrorData {
                    field: FieldType::Username.into(),
                    errors: username_result.errors,
                })),
            }),
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    if !email_result.errors.is_empty() {
        debug!("Email validation failed for '{}': {:?}", payload.email, email_result.errors);
        let response = ApiResponse {
            code: ResponseCode::ErrorValidation.into(),
            data: Some(ApiData {
                data: Some(api_data::Data::ValidationError(ValidationErrorData {
                    field: FieldType::Email.into(),
                    errors: email_result.errors,
                })),
            }),
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    if !password_result.errors.is_empty() {
        debug!("Password validation failed for '{}': {:?}", payload.username, password_result.errors);
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

    match db
        .user_login_table
        .is_username_free(&payload.username)
        .await
    {
        Ok(true) => {
            debug!("Username '{}' is available", payload.username);
        }
        Ok(false) => {
            debug!(
                "Registration failed: username '{}' already taken",
                payload.username
            );
            let response = ApiResponse {
                code: ResponseCode::ErrorUsernameTaken.into(),
                data: None,
            };
            return (StatusCode::CONFLICT, Protobuf(response));
        }
        Err(e) => {
            debug!(
                "Database error checking username '{}': {:?}",
                payload.username, e
            );
            return internal_error();
        }
    }

    let hashed_password = match crate::db::hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => {
            debug!(
                "Failed to hash password for '{}': {:?}",
                payload.username, e
            );
            return internal_error();
        }
    };

    let user_login = UserLogin {
        user_id: 0,
        username: payload.username.clone(),
        email: payload.email.clone(),
        password: Some(hashed_password),
        email_verified: false,
        email_verified_at: None,
        counter: 0,
        password_reset: false,
    };

    match db.user_login_table.new_user(&user_login).await {
        Ok(user_id) => {
            debug!(
                "User record created for '{}' with id {}",
                payload.username, user_id
            );

            let token = generate_verification_token();
            let token_hash = match hash_token(&token) {
                Ok(hash) => hash,
                Err(e) => {
                    debug!(
                        "Failed to hash token for '{}': {:?}. Attempting cleanup...",
                        payload.username, e
                    );
                    match db.user_login_table.delete(&payload.username).await {
                        Ok(_) => debug!("Cleanup successful: deleted user '{}'", payload.username),
                        Err(cleanup_e) => error!(
                            "Cleanup failed for user '{}': {:?}",
                            payload.username, cleanup_e
                        ),
                    }
                    return internal_error();
                }
            };

            let expires_at = OffsetDateTime::now_utc() + time::Duration::hours(EMAIL_VERIFICATION_TOKEN_DURATION_HOURS);

            match db
                .email_verification_tokens_table
                .insert(user_id, &token_hash, expires_at)
                .await
            {
                Ok(_) => {
                    debug!("Verification token stored for user_id: {}", user_id);

                    let base_url = if db.is_dev {
                        BASE_URL_DEV
                    } else {
                        BASE_URL_PROD
                    };

                    let verification_link = format!("{}/verify-email?token={}", base_url, token);

                    let email_sender = EmailSender::new_mailhog();
                    if let Err(e) = email_sender
                        .send_verification_email(&payload.email, &verification_link)
                        .await
                    {
                        error!(
                            "Failed to send verification email to {}: {:?}. Attempting cleanup...",
                            payload.email, e
                        );
                        match db.user_login_table.delete(&payload.username).await {
                            Ok(_) => {
                                debug!("Cleanup successful: deleted user '{}'", payload.username)
                            }
                            Err(cleanup_e) => error!(
                                "Cleanup failed for user '{}': {:?}",
                                payload.username, cleanup_e
                            ),
                        }
                        return internal_error();
                    }

                    let response = ApiResponse {
                        code: ResponseCode::SuccessRegistered.into(),
                        data: None,
                    };
                    (StatusCode::CREATED, Protobuf(response))
                }
                Err(e) => {
                    debug!(
                        "Failed to store verification token for user_id {}: {:?}. Attempting cleanup...",
                        user_id, e
                    );
                    match db.user_login_table.delete(&payload.username).await {
                        Ok(_) => debug!("Cleanup successful: deleted user '{}'", payload.username),
                        Err(cleanup_e) => error!(
                            "Cleanup failed for user '{}': {:?}",
                            payload.username, cleanup_e
                        ),
                    }
                    return internal_error();
                }
            }
        }
        Err(e) => {
            let (code, status) = if e.to_string().contains("username already taken") {
                (ResponseCode::ErrorUsernameTaken, StatusCode::CONFLICT)
            } else {
                (ResponseCode::ErrorInternal, StatusCode::BAD_REQUEST)
            };
            debug!("Failed to create user '{}': {:?}", payload.username, e);
            let response = ApiResponse {
                code: code.into(),
                data: None,
            };
            (status, Protobuf(response))
        }
    }
}
