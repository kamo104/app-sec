#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
use rust_i18n::t;
use prost::Message;

use proto_types::v1::{SuccessCode, ErrorCode, FieldType, ValidationErrorCode, ValidationErrorData, ApiResponse, ResponseCode};
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

/// Translates an ApiResponse to a localized string.
/// Automatically determines if it's a success or error code.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate_response(response_bytes: &[u8], locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());

    let response = match ApiResponse::decode(response_bytes) {
        Ok(r) => r,
        Err(_) => return t!("INTERNAL", locale = &locale).to_string(),
    };

    match ResponseCode::try_from(response.code) {
        Ok(ResponseCode::Success) => {
            if let Some(proto_types::v1::api_response::Detail::Success(success_code)) = response.detail {
                let key = match SuccessCode::try_from(success_code) {
                    Ok(sc) => sc.as_str_name(),
                    Err(_) => SuccessCode::Unspecified.as_str_name(),
                };
                t!(key, locale = &locale).to_string()
            } else {
                t!("SUCCESS_OK", locale = &locale).to_string()
            }
        }
        Ok(ResponseCode::Error) => {
            if let Some(proto_types::v1::api_response::Detail::Error(error_code)) = response.detail {
                let key = match ErrorCode::try_from(error_code) {
                    Ok(ec) => ec.as_str_name(),
                    Err(_) => ErrorCode::Unspecified.as_str_name(),
                };
                t!(key, locale = &locale).to_string()
            } else {
                t!("INTERNAL", locale = &locale).to_string()
            }
        }
        _ => t!("INTERNAL", locale = &locale).to_string(),
    }
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

/// Translates validation error data into a localized string.
/// Takes protobuf-encoded ValidationErrorData and returns a human-readable error message.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate_validation_error(validation_error_bytes: &[u8], locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());

    let validation_error = match ValidationErrorData::decode(validation_error_bytes) {
        Ok(v) => v,
        Err(_) => return t!("INTERNAL", locale = &locale).to_string(),
    };

    let mut all_error_messages = Vec::new();

    for field_error in validation_error.field_errors {
        let field_type = match FieldType::try_from(field_error.field) {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        for error_code in field_error.errors {
            let validation_code = match ValidationErrorCode::try_from(error_code) {
                Ok(vc) => vc,
                Err(_) => continue,
            };

            let message = translate_field_validation_error(field_error.field, error_code, Some(locale.clone()));
            all_error_messages.push(message);
        }
    }

    all_error_messages.join(", ")
}

/// Translates a single validation error code for a specific field.
/// This is a public helper function for translating individual field errors.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate_field_validation_error(field: i32, error_code: i32, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());

    let field_type = match FieldType::try_from(field) {
        Ok(ft) => ft,
        Err(_) => return t!("INTERNAL", locale = &locale).to_string(),
    };

    let validation_code = match ValidationErrorCode::try_from(error_code) {
        Ok(vc) => vc,
        Err(_) => return t!("INTERNAL", locale = &locale).to_string(),
    };

    let field_name = field_type.as_str_name();
    let code_name = validation_code.as_str_name();
    let translation_key = format!("{}_{}", field_name, code_name);
    translate_validation_error_with_params(&translation_key, field_type, validation_code, &locale)
}
