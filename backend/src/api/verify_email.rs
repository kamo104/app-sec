use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::debug;

use crate::db::{DBHandle, hash_token};
use api_types::{VerifyEmailResponse, VerifyEmailError, VerifyEmailErrorResponse, EmailVerificationRequest};

#[utoipa::path(
    post,
    path = "/api/verify-email",
    request_body = EmailVerificationRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = VerifyEmailResponse),
        (status = 400, description = "Invalid or expired token", body = VerifyEmailErrorResponse),
        (status = 500, description = "Internal server error", body = VerifyEmailErrorResponse)
    ),
    tag = "auth"
)]
pub async fn verify_email(
    State(db): State<Arc<DBHandle>>,
    Json(payload): Json<EmailVerificationRequest>,
) -> impl IntoResponse {
    debug!(
        "Received email verification request - token: {}",
        payload.token
    );

    if payload.token.is_empty() {
        debug!("Email verification failed: token is empty");
        return (
            StatusCode::BAD_REQUEST,
            Json(VerifyEmailErrorResponse {
                error: VerifyEmailError::InvalidToken,
            })
        ).into_response();
    }

    let token_hash = match hash_token(&payload.token) {
        Ok(hash) => hash,
        Err(e) => {
            debug!("Failed to hash verification token: {:?}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(VerifyEmailErrorResponse {
                    error: VerifyEmailError::InvalidToken,
                })
            ).into_response();
        }
    };

    let token_record = match db
        .email_verification_tokens_table
        .get_by_token_hash(&token_hash)
        .await
    {
        Ok(record) => record,
        Err(sqlx::Error::RowNotFound) => {
            debug!("Verification token not found in database");
            return (
                StatusCode::BAD_REQUEST,
                Json(VerifyEmailErrorResponse {
                    error: VerifyEmailError::InvalidToken,
                })
            ).into_response();
        }
        Err(e) => {
            debug!("Database error looking up token: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(VerifyEmailErrorResponse {
                    error: VerifyEmailError::Internal,
                })
            ).into_response();
        }
    };

    if OffsetDateTime::now_utc() > token_record.expires_at {
        debug!("Verification token has expired");
        return (
            StatusCode::BAD_REQUEST,
            Json(VerifyEmailErrorResponse {
                error: VerifyEmailError::InvalidToken,
            })
        ).into_response();
    }

    let user = match db
        .user_login_table
        .get_by_user_id(token_record.user_id)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            debug!("Failed to get user for verification: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(VerifyEmailErrorResponse {
                    error: VerifyEmailError::Internal,
                })
            ).into_response();
        }
    };

    if user.email_verified {
        debug!("User already verified");
        return (StatusCode::OK, Json(VerifyEmailResponse::default())).into_response();
    }

    match db
        .user_login_table
        .mark_email_verified(token_record.user_id)
        .await
    {
        Ok(_) => {
            debug!(
                "Email verified successfully for user_id: {}",
                token_record.user_id
            );

            match db
                .email_verification_tokens_table
                .delete_by_user_id(token_record.user_id)
                .await
            {
                Ok(_) => debug!(
                    "Verification token deleted for user_id: {}",
                    token_record.user_id
                ),
                Err(e) => debug!(
                    "Failed to delete token for user_id {}: {:?}",
                    token_record.user_id, e
                ),
            }

            (StatusCode::OK, Json(VerifyEmailResponse::default())).into_response()
        }
        Err(e) => {
            debug!("Failed to mark email as verified: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(VerifyEmailErrorResponse {
                    error: VerifyEmailError::Internal,
                })
            ).into_response()
        }
    }
}
