//! Password validation library that can be used in both Rust backend and WebAssembly frontend.
//!
//! This library provides password strength validation with consistent rules across platforms.

use std::fmt;

/// Represents the result of a password validation check.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PasswordValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl PasswordValidationResult {
    pub fn new(is_valid: bool, errors: Vec<String>) -> Self {
        Self { is_valid, errors }
    }

    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }
}

impl fmt::Display for PasswordValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid {
            write!(f, "Valid")
        } else {
            write!(f, "Invalid: {}", self.errors.join(", "))
        }
    }
}

/// Validates password strength according to security best practices.
///
/// # Rules
/// - At least 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one number
/// - At least one special character
///
/// # Examples
///
/// ```
/// use password_validator::{validate_password, PasswordValidationResult};
///
/// let result = validate_password("StrongPass123!");
/// assert!(result.is_valid);
/// ```
pub fn validate_password(password: &str) -> PasswordValidationResult {
    let mut errors = Vec::new();

    if password.is_empty() {
        return PasswordValidationResult::invalid(vec!["Password is required".to_string()]);
    }

    // Check length
    if password.len() < 8 {
        errors.push("Password must be at least 8 characters".to_string());
    }

    // Check for uppercase letter
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        errors.push("Password must contain at least one uppercase letter".to_string());
    }

    // Check for lowercase letter
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        errors.push("Password must contain at least one lowercase letter".to_string());
    }

    // Check for number
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("Password must contain at least one number".to_string());
    }

    // Check for special character
    if !password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        errors.push("Password must contain at least one special character".to_string());
    }

    if errors.is_empty() {
        PasswordValidationResult::valid()
    } else {
        PasswordValidationResult::invalid(errors)
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
    pub errors: Vec<String>,
    pub strength: PasswordStrength,
    pub score: u32,
}

impl PasswordValidationDetailed {
    pub fn new(is_valid: bool, errors: Vec<String>, strength: PasswordStrength, score: u32) -> Self {
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

    let strength = if score <= 3 {
        PasswordStrength::Weak
    } else if score <= 5 {
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

/// WebAssembly-compatible interface for password validation.
/// This function is compiled to WebAssembly and can be called from JavaScript.
#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    /// Validates password and returns a JSON string with the result.
    /// This is the WebAssembly interface function.
    #[wasm_bindgen]
    pub fn validate_password_wasm(password: &str) -> String {
        let result = validate_password(password);
        serde_json::to_string(&result).unwrap_or_else(|_| {
            serde_json::to_string(&PasswordValidationResult::invalid(vec![
                "Internal error".to_string()
            ]))
            .unwrap()
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
                vec!["Internal error".to_string()],
                PasswordStrength::Weak,
                0,
            )).unwrap()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_password() {
        let result = validate_password("");
        assert!(!result.is_valid);
        assert_eq!(result.errors, vec!["Password is required".to_string()]);
    }

    #[test]
    fn test_short_password() {
        let result = validate_password("Short1!");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&"Password must be at least 8 characters".to_string()));
    }

    #[test]
    fn test_no_uppercase() {
        let result = validate_password("password123!");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&"Password must contain at least one uppercase letter".to_string()));
    }

    #[test]
    fn test_no_lowercase() {
        let result = validate_password("PASSWORD123!");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&"Password must contain at least one lowercase letter".to_string()));
    }

    #[test]
    fn test_no_number() {
        let result = validate_password("Password!");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&"Password must contain at least one number".to_string()));
    }

    #[test]
    fn test_no_special_char() {
        let result = validate_password("Password123");
        assert!(!result.is_valid);
        assert!(result.errors.contains(&"Password must contain at least one special character".to_string()));
    }

    #[test]
    fn test_valid_password() {
        let result = validate_password("StrongPass123!");
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_valid_password_with_various_special_chars() {
        let valid_passwords = vec![
            "StrongPass123!",
            "StrongPass123@",
            "StrongPass123#",
            "StrongPass123$",
            "StrongPass123%",
            "StrongPass123^",
            "StrongPass123&",
            "StrongPass123*",
            "StrongPass123(",
            "StrongPass123)",
            "StrongPass123-",
            "StrongPass123_",
            "StrongPass123=",
            "StrongPass123+",
            "StrongPass123[",
            "StrongPass123]",
            "StrongPass123{",
            "StrongPass123}",
            "StrongPass123|",
            "StrongPass123\\",
            "StrongPass123:",
            "StrongPass123;",
            "StrongPass123\"",
            "StrongPass123'",
            "StrongPass123<",
            "StrongPass123>",
            "StrongPass123,",
            "StrongPass123.",
            "StrongPass123?",
            "StrongPass123/",
            "StrongPass123~",
            "StrongPass123`",
        ];

        for password in valid_passwords {
            let result = validate_password(password);
            assert!(result.is_valid, "Expected {} to be valid", password);
        }
    }

    #[test]
    fn test_multiple_errors() {
        let result = validate_password("abc");
        println!("Errors for 'abc': {:?}", result.errors);
        assert!(!result.is_valid);
        // "abc" has lowercase, so it fails: length, uppercase, number, special character (4 errors)
        assert_eq!(result.errors.len(), 4);
    }

    #[test]
    fn test_result_display() {
        let valid_result = PasswordValidationResult::valid();
        assert_eq!(format!("{}", valid_result), "Valid");

        let invalid_result = PasswordValidationResult::invalid(vec!["Too short".to_string()]);
        assert_eq!(format!("{}", invalid_result), "Invalid: Too short");
    }

    #[test]
    fn test_password_strength_detailed() {
        // Weak password
        let result = validate_password_detailed("abc");
        println!("abc: score={}, valid={}, errors={:?}", result.score, result.is_valid, result.errors);
        assert!(!result.is_valid);
        assert_eq!(result.strength, PasswordStrength::Weak);
        assert_eq!(result.score, 1); // Only lowercase

        // Medium password
        let result = validate_password_detailed("Password123");
        println!("Password123: score={}, valid={}, errors={:?}", result.score, result.is_valid, result.errors);
        assert!(!result.is_valid); // Missing special character
        assert_eq!(result.strength, PasswordStrength::Medium);
        // Length 8+, uppercase, lowercase, number = 4, not 5
        assert_eq!(result.score, 4);

        // Strong password
        let result = validate_password_detailed("StrongPass123!");
        println!("StrongPass123!: score={}, valid={}, errors={:?}", result.score, result.is_valid, result.errors);
        assert!(result.is_valid);
        assert_eq!(result.strength, PasswordStrength::Strong);
        // Length 8+, length 12+, uppercase, lowercase, number, special = 6, not 7
        assert_eq!(result.score, 6);
    }

    #[test]
    fn test_password_strength_scoring() {
        let test_cases = vec![
            ("abc", 1), // lowercase only
            ("ABC", 1), // uppercase only
            ("123", 1), // numbers only
            ("!", 1),   // special only
            ("abc123", 2), // lowercase + number
            ("Abc123", 3), // length 6, uppercase, lowercase, number
            ("Abcdefg1", 4), // length 8+, uppercase, lowercase, number
            ("Abcdefg1!", 5), // length 8+, uppercase, lowercase, number, special
            ("Abcdefgh1!", 5), // length 12-, uppercase, lowercase, number, special
            ("Abcdefghijkl1!", 6), // length 16+, uppercase, lowercase, number, special
        ];

        for (password, expected_score) in test_cases {
            let result = validate_password_detailed(password);
            println!("{}: score={}, expected={}", password, result.score, expected_score);
            assert_eq!(result.score, expected_score, "Failed for password: {}", password);
        }
    }

    #[test]
    fn test_password_strength_enum() {
        let weak = PasswordStrength::Weak;
        let medium = PasswordStrength::Medium;
        let strong = PasswordStrength::Strong;

        assert_eq!(format!("{:?}", weak), "Weak");
        assert_eq!(format!("{:?}", medium), "Medium");
        assert_eq!(format!("{:?}", strong), "Strong");
    }
}