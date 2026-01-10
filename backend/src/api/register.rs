use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::{debug, error};

use crate::db::{DBHandle, UserLogin, generate_verification_token, hash_token};
use crate::email::EmailSender;
use api_types::{RegisterError, RegisterErrorResponse, RegistrationRequest, ValidationErrorData};
use super::utils::{BASE_URL_DEV, BASE_URL_PROD, EMAIL_VERIFICATION_TOKEN_DURATION_HOURS};

#[utoipa::path(
    post,
    path = "/api/register",
    request_body = RegistrationRequest,
    responses(
        (status = 200, description = "User registered successfully"),
        (status = 400, description = "Validation error", body = RegisterErrorResponse),
        (status = 409, description = "Username or email already taken", body = RegisterErrorResponse),
        (status = 500, description = "Internal server error", body = RegisterErrorResponse)
    ),
    tag = "auth"
)]
pub async fn register_user(
    State(db): State<Arc<DBHandle>>,
    Json(payload): Json<RegistrationRequest>,
) -> impl IntoResponse {
    debug!(
        "Received registration request - username: {}, email: {}",
        payload.username, payload.email
    );

    let username_result = field_validator::validate_username(&payload.username);
    let email_result = field_validator::validate_email(&payload.email);
    let password_result = field_validator::validate_password(&payload.password);

    let mut errors = Vec::new();
    if !username_result.is_valid() {
        errors.push(username_result);
    }
    if !email_result.is_valid() {
        errors.push(email_result);
    }
    if !password_result.is_valid() {
        errors.push(password_result);
    }

    if !errors.is_empty() {
        debug!("Validation failed: {:?}", errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(RegisterErrorResponse {
                error: RegisterError::Validation,
                validation: Some(ValidationErrorData::from_errors(errors)),
            })
        ).into_response();
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
            return (
                StatusCode::CONFLICT,
                Json(RegisterErrorResponse {
                    error: RegisterError::UsernameTaken,
                    validation: None,
                })
            ).into_response();
        }
        Err(e) => {
            debug!(
                "Database error checking username '{}': {:?}",
                payload.username, e
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterErrorResponse {
                    error: RegisterError::Internal,
                    validation: None,
                })
            ).into_response();
        }
    }

    match db
        .user_login_table
        .is_email_free(&payload.email)
        .await
    {
        Ok(true) => {
            debug!("Email '{}' is available", payload.email);
        }
        Ok(false) => {
            debug!(
                "Registration failed: email '{}' already taken",
                payload.email
            );
            return (
                StatusCode::CONFLICT,
                Json(RegisterErrorResponse {
                    error: RegisterError::EmailTaken,
                    validation: None,
                })
            ).into_response();
        }
        Err(e) => {
            debug!(
                "Database error checking email '{}': {:?}",
                payload.email, e
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterErrorResponse {
                    error: RegisterError::Internal,
                    validation: None,
                })
            ).into_response();
        }
    }

    let hashed_password = match crate::db::hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => {
            debug!(
                "Failed to hash password for '{}': {:?}",
                payload.username, e
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterErrorResponse {
                    error: RegisterError::Internal,
                    validation: None,
                })
            ).into_response();
        }
    };

    let user_login = UserLogin {
        user_id: 0,
        username: payload.username.clone(),
        email: payload.email.clone(),
        password: Some(hashed_password),
        email_verified: false,
        email_verified_at: None,
        password_reset: false,
    };

    match db.user_login_table.new_user(&user_login).await {
        Ok(user_id) => {
            debug!(
                "User record created for '{}' with id {}",
                payload.username, user_id
            );

            // Create user_data record
            if let Err(e) = db.user_data_table.insert(user_id).await {
                debug!(
                    "Failed to create user_data for '{}': {:?}. Attempting cleanup...",
                    payload.username, e
                );
                match db.user_login_table.delete(&payload.username).await {
                    Ok(_) => debug!("Cleanup successful: deleted user '{}'", payload.username),
                    Err(cleanup_e) => error!(
                        "Cleanup failed for user '{}': {:?}",
                        payload.username, cleanup_e
                    ),
                }
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RegisterErrorResponse {
                        error: RegisterError::Internal,
                        validation: None,
                    })
                ).into_response();
            }

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
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(RegisterErrorResponse {
                            error: RegisterError::Internal,
                            validation: None,
                        })
                    ).into_response();
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
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(RegisterErrorResponse {
                                error: RegisterError::Internal,
                                validation: None,
                            })
                        ).into_response();
                    }

                    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
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
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(RegisterErrorResponse {
                            error: RegisterError::Internal,
                            validation: None,
                        })
                    ).into_response()
                }
            }
        }
        Err(e) => {
            let (error_code, status) = if e.to_string().contains("username already taken") {
                (RegisterError::UsernameTaken, StatusCode::CONFLICT)
            } else {
                (RegisterError::Internal, StatusCode::BAD_REQUEST)
            };
            debug!("Failed to create user '{}': {:?}", payload.username, e);
            (
                status,
                Json(RegisterErrorResponse {
                    error: error_code,
                    validation: None,
                })
            ).into_response()
        }
    }
}
