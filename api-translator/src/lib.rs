#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
use rust_i18n::t;

pub mod generated;
use generated::v1::ResponseCode;

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

/// Translates a validation error into a localized string.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn translate_validation_error(error_json: &str, locale: Option<String>) -> String {
    let locale = locale.unwrap_or_else(|| "en".to_string());
    let val: serde_json::Value = match serde_json::from_str(error_json) {
        Ok(v) => v,
        Err(_) => return t!("RESPONSE_CODE_UNSPECIFIED", locale = &locale).to_string(),
    };

    let key = match val.get("type") {
        Some(serde_json::Value::String(s)) => s.as_str(),
        _ => return t!("RESPONSE_CODE_UNSPECIFIED", locale = &locale).to_string(),
    };

    if let Some(params) = val.get("params") {
        if let Some(obj) = params.as_object() {
            let mut translation = t!(key, locale = &locale).to_string();
            for (k, v) in obj {
                let placeholder = format!("%{{{}}}", k);
                let val_str = match v {
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                };
                translation = translation.replace(&placeholder, &val_str);
            }
            return translation;
        }
    }

    t!(key, locale = &locale).to_string()
}
