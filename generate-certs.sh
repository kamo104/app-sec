#!/bin/bash
# Generate self-signed TLS certificates for local development/testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CERTS_DIR="$SCRIPT_DIR/certs"

mkdir -p "$CERTS_DIR"

echo "Generating self-signed certificate..."

openssl req -x509 -newkey rsa:4096 \
    -keyout "$CERTS_DIR/key.pem" \
    -out "$CERTS_DIR/cert.pem" \
    -days 365 \
    -nodes \
    -subj "/CN=localhost/O=AppSec/C=US" \
    -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"

echo "Certificates generated:"
echo "  - $CERTS_DIR/cert.pem"
echo "  - $CERTS_DIR/key.pem"
echo ""
echo "Valid for 365 days."
