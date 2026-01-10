//! API response types.
//!
//! Each endpoint has specific response types that document exactly which
//! success/error codes are possible, making the API documentation precise.

use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::ValidationErrorData;

// =============================================================================
// Data response types (used for successful responses with data)
// =============================================================================

/// Login response data containing user info and session details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct LoginResponseData {
    pub username: String,
    pub email: String,
    /// Unix timestamp in seconds when the session expires.
    pub session_expires_at: i64,
    /// Unix timestamp in seconds when the session was created.
    pub session_created_at: i64,
}

/// Counter data containing the current counter value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CounterData {
    pub value: i64,
}

// =============================================================================
// Health endpoint responses
// =============================================================================

/// Success code for health check endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum HealthSuccess {
    #[serde(rename = "SUCCESS_OK")]
    Ok,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct HealthResponse {
    pub success: HealthSuccess,
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self { success: HealthSuccess::Ok }
    }
}

// =============================================================================
// Registration endpoint responses
// =============================================================================

/// Success code for registration endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum RegisterSuccess {
    #[serde(rename = "SUCCESS_REGISTERED")]
    Registered,
}

/// Registration success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct RegisterResponse {
    pub success: RegisterSuccess,
}

impl Default for RegisterResponse {
    fn default() -> Self {
        Self { success: RegisterSuccess::Registered }
    }
}

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
// Logout endpoint responses
// =============================================================================

/// Success code for logout endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum LogoutSuccess {
    #[serde(rename = "SUCCESS_LOGGED_OUT")]
    LoggedOut,
}

/// Logout success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct LogoutResponse {
    pub success: LogoutSuccess,
}

impl Default for LogoutResponse {
    fn default() -> Self {
        Self { success: LogoutSuccess::LoggedOut }
    }
}

// =============================================================================
// Email verification endpoint responses
// =============================================================================

/// Success code for email verification endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum VerifyEmailSuccess {
    #[serde(rename = "SUCCESS_EMAIL_VERIFIED")]
    EmailVerified,
}

/// Email verification success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct VerifyEmailResponse {
    pub success: VerifyEmailSuccess,
}

impl Default for VerifyEmailResponse {
    fn default() -> Self {
        Self { success: VerifyEmailSuccess::EmailVerified }
    }
}

/// Error codes specific to email verification endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum VerifyEmailError {
    #[serde(rename = "INVALID_TOKEN")]
    InvalidToken,
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

/// Success code for password reset request endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum RequestPasswordResetSuccess {
    #[serde(rename = "SUCCESS_PASSWORD_RESET_REQUESTED")]
    PasswordResetRequested,
}

/// Password reset request success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct RequestPasswordResetResponse {
    pub success: RequestPasswordResetSuccess,
}

impl Default for RequestPasswordResetResponse {
    fn default() -> Self {
        Self { success: RequestPasswordResetSuccess::PasswordResetRequested }
    }
}

/// Error codes for password reset request endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum RequestPasswordResetError {
    #[serde(rename = "INTERNAL")]
    Internal,
}

/// Password reset request error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct RequestPasswordResetErrorResponse {
    pub error: RequestPasswordResetError,
}

impl Default for RequestPasswordResetErrorResponse {
    fn default() -> Self {
        Self { error: RequestPasswordResetError::Internal }
    }
}

// =============================================================================
// Password reset completion endpoint responses
// =============================================================================

/// Success code for password reset completion endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum CompletePasswordResetSuccess {
    #[serde(rename = "SUCCESS_PASSWORD_RESET_COMPLETED")]
    PasswordResetCompleted,
}

/// Password reset completion success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CompletePasswordResetResponse {
    pub success: CompletePasswordResetSuccess,
}

impl Default for CompletePasswordResetResponse {
    fn default() -> Self {
        Self { success: CompletePasswordResetSuccess::PasswordResetCompleted }
    }
}

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

/// Error codes for counter endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum CounterError {
    #[serde(rename = "INTERNAL")]
    Internal,
}

/// Counter error response (for internal server errors).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CounterErrorResponse {
    pub error: CounterError,
}

impl Default for CounterErrorResponse {
    fn default() -> Self {
        Self { error: CounterError::Internal }
    }
}
