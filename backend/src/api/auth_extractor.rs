use axum::{
    Json,
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tower_cookies::Cookies;

use crate::db::{DBHandle, UserLogin, UserSession, hash_token};
use api_types::{AuthError, AuthErrorResponse, UserRole};

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

        let hash = hash_token(token.value()).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: AuthError::Internal,
                }),
            )
        })?;

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

/// Extractor for admin-only endpoints.
/// Returns 401 if not authenticated, 403 if authenticated but not admin.
pub struct AdminUser {
    pub user: UserLogin,
    pub session: UserSession,
}

/// Rejection type that can be either auth error (401) or forbidden (403)
pub enum AdminRejection {
    Unauthorized(Json<AuthErrorResponse>),
    Forbidden,
}

impl axum::response::IntoResponse for AdminRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            AdminRejection::Unauthorized(json) => (StatusCode::UNAUTHORIZED, json).into_response(),
            AdminRejection::Forbidden => StatusCode::FORBIDDEN.into_response(),
        }
    }
}

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    Arc<DBHandle>: FromRef<S>,
{
    type Rejection = AdminRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First, authenticate the user
        let auth_user = AuthenticatedUser::from_request_parts(parts, state)
            .await
            .map_err(|(_, json)| AdminRejection::Unauthorized(json))?;

        // Then check if they're an admin
        if auth_user.user.role != UserRole::Admin {
            return Err(AdminRejection::Forbidden);
        }

        Ok(Self {
            user: auth_user.user,
            session: auth_user.session,
        })
    }
}
