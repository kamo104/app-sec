# AppSec Demo

A web application focused on good web security practices.

## Deployment

### 1. Generate TLS certificates

```bash
./generate-certs.sh
```

For production, replace `certs/cert.pem` and `certs/key.pem` with certificates from a trusted CA.

### 2. Configure

Edit `config.toml` with your settings. Key sections:

- `[admin]` - Set admin credentials
- `[urls]` - Set `base_url` to your domain
- `[mail]` - Configure SMTP server (MailHog is included for testing)

### 3. Set database encryption key

```bash
echo "APPSEC__DATABASE__KEY=$(openssl rand -hex 32)" > .env
```

### 4. Start

```bash
docker-compose up -d
```

The application is available at https://localhost:4000.

MailHog web UI (for testing emails) is at http://localhost:8025.
