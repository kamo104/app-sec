//! Validation types for field-level error reporting.

use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::{FieldType, PasswordStrength, ValidationErrorCode};

/// Validation error for a single field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ValidationFieldError {
    pub field: FieldType,
    pub errors: Vec<ValidationErrorCode>,
}

impl ValidationFieldError {
    /// Creates a new validation field error.
    pub fn new(field: FieldType) -> Self {
        Self {
            field,
            errors: Vec::new(),
        }
    }

    /// Returns true if there are no errors.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Adds an error code to this field error.
    pub fn add_error(&mut self, code: ValidationErrorCode) {
        self.errors.push(code);
    }
}

/// Validation error data containing multiple field errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrorData {
    pub field_errors: Vec<ValidationFieldError>,
}

impl ValidationErrorData {
    /// Creates a new empty validation error data.
    pub fn new() -> Self {
        Self {
            field_errors: Vec::new(),
        }
    }

    /// Creates validation error data from a list of field errors.
    pub fn from_errors(errors: Vec<ValidationFieldError>) -> Self {
        Self {
            field_errors: errors,
        }
    }

    /// Returns true if there are no field errors.
    pub fn is_valid(&self) -> bool {
        self.field_errors.iter().all(|e| e.is_valid())
    }
}

impl Default for ValidationErrorData {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed password validation data including strength assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ValidationDetailedPasswordData {
    pub data: Option<ValidationFieldError>,
    pub strength: PasswordStrength,
    pub score: u32,
}

impl ValidationDetailedPasswordData {
    /// Creates new detailed password data.
    pub fn new(data: ValidationFieldError, strength: PasswordStrength, score: u32) -> Self {
        Self {
            data: Some(data),
            strength,
            score,
        }
    }
}
