use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::db::{self, DB};
#[cfg(feature = "server")]
use crate::email::EmailSender;
#[cfg(feature = "server")]
use tower_cookies::{Cookie, cookie::SameSite, Cookies};
#[cfg(feature = "server")]
use dioxus::fullstack::FullstackContext;
#[cfg(feature = "server")]
use tracing::error;

// Common types shared between client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationData {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginResponse {
    pub username: String,
    pub redirect_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationData {
    pub field_type: String,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetRequestData {
    pub identifier: String, // email or username
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordData {
    pub token: String,
    pub new_password: String,
}

// --- CONSTANTS ---
#[cfg(feature = "server")]
const COOKIE_NAME: &str = "session_token";

#[cfg(feature = "server")]
async fn get_db() -> Result<std::sync::Arc<db::DBHandle>, ServerFnError> {
    DB.get()
        .cloned()
        .ok_or_else(|| ServerFnError::new("Database not initialized"))
}

#[cfg(feature = "server")]
fn get_cookies() -> Result<Cookies, ServerFnError> {
    // In Dioxus 0.6+, we can access the axum request parts
    // But direct "Cookies" extractor support in server_context is limited or different depending on setup
    // For now, we'll try to use the raw headers to parse/set cookies or assume Dioxus integrates tower-cookies if we set it up in main.
    //
    // ACTUALLY: Dioxus server functions integrate with Axum extractors.
    // We can use `extract::<Cookies>().await` inside the function body.
    // Ref: https://dioxuslabs.com/learn/0.6/fullstack/server_functions#extractors

    // NOTE: This call must be awaited inside the server function.
    // Since we are in a helper function here, we can't await it easily unless this is async.
    // We will inline extraction in the handlers.
    Err(ServerFnError::new("Use extract in handler"))
}

#[server]
pub async fn register(data: RegistrationData) -> Result<(), ServerFnError> {
    // 1. Validate Input
    let msgs_username = field_validator::validate_username(&data.username, 3, 20, true).errors;
    if !msgs_username.is_empty() {
        return Err(ServerFnError::new(serde_json::to_string(&ValidationData {
            field_type: "username".to_string(),
            messages: msgs_username.iter().map(|e| format!("{:?}", e)).collect(),
        })?));
    }

    let msgs_email = field_validator::validate_email(&data.email).errors;
    if !msgs_email.is_empty() {
        return Err(ServerFnError::new(serde_json::to_string(&ValidationData {
            field_type: "email".to_string(),
            messages: msgs_email.iter().map(|e| format!("{:?}", e)).collect(),
        })?));
    }

    let msgs_password = field_validator::validate_password(&data.password).errors;
    if !msgs_password.is_empty() {
        return Err(ServerFnError::new(serde_json::to_string(&ValidationData {
            field_type: "password".to_string(),
            messages: msgs_password.iter().map(|e| format!("{:?}", e)).collect(),
        })?));
    }

    // 2. Check DB
    let db = get_db().await?;

    // Check if username taken
    if !db.user_login_table.is_username_free(&data.username).await.map_err(|e| ServerFnError::new(e.to_string()))? {
        return Err(ServerFnError::new("Username is already taken"));
    }

    // 3. Create User
    let new_user = db::UserLogin {
        user_id: 0, // auto increment
        username: data.username.clone(),
        email: data.email.clone(),
        password: Some(db::hash_password(&data.password).map_err(|e| ServerFnError::new(e.to_string()))?),
        email_verified: false,
        email_verified_at: None,
        counter: 0,
        password_reset: false,
    };

    let user_id = db.user_login_table.new_user(&new_user).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    // 4. Verification Token
    let token_raw = db::generate_verification_token();
    let token_hash = db::hash_token(&token_raw).map_err(|e| ServerFnError::new(e.to_string()))?;

    let expiry = time::OffsetDateTime::now_utc() + time::Duration::hours(2);
    db.email_verification_tokens_table.insert(user_id, &token_hash, expiry).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    // 5. Send Email
    let base_url = if db.is_dev { "http://localhost:8080" } else { "https://example.com" }; // Adjusted port for dioxus default
    let link = format!("{}/verify-email?token={}", base_url, token_raw);

    let email_sender = EmailSender::new_mailhog();
    if let Err(e) = email_sender.send_verification_email(&data.email, &link).await {
        error!("Failed to send email: {:?}", e);
        // Don't fail the request, just log it? Or fail it? Backend implementation returns success but logs error.
    }

    Ok(())
}

#[server]
pub async fn login(data: LoginData) -> Result<LoginResponse, ServerFnError> {
    let db = get_db().await?;

    // 1. Verify Credentials
    // Check validation first? Backend does.
    let msgs_username = field_validator::validate_username(&data.username, 3, 20, false).errors;
    if !msgs_username.is_empty() {
        return Err(ServerFnError::new("Invalid credentials"));
    }

    let is_valid = db.user_login_table.is_password_correct(&data.username, &data.password).await.unwrap_or(false);

    if !is_valid {
         return Err(ServerFnError::new("Invalid credentials"));
    }

    let user = db.user_login_table.get_by_username(&data.username).await.map_err(|_| ServerFnError::new("Invalid credentials"))?;

    // 2. Check Email Verification
    if !user.email_verified {
        return Err(ServerFnError::new("Email not verified"));
    }

    // 3. Create Session
    let session_token = db::generate_session_token();
    let session_hash = db::hash_token(&session_token).map_err(|e| ServerFnError::new(e.to_string()))?;
    let session_id = db::generate_session_id();
    let expiry = time::OffsetDateTime::now_utc() + time::Duration::days(7);

    let session = db::UserSession {
        user_id: user.user_id,
        session_id,
        session_hash,
        session_expiry: expiry,
    };

    db.user_sessions_table.insert(&session).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    // 4. Set Cookie
    let cookies: Cookies = FullstackContext::extract().await?;
    let mut cookie = Cookie::new(COOKIE_NAME, session_token);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(!db.is_dev);
    // cookie.set_expires(expiry); // tower_cookies might expect OffsetDateTime or similar?
    // tower_cookies::CookieBuilder::new...

    cookies.add(cookie);

    Ok(LoginResponse {
        username: user.username,
        redirect_url: "/dashboard".to_string(),
    })
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    let cookies: Cookies = FullstackContext::extract().await?;

    // We need to invalidate the session in DB too?
    // Backend implementation:
    // 1. Get cookie
    // 2. Hash token
    // 3. Delete from DB
    // 4. Clear cookie

    if let Some(cookie) = cookies.get(COOKIE_NAME) {
        let token = cookie.value();
        if let Ok(hash) = db::hash_token(token) {
            let db = get_db().await?;
            // Ideally we need user_id but delete_by_hash is implemented in our db logic
            let _ = db.user_sessions_table.delete_by_hash(&hash).await;
        }
    }

    let mut cookie = Cookie::new(COOKIE_NAME, "");
    cookie.set_path("/");
    cookie.set_max_age(time::Duration::seconds(0));
    cookies.add(cookie);

    Ok(())
}

#[server]
pub async fn get_counter() -> Result<i64, ServerFnError> {
    let user = get_authenticated_user().await?;
    Ok(user.counter)
}

#[server]
pub async fn set_counter(value: i64) -> Result<(), ServerFnError> {
    let user = get_authenticated_user().await?;
    let db = get_db().await?;
    db.user_login_table.update_counter(user.user_id, value).await.map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub async fn verify_email(token: String) -> Result<(), ServerFnError> {
    let db = get_db().await?;
    let token_hash = db::hash_token(&token).map_err(|e| ServerFnError::new(e.to_string()))?;

    let record = db.email_verification_tokens_table.get_by_token_hash(&token_hash).await
        .map_err(|_| ServerFnError::new("Invalid or expired token"))?;

    if record.expires_at < time::OffsetDateTime::now_utc() {
        return Err(ServerFnError::new("Token expired"));
    }

    db.user_login_table.mark_email_verified(record.user_id).await.map_err(|e| ServerFnError::new(e.to_string()))?;
    db.email_verification_tokens_table.delete_by_user_id(record.user_id).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn request_password_reset(data: ResetRequestData) -> Result<(), ServerFnError> {
    let db = get_db().await?;

    // Find user
    let user = match db.user_login_table.get_by_username_or_email(&data.identifier).await {
        Ok(u) => u,
        Err(_) => {
            // Don't reveal user existence
            return Ok(());
        }
    };

    let token_raw = db::generate_verification_token();
    let token_hash = db::hash_token(&token_raw).map_err(|e| ServerFnError::new(e.to_string()))?;
    let expiry = time::OffsetDateTime::now_utc() + time::Duration::hours(1);

    db.password_reset_tokens_table.insert(user.user_id, &token_hash, expiry).await.map_err(|e| ServerFnError::new(e.to_string()))?;
    db.user_login_table.set_password_reset_flag(user.user_id, true).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    let base_url = if db.is_dev { "http://localhost:8080" } else { "https://example.com" };
    let link = format!("{}/reset-password?token={}", base_url, token_raw);

    let email_sender = EmailSender::new_mailhog();
    let _ = email_sender.send_password_reset_email(&user.email, &link).await;

    Ok(())
}

#[server]
pub async fn reset_password(data: ResetPasswordData) -> Result<(), ServerFnError> {
    let msgs_password = field_validator::validate_password(&data.new_password).errors;
    if !msgs_password.is_empty() {
        return Err(ServerFnError::new("Weak password"));
    }

    let db = get_db().await?;
    let token_hash = db::hash_token(&data.token).map_err(|e| ServerFnError::new(e.to_string()))?;

    let record = db.password_reset_tokens_table.get_by_token_hash(&token_hash).await
        .map_err(|_| ServerFnError::new("Invalid or expired token"))?;

    if record.expires_at < time::OffsetDateTime::now_utc() {
        return Err(ServerFnError::new("Token expired"));
    }

    db.user_login_table.set_password_by_user_id(record.user_id, &data.new_password).await.map_err(|e| ServerFnError::new(e.to_string()))?;
    db.user_login_table.set_password_reset_flag(record.user_id, false).await.map_err(|e| ServerFnError::new(e.to_string()))?;
    db.password_reset_tokens_table.delete_by_user_id(record.user_id).await.map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}


// --- HELPERS ---
#[cfg(feature = "server")]
async fn get_authenticated_user() -> Result<db::UserLogin, ServerFnError> {
    let cookies: Cookies = FullstackContext::extract().await?;
    let session_token = cookies.get(COOKIE_NAME)
        .ok_or(ServerFnError::new("No session"))?
        .value()
        .to_string();

    let db = get_db().await?;
    let token_hash = db::hash_token(&session_token).map_err(|_| ServerFnError::new("Auth error"))?;

    let session = db.user_sessions_table.get_by_hash(&token_hash).await.map_err(|_| ServerFnError::new("Invalid session"))?;

    if session.session_expiry < time::OffsetDateTime::now_utc() {
        // Cleanup?
        return Err(ServerFnError::new("Session expired"));
    }

    let user = db.user_login_table.get_by_user_id(session.user_id).await.map_err(|_| ServerFnError::new("User not found"))?;
    Ok(user)
}
