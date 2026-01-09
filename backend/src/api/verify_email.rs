use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::debug;

use crate::db::{DBHandle, hash_token};
use crate::generated::v1::{ApiResponse, ResponseCode};
use super::utils::internal_error;

pub async fn verify_email(
    State(db): State<Arc<DBHandle>>,
    Protobuf(payload): Protobuf<crate::generated::v1::EmailVerificationRequest>,
) -> impl IntoResponse {
    debug!(
        "Received email verification request - token: {}",
        payload.token
    );

    if payload.token.is_empty() {
        debug!("Email verification failed: token is empty");
        let response = ApiResponse {
            code: ResponseCode::ErrorInvalidToken.into(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    let token_hash = match hash_token(&payload.token) {
        Ok(hash) => hash,
        Err(e) => {
            debug!("Failed to hash verification token: {:?}", e);
            let response = ApiResponse {
                code: ResponseCode::ErrorInvalidToken.into(),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Protobuf(response));
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
            let response = ApiResponse {
                code: ResponseCode::ErrorInvalidToken.into(),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Protobuf(response));
        }
        Err(e) => {
            debug!("Database error looking up token: {:?}", e);
            return internal_error();
        }
    };

    if OffsetDateTime::now_utc() > token_record.expires_at {
        debug!("Verification token has expired");
        let response = ApiResponse {
            code: ResponseCode::ErrorInvalidToken.into(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    let user = match db
        .user_login_table
        .get_by_user_id(token_record.user_id)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            debug!("Failed to get user for verification: {:?}", e);
            let response = ApiResponse {
                code: ResponseCode::ErrorInternal.into(),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Protobuf(response));
        }
    };

    if user.email_verified {
        debug!("User already verified");
        let response = ApiResponse {
            code: ResponseCode::Success.into(),
            data: None,
        };
        return (StatusCode::OK, Protobuf(response));
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

            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: None,
            };
            (StatusCode::OK, Protobuf(response))
        }
        Err(e) => {
            debug!("Failed to mark email as verified: {:?}", e);
            return internal_error();
        }
    }
}
