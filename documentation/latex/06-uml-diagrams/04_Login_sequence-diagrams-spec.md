# Sequence Diagrams Specifications

This document describes the content for each UML sequence diagram needed for the documentation.

## 1. Login Flow Sequence Diagram (`login-sequence.png`)

### Participants
- User (Actor)
- Frontend (Vue.js)
- Backend (Axum API)
- Database (SQLite)

### Flow

1. **User** enters username and password, clicks Login
2. **Frontend** validates username format (UsernameField component, format only - no length check for login)
3. **Frontend** sends `POST /api/login` with `{username, password}`
4. **Backend** validates username format
5. **Backend** queries Database for user by username
   - **Alt: User not found** -> Return 401 INVALID_CREDENTIALS
6. **Backend** checks if email_verified = true
   - **Alt: Not verified** -> Return 401 EMAIL_NOT_VERIFIED
7. **Backend** verifies password against Argon2 hash
   - **Alt: Wrong password** -> Return 401 INVALID_CREDENTIALS
8. **Backend** generates 32-byte session token
9. **Backend** computes SHA256 hash of token
10. **Backend** generates session_id
11. **Backend** inserts session into `user_sessions` table with expiry
12. **Backend** creates HTTP-only cookie with session token
13. **Backend** returns 200 with `{username, email, role, sessionExpiresAt, sessionCreatedAt}`
14. **Frontend** stores user data in auth store
15. **Frontend** stores session info in localStorage
16. **Frontend** schedules session refresh timer
17. **Frontend** redirects to home page

### Alternative Paths (shown with boxes/fragments)
- User not found in database
- Email not verified
- Password incorrect
- Database error

---

## 2. Logout Flow Sequence Diagram (`logout-sequence.png`)

### Participants
- User (Actor)
- Frontend (Vue.js)
- Backend (Axum API)
- Database (SQLite)

### Flow

1. **User** clicks Logout button
2. **Frontend** sends `POST /api/logout`
3. **Backend** reads session_token from cookie
4. **Backend** computes SHA256 hash of token
5. **Backend** deletes session from `user_sessions` table by hash
6. **Backend** creates expired cookie to clear session
7. **Backend** returns 200 OK
8. **Frontend** clears user from auth store
9. **Frontend** clears localStorage
10. **Frontend** cancels refresh timer
11. **Frontend** redirects to login page

### Notes
- Logout is idempotent - always returns success
- Missing/invalid cookie is handled gracefully

---

## 3. Password Reset Flow Sequence Diagram (`password-reset-sequence.png`)

This diagram shows the complete password reset process in two phases.

### Participants
- User (Actor)
- Frontend (Vue.js)
- Backend (Axum API)
- Database (SQLite)
- Email Service (SMTP/MailHog)

### Phase 1: Reset Request

1. **User** enters email address, clicks "Send Reset Link"
2. **Frontend** validates email format (WASM)
3. **Frontend** sends `POST /api/request-password-reset` with `{email}`
4. **Backend** queries Database for user by email
   - **Alt: User not found** -> Return 200 OK (for security)
5. **Backend** generates 32-byte reset token
6. **Backend** computes SHA256 hash of token
7. **Backend** inserts/updates `password_reset_tokens` table (UPSERT)
8. **Backend** sets `password_reset = true` on user record
9. **Backend** constructs reset link with token
10. **Backend** sends reset email via SMTP
    - **Alt: Email fails** -> Return 500 INTERNAL
11. **Backend** returns 200 OK
12. **Frontend** shows success message (always, for security)

### Phase 2: Reset Completion

1. **User** clicks reset link in email, lands on reset page
2. **Frontend** extracts token from URL query parameter
3. **User** enters new password and confirmation
4. **Frontend** validates password (WASM) - complexity requirements
5. **Frontend** validates passwords match
6. **Frontend** sends `POST /api/complete-password-reset` with `{token, newPassword}`
7. **Backend** computes SHA256 hash of token
8. **Backend** queries `password_reset_tokens` by hash
   - **Alt: Token not found** -> Return 400 INVALID_TOKEN
9. **Backend** checks token expiry
   - **Alt: Token expired** -> Return 400 INVALID_TOKEN
10. **Backend** validates new password
    - **Alt: Invalid password** -> Return 400 VALIDATION with errors
11. **Backend** hashes new password with Argon2
12. **Backend** updates password in `user_login` table
13. **Backend** sets `password_reset = false` on user record
14. **Backend** deletes token from `password_reset_tokens`
15. **Backend** returns 200 OK
16. **Frontend** shows success message
17. **Frontend** provides link to login page

### Alternative Paths
- User not found (returns success anyway)
- Email sending failure
- Invalid/missing token
- Expired token
- Password validation failure

### Security Notes (to show in diagram)
- Always returns 200 on request to prevent email enumeration
- Token stored as hash only
- Existing token is overwritten on new request
- Token is single-use and deleted after completion

### Diagram Layout Suggestion
- Use a horizontal divider or "ref" fragment to separate Phase 1 and Phase 2
- Or use two separate interaction frames within the same diagram
- Label them "Password Reset Request" and "Password Reset Completion"

---

## General Diagram Guidelines

### Style
- Use standard UML sequence diagram notation
- Lifelines for each participant
- Synchronous messages with filled arrowheads
- Return messages with dashed lines
- Alt/Opt fragments for conditional flows
- Notes for security considerations

### Colors (suggested)
- Success paths: default/black
- Error paths: red or in alt fragments
- Security notes: blue or highlighted

### Tool Recommendation
- Draw.io (app.diagrams.net) as recommended in course instructions
- Export as PNG at high resolution (at least 1920px width)

### File Locations
Place generated diagrams in:
```
documentation/latex/06-uml-diagrams/
  - login-sequence.png
  - logout-sequence.png
  - password-reset-sequence.png
```
