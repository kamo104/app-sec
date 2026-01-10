//! Translation library for API responses and validation errors.
//! Can be used in both Rust backend and WebAssembly frontend.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
use rust_i18n::t;

use api_types::{
    FieldType, ValidationErrorCode, ValidationErrorData, SuccessCode, ErrorCode,
};
use field_validator::{
    USERNAME_CHAR_MIN, USERNAME_CHAR_MAX,
    PASSWORD_CHAR_MIN, PASSWORD_CHAR_MAX,
    PASSWORD_UPPERCASE_MIN, PASSWORD_LOWERCASE_MIN,
    PASSWORD_NUMBER_MIN, PASSWORD_SPECIAL_MIN,
};

// Initialize i18n
rust_i18n::i18n!("locales");

/// Translates an arbitrary key with optional locale.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate(key: &str, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());
    t!(key, locale = &locale).to_string()
}

/// Translates a success code string to a localized message.
/// Takes the success code as a string (e.g., "SUCCESS_LOGGED_IN").
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate_success_code(code: &str, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());
    t!(code, locale = &locale).to_string()
}

/// Translates an error code string to a localized message.
/// Takes the error code as a string (e.g., "INVALID_CREDENTIALS").
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate_error_code(code: &str, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());
    t!(code, locale = &locale).to_string()
}

/// Translates a SuccessCode enum to a localized message.
pub fn translate_success(code: SuccessCode, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());
    let key = code.as_str_name();
    t!(key, locale = &locale).to_string()
}

/// Translates an ErrorCode enum to a localized message.
pub fn translate_error(code: ErrorCode, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());
    let key = code.as_str_name();
    t!(key, locale = &locale).to_string()
}

/// Helper function to interpolate values in translation strings.
fn translate_validation_error_with_params(key: &str, field_type: FieldType, error_code: ValidationErrorCode, locale: &str) -> String {
    match (field_type, error_code) {
        (FieldType::Username, ValidationErrorCode::TooShort) => {
            t!(key, locale = locale, min = USERNAME_CHAR_MIN).to_string()
        }
        (FieldType::Username, ValidationErrorCode::TooLong) => {
            t!(key, locale = locale, max = USERNAME_CHAR_MAX).to_string()
        }
        (FieldType::Password, ValidationErrorCode::TooShort) => {
            t!(key, locale = locale, min = PASSWORD_CHAR_MIN).to_string()
        }
        (FieldType::Password, ValidationErrorCode::TooLong) => {
            t!(key, locale = locale, max = PASSWORD_CHAR_MAX).to_string()
        }
        (FieldType::Password, ValidationErrorCode::TooFewUppercaseLetters) => {
            t!(key, locale = locale, min = PASSWORD_UPPERCASE_MIN).to_string()
        }
        (FieldType::Password, ValidationErrorCode::TooFewLowercaseLetters) => {
            t!(key, locale = locale, min = PASSWORD_LOWERCASE_MIN).to_string()
        }
        (FieldType::Password, ValidationErrorCode::TooFewDigits) => {
            t!(key, locale = locale, min = PASSWORD_NUMBER_MIN).to_string()
        }
        (FieldType::Password, ValidationErrorCode::TooFewSpecialCharacters) => {
            t!(key, locale = locale, min = PASSWORD_SPECIAL_MIN).to_string()
        }
        _ => t!(key, locale = locale).to_string(),
    }
}

/// Translates validation error data JSON into a localized string.
/// Takes JSON-encoded ValidationErrorData and returns a human-readable error message.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate_validation_error(validation_error_json: &str, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());

    let validation_error: ValidationErrorData = match serde_json::from_str(validation_error_json) {
        Ok(v) => v,
        Err(_) => return t!("INTERNAL", locale = &locale).to_string(),
    };

    let mut all_error_messages = Vec::new();

    for field_error in validation_error.field_errors {
        let field_type = field_error.field;

        for error_code in field_error.errors {
            let message = translate_field_validation_error_internal(field_type, error_code, &locale);
            all_error_messages.push(message);
        }
    }

    all_error_messages.join(", ")
}

/// Internal function for translating a single field validation error.
fn translate_field_validation_error_internal(field_type: FieldType, validation_code: ValidationErrorCode, locale: &str) -> String {
    let field_name = field_type.as_str_name();
    let code_name = validation_code.as_str_name();
    let translation_key = format!("{}_{}", field_name, code_name);
    translate_validation_error_with_params(&translation_key, field_type, validation_code, locale)
}

/// Translates a single validation error code for a specific field.
/// This is a public helper function for translating individual field errors.
/// Takes field and error_code as string names.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate_field_validation_error(field: &str, error_code: &str, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());

    let field_type = match FieldType::from_str_name(field) {
        Some(ft) => ft,
        None => return t!("INTERNAL", locale = &locale).to_string(),
    };

    let validation_code = match ValidationErrorCode::from_str_name(error_code) {
        Some(vc) => vc,
        None => return t!("INTERNAL", locale = &locale).to_string(),
    };

    translate_field_validation_error_internal(field_type, validation_code, &locale)
}
