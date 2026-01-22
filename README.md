# AppSec Demo

A web application demonstration focused on using good web security practices.

## Features

- User authentication with Argon2 password hashing
- Session management with secure HTTP-only cookies
- Email verification and password reset flows
- Role-based access control (User/Admin)
- TLS/HTTPS support with configurable certificates
- HSTS (HTTP Strict Transport Security)
- SQLCipher database encryption
- CORS protection
- Posts, comments, and ratings system

## Quick Start (Development)

### Prerequisites

- Rust 1.85+
- wasm-pack
- Deno (for OpenAPI client generation)
- Node.js (for frontend build)

### Build

```bash
./build.sh
```

### Run

1. Start a local SMTP server (e.g., MailHog):
   ```bash
   docker run -d -p 1025:1025 -p 8025:8025 mailhog/mailhog
   ```

2. Run the server:
   ```bash
   cd backend && cargo run
   ```

3. Open http://localhost:4000 in your browser.

4. Login with default admin credentials:
   - Username: `admin`
   - Password: `AdminPassword123!`

## Production Deployment

### Option 1: Docker Compose (Recommended)

1. **Generate TLS certificates**:
   ```bash
   mkdir -p certs
   # For testing with self-signed certificates:
   ./generate-certs.sh
   
   # For production, use certificates from a trusted CA (e.g., Let's Encrypt)
   ```

2. **Create a production config.toml**:
   ```toml
   [database]
   path = "/app/data/data.db"
   encrypt = true
   # key = "your-64-char-hex-key"  # Or set via APPSEC__DATABASE__KEY env var
   
   [server]
   bind_addr = "0.0.0.0"
   port = 4000
   openapi = false
   
   [urls]
   base_url = "https://your-domain.com"
   
   [tls]
   enabled = true
   cert_path = "/app/certs/cert.pem"
   key_path = "/app/certs/key.pem"
   
   [security]
   hsts_enabled = true
   # cors_allowed_origins defaults to base_url if empty
   
   [admin]
   username = "admin"
   email = "admin@your-domain.com"
   password = "YourSecurePassword123!"
   
   [mail]
   smtp_host = "your-smtp-server"
   smtp_port = 587
   from_email = "noreply@your-domain.com"
   ```

3. **Set the database encryption key** (if using encryption):
   ```bash
   # Generate a key
   openssl rand -hex 32
   
   # Set in .env file
   echo "APPSEC__DATABASE__KEY=<your-64-char-hex-key>" > .env
   ```

4. **Start the application**:
   ```bash
   docker-compose up -d
   ```

5. **Access the application** at https://localhost:4000

### Option 2: Native Binary

1. **Build the release binary**:
   ```bash
   ./build.sh
   ```

2. **Configure** by editing `config.toml` with production settings.

3. **Set environment variables** (optional, overrides config.toml):
   ```bash
   export APPSEC__DATABASE__KEY=$(openssl rand -hex 32)
   export APPSEC__ADMIN__PASSWORD="YourSecurePassword"
   ```

4. **Run the server**:
   ```bash
   cd backend
   RUST_LOG=info ./target/release/appsec-server
   ```

## Configuration Reference

All configuration is done via `config.toml`. Environment variables can override any setting using the `APPSEC__` prefix with `__` as separator.

Example: `APPSEC__SERVER__PORT=8080` overrides `[server] port`.

### Database

| Setting | Description | Default |
|---------|-------------|---------|
| `path` | Path to SQLite database file | `data.db` |
| `encrypt` | Enable SQLCipher encryption | `false` |
| `key` | Encryption key (64 hex chars) | (uses keyring) |
| `keyring_service_name` | Keyring service for encryption key | `APPSEC_DB_KEY` |
| `keyring_username` | Keyring username | `APPSEC` |
| `db_key_length` | Encryption key length in bytes | `32` |

### Server

| Setting | Description | Default |
|---------|-------------|---------|
| `bind_addr` | IP address to bind to | `127.0.0.1` |
| `port` | Port to listen on | `4000` |
| `openapi` | Enable OpenAPI docs at /api/docs | `true` |
| `max_body_size` | Maximum request body size | `10485760` |
| `uploads_folder` | Directory for file uploads | `uploads` |

### URLs

| Setting | Description | Default |
|---------|-------------|---------|
| `base_url` | Base URL for email links and default CORS origin | `http://localhost:4000` |

### TLS

| Setting | Description | Default |
|---------|-------------|---------|
| `enabled` | Enable HTTPS | `false` |
| `cert_path` | Path to certificate PEM file | `certs/cert.pem` |
| `key_path` | Path to private key PEM file | `certs/key.pem` |

### Security

| Setting | Description | Default |
|---------|-------------|---------|
| `hsts_enabled` | Enable HSTS header | `false` |
| `hsts_max_age_seconds` | HSTS max-age | `31536000` (1 year) |
| `hsts_include_subdomains` | Include subdomains in HSTS | `true` |
| `hsts_preload` | Enable HSTS preload | `false` |
| `cors_allowed_origins` | Allowed CORS origins (comma-separated) | (defaults to `base_url`) |

#### CORS Configuration

CORS (Cross-Origin Resource Sharing) controls which origins can access the API.

- **Default behavior**: If `cors_allowed_origins` is empty, the application uses `base_url` as the only allowed origin.
- **Multiple origins**: Set `cors_allowed_origins` to a comma-separated list of URLs.
  ```toml
  cors_allowed_origins = "https://example.com,https://www.example.com"
  ```
- **Credentials**: The application always allows credentials (cookies) for authenticated requests.

### Admin

| Setting | Description | Default |
|---------|-------------|---------|
| `username` | Default admin username | `admin` |
| `email` | Default admin email | `admin@localhost` |
| `password` | Default admin password | `AdminPassword123!` |

The default admin user is created automatically on first startup if no users exist in the database.

### Session & Tokens

| Setting | Description | Default |
|---------|-------------|---------|
| `session.duration_days` | Session validity period | `7` |
| `session.token_bytes` | Session token size | `32` |
| `token.email_verification_duration_hours` | Email token validity | `2` |
| `token.password_reset_duration_hours` | Password reset token validity | `1` |

### Mail

| Setting | Description | Default |
|---------|-------------|---------|
| `smtp_host` | SMTP server hostname | `127.0.0.1` |
| `smtp_port` | SMTP server port | `1025` |
| `from_email` | Sender email address | `noreply@appsec.local` |

## Security Considerations

1. **Change default admin credentials** before deploying to production.

2. **Use strong database encryption key** - generate with `openssl rand -hex 32`.

3. **Enable TLS** with certificates from a trusted CA for production.

4. **CORS is secure by default** - only allows requests from `base_url`. Add additional origins to `cors_allowed_origins` only if needed.

5. **Disable OpenAPI** (`openapi = false`) in production to reduce attack surface.

6. **Use a proper SMTP server** for production email delivery.

7. **Set appropriate HSTS settings** if submitting to the HSTS preload list.

## Project Structure

```
.
├── backend/          # Rust Axum backend server
├── frontend/         # Vue.js + Vuetify frontend
├── translator/       # Rust crate for i18n (compiled to WASM)
├── field-validator/  # Rust crate for validation (compiled to WASM)
├── api-types/        # Shared API type definitions
├── documentation/    # LaTeX documentation
├── config.toml       # Application configuration
├── Dockerfile        # Multi-stage Docker build
└── docker-compose.yml
```

## License

This project is for educational/demonstration purposes.
