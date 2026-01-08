#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
use rust_i18n::t;
use prost::Message;

pub mod generated;
use generated::v1::{ResponseCode, FieldType, ValidationErrorCode, ValidationErrorData};
use field_validator::{
    USERNAME_CHAR_MIN, USERNAME_CHAR_MAX,
    PASSWORD_CHAR_MIN, PASSWORD_CHAR_MAX,
    PASSWORD_UPPERCASE_MIN, PASSWORD_LOWERCASE_MIN,
    PASSWORD_NUMBER_MIN, PASSWORD_SPECIAL_MIN,
};

// Initialize i18n
rust_i18n::i18n!("locales");

/// Translates a response code into a localized string.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate_response_code(code: i32, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());
    let key = match ResponseCode::try_from(code) {
        Ok(rc) => rc.as_str_name(),
        Err(_) => ResponseCode::Unspecified.as_str_name(),
    };

    t!(key, locale = &locale).to_string()
}

/// Helper function to interpolate values in translation strings.
fn translate_with_params(key: &str, field_type: FieldType, error_code: ValidationErrorCode, locale: &str) -> String {
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
        Err(_) => return t!("RESPONSE_CODE_UNSPECIFIED", locale = &locale).to_string(),
    };

    let field_type = match FieldType::try_from(validation_error.field) {
        Ok(ft) => ft,
        Err(_) => return t!("RESPONSE_CODE_UNSPECIFIED", locale = &locale).to_string(),
    };

    let field_name = field_type.as_str_name();

    let mut error_messages = Vec::new();
    for error_code in validation_error.errors {
        let validation_code = match ValidationErrorCode::try_from(error_code) {
            Ok(vc) => vc,
            Err(_) => continue,
        };

        let code_name = validation_code.as_str_name();
        let translation_key = format!("{}_{}", field_name, code_name);
        let message = translate_with_params(&translation_key, field_type, validation_code, &locale);
        error_messages.push(message);
    }

    error_messages.join(", ")
}

/// Translates a single validation error code for a specific field.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate_field_validation_error(field: i32, error_code: i32, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());

    let field_type = match FieldType::try_from(field) {
        Ok(ft) => ft,
        Err(_) => return t!("RESPONSE_CODE_UNSPECIFIED", locale = &locale).to_string(),
    };

    let validation_code = match ValidationErrorCode::try_from(error_code) {
        Ok(vc) => vc,
        Err(_) => return t!("RESPONSE_CODE_UNSPECIFIED", locale = &locale).to_string(),
    };

    let field_name = field_type.as_str_name();
    let code_name = validation_code.as_str_name();
    let translation_key = format!("{}_{}", field_name, code_name);

    translate_with_params(&translation_key, field_type, validation_code, &locale)
}
