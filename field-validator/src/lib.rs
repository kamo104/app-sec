//! Field validation library that can be used in both Rust backend and WebAssembly frontend.
//!
//! This library provides consistent validation rules for all fields across platforms.
//! It supports validation for usernames, emails, passwords, and other fields.

use lettre::Address;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// Re-export types from api-types for WASM consumers
pub use api_types::{
    FieldType, PasswordStrength, ValidationDetailedPasswordData, ValidationErrorCode,
    ValidationFieldError,
};

pub const USERNAME_CHAR_MIN: usize = 3;
pub const USERNAME_CHAR_MAX: usize = 20;

pub const PASSWORD_CHAR_MIN: usize = 8;
pub const PASSWORD_CHAR_MAX: usize = 64;
pub const PASSWORD_UPPERCASE_MIN: usize = 1;
pub const PASSWORD_LOWERCASE_MIN: usize = 1;
pub const PASSWORD_NUMBER_MIN: usize = 1;
pub const PASSWORD_SPECIAL_MIN: usize = 1;

pub const PASSWORD_SCORE_WEAK_MAX: u32 = 3;
pub const PASSWORD_SCORE_MEDIUM_MAX: u32 = 5;
pub const PASSWORD_SCORE_STRONG_MAX: u32 = 6;

pub const POST_TITLE_CHAR_MIN: usize = 1;
pub const POST_TITLE_CHAR_MAX: usize = 100;

pub const POST_DESCRIPTION_CHAR_MAX: usize = 500;

pub const COMMENT_CONTENT_CHAR_MIN: usize = 1;
pub const COMMENT_CONTENT_CHAR_MAX: usize = 1000;

/// Validates a username.
///
/// # Rules
/// - Must be between USERNAME_CHAR_MIN and USERNAME_CHAR_MAX characters
/// - Must be a printable UTF-8 character
///
/// # Parameters
/// - `username`: The username to validate
pub fn validate_username(username: &str) -> ValidationFieldError {
    let mut ret = ValidationFieldError::new(FieldType::Username);

    if username.len() < USERNAME_CHAR_MIN {
        ret.add_error(ValidationErrorCode::TooShort);
    }
    if username.len() > USERNAME_CHAR_MAX {
        ret.add_error(ValidationErrorCode::TooLong);
    }

    // Check for valid characters (printable UTF-8)
    if !username.chars().all(|c| !c.is_control()) {
        ret.add_error(ValidationErrorCode::InvalidCharacters);
    }
    ret
}

/// Validates an email address.
///
/// # Rules
/// - Must not be empty
/// - Must be of a valid format
pub fn validate_email(email: &str) -> ValidationFieldError {
    let mut ret = ValidationFieldError::new(FieldType::Email);
    if email.is_empty() {
        ret.add_error(ValidationErrorCode::Required);
        return ret;
    }
    let address = email.parse::<Address>();
    if address.is_ok() {
        return ret;
    }
    ret.add_error(ValidationErrorCode::InvalidFormat);
    ret
}

/// Validates a password according to security best practices.
///
/// # Rules
/// - At least PASSWORD_CHAR_MIN characters
/// - At most PASSWORD_CHAR_MAX characters
/// - At least PASSWORD_UPPERCASE_MIN uppercase letter
/// - At least PASSWORD_LOWERCASE_MIN lowercase letter
/// - At least PASSWORD_NUMBER_MIN number
/// - At least PASSWORD_SPECIAL_MIN special character
pub fn validate_password(password: &str) -> ValidationFieldError {
    let mut ret = ValidationFieldError::new(FieldType::Password);

    // Check length
    if password.len() < PASSWORD_CHAR_MIN {
        ret.add_error(ValidationErrorCode::TooShort);
    }
    if password.len() > PASSWORD_CHAR_MAX {
        ret.add_error(ValidationErrorCode::TooLong);
    }

    // Check for uppercase letter
    if password.chars().filter(|c| c.is_uppercase()).count() < PASSWORD_UPPERCASE_MIN {
        ret.add_error(ValidationErrorCode::TooFewUppercaseLetters);
    }

    // Check for lowercase letter
    if password.chars().filter(|c| c.is_lowercase()).count() < PASSWORD_LOWERCASE_MIN {
        ret.add_error(ValidationErrorCode::TooFewLowercaseLetters);
    }

    // Check for number
    if password.chars().filter(|c| c.is_numeric()).count() < PASSWORD_NUMBER_MIN {
        ret.add_error(ValidationErrorCode::TooFewDigits);
    }

    // Check for special character
    if password.chars().filter(|c| !c.is_alphanumeric()).count() < PASSWORD_SPECIAL_MIN {
        ret.add_error(ValidationErrorCode::TooFewSpecialCharacters);
    }
    ret
}

/// Validates a post title.
///
/// # Rules
/// - Must be between POST_TITLE_CHAR_MIN and POST_TITLE_CHAR_MAX characters
/// - Must not contain control characters
pub fn validate_post_title(title: &str) -> ValidationFieldError {
    let mut ret = ValidationFieldError::new(FieldType::PostTitle);

    let trimmed = title.trim();
    if trimmed.len() < POST_TITLE_CHAR_MIN {
        ret.add_error(ValidationErrorCode::TooShort);
    }
    if trimmed.len() > POST_TITLE_CHAR_MAX {
        ret.add_error(ValidationErrorCode::TooLong);
    }
    if !trimmed.chars().all(|c| !c.is_control()) {
        ret.add_error(ValidationErrorCode::InvalidCharacters);
    }
    ret
}

/// Validates a post description.
///
/// # Rules
/// - At most POST_DESCRIPTION_CHAR_MAX characters
/// - Must not contain control characters (except newlines)
pub fn validate_post_description(description: &str) -> ValidationFieldError {
    let mut ret = ValidationFieldError::new(FieldType::PostDescription);

    if description.len() > POST_DESCRIPTION_CHAR_MAX {
        ret.add_error(ValidationErrorCode::TooLong);
    }
    // Allow newlines but not other control characters
    if !description
        .chars()
        .all(|c| !c.is_control() || c == '\n' || c == '\r')
    {
        ret.add_error(ValidationErrorCode::InvalidCharacters);
    }
    ret
}

/// Validates comment content.
///
/// # Rules
/// - Must be between COMMENT_CONTENT_CHAR_MIN and COMMENT_CONTENT_CHAR_MAX characters
/// - Must not contain control characters (except newlines)
pub fn validate_comment_content(content: &str) -> ValidationFieldError {
    let mut ret = ValidationFieldError::new(FieldType::CommentContent);

    let trimmed = content.trim();
    if trimmed.len() < COMMENT_CONTENT_CHAR_MIN {
        ret.add_error(ValidationErrorCode::TooShort);
    }
    if trimmed.len() > COMMENT_CONTENT_CHAR_MAX {
        ret.add_error(ValidationErrorCode::TooLong);
    }
    // Allow newlines but not other control characters
    if !trimmed
        .chars()
        .all(|c| !c.is_control() || c == '\n' || c == '\r')
    {
        ret.add_error(ValidationErrorCode::InvalidCharacters);
    }
    ret
}

/// Validates a field based on its type.
///
/// # Parameters
/// - `field_type`: The type of field to validate (FieldType as a string)
/// - `value`: The value to validate
/// # Returns
/// - JSON string of `ValidationFieldError`
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn validate_field(field_type: &str, value: &str) -> String {
    let result = match FieldType::from_str_name(field_type) {
        Some(field_type) => match field_type {
            FieldType::Username => validate_username(value),
            FieldType::Email => validate_email(value),
            FieldType::Password => validate_password(value),
            FieldType::PostTitle => validate_post_title(value),
            FieldType::PostDescription => validate_post_description(value),
            FieldType::CommentContent => validate_comment_content(value),
            FieldType::Unspecified => ValidationFieldError::new(FieldType::Unspecified),
        },
        None => ValidationFieldError::new(FieldType::Unspecified),
    };
    serde_json::to_string(&result).unwrap_or_default()
}

/// Validates password and returns detailed strength information
/// # Returns
/// - JSON string of `ValidationDetailedPasswordData`
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn validate_password_detailed(password: &str) -> String {
    let validation_errors = validate_password(password);

    // Calculate score based on various factors
    let mut score = 0u32;

    let len = password.len();

    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_special = false;

    for c in password.chars() {
        has_upper |= c.is_uppercase();
        has_lower |= c.is_lowercase();
        has_digit |= c.is_numeric();
        has_special |= !c.is_alphanumeric();
    }

    score += (len >= 8) as u32;
    score += (len >= 12) as u32;
    score += (len >= 16) as u32;
    score += has_upper as u32;
    score += has_lower as u32;
    score += has_digit as u32;
    score += has_special as u32;

    let strength = match score {
        ..=PASSWORD_SCORE_WEAK_MAX => PasswordStrength::Weak,
        ..=PASSWORD_SCORE_MEDIUM_MAX => PasswordStrength::Medium,
        ..=PASSWORD_SCORE_STRONG_MAX => PasswordStrength::Strong,
        _ => PasswordStrength::Cia,
    };
    let ret = ValidationDetailedPasswordData::new(validation_errors, strength, score);
    serde_json::to_string(&ret).unwrap_or_default()
}
