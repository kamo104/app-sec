//! Enum types for API responses and validation.

use serde::{Deserialize, Serialize};
use strum::{EnumString, IntoStaticStr};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

/// Helper macro to define an enum with consistent strum/serde serialization.
/// For variants that need a custom name (not matching SCREAMING_SNAKE_CASE of variant name),
/// use the tuple form: `VariantName = "CUSTOM_NAME"`.
macro_rules! define_enum {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $(= $rename:literal)?
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, IntoStaticStr, EnumString)]
        #[cfg_attr(feature = "openapi", derive(ToSchema))]
        #[cfg_attr(feature = "wasm", derive(Tsify))]
        #[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        #[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $(
                    #[strum(serialize = $rename)]
                    #[serde(rename = $rename)]
                )?
                $variant,
            )*
        }

        impl $name {
            pub fn as_str_name(&self) -> &'static str {
                (*self).into()
            }

            pub fn from_str_name(name: &str) -> Option<Self> {
                name.parse().ok()
            }
        }
    };
}

define_enum! {
    /// Success codes for specific success outcomes.
    pub enum SuccessCode {
        #[default]
        Unspecified = "SUCCESS_CODE_UNSPECIFIED",
        Ok = "SUCCESS_OK",
        Registered = "SUCCESS_REGISTERED",
        LoggedIn = "SUCCESS_LOGGED_IN",
        LoggedOut = "SUCCESS_LOGGED_OUT",
        EmailVerified = "SUCCESS_EMAIL_VERIFIED",
        SessionRefreshed = "SUCCESS_SESSION_REFRESHED",
        PasswordResetRequested = "SUCCESS_PASSWORD_RESET_REQUESTED",
        PasswordResetCompleted = "SUCCESS_PASSWORD_RESET_COMPLETED",
        CounterUpdated = "SUCCESS_COUNTER_UPDATED",
    }
}

define_enum! {
    /// Error codes for specific error types.
    pub enum ErrorCode {
        #[default]
        Unspecified = "ERROR_CODE_UNSPECIFIED",
        InvalidInput,
        UsernameTaken,
        InvalidCredentials,
        EmailNotVerified,
        InvalidToken,
        Database,
        Internal,
        Validation,
        EmailTaken,
    }
}

define_enum! {
    /// Field types for validation errors.
    pub enum FieldType {
        #[default]
        Unspecified = "FIELD_TYPE_UNSPECIFIED",
        Username,
        Email,
        Password,
    }
}

define_enum! {
    /// Validation error codes.
    pub enum ValidationErrorCode {
        #[default]
        Unspecified = "VALIDATION_ERROR_CODE_UNSPECIFIED",
        Required,
        TooShort,
        TooLong,
        InvalidCharacters,
        InvalidFormat,
        TooFewUppercaseLetters,
        TooFewLowercaseLetters,
        TooFewDigits,
        TooFewSpecialCharacters,
    }
}

define_enum! {
    /// Password strength levels.
    pub enum PasswordStrength {
        #[default]
        Unspecified = "PASSWORD_STRENGTH_UNSPECIFIED",
        Weak = "PASSWORD_STRENGTH_WEAK",
        Medium = "PASSWORD_STRENGTH_MEDIUM",
        Strong = "PASSWORD_STRENGTH_STRONG",
        Cia = "PASSWORD_STRENGTH_CIA",
    }
}
