use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
    Json,
};
use tower_cookies::Cookies;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;

use crate::db::{DBHandle, UserLogin, UserSession, hash_token};
use api_types::{AuthErrorResponse, AuthError};

pub struct AuthenticatedUser {
    pub user: UserLogin,
    pub session: UserSession,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    Arc<DBHandle>: FromRef<S>,
{
    type Rejection = (StatusCode, Json<AuthErrorResponse>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db = Arc::<DBHandle>::from_ref(state);

        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, Json(AuthErrorResponse::default())))?;

        let token = cookies
            .get("session_token")
            .ok_or((StatusCode::UNAUTHORIZED, Json(AuthErrorResponse::default())))?;

        let hash = hash_token(token.value())
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthErrorResponse { error: AuthError::Internal })))?;

        let session = db
            .user_sessions_table
            .get_by_hash(&hash)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, Json(AuthErrorResponse::default())))?;

        if session.session_expiry < OffsetDateTime::now_utc() {
            return Err((StatusCode::UNAUTHORIZED, Json(AuthErrorResponse::default())));
        }

        let user = db
            .user_login_table
            .get_by_user_id(session.user_id)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, Json(AuthErrorResponse::default())))?;

        Ok(Self { user, session })
    }
}
