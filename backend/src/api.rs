use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use tracing::debug;

use crate::db::{DBHandle, UserLogin, ApiResponse, RegistrationRequest};

/// Handler for user registration
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
