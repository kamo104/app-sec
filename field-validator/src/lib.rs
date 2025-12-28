//! Field validation library that can be used in both Rust backend and WebAssembly frontend.
//!
//! This library provides consistent validation rules for all fields across platforms.
//! It supports validation for usernames, emails, passwords, and other fields.

use std::fmt;

pub mod generated;

pub const SCORE_WEAK_MAX: u32 = 3;
pub const SCORE_MEDIUM_MAX: u32 = 5;

/// Represents the type of field being validated.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FieldType {
    Username,
    Email,
    Password,
    Generic,
}

/// Represents a specific validation error.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type", content = "params")]
pub enum ValidationError {
    // Username errors
    UsernameRequired,
    UsernameTooShort { min: usize },
    UsernameTooLong { max: usize },
    UsernameInvalidCharacters,

    // Email errors
    EmailRequired,
    EmailInvalidFormat,
    EmailMissingAt,
    EmailInvalidDomainCharacters,
    EmailMissingDot,

    // Password errors
    PasswordRequired,
    PasswordTooShort { min: usize },
    PasswordNoUppercase,
    PasswordNoLowercase,
    PasswordNoNumber,
    PasswordNoSpecialChar,

    // Generic
    FieldRequired,
}

/// Represents the result of a field validation check.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

impl FieldValidationResult {
    pub fn new(is_valid: bool, errors: Vec<ValidationError>) -> Self {
        Self { is_valid, errors }
    }

    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }
}

impl fmt::Display for FieldValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid {
            write!(f, "Valid")
        } else {
            write!(f, "Invalid: {:?}", self.errors)
        }
    }
}

/// Validates a username.
///
/// # Rules
/// - Must not be empty
/// - Must be between 3 and 20 characters (optional, based on parameters)
/// - Can only contain letters, numbers, and underscores
///
/// # Parameters
/// - `username`: The username to validate
/// - `min_length`: Minimum length (default: 3)
/// - `max_length`: Maximum length (default: 20)
/// - `validate_length`: Whether to validate length (default: true)
pub fn validate_username(
    username: &str,
    min_length: usize,
    max_length: usize,
    validate_length: bool,
) -> FieldValidationResult {
    let mut errors = Vec::new();

    if username.is_empty() {
        return FieldValidationResult::invalid(vec![ValidationError::UsernameRequired]);
    }

    if validate_length {
        if username.len() < min_length {
            errors.push(ValidationError::UsernameTooShort { min: min_length });
        }
        if username.len() > max_length {
            errors.push(ValidationError::UsernameTooLong { max: max_length });
        }
    }

    // Check for valid characters (letters, numbers, underscores)
    if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        errors.push(ValidationError::UsernameInvalidCharacters);
    }

    if errors.is_empty() {
        FieldValidationResult::valid()
    } else {
        FieldValidationResult::invalid(errors)
    }
}

/// Validates an email address.
///
/// # Rules
/// - Must not be empty
/// - Must contain @ symbol
/// - Must have a domain part
pub fn validate_email(email: &str) -> FieldValidationResult {
    let mut errors = Vec::new();

    if email.is_empty() {
        return FieldValidationResult::invalid(vec![ValidationError::EmailRequired]);
    }

    // Basic email validation
    if !email.contains('@') {
        errors.push(ValidationError::EmailMissingAt);
    } else {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            errors.push(ValidationError::EmailInvalidFormat);
        }

        // Check for valid domain characters
        if let Some(domain) = parts.get(1) {
            if !domain.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_') {
                errors.push(ValidationError::EmailInvalidDomainCharacters);
            }
            if !domain.contains('.') {
                errors.push(ValidationError::EmailMissingDot);
            }
        }
    }

    if errors.is_empty() {
        FieldValidationResult::valid()
    } else {
        FieldValidationResult::invalid(errors)
    }
}

/// Validates a password according to security best practices.
///
/// # Rules
/// - At least 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one number
/// - At least one special character
pub fn validate_password(password: &str) -> FieldValidationResult {
    let mut errors = Vec::new();

    if password.is_empty() {
        return FieldValidationResult::invalid(vec![ValidationError::PasswordRequired]);
    }

    // Check length
    if password.len() < 8 {
        errors.push(ValidationError::PasswordTooShort { min: 8 });
    }

    // Check for uppercase letter
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        errors.push(ValidationError::PasswordNoUppercase);
    }

    // Check for lowercase letter
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        errors.push(ValidationError::PasswordNoLowercase);
    }

    // Check for number
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push(ValidationError::PasswordNoNumber);
    }

    // Check for special character
    if !password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        errors.push(ValidationError::PasswordNoSpecialChar);
    }

    if errors.is_empty() {
        FieldValidationResult::valid()
    } else {
        FieldValidationResult::invalid(errors)
    }
}

/// Validates a field based on its type.
///
/// # Parameters
/// - `field_type`: The type of field to validate
/// - `value`: The value to validate
/// - `min_length`: Minimum length (used for username, optional)
/// - `max_length`: Maximum length (used for username, optional)
/// - `validate_length`: Whether to validate length (used for username, optional)
pub fn validate_field(
    field_type: FieldType,
    value: &str,
    min_length: Option<usize>,
    max_length: Option<usize>,
    validate_length: Option<bool>,
) -> FieldValidationResult {
    match field_type {
        FieldType::Username => validate_username(
            value,
            min_length.unwrap_or(3),
            max_length.unwrap_or(20),
            validate_length.unwrap_or(true),
        ),
        FieldType::Email => validate_email(value),
        FieldType::Password => validate_password(value),
        FieldType::Generic => {
            if value.is_empty() {
                FieldValidationResult::invalid(vec![ValidationError::FieldRequired])
            } else {
                FieldValidationResult::valid()
            }
        }
    }
}

/// Represents password strength level
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
}

/// Detailed password validation result with strength information
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PasswordValidationDetailed {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub strength: PasswordStrength,
    pub score: u32,
}

impl PasswordValidationDetailed {
    pub fn new(is_valid: bool, errors: Vec<ValidationError>, strength: PasswordStrength, score: u32) -> Self {
        Self {
            is_valid,
            errors,
            strength,
            score,
        }
    }
}

/// Validates password and returns detailed strength information
pub fn validate_password_detailed(password: &str) -> PasswordValidationDetailed {
    let basic_result = validate_password(password);

    // Calculate score based on various factors
    let mut score = 0;

    if password.len() >= 8 { score += 1; }
    if password.len() >= 12 { score += 1; }
    if password.chars().any(|c| c.is_ascii_uppercase()) { score += 1; }
    if password.chars().any(|c| c.is_ascii_lowercase()) { score += 1; }
    if password.chars().any(|c| c.is_ascii_digit()) { score += 1; }
    if password.chars().any(|c| !c.is_ascii_alphanumeric()) { score += 1; }
    if password.len() >= 16 { score += 1; }

    let strength = if score <= SCORE_WEAK_MAX {
        PasswordStrength::Weak
    } else if score <= SCORE_MEDIUM_MAX {
        PasswordStrength::Medium
    } else {
        PasswordStrength::Strong
    };

    PasswordValidationDetailed::new(
        basic_result.is_valid,
        basic_result.errors,
        strength,
        score,
    )
}

/// WebAssembly-compatible interface for field validation.
/// This function is compiled to WebAssembly and can be called from JavaScript.
#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    /// Validates a field and returns a JSON string with the result.
    /// This is the WebAssembly interface function.
    #[wasm_bindgen]
    pub fn validate_field_wasm(
        field_type: &str,
        value: &str,
        min_length: usize,
        max_length: usize,
        validate_length: bool,
    ) -> String {
        let field_type = match field_type {
            "username" => FieldType::Username,
            "email" => FieldType::Email,
            "password" => FieldType::Password,
            _ => FieldType::Generic,
        };

        let result = validate_field(field_type, value, Some(min_length), Some(max_length), Some(validate_length));
        serde_json::to_string(&result).unwrap_or_else(|_| {
            serde_json::to_string(&FieldValidationResult::invalid(vec![
                ValidationError::FieldRequired
            ])).unwrap()
        })
    }

    /// Validates a username and returns a JSON string with the result.
    #[wasm_bindgen]
    pub fn validate_username_wasm(username: &str, min_length: usize, max_length: usize, validate_length: bool) -> String {
        let result = validate_username(username, min_length, max_length, validate_length);
        serde_json::to_string(&result).unwrap_or_else(|_| {
            serde_json::to_string(&FieldValidationResult::invalid(vec![
                ValidationError::UsernameRequired
            ])).unwrap()
        })
    }

    /// Validates an email and returns a JSON string with the result.
    #[wasm_bindgen]
    pub fn validate_email_wasm(email: &str) -> String {
        let result = validate_email(email);
        serde_json::to_string(&result).unwrap_or_else(|_| {
            serde_json::to_string(&FieldValidationResult::invalid(vec![
                ValidationError::EmailRequired
            ])).unwrap()
        })
    }

    /// Validates a password and returns a JSON string with the result.
    #[wasm_bindgen]
    pub fn validate_password_wasm(password: &str) -> String {
        let result = validate_password(password);
        serde_json::to_string(&result).unwrap_or_else(|_| {
            serde_json::to_string(&FieldValidationResult::invalid(vec![
                ValidationError::PasswordRequired
            ])).unwrap()
        })
    }

    /// Validates password and returns a boolean indicating validity.
    #[wasm_bindgen]
    pub fn is_password_valid(password: &str) -> bool {
        validate_password(password).is_valid
    }

    /// Validates password and returns errors as a JSON array string.
    #[wasm_bindgen]
    pub fn get_password_errors(password: &str) -> String {
        let result = validate_password(password);
        serde_json::to_string(&result.errors).unwrap_or_default()
    }

    /// Validates password and returns detailed strength information as JSON.
    #[wasm_bindgen]
    pub fn get_password_strength(password: &str) -> String {
        let result = validate_password_detailed(password);
        serde_json::to_string(&result).unwrap_or_else(|_| {
            serde_json::to_string(&PasswordValidationDetailed::new(
                false,
                vec![ValidationError::PasswordRequired],
                PasswordStrength::Weak,
                0,
            )).unwrap()
        })
    }

    /// Returns the strength level as an integer matching the protobuf definition.
    #[wasm_bindgen]
    pub fn get_password_strength_class(score: u32) -> i32 {
        if score <= SCORE_WEAK_MAX {
            generated::v1::PasswordStrength::Weak as i32
        } else if score <= SCORE_MEDIUM_MAX {
            generated::v1::PasswordStrength::Medium as i32
        } else {
            generated::v1::PasswordStrength::Strong as i32
        }
    }

    /// Validates username and returns a boolean indicating validity.
    #[wasm_bindgen]
    pub fn is_username_valid(username: &str, min_length: usize, max_length: usize, validate_length: bool) -> bool {
        validate_username(username, min_length, max_length, validate_length).is_valid
    }

    /// Validates email and returns a boolean indicating validity.
    #[wasm_bindgen]
    pub fn is_email_valid(email: &str) -> bool {
        validate_email(email).is_valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Username tests
    #[test]
    fn test_empty_username() {
        let result = validate_username("", 3, 20, true);
        assert!(!result.is_valid);
        assert_eq!(result.errors, vec![ValidationError::UsernameRequired]);
    }

    #[test]
    fn test_short_username() {
        let result = validate_username("ab", 3, 20, true);
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::UsernameTooShort { min: 3 }));
    }

    #[test]
    fn test_long_username() {
        let result = validate_username("thisusernameistoolong", 3, 10, true);
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::UsernameTooLong { max: 10 }));
    }

    #[test]
    fn test_invalid_username_chars() {
        let result = validate_username("user@name", 3, 20, true);
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::UsernameInvalidCharacters));
    }

    #[test]
    fn test_valid_username() {
        let result = validate_username("user_123", 3, 20, true);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_username_no_length_validation() {
        let result = validate_username("ab", 3, 20, false);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    // Email tests
    #[test]
    fn test_empty_email() {
        let result = validate_email("");
        assert!(!result.is_valid);
        assert_eq!(result.errors, vec![ValidationError::EmailRequired]);
    }

    #[test]
    fn test_invalid_email_no_at() {
        let result = validate_email("userdomain.com");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::EmailMissingAt));
    }

    #[test]
    fn test_invalid_email_format() {
        let result = validate_email("@domain.com");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::EmailInvalidFormat));
    }

    #[test]
    fn test_valid_email() {
        let result = validate_email("user@example.com");
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    // Password tests
    #[test]
    fn test_empty_password() {
        let result = validate_password("");
        assert!(!result.is_valid);
        assert_eq!(result.errors, vec![ValidationError::PasswordRequired]);
    }

    #[test]
    fn test_short_password() {
        let result = validate_password("Short1!");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::PasswordTooShort { min: 8 }));
    }

    #[test]
    fn test_no_uppercase() {
        let result = validate_password("password123!");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::PasswordNoUppercase));
    }

    #[test]
    fn test_no_lowercase() {
        let result = validate_password("PASSWORD123!");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::PasswordNoLowercase));
    }

    #[test]
    fn test_no_number() {
        let result = validate_password("Password!");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::PasswordNoNumber));
    }

    #[test]
    fn test_no_special_char() {
        let result = validate_password("Password123");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::PasswordNoSpecialChar));
    }

    #[test]
    fn test_valid_password() {
        let result = validate_password("StrongPass123!");
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    // Generic field tests
    #[test]
    fn test_generic_field_empty() {
        let result = validate_field(FieldType::Generic, "", None, None, None);
        assert!(!result.is_valid);
        assert_eq!(result.errors, vec![ValidationError::FieldRequired]);
    }

    #[test]
    fn test_generic_field_valid() {
        let result = validate_field(FieldType::Generic, "anything", None, None, None);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    // Password strength tests
    #[test]
    fn test_password_strength_detailed() {
        // Weak password
        let result = validate_password_detailed("abc");
        assert!(!result.is_valid);
        assert_eq!(result.strength, PasswordStrength::Weak);
        assert_eq!(result.score, 1);

        // Medium password
        let result = validate_password_detailed("Password123");
        assert!(!result.is_valid);
        assert_eq!(result.strength, PasswordStrength::Medium);
        assert_eq!(result.score, 4);

        // Strong password
        let result = validate_password_detailed("StrongPass123!");
        assert!(result.is_valid);
        assert_eq!(result.strength, PasswordStrength::Strong);
        assert_eq!(result.score, 6);
    }
}
