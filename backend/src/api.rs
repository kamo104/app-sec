use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use std::sync::Arc;
use tracing::debug;

use crate::db::{DBHandle, UserLogin};
use crate::generated::v1::{api_response, ApiResponse, EmptyData, HealthData, LoginResponseData};

/// Handler for user registration
pub async fn register_user(
    State(db): State<Arc<DBHandle>>,
    Protobuf(payload): Protobuf<crate::generated::v1::RegistrationRequest>,
) -> impl IntoResponse {
    debug!(
        "Received registration request - username: {}, email: {}",
        payload.username, payload.email
    );

    // Validate input
    if payload.username.is_empty() || payload.email.is_empty() || payload.password.is_empty() {
        debug!(
            "Registration validation failed for '{}': fields cannot be empty",
            payload.username
        );
        let response = ApiResponse {
            success: false,
            message: "Username, email, and password are required".to_string(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Validate username using shared Rust library
    let username_result = field_validator::validate_username(&payload.username, 3, 20, true);
    if !username_result.is_valid {
        debug!(
            "Username validation failed for '{}': {:?}",
            payload.username, username_result.errors
        );
        let response = ApiResponse {
            success: false,
            message: format!("Username does not meet requirements: {}", username_result.errors.join(", ")),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Validate email using shared Rust library
    let email_result = field_validator::validate_email(&payload.email);
    if !email_result.is_valid {
        debug!(
            "Email validation failed for '{}': {:?}",
            payload.email, email_result.errors
        );
        let response = ApiResponse {
            success: false,
            message: format!("Email does not meet requirements: {}", email_result.errors.join(", ")),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Validate password strength using shared Rust library
    let password_result = field_validator::validate_password(&payload.password);
    if !password_result.is_valid {
        debug!(
            "Password validation failed for '{}': {:?}",
            payload.username, password_result.errors
        );
        let response = ApiResponse {
            success: false,
            message: format!("Password does not meet requirements: {}", password_result.errors.join(", ")),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Check if username already exists
    match db.user_login_table.is_username_free(&payload.username).await {
        Ok(true) => {
            debug!("Username '{}' is available", payload.username);
        }
        Ok(false) => {
            debug!("Registration failed: username '{}' already taken", payload.username);
            let response = ApiResponse {
                success: false,
                message: "Username already taken".to_string(),
                data: None,
            };
            return (StatusCode::CONFLICT, Protobuf(response));
        }
        Err(e) => {
            debug!(
                "Database error checking username '{}': {:?}",
                payload.username, e
            );
            let response = ApiResponse {
                success: false,
                message: "Database error while checking username".to_string(),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Protobuf(response));
        }
    }

    // Create user login entry
    let user_login = UserLogin {
        username: payload.username.clone(),
        email: payload.email.clone(),
        password: None, // Initially no password, will be set separately
    };

    // Insert user into database
    match db.user_login_table.new_user(&user_login).await {
        Ok(_) => {
            debug!("User record created for '{}'", payload.username);
            // Set the password
            match db.user_login_table
                .set_password(&payload.username, &payload.password)
                .await
            {
                Ok(_) => {
                    debug!("Password set successfully for '{}'", payload.username);
                    let response = ApiResponse {
                        success: true,
                        message: "User registered successfully".to_string(),
                        data: Some(api_response::Data::Empty(EmptyData {})),
                    };
                    (StatusCode::CREATED, Protobuf(response))
                }
                Err(e) => {
                    debug!(
                        "Password set failed for '{}': {:?}. Attempting cleanup...",
                        payload.username, e
                    );
                    // If password setting fails, try to clean up the user
                    let _ = db.user_login_table.delete(&payload.username).await;
                    let response = ApiResponse {
                        success: false,
                        message: "Failed to set password".to_string(),
                        data: None,
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Protobuf(response))
                }
            }
        }
        Err(e) => {
            let error_msg = if e.to_string().contains("username already taken") {
                "Username already taken"
            } else {
                "Failed to create user"
            };
            debug!("Failed to create user '{}': {}", payload.username, error_msg);
            let response = ApiResponse {
                success: false,
                message: error_msg.to_string(),
                data: None,
            };
            (StatusCode::BAD_REQUEST, Protobuf(response))
        }
    }
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    let response = ApiResponse {
        success: true,
        message: "Service is running".to_string(),
        data: Some(api_response::Data::HealthData(HealthData {
            status: "healthy".to_string(),
        })),
    };
    (StatusCode::OK, Protobuf(response))
}

/// Handler for user login
pub async fn login_user(
    State(db): State<Arc<DBHandle>>,
    Protobuf(payload): Protobuf<crate::generated::v1::LoginRequest>,
) -> impl IntoResponse {
    debug!(
        "Received login request - username: {}",
        payload.username
    );

    // Validate input
    if payload.username.is_empty() || payload.password.is_empty() {
        debug!(
            "Login validation failed for '{}': fields cannot be empty",
            payload.username
        );
        let response = ApiResponse {
            success: false,
            message: "Username and password are required".to_string(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response)).into_response();
    }

    // Validate username format (without length validation for login)
    let username_result = field_validator::validate_username(&payload.username, 3, 20, false);
    if !username_result.is_valid {
        debug!(
            "Username validation failed for '{}': {:?}",
            payload.username, username_result.errors
        );
        let response = ApiResponse {
            success: false,
            message: format!("Invalid username format: {}", username_result.errors.join(", ")),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response)).into_response();
    }

    // Check if user exists
    let user = match db.user_login_table.get_by_username(&payload.username).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            debug!("Login failed: user '{}' not found", payload.username);
            let response = ApiResponse {
                success: false,
                message: "Invalid username or password".to_string(),
                data: None,
            };
            return (StatusCode::UNAUTHORIZED, Protobuf(response)).into_response();
        }
        Err(e) => {
            debug!(
                "Database error checking user '{}': {:?}",
                payload.username, e
            );
            let response = ApiResponse {
                success: false,
                message: "Database error while checking user".to_string(),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Protobuf(response)).into_response();
        }
    };

    // Verify password
    match db.user_login_table
        .is_password_correct(&payload.username, &payload.password)
        .await
    {
        Ok(true) => {
            debug!("Login successful for '{}'", payload.username);
            let response_data = LoginResponseData {
                username: user.username,
                email: user.email,
            };
            let response = ApiResponse {
                success: true,
                message: "Login successful".to_string(),
                data: Some(api_response::Data::LoginResponse(response_data)),
            };
            (StatusCode::OK, Protobuf(response)).into_response()
        }
        Ok(false) => {
            debug!("Login failed: incorrect password for '{}'", payload.username);
            let response = ApiResponse {
                success: false,
                message: "Invalid username or password".to_string(),
                data: None,
            };
            (StatusCode::UNAUTHORIZED, Protobuf(response)).into_response()
        }
        Err(e) => {
            debug!(
                "Password verification error for '{}': {:?}",
                payload.username, e
            );
            let response = ApiResponse {
                success: false,
                message: e.to_string(),
                data: None,
            };
            (StatusCode::UNAUTHORIZED, Protobuf(response)).into_response()
        }
    }
}
