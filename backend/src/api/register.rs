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
use proto_types::v1::{SuccessCode, ErrorCode};
use super::utils::{internal_error, validation_error, error_response, success_response, BASE_URL_DEV, BASE_URL_PROD, EMAIL_VERIFICATION_TOKEN_DURATION_HOURS};

pub async fn register_user(
    State(db): State<Arc<DBHandle>>,
    Protobuf(payload): Protobuf<proto_types::v1::RegistrationRequest>,
) -> impl IntoResponse {
    debug!(
        "Received registration request - username: {}, email: {}",
        payload.username, payload.email
    );

    let username_result = field_validator::validate_username(&payload.username);
    let email_result = field_validator::validate_email(&payload.email);
    let password_result = field_validator::validate_password(&payload.password);

    let mut errors = Vec::new();
    if !username_result.errors.is_empty() {
        errors.push(username_result);
    }
    if !email_result.errors.is_empty() {
        errors.push(email_result);
    }
    if !password_result.errors.is_empty() {
        errors.push(password_result);
    }

    if !errors.is_empty() {
        debug!("Validation failed: {:?}", errors);
        return validation_error(errors);
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
            return error_response(StatusCode::CONFLICT, ErrorCode::UsernameTaken, None);
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

                    success_response(SuccessCode::SuccessRegistered, None)
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
            let (error_code, status) = if e.to_string().contains("username already taken") {
                (ErrorCode::UsernameTaken, StatusCode::CONFLICT)
            } else {
                (ErrorCode::Internal, StatusCode::BAD_REQUEST)
            };
            debug!("Failed to create user '{}': {:?}", payload.username, e);
            error_response(status, error_code, None)
        }
    }
}
