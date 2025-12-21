use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

use crate::db::{DBHandle, UserLogin};

/* --- API REQUEST/RESPONSE TYPES --- */

/// Generic API response type
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        ApiResponse {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> Self {
        ApiResponse {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

/// Registration request payload
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Login request payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response data
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub username: String,
    pub email: String,
}

/* --- API REQUEST/RESPONSE TYPES --- */

/// Handler for user login
pub async fn register_user(
    State(db): State<Arc<DBHandle>>,
    Json(payload): Json<RegistrationRequest>,
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
        let response = ApiResponse::<()>::error("Username, email, and password are required");
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    // Check if username already exists
    match db.user_login_table.is_username_free(&payload.username).await {
        Ok(true) => {
            debug!("Username '{}' is available", payload.username);
        }
        Ok(false) => {
            debug!("Registration failed: username '{}' already taken", payload.username);
            let response = ApiResponse::<()>::error("Username already taken");
            return (StatusCode::CONFLICT, Json(response));
        }
        Err(e) => {
            debug!(
                "Database error checking username '{}': {:?}",
                payload.username, e
            );
            let response = ApiResponse::<()>::error("Database error while checking username");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
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
                    let response = ApiResponse::<()>::success((), "User registered successfully");
                    (StatusCode::CREATED, Json(response))
                }
                Err(e) => {
                    debug!(
                        "Password set failed for '{}': {:?}. Attempting cleanup...",
                        payload.username, e
                    );
                    // If password setting fails, try to clean up the user
                    let _ = db.user_login_table.delete(&payload.username).await;
                    let response = ApiResponse::<()>::error("Failed to set password");
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
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
            let response = ApiResponse::<()>::error(error_msg);
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    #[derive(Serialize)]
    struct HealthData {
        status: String,
    }
    let response = ApiResponse::success(
        HealthData { status: "healthy".to_string() },
        "Service is running"
    );
    (StatusCode::OK, Json(response))
}

/// Handler for user login
pub async fn login_user(
    State(db): State<Arc<DBHandle>>,
    Json(payload): Json<LoginRequest>,
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
        let response = ApiResponse::<()>::error("Username and password are required");
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    // Check if user exists
    let user = match db.user_login_table.get_by_username(&payload.username).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            debug!("Login failed: user '{}' not found", payload.username);
            let response = ApiResponse::<()>::error("Invalid username or password");
            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
        Err(e) => {
            debug!(
                "Database error checking user '{}': {:?}",
                payload.username, e
            );
            let response = ApiResponse::<()>::error("Database error while checking user");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Verify password
    match db.user_login_table
        .is_password_correct(&payload.username, &payload.password)
        .await
    {
        Ok(true) => {
            debug!("Login successful for '{}'", payload.username);
            let response_data = LoginResponse {
                username: user.username,
                email: user.email,
            };
            let response = ApiResponse::success(response_data, "Login successful");
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(false) => {
            debug!("Login failed: incorrect password for '{}'", payload.username);
            let response = ApiResponse::<()>::error("Invalid username or password");
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
        Err(e) => {
            debug!(
                "Password verification error for '{}': {:?}",
                payload.username, e
            );
            let response = ApiResponse::<()>::error(&e.to_string());
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    }
}
