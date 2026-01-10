//! API response types.
//!
//! Simple success responses (200 OK) return empty JSON `{}`.
//! The frontend uses translate('SUCCESS_KEY') to display success messages.

use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::ValidationErrorData;

// =============================================================================
// Registration endpoint responses
// =============================================================================

/// Error codes specific to registration endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum RegisterError {
    #[serde(rename = "USERNAME_TAKEN")]
    UsernameTaken,
    #[serde(rename = "EMAIL_TAKEN")]
    EmailTaken,
    #[serde(rename = "VALIDATION")]
    Validation,
    #[serde(rename = "INTERNAL")]
    Internal,
}

/// Registration error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct RegisterErrorResponse {
    pub error: RegisterError,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationErrorData>,
}

// =============================================================================
// Login endpoint responses
// =============================================================================

/// Login success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub username: String,
    pub email: String,
    /// Unix timestamp in seconds when the session expires.
    pub session_expires_at: i64,
    /// Unix timestamp in seconds when the session was created.
    pub session_created_at: i64,
}

/// Error codes specific to login endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum LoginError {
    #[serde(rename = "INVALID_CREDENTIALS")]
    InvalidCredentials,
    #[serde(rename = "EMAIL_NOT_VERIFIED")]
    EmailNotVerified,
    #[serde(rename = "VALIDATION")]
    Validation,
    #[serde(rename = "INTERNAL")]
    Internal,
}

/// Login error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct LoginErrorResponse {
    pub error: LoginError,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationErrorData>,
}

// =============================================================================
// Email verification endpoint responses
// =============================================================================

/// Error codes specific to email verification endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum VerifyEmailError {
    #[serde(rename = "TOKEN_EXPIRED")]
    TokenExpired,
    #[serde(rename = "INTERNAL")]
    Internal,
}

/// Email verification error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct VerifyEmailErrorResponse {
    pub error: VerifyEmailError,
}

// =============================================================================
// Password reset request endpoint responses
// =============================================================================

// Password reset request endpoint only returns INTERNAL errors,
// which can be returned as a simple JSON: {"error": "INTERNAL"}

// =============================================================================
// Password reset completion endpoint responses
// =============================================================================

/// Error codes specific to password reset completion endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum CompletePasswordResetError {
    #[serde(rename = "INVALID_TOKEN")]
    InvalidToken,
    #[serde(rename = "VALIDATION")]
    Validation,
    #[serde(rename = "INTERNAL")]
    Internal,
}

/// Password reset completion error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CompletePasswordResetErrorResponse {
    pub error: CompletePasswordResetError,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationErrorData>,
}

// =============================================================================
// Auth check/refresh endpoint responses
// =============================================================================

/// Auth session response - used for both auth check and refresh endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct AuthSessionResponse {
    pub username: String,
    pub email: String,
    pub session_expires_at: i64,
    pub session_created_at: i64,
}

/// Error codes for auth check and refresh endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum AuthError {
    #[serde(rename = "INVALID_CREDENTIALS")]
    InvalidCredentials,
    #[serde(rename = "INTERNAL")]
    Internal,
}

/// Auth error response (used for auth check and refresh).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AuthErrorResponse {
    pub error: AuthError,
}

impl Default for AuthErrorResponse {
    fn default() -> Self {
        Self { error: AuthError::InvalidCredentials }
    }
}

// =============================================================================
// Counter endpoint responses
// =============================================================================

/// Counter data containing the current counter value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CounterData {
    pub value: i64,
}

// Counter endpoint only returns INTERNAL errors,
// which can be returned as a simple JSON: {"error": "INTERNAL"}
