//! API response types.
//!
//! Simple success responses (200 OK) return empty JSON `{}`.
//! The frontend uses translate('SUCCESS_KEY') to display success messages.

use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::{UserRole, ValidationErrorData};

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
    pub role: UserRole,
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
    pub role: UserRole,
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
        Self {
            error: AuthError::InvalidCredentials,
        }
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

// =============================================================================
// Post responses
// =============================================================================

/// Post data returned from API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PostResponse {
    pub post_id: i64,
    pub user_id: i64,
    pub username: String,
    /// Whether the post author's account has been deleted.
    pub is_user_deleted: bool,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub image_url: String,
    pub score: i64,
    pub comment_count: i64,
    /// User's rating on this post: 1, -1, or null if not rated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}

/// Post list response with pagination info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PostListResponse {
    pub posts: Vec<PostResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Post creation success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CreatePostResponse {
    pub post_id: i64,
}

/// Error codes for post operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PostError {
    #[serde(rename = "NOT_FOUND")]
    NotFound,
    #[serde(rename = "VALIDATION")]
    Validation,
    #[serde(rename = "INVALID_IMAGE")]
    InvalidImage,
    #[serde(rename = "FILE_TOO_LARGE")]
    FileTooLarge,
    #[serde(rename = "INTERNAL")]
    Internal,
}

/// Post error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PostErrorResponse {
    pub error: PostError,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationErrorData>,
}

// =============================================================================
// Comment responses
// =============================================================================

/// Comment data returned from API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CommentResponse {
    pub comment_id: i64,
    pub post_id: i64,
    pub user_id: i64,
    pub username: String,
    /// Whether the comment author's account has been deleted.
    pub is_user_deleted: bool,
    pub content: String,
    pub created_at: i64,
}

/// Comment list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CommentListResponse {
    pub comments: Vec<CommentResponse>,
    pub total: i64,
}

/// Comment creation success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentResponse {
    pub comment_id: i64,
}

/// Error codes for comment operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum CommentError {
    #[serde(rename = "NOT_FOUND")]
    NotFound,
    #[serde(rename = "POST_NOT_FOUND")]
    PostNotFound,
    #[serde(rename = "VALIDATION")]
    Validation,
    #[serde(rename = "INTERNAL")]
    Internal,
}

/// Comment error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CommentErrorResponse {
    pub error: CommentError,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationErrorData>,
}

// =============================================================================
// Rating responses
// =============================================================================

/// Rating response showing current score after rating.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct RatingResponse {
    pub score: i64,
    /// User's current rating: 1, -1, or null if removed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
}

/// Error codes for rating operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum RatingError {
    #[serde(rename = "POST_NOT_FOUND")]
    PostNotFound,
    #[serde(rename = "INVALID_VALUE")]
    InvalidValue,
    #[serde(rename = "INTERNAL")]
    Internal,
}

/// Rating error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct RatingErrorResponse {
    pub error: RatingError,
}

// =============================================================================
// Admin responses
// =============================================================================

/// User info for admin endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResponse {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub email_verified: bool,
    pub is_deleted: bool,
}

/// User list response for admin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct UserListResponse {
    pub users: Vec<UserInfoResponse>,
}

/// Deleted post data for admin restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct DeletedPostResponse {
    pub post_id: i64,
    pub user_id: i64,
    pub username: String,
    /// Whether the post author's account has been deleted.
    pub is_user_deleted: bool,
    pub title: String,
    pub deleted_at: i64,
}

/// Deleted posts list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct DeletedPostsListResponse {
    pub posts: Vec<DeletedPostResponse>,
    pub total: i64,
}
