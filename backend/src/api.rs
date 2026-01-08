use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use tower_cookies::{Cookie, Cookies};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tracing::{debug, error};

use crate::db::{
    DBHandle, UserLogin, UserSession, generate_session_id, generate_session_token,
    generate_verification_token, hash_token,
};
use crate::email::EmailSender;
use crate::generated::v1::{
    ApiData, ApiResponse, CounterData, FieldType, LoginResponseData, ResponseCode,
    ValidationErrorData, api_data,
};

// Session and token expiry constants
const SESSION_DURATION_DAYS: i64 = 7;
const EMAIL_VERIFICATION_TOKEN_DURATION_HOURS: i64 = 2;
const PASSWORD_RESET_TOKEN_DURATION_HOURS: i64 = 1;

fn auth_error() -> (StatusCode, Protobuf<ApiResponse>) {
    (
        StatusCode::UNAUTHORIZED,
        Protobuf(ApiResponse {
            code: ResponseCode::ErrorInvalidCredentials.into(),
            data: None,
        }),
    )
}

fn internal_error() -> (StatusCode, Protobuf<ApiResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Protobuf(ApiResponse {
            code: ResponseCode::ErrorInternal.into(),
            data: None,
        }),
    )
}

/// Extractor for the current user session
pub struct AuthenticatedUser {
    pub user: UserLogin,
    pub session: UserSession,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    Arc<DBHandle>: FromRef<S>,
{
    type Rejection = (StatusCode, Protobuf<ApiResponse>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db = Arc::<DBHandle>::from_ref(state);

        // Extract cookies using tower-cookies
        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| auth_error())?;

        let token = cookies
            .get("session_token")
            .ok_or(auth_error())?;

        let hash = hash_token(token.value()).map_err(|_| internal_error())?;

        let session = db
            .user_sessions_table
            .get_by_hash(&hash)
            .await
            .map_err(|_| auth_error())?;

        if session.session_expiry < OffsetDateTime::now_utc() {
            return Err(auth_error());
        }

        let user = db
            .user_login_table
            .get_by_user_id(session.user_id)
            .await
            .map_err(|_| auth_error())?;

        Ok(Self { user, session })
    }
}

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
            code: ResponseCode::ErrorInvalidInput.into(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Validate username using shared Rust library
    let username_result = field_validator::validate_username(&payload.username);
    if !username_result.errors.is_empty() {
        debug!(
            "Username validation failed for '{}': {:?}",
            payload.username, username_result.errors
        );
        let response = ApiResponse {
            code: ResponseCode::ErrorValidation.into(),
            data: Some(ApiData {
                data: Some(api_data::Data::ValidationError(ValidationErrorData {
                    field: FieldType::Username.into(),
                    errors: username_result.errors,
                })),
            }),
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Validate email using shared Rust library
    let email_result = field_validator::validate_email(&payload.email);
    if !email_result.errors.is_empty() {
        debug!(
            "Email validation failed for '{}': {:?}",
            payload.email, email_result.errors
        );
        let response = ApiResponse {
            code: ResponseCode::ErrorValidation.into(),
            data: Some(ApiData {
                data: Some(api_data::Data::ValidationError(ValidationErrorData {
                    field: FieldType::Email.into(),
                    errors: email_result.errors,
                })),
            }),
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Validate password strength using shared Rust library
    let password_result = field_validator::validate_password(&payload.password);
    if !password_result.errors.is_empty() {
        debug!(
            "Password validation failed for '{}': {:?}",
            payload.username, password_result.errors
        );
        let response = ApiResponse {
            code: ResponseCode::ErrorValidation.into(),
            data: Some(ApiData {
                data: Some(api_data::Data::ValidationError(ValidationErrorData {
                    field: FieldType::Password.into(),
                    errors: password_result.errors,
                })),
            }),
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Check if username already exists
    match db
        .user_login_table
        .is_username_free(&payload.username)
        .await
    {
        Ok(true) => {
            debug!("Username '{}' is available", payload.username);
        }
        Ok(false) => {
            debug!(
                "Registration failed: username '{}' already taken",
                payload.username
            );
            let response = ApiResponse {
                code: ResponseCode::ErrorUsernameTaken.into(),
                data: None,
            };
            return (StatusCode::CONFLICT, Protobuf(response));
        }
        Err(e) => {
            debug!(
                "Database error checking username '{}': {:?}",
                payload.username, e
            );
            return internal_error();
        }
    }

    // Create user login entry
    // user_id will be auto-generated by the database
    let hashed_password = match crate::db::hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => {
            debug!(
                "Failed to hash password for '{}': {:?}",
                payload.username, e
            );
            return internal_error();
        }
    };

    let user_login = UserLogin {
        user_id: 0, // Will be set by database AUTOINCREMENT
        username: payload.username.clone(),
        email: payload.email.clone(),
        password: Some(hashed_password),
        email_verified: false,   // Initially not verified
        email_verified_at: None, // Will be set when verified
        counter: 0,
        password_reset: false,
    };

    // Insert user into database
    match db.user_login_table.new_user(&user_login).await {
        Ok(user_id) => {
            debug!(
                "User record created for '{}' with id {}",
                payload.username, user_id
            );

            // Generate verification token
            let token = generate_verification_token();
            let token_hash = match hash_token(&token) {
                Ok(hash) => hash,
                Err(e) => {
                    debug!(
                        "Failed to hash token for '{}': {:?}. Attempting cleanup...",
                        payload.username, e
                    );
                    match db.user_login_table.delete(&payload.username).await {
                        Ok(_) => debug!("Cleanup successful: deleted user '{}'", payload.username),
                        Err(cleanup_e) => error!(
                            "Cleanup failed for user '{}': {:?}",
                            payload.username, cleanup_e
                        ),
                    }
                    return internal_error();
                }
            };

            // Set token expiry
            let expires_at = OffsetDateTime::now_utc() + time::Duration::hours(EMAIL_VERIFICATION_TOKEN_DURATION_HOURS);

            // Store token in database
            match db
                .email_verification_tokens_table
                .insert(user_id, &token_hash, expires_at)
                .await
            {
                Ok(_) => {
                    debug!("Verification token stored for user_id: {}", user_id);

                    // Get base URL for verification link
                    let base_url = if db.is_dev {
                        "http://localhost:4000"
                    } else {
                        "https://example.com"
                    };

                    let verification_link = format!("{}/verify-email?token={}", base_url, token);

                    // Send real email via MailHog
                    let email_sender = EmailSender::new_mailhog();
                    if let Err(e) = email_sender
                        .send_verification_email(&payload.email, &verification_link)
                        .await
                    {
                        error!(
                            "Failed to send verification email to {}: {:?}. Attempting cleanup...",
                            payload.email, e
                        );
                        match db.user_login_table.delete(&payload.username).await {
                            Ok(_) => {
                                debug!("Cleanup successful: deleted user '{}'", payload.username)
                            }
                            Err(cleanup_e) => error!(
                                "Cleanup failed for user '{}': {:?}",
                                payload.username, cleanup_e
                            ),
                        }
                        return internal_error();
                    }

                    let response = ApiResponse {
                        code: ResponseCode::SuccessRegistered.into(),
                        data: None,
                    };
                    (StatusCode::CREATED, Protobuf(response))
                }
                Err(e) => {
                    debug!(
                        "Failed to store verification token for user_id {}: {:?}. Attempting cleanup...",
                        user_id, e
                    );
                    match db.user_login_table.delete(&payload.username).await {
                        Ok(_) => debug!("Cleanup successful: deleted user '{}'", payload.username),
                        Err(cleanup_e) => error!(
                            "Cleanup failed for user '{}': {:?}",
                            payload.username, cleanup_e
                        ),
                    }
                    return internal_error();
                }
            }
        }
        Err(e) => {
            let (code, status) = if e.to_string().contains("username already taken") {
                (ResponseCode::ErrorUsernameTaken, StatusCode::CONFLICT)
            } else {
                (ResponseCode::ErrorInternal, StatusCode::BAD_REQUEST)
            };
            debug!("Failed to create user '{}': {:?}", payload.username, e);
            let response = ApiResponse {
                code: code.into(),
                data: None,
            };
            (status, Protobuf(response))
        }
    }
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    let response = ApiResponse {
        code: ResponseCode::Success.into(),
        data: None,
    };
    (StatusCode::OK, Protobuf(response))
}

/// Authentication check endpoint - verifies if the user has a valid session
pub async fn auth_check(auth: AuthenticatedUser) -> impl IntoResponse {
    let response = ApiResponse {
        code: ResponseCode::Success.into(),
        data: Some(ApiData {
            data: Some(api_data::Data::LoginResponse(LoginResponseData {
                username: auth.user.username,
                email: auth.user.email,
                session_expires_at: auth.session.session_expiry.unix_timestamp(),
                session_created_at: auth.session.session_created_at.unix_timestamp(),
            })),
        }),
    };
    (StatusCode::OK, Protobuf(response))
}

/// Session refresh endpoint - extends the session and returns new expiry time
pub async fn refresh_session(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    cookies: Cookies,
) -> impl IntoResponse {
    debug!("Refreshing session for user: {}", auth.user.username);

    // Get the current session token
    let token = match cookies.get("session_token") {
        Some(t) => t,
        None => {
            error!("No session token found in refresh request");
            return auth_error().into_response();
        }
    };

    let session_hash = match hash_token(token.value()) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash session token: {:?}", e);
            return internal_error().into_response();
        }
    };

    // Update session expiry in database
    let new_expiry = OffsetDateTime::now_utc() + time::Duration::days(SESSION_DURATION_DAYS);

    match db.user_sessions_table.update_expiry(&session_hash, new_expiry).await {
        Ok(_) => {
            debug!("Session refreshed successfully for user: {}", auth.user.username);

            // Update cookie with new expiry
            let mut cookie = Cookie::new("session_token", token.value().to_string());
            cookie.set_path("/");
            cookie.set_http_only(true);
            cookie.set_secure(!db.is_dev);
            cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
            cookie.set_expires(Some(new_expiry.into()));
            cookies.add(cookie);

            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: Some(ApiData {
                    data: Some(api_data::Data::LoginResponse(LoginResponseData {
                        username: auth.user.username,
                        email: auth.user.email,
                        session_expires_at: new_expiry.unix_timestamp(),
                        session_created_at: auth.session.session_created_at.unix_timestamp(),
                    })),
                }),
            };
            (StatusCode::OK, Protobuf(response)).into_response()
        }
        Err(e) => {
            error!("Failed to refresh session: {:?}", e);
            internal_error().into_response()
        }
    }
}

/// Handler for user login
pub async fn login_user(
    State(db): State<Arc<DBHandle>>,
    cookies: Cookies,
    Protobuf(payload): Protobuf<crate::generated::v1::LoginRequest>,
) -> impl IntoResponse {
    debug!("Received login request - username: {}", payload.username);

    // Validate input
    if payload.username.is_empty() || payload.password.is_empty() {
        debug!(
            "Login validation failed for '{}': fields cannot be empty",
            payload.username
        );
        let response = ApiResponse {
            code: ResponseCode::ErrorInvalidInput.into(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response)).into_response();
    }

    // Validate username format
    let username_result = field_validator::validate_username(&payload.username);
    if !username_result.errors.is_empty() {
        debug!(
            "Username validation failed for '{}': {:?}",
            payload.username, username_result.errors
        );
        let response = ApiResponse {
            code: ResponseCode::ErrorValidation.into(),
            data: Some(ApiData {
                data: Some(api_data::Data::ValidationError(ValidationErrorData {
                    field: FieldType::Username.into(),
                    errors: username_result.errors,
                })),
            }),
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response)).into_response();
    }

    // Check if user exists
    let user = match db.user_login_table.get_by_username(&payload.username).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            debug!("Login failed: user '{}' not found", payload.username);
            let response = ApiResponse {
                code: ResponseCode::ErrorInvalidCredentials.into(),
                data: None,
            };
            return (StatusCode::UNAUTHORIZED, Protobuf(response)).into_response();
        }
        Err(e) => {
            debug!(
                "Database error checking user '{}': {:?}",
                payload.username, e
            );
            return internal_error().into_response();
        }
    };

    // Check if email is verified
    if !user.email_verified {
        debug!(
            "Login failed: user '{}' has not verified their email",
            payload.username
        );
        let response = ApiResponse {
            code: ResponseCode::ErrorEmailNotVerified.into(),
            data: None,
        };
        return (StatusCode::UNAUTHORIZED, Protobuf(response)).into_response();
    }

    // Verify password
    match db
        .user_login_table
        .is_password_correct(&payload.username, &payload.password)
        .await
    {
        Ok(true) => {
            debug!("Login successful for '{}'", payload.username);

            // Create a session
            let session_token = generate_session_token();
            let session_hash = match hash_token(&session_token) {
                Ok(hash) => hash,
                Err(e) => {
                    error!("Failed to hash session token: {:?}", e);
                    return internal_error().into_response();
                }
            };

            let now = OffsetDateTime::now_utc();
            let expiry = now + time::Duration::days(SESSION_DURATION_DAYS);
            let session = UserSession {
                user_id: user.user_id,
                session_id: generate_session_id(),
                session_hash,
                session_expiry: expiry,
                session_created_at: now,
            };

            if let Err(e) = db.user_sessions_table.insert(&session).await {
                error!("Failed to store session: {:?}", e);
                return internal_error().into_response();
            }

            // Create cookie
            let mut cookie = Cookie::new("session_token", session_token);
            cookie.set_path("/");
            cookie.set_http_only(true);
            cookie.set_secure(!db.is_dev);
            cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
            cookie.set_expires(Some(expiry.into()));

            // Add cookie to response via tower-cookies
            cookies.add(cookie);

            let response_data = LoginResponseData {
                username: user.username,
                email: user.email,
                session_expires_at: expiry.unix_timestamp(),
                session_created_at: now.unix_timestamp(),
            };
            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: Some(ApiData {
                    data: Some(api_data::Data::LoginResponse(response_data)),
                }),
            };

            (StatusCode::OK, Protobuf(response)).into_response()
        }
        Ok(false) => {
            debug!(
                "Login failed: incorrect password for '{}'",
                payload.username
            );
            let response = ApiResponse {
                code: ResponseCode::ErrorInvalidCredentials.into(),
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
                code: ResponseCode::ErrorInternal.into(),
                data: None,
            };
            (StatusCode::UNAUTHORIZED, Protobuf(response)).into_response()
        }
    }
}

/// Handler for email verification
pub async fn verify_email(
    State(db): State<Arc<DBHandle>>,
    Protobuf(payload): Protobuf<crate::generated::v1::EmailVerificationRequest>,
) -> impl IntoResponse {
    debug!(
        "Received email verification request - token: {}",
        payload.token
    );

    if payload.token.is_empty() {
        debug!("Email verification failed: token is empty");
        let response = ApiResponse {
            code: ResponseCode::ErrorInvalidToken.into(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Hash the provided token
    let token_hash = match hash_token(&payload.token) {
        Ok(hash) => hash,
        Err(e) => {
            debug!("Failed to hash verification token: {:?}", e);
            let response = ApiResponse {
                code: ResponseCode::ErrorInvalidToken.into(),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Protobuf(response));
        }
    };

    // Look up the token in the database
    let token_record = match db
        .email_verification_tokens_table
        .get_by_token_hash(&token_hash)
        .await
    {
        Ok(record) => record,
        Err(sqlx::Error::RowNotFound) => {
            debug!("Verification token not found in database");
            let response = ApiResponse {
                code: ResponseCode::ErrorInvalidToken.into(),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Protobuf(response));
        }
        Err(e) => {
            debug!("Database error looking up token: {:?}", e);
            return internal_error();
        }
    };

    // Check if token is expired
    if OffsetDateTime::now_utc() > token_record.expires_at {
        debug!("Verification token has expired");
        let response = ApiResponse {
            code: ResponseCode::ErrorInvalidToken.into(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Check if user is already verified
    let user = match db
        .user_login_table
        .get_by_user_id(token_record.user_id)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            debug!("Failed to get user for verification: {:?}", e);
            let response = ApiResponse {
                code: ResponseCode::ErrorInternal.into(),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Protobuf(response));
        }
    };

    if user.email_verified {
        debug!("User already verified");
        let response = ApiResponse {
            code: ResponseCode::Success.into(),
            data: None,
        };
        return (StatusCode::OK, Protobuf(response));
    }

    // Mark email as verified
    match db
        .user_login_table
        .mark_email_verified(token_record.user_id)
        .await
    {
        Ok(_) => {
            debug!(
                "Email verified successfully for user_id: {}",
                token_record.user_id
            );

            // Delete the token to ensure single-use
            match db
                .email_verification_tokens_table
                .delete_by_user_id(token_record.user_id)
                .await
            {
                Ok(_) => debug!(
                    "Verification token deleted for user_id: {}",
                    token_record.user_id
                ),
                Err(e) => debug!(
                    "Failed to delete token for user_id {}: {:?}",
                    token_record.user_id, e
                ),
            }

            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: None,
            };
            (StatusCode::OK, Protobuf(response))
        }
        Err(e) => {
            debug!("Failed to mark email as verified: {:?}", e);
            return internal_error();
        }
    }
}

/// Handler to get the current counter value
pub async fn get_counter(auth: AuthenticatedUser) -> impl IntoResponse {
    let response = ApiResponse {
        code: ResponseCode::Success.into(),
        data: Some(ApiData {
            data: Some(api_data::Data::CounterData(CounterData {
                value: auth.user.counter,
            })),
        }),
    };
    (StatusCode::OK, Protobuf(response))
}

/// Handler to set the counter value
pub async fn set_counter(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    Protobuf(payload): Protobuf<crate::generated::v1::SetCounterRequest>,
) -> impl IntoResponse {
    match db
        .user_login_table
        .update_counter(auth.user.user_id, payload.value)
        .await
    {
        Ok(_) => {
            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: Some(ApiData {
                    data: Some(api_data::Data::CounterData(CounterData {
                        value: payload.value,
                    })),
                }),
            };
            (StatusCode::OK, Protobuf(response))
        }
        Err(e) => {
            error!("Failed to update counter: {:?}", e);
            return internal_error();
        }
    }
}

/// Handler to initiate password reset
pub async fn request_password_reset(
    State(db): State<Arc<DBHandle>>,
    Protobuf(payload): Protobuf<crate::generated::v1::PasswordResetRequest>,
) -> impl IntoResponse {
    debug!("Received password reset request for '{}'", payload.email);

    // Check if user exists
    let user = match db.user_login_table.get_by_username(&payload.email).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            // Security: Don't reveal if user exists
            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: None,
            };
            return (StatusCode::OK, Protobuf(response));
        }
        Err(e) => {
            error!("Database error checking user: {:?}", e);
            return internal_error();
        }
    };

    // Generate reset token
    let token = generate_verification_token();
    let token_hash = match hash_token(&token) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash reset token: {:?}", e);
            return internal_error();
        }
    };

    let expires_at = OffsetDateTime::now_utc() + time::Duration::hours(PASSWORD_RESET_TOKEN_DURATION_HOURS);

    // Store reset token
    if let Err(e) = db.password_reset_tokens_table.insert(user.user_id, &token_hash, expires_at).await {
        error!("Failed to store reset token: {:?}", e);
        return internal_error();
    }

    // Set password reset flag
    if let Err(e) = db.user_login_table.set_password_reset_flag(user.user_id, true).await {
        error!("Failed to set password reset flag: {:?}", e);
        return internal_error();
    }

    // Send email
    let base_url = if db.is_dev {
        "http://localhost:4000"
    } else {
        "https://example.com"
    };
    let reset_link = format!("{}/reset-password?token={}", base_url, token);
    let email_sender = EmailSender::new_mailhog();

    if let Err(e) = email_sender.send_password_reset_email(&user.email, &reset_link).await {
        error!("Failed to send reset email: {:?}", e);
        return internal_error();
    }

    let response = ApiResponse {
        code: ResponseCode::Success.into(),
        data: None,
    };
    (StatusCode::OK, Protobuf(response))
}

/// Handler to complete password reset
pub async fn complete_password_reset(
    State(db): State<Arc<DBHandle>>,
    Protobuf(payload): Protobuf<crate::generated::v1::PasswordResetCompleteRequest>,
) -> impl IntoResponse {
    debug!("Received password reset completion request");

    // Validate token
    let token_hash = match hash_token(&payload.token) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash reset token: {:?}", e);
            return internal_error();
        }
    };

    let reset_record = match db.password_reset_tokens_table.get_by_token_hash(&token_hash).await {
        Ok(record) => record,
        Err(sqlx::Error::RowNotFound) => {
            let response = ApiResponse {
                code: ResponseCode::ErrorInvalidToken.into(),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Protobuf(response));
        }
        Err(e) => {
            error!("Database error looking up reset token: {:?}", e);
            return internal_error();
        }
    };

    if OffsetDateTime::now_utc() > reset_record.expires_at {
        let response = ApiResponse {
            code: ResponseCode::ErrorInvalidToken.into(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Validate new password
    let password_result = field_validator::validate_password(&payload.new_password);
    if !password_result.errors.is_empty() {
        let response = ApiResponse {
            code: ResponseCode::ErrorValidation.into(),
            data: Some(ApiData {
                data: Some(api_data::Data::ValidationError(ValidationErrorData {
                    field: FieldType::Password.into(),
                    errors: password_result.errors,
                })),
            }),
        };
        return (StatusCode::BAD_REQUEST, Protobuf(response));
    }

    // Update password
    match db.user_login_table.set_password_by_user_id(reset_record.user_id, &payload.new_password).await {
        Ok(_) => {
            // Clear flag and delete token
            let _ = db.user_login_table.set_password_reset_flag(reset_record.user_id, false).await;
            let _ = db.password_reset_tokens_table.delete_by_user_id(reset_record.user_id).await;

            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: None,
            };
            (StatusCode::OK, Protobuf(response))
        }
        Err(e) => {
            error!("Failed to update password: {:?}", e);
            internal_error()
        }
    }
}

/// Handler for user logout
pub async fn logout_user(
    State(db): State<Arc<DBHandle>>,
    cookies: Cookies,
) -> impl IntoResponse {
    if let Some(token) = cookies.get("session_token") {
        if let Ok(hash) = hash_token(token.value()) {
            let _ = db.user_sessions_table.delete_by_hash(&hash).await;
        }
    }

    let mut cookie = Cookie::new("session_token", "");
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_secure(!db.is_dev);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_expires(Some(OffsetDateTime::UNIX_EPOCH.into()));

    cookies.add(cookie);

    let response = ApiResponse {
        code: ResponseCode::Success.into(),
        data: None,
    };

    (StatusCode::OK, Protobuf(response))
}
