use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
    Json,
};
use tower_cookies::Cookies;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;

use crate::db::{DBHandle, UserLogin, UserSession, hash_token};
use api_types::responses::{AuthErrorResponse, AuthError};

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
            .map_err(|_| auth_error())?;

        let token = cookies
            .get("session_token")
            .ok_or(auth_error())?;

        let hash = hash_token(token.value()).map_err(|_| internal_error())?;

        let session = db
            .user_sessions_table
            .get_by_hash(&hash)
            .await
            .map_err(|_| auth_error())?;

        if session.session_expiry < OffsetDateTime::now_utc() {
            return Err(auth_error());
        }

        let user = db
            .user_login_table
            .get_by_user_id(session.user_id)
            .await
            .map_err(|_| auth_error())?;

        Ok(Self { user, session })
    }
}
