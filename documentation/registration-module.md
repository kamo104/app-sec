# Registration Module Documentation

[[_TOC_]]

---

## 1. Student Group, Course, and Exercise Details

### Student Group Information
- **Group Name**: Application Security Project Team
- **Project Focus**: Web Security Best Practices Demonstration
- **Academic Context**: Master's Degree Program, Semester II

### Course Details
- **Course**: Application Security
- **Level**: Graduate/Master's
- **Focus Areas**:
  - Secure authentication and authorization
  - Password security and hashing
  - Email verification systems
  - Input validation and sanitization
  - Cryptographic token generation

### Exercise Description
This exercise demonstrates a complete registration and email verification system implementing modern security practices:
- Secure user registration with validated input
- Cryptographically secure token generation
- Email verification workflow
- Password security with Argon2 hashing
- Defense against common attacks (replay, enumeration, etc.)

---

## 2. Component Description

### Purpose
The registration module provides a secure user onboarding system that:
1. Collects and validates user registration data
2. Creates user accounts with verified email requirements
3. Generates secure, single-use email verification tokens
4. Prevents unauthorized access through email verification
5. Ensures data integrity and security throughout the process

### Data Collected
During registration, the following data is collected and processed:

| Field | Type | Validation | Storage |
|-------|------|------------|---------|
| Username | String | 3-20 chars, alphanumeric + underscore | Plaintext in DB |
| Email | String | Valid email format, unique | Plaintext in DB |
| Password | String | Strength requirements (see validation) | Argon2 PHC format hash |
| Verification Token | 32-byte hex | Cryptographically secure | SHA256 hash only |

### Security Assumptions
1. **Transport Security**: HTTPS is used for all communications
2. **Database Security**: SQLite database encrypted at rest (SQLCipher)
3. **Token Security**: Tokens are never stored in plaintext
4. **Password Security**: Argon2id algorithm with unique salts per user
5. **Email Security**: Email verification required before account activation
6. **Session Security**: Future session management planned with JWT

---

## 3. Component Requirements

### Functional Requirements

#### FR-1: User Registration
- **ID**: FR-1
- **Description**: Users can register with username, email, and password
- **Acceptance Criteria**:
  - All fields are required
  - Input validation passes
  - Username is unique
  - User record created with `email_verified = false`
  - Verification token generated and stored
  - Success response returned

#### FR-2: Input Validation
- **ID**: FR-2
- **Description**: All inputs must be validated before processing
- **Acceptance Criteria**:
  - Username: 3-20 chars, alphanumeric + underscore
  - Email: Valid format, RFC 5322 compliant
  - Password: Minimum strength requirements (8+ chars, mixed case, numbers, symbols)
  - All validation errors returned to user

#### FR-3: Email Verification Token Generation
- **ID**: FR-3
- **Description**: Generate cryptographically secure verification tokens
- **Acceptance Criteria**:
  - 32 bytes (64 hex characters) of entropy
  - Generated using OS CSPRNG (OsRng)
  - Hashed with SHA256 before storage
  - Expires after 2 hours
  - Single-use only

#### FR-4: Email Verification
- **ID**: FR-4
- **Description**: Verify email address via token link
- **Acceptance Criteria**:
  - Token lookup by hash
  - Expiration check
  - User status verification
  - Mark user as verified
  - Delete token after use
  - Return appropriate response

#### FR-5: Login with Email Verification
- **ID**: FR-5
- **Description**: Users cannot login until email is verified
- **Acceptance Criteria**:
  - Check email_verified flag
  - Return error if not verified
  - Allow login after verification

### Non-Functional Requirements

#### NFR-1: Security
- **ID**: NFR-1
- **Description**: All security best practices must be followed
- **Details**:
  - No plaintext passwords or tokens stored
  - Generic error messages (no enumeration)
  - HTTPS only
  - No PII in URLs
  - Rate limiting planned for future

#### NFR-2: Performance
- **ID**: NFR-2
- **Description**: System must handle concurrent registrations
- **Details**:
  - Database operations < 100ms
  - Token generation < 50ms
  - API response < 200ms

#### NFR-3: Reliability
- **ID**: NFR-3
- **Description**: System must be fault-tolerant
- **Details**:
  - Transaction rollback on failure
  - Cleanup of partial data
  - Graceful error handling
  - Logging for audit trail

#### NFR-4: Usability
- **ID**: NFR-4
- **Description**: Clear user feedback required
- **Details**:
  - Success/error messages
  - Password strength indicator
  - Field validation feedback
  - Email verification instructions

---

## 4. Component Architecture

### Technology Stack

#### Backend
- **Language**: Rust 1.75+
- **Web Framework**: Axum
- **Database**: SQLite with SQLCipher encryption
- **ORM**: SQLx (async)
- **Password Hashing**: Argon2
- **Token Hashing**: SHA256
- **CSPRNG**: OsRng (rand_core)
- **API Format**: Protocol Buffers v3

#### Frontend
- **Framework**: Vue 3 with Composition API
- **UI Library**: Vuetify 3
- **State Management**: Pinia (planned)
- **API Client**: Native fetch with protobuf
- **Validation**: Shared Rust validators compiled to WASM

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (Vue)                       │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │Register Page│→│API Service   │→│Validation Layer  │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↓ HTTP/Protobuf
┌─────────────────────────────────────────────────────────────┐
│                      Backend (Axum)                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │API Handlers │→│Business Logic│→│Database Layer    │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↓ SQL Encrypted
┌─────────────────────────────────────────────────────────────┐
│                    Database (SQLite)                        │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │user_login   │  │user_sessions │  │email_verif_tokens│  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Registration Flow**:
   ```
   User → Frontend Form → Validation → API Request → Backend Handler
   → Input Validation → Username Check → User Creation → Password Hash
   → Token Generation → Token Hash → Token Storage → Email Mock
   → Success Response
   ```

2. **Verification Flow**:
   ```
   Email Link → Token Extract → Hash Token → DB Lookup → Expiration Check
 User Status Verified → Mark Verified → Delete Token → Success Response
 ```

## **ussion Module**### **  **

`` ** ** 3`` **  API-ation
 The   Backend


`` │ User Login

 →
 User


:  Email Verification


 Token
``



 User










 →










``




























- **UserLoginTable**: Manages user credentials and verification status
- **EmailVerificationTokensTable**: Stores token hashes with expiration
-- ** ****:**:**:: ** ** ****:**:**:: **#### **### Database Schema

```sql
CREATE TABLE user_login (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    password TEXT,
    email_verified INTEGER NOT NULL DEFAULT 0,
    email_verified_at INTEGER
);

CREATE TABLE email_verification_tokens (
    user_id INTEGER NOT NULL REFERENCES,
    token_hash TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(user_login(user_id REFERENCES ON user_id

 REFERENCES
 ** token_hash TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (user_id),
    FOREIGN KEY (user_id) REFERENCES user_login(user_id) ON DELETE CASCADE
);
```

### Field Constraints
- **user_login**:
  - `user_id`: AUTOINCREMENT, PRIMARY KEY
  - `username`: UNIQUE, NOT NULL, 3-20 chars
  - `email`: NOT NULL, valid format
  - `password`: NULL allowed (for reset), PHC format
  - `email_verified`: DEFAULT 0
  - `email_verified_at`: NULL until verified

- **email_verification_tokens**:
  - `user_id`: FOREIGN KEY, ON DELETE CASCADE
  - `token_hash`: SHA256 hex string, NOT NULL
  - `expires_at`: Unix timestamp, 2 hours from creation
  - `created_at`: Audit timestamp

---

## 6. UML Sequence Diagrams

### Registration Sequence Diagram

![Registration Sequence Diagram](./sequence-diagrams/registration.png)

**Text Description of Registration Flow:**

```
User -> Frontend: Submit Registration Form
Frontend -> Validation: Validate Fields
Validation -> Frontend: Return Validation Result

Frontend -> Backend: POST /api/register (Protobuf)
Backend -> DB: Check Username Availability
DB -> Backend: Username Available

Backend -> DB: Create User Record
DB -> Backend: User Created (user_id)

Backend -> Crypto: Generate Token (32 bytes)
Crypto -> Backend: Raw Token

Backend -> Crypto: Hash Token (SHA256)
Crypto -> Backend: Token Hash

Backend -> DB: Store Token Hash + Expiry
DB -> Backend: Token Stored

Backend -> Email: Mock Email (Log to Console)
Backend -> Frontend: Registration Success

Frontend -> User: Display Success Message
```

### Email Verification Sequence Diagram

![Email Verification Sequence Diagram](./sequence-diagrams/verify-email.png)

**Text Description of Verification Flow:**

```
User -> Email: Click Verification Link
Email -> Frontend: GET /verify-email?token=abc123

Frontend -> Backend: POST /api/verify-email (token)
Backend -> Crypto: Hash Token (SHA256)
Crypto -> Backend: Token Hash

Backend -> DB: Lookup Token by Hash
DB -> Backend: Token Record

Backend -> Validation: Check Expiration
Validation -> Backend: Valid (not expired)

Backend -> DB: Get User by user_id
DB -> Backend: User Record

Backend -> Validation: Check if Verification Verified Verified verified verified verified verified verified











 ->:































:













:


::

























:














,



























:









:

,

:




,



: |:

 |: }



:

: Backend -> DB: Mark Email Verified
DB -> Backend: Update Success

Backend -> DB: Delete Token
DB -> Backend: Delete Success

Backend -> Frontend: Verification Success
Frontend -> User: Display Success Page
```

### Alternative Paths and Error Handling

#### Registration Error Paths:

1. **Validation Failure**:
   ```
   Frontend -> Validation: Validate Fields
   Validation -> Frontend: Errors Found
   Frontend -> User: Display Field Errors
   ```

2. **Username Already Taken**:
   ```
   Backend -> DB: Check Username
   DB -> Backend: Username Exists
   Backend -> Frontend: 409 Conflict
   Frontend -> User: "Username already taken"
   ```

3. **Database Error**:
   ```
   Backend -> DB: Operation
   DB -> Backend: Error
   Backend -> Frontend: 500 Internal Error
   Frontend -> User: Generic error message
   ```

#### Verification Error Paths:

1. **Invalid Token**:
   ```
   Backend -> DB: Lookup Token
   DB -> Backend: Not Found
   Backend -> Frontend: 400 Bad Request
   Frontend -> User: "Invalid or expired link"
   ```

2. **Expired Token**:
   ```
   Backend -> Validation: Check Expiration
   Validation -> Backend: Expired
   Backend -> Frontend: 400 Bad Request
   Frontend -> User: "Invalid or expired link"
   ```

3. **Already Verified**:
   ```
   Backend -> DB: Check User Status
   DB -> Backend: Already Verified
   Backend -> Frontend: 200 OK
   Frontend -> User: "Email already verified"
   ```

---

## 7. Security Features Summary

### Implemented Security Measures

1. **Password Security**
   - Argon2id hashing with unique salts
   - Minimum complexity requirements
   - Never stored in plaintext

2. **Token Security**
   - 32 bytes (256 bits) of entropy
   - SHA256 hashing before storage
   - 2-hour expiration
   - Single-use (deleted after verification)
   - No PII in URLs

3. **Input Validation**
   - Client-side (WASM validators)
   - Server-side (Rust validators)
   - Length and format checks
   - Email RFC compliance

4. **Error Handling**
   - Generic error messages
   - No enumeration hints
   - Proper HTTP status codes
   - Logging for debugging only

5. **Database Security**
   - SQLCipher encryption
   - Foreign key constraints
   - ON DELETE CASCADE
   - Prepared statements (SQLx)

### Future Security Enhancements

- Rate limiting on registration/verification
- CAPTCHA for bot prevention
- Email service integration (SMTP)
- Token cleanup scheduler
- JWT session management
- 2FA/MFA support
- Password reset flow
- Account lockout after failed attempts

---

## 8. API Reference

### Endpoints

#### POST /api/register
**Request**:
```protobuf
message RegistrationRequest {
  string username = 1;
  string email = 2;
  string password = 3;
}
```

**Response**:
```protobuf
message ApiResponse {
  bool success = 1;
  string message = 2






 {




:






**









****



  ****



****


**
****
********

******  ************************
** ** **** is**
******** ****************** ****** /****   ** **     |**** **  ( ** ** **Response**:
```protobuf
message ApiResponse {
  bool success = 1;
  string message = 2;
  oneof data {
    LoginResponseData login_response = 3;
  }
}

message LoginResponseData {
  string username = 1;
  string email = 2;
}
```

**Status Codes**:
- 200: Success
- 400: Bad request (validation)
- 401: Unauthorized (invalid credentials or unverified)
- 500: Server error

#### POST /api/verify-email
**Request**:
```protobuf
message EmailVerificationRequest {
  string token = 1;
}
```

**Response**:
```protobuf
message ApiResponse {
  bool success = 1;
  string message = 2;
  oneof data {
    EmptyData empty = 5;
  }
}
```

**Status Codes**:
- 20: Success
- 400: Invalid/expired token
- 500: Server error

#### GET /api/health
**Response**:
```protobuf
message ApiResponse {
  bool success =1;
1;
  string message = 2;
  oneof data {
    HealthData health_data = 4;
  }
}

message HealthData {
  string status = 1;
}
```

---

## 9. Implementation Files

### Backend Files
- `backend/src/db/mod.rs` - Database schema and operations
- `backend/src/api.rs` - API handlers (registration, login, verification)
- `backend/src/main.rs` - Server setup and routing
- `backend/src/generated/api.v1.rs` - Protobuf definitions (generated)

### Frontend Files
- `frontend/src/pages/register.vue` - Registration page
- `frontend/src/pages/verify-email.vue` - Email verification page
- `frontend/src/components/UserRegistration.vue` - Registration component
- `frontend/src/services/api.ts` - API service layer
- `frontend/src/generated/api.ts` - Protobuf definitions (generated)

### Proto Files
- `proto/api.proto` - API message definitions

### Documentation Files
- `documentation/email-verify-feat.md` - Email verification feature spec
- `documentation/email-verify-implementation.md` - Implementation summary
- `documentation/registration-module.md` - This file

---

## 10. Usage Examples

### Development Mode
```bash
# Backend
cd backend
cargo run -- --dev

# Frontend (separate terminal)
cd frontend
npm run dev
```

### Registration Flow Example
```typescript
// Frontend API call
import { registerUser } from '@/services/api';

const response = await registerUser({
  username: 'john_doe',
  email: 'john@example.com',
  password: 'SecureP@ssw0rd123'
});

if (response.success) {
  // User sees: "Registration successful! Please check your email to verify your account."
  // Mock email logged to console with verification link
}
```

### Verification Flow Example
```typescript
// User clicks email link: https://example.com/verify-email?token=abc123...
// Frontend automatically calls verification

import { verifyEmail } from '@/services/api';

const token = new URLSearchParams(window.location.search).get('token');
const response = await verifyEmail(token);

if (response.success) {
  // User sees: "Email verified successfully! You can now log in."
}
```

---

## 11. Testing Checklist

### Unit Tests
- [ ] Input validation (username, email, password)
- [ ] Token generation (entropy, format)
- [ ] Token hashing (deterministic)
- [ ] Password hashing (Argon2)
- [ ] Database operations (CRUD)

### Integration Tests
- [ ] Complete registration flow
- [ ] Email verification flow
- [ ] Login with verified/unverified email
- [ ] Token expiration
 [ ] Error handling (invalid inputs, duplicate usernames)
- [ ] Concurrent registrations

### Security Tests
- [ ] No plaintext passwords in logs
- [ ] No plaintext tokens in database
- [ ] Generic error messages only
- [ ] SQL injection prevention
- [ ] Replay attack prevention (single-use tokens)
- [ ] Timing attack resistance

### Manual Testing
- [ ] Frontend form validation
- [ ] API response handling
- [ ] Email link format
- [ ] User experience flow
- [ ] Error message clarity

---

## 12. Deployment Considerations

### Prerequisites
- Rust 1.75+ with cargo
- Node.js 18+ with npm
- SQLite3 with SQLCipher support
- Keyring access (production mode)

### Environment Variables
```bash
# Development
RUST_LOG=backend=debug,tower_http=debug

# Production (uses keyring for DB encryption key)
RUST_LOG=backend=info
```

### Database Setup
- Development: `data_dev.db` (static encryption key)
- Production: `data.db` (keyring-stored encryption key)
- Automatic table creation on first run

### Security Checklist
- [ ] Use HTTPS in production
- [ ] Configure proper CORS policies
- [ ] Set up email service (SMTP)
- [ ] Implement rate limiting
- [ ] Enable production logging
- [ ] Configure keyring properly
- [ ] Set up monitoring/alerting

---

## 13. References

### Security Standards
- OWASP Authentication Cheat Sheet
- OWASP Password Storage Cheat Sheet
- OWASP Email Verification Cheat Sheet
- RFC 5322 (Email format)
- NIST SP 800-63B (Digital Identity Guidelines)

### Libraries & Tools
- Axum (Rust web framework)
- SQLx (async SQL toolkit)
- Argon2 (password hashing)
- SHA2 (token hashing)
- rand_core (CSPRNG)
- Protocol Buffers (API format)
- Vue 3 + Vuetify (Frontend)

---

## 14. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-23 | Initial documentation based on implementation |

---

**Document Generated**: 2025-12-23
**Last Updated**: 2025-12-23
**Status**: Complete
**Review Status**: Ready for review
