//! Field validation library that can be used in both Rust backend and WebAssembly frontend.
//!
//! This library provides consistent validation rules for all fields across platforms.
//! It supports validation for usernames, emails, passwords, and other fields.

use lettre::Address;
use prost::Message;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub mod generated;
use generated::v1::{
    FieldType, PasswordStrength, ValidationDetailedPasswordData, ValidationErrorCode,
    ValidationErrorData,
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

/// Validates a username.
///
/// # Rules
/// - Must be between USERNAME_CHAR_MIN and USERNAME_CHAR_MAX characters
/// - Must be a printable UTF-8 character
///
/// # Parameters
/// - `username`: The username to validate
pub fn validate_username(username: &str) -> ValidationErrorData {
    let mut ret = ValidationErrorData {
        field: FieldType::Username as i32,
        errors: Vec::new(),
    };

    if username.len() < USERNAME_CHAR_MIN {
        ret.errors.push(ValidationErrorCode::TooShort as i32);
    }
    if username.len() > USERNAME_CHAR_MAX {
        ret.errors.push(ValidationErrorCode::TooLong as i32);
    }

    // Check for valid characters (printable UTF-8)
    if !username.chars().all(|c| !c.is_control()) {
        ret.errors
            .push(ValidationErrorCode::InvalidCharacters as i32);
    }
    return ret;
}

/// Validates an email address.
///
/// # Rules
/// - Must not be empty
/// - Must be of a valid format
pub fn validate_email(email: &str) -> ValidationErrorData {
    let mut ret = ValidationErrorData {
        field: FieldType::Email as i32,
        errors: Vec::new(),
    };
    if email.is_empty() {
        ret.errors.push(ValidationErrorCode::Required as i32);
        return ret;
    }
    let address = email.parse::<Address>();
    if address.is_ok() {
        return ret;
    }
    ret.errors.push(ValidationErrorCode::InvalidFormat as i32);
    return ret;
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
pub fn validate_password(password: &str) -> ValidationErrorData {
    let mut ret = ValidationErrorData {
        field: FieldType::Password as i32,
        errors: Vec::new(),
    };

    // Check length
    if password.len() < PASSWORD_CHAR_MIN {
        ret.errors.push(ValidationErrorCode::TooShort as i32)
    }
    if password.len() > PASSWORD_CHAR_MAX {
        ret.errors.push(ValidationErrorCode::TooLong as i32)
    }

    // Check for uppercase letter
    if password.chars().filter(|c| c.is_uppercase()).count() < PASSWORD_UPPERCASE_MIN {
        ret.errors
            .push(ValidationErrorCode::TooFewUppercaseLetters as i32);
    }

    // Check for lowercase letter
    if password.chars().filter(|c| c.is_lowercase()).count() < PASSWORD_LOWERCASE_MIN {
        ret.errors
            .push(ValidationErrorCode::TooFewLowercaseLetters as i32);
    }

    // Check for number
    if password.chars().filter(|c| c.is_numeric()).count() < PASSWORD_NUMBER_MIN {
        ret.errors.push(ValidationErrorCode::TooFewDigits as i32);
    }

    // Check for special character
    if password.chars().filter(|c| !c.is_alphanumeric()).count() < PASSWORD_SPECIAL_MIN {
        ret.errors
            .push(ValidationErrorCode::TooFewSpecialCharacters as i32);
    }
    return ret;
}

/// Validates a field based on its type.
///
/// # Parameters
/// - `field_type`: The type of field to validate (FieldType encoded as a string)
/// - `value`: The value to validate
/// # Returns
/// - `ValidationErrorData` encoded as bytes
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn validate_field(field_type: &str, value: &str) -> Vec<u8> {
    match FieldType::from_str_name(field_type) {
        Some(field_type) => match field_type {
            FieldType::Username => validate_username(value).encode_to_vec(),
            FieldType::Email => validate_email(value).encode_to_vec(),
            FieldType::Password => validate_password(value).encode_to_vec(),
            FieldType::Unspecified => todo!(),
        },
        None => todo!(),
    }
}

/// Validates password and returns detailed strength information
/// # Returns
/// - `ValidationDetailedPasswordData` encoded as bytes
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn validate_password_detailed(password: &str) -> Vec<u8> {
    let validation_errors = validate_password(password);

    // Calculate score based on various factors
    let mut score = 0;

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
    let ret = ValidationDetailedPasswordData {
        data: Some(validation_errors),
        strength: strength as i32,
        score: score,
    };
    return ret.encode_to_vec();
}
