#!/usr/bin/env bash

# Development script with full-app hot reloading
# Watches backend, frontend, and shared modules for changes
# Rebuilds and restarts the entire application on any change

set -e

# Check for watchexec
if ! command -v watchexec &> /dev/null; then
    echo "Error: watchexec not found."
    echo "Install with: cargo install watchexec-cli"
    echo "        or:   brew install watchexec"
    exit 1
fi

# Check for mailhog
if ! command -v mailhog &> /dev/null; then
    echo "Warning: mailhog not found. Email testing will not work."
    echo "Install with: brew install mailhog"
    echo "        or:   go install github.com/mailhog/MailHog@latest"
fi

cleanup() {
    echo ""
    echo "Shutting down..."
    kill $(jobs -p) 2>/dev/null
    exit 0
}

trap cleanup EXIT INT TERM

# Start MailHog in background (if available)
if command -v mailhog &> /dev/null; then
    mailhog &
    echo "MailHog started at http://localhost:8025"
fi

echo "Starting development mode with full-app watching..."
echo ""
echo "Watching:"
echo "  - backend/src"
echo "  - frontend/src"
echo "  - field-validator/src"
echo "  - translator/src"
echo "  - api-types/src"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Watch and rebuild on changes
# --on-busy-update restart: kills running process and starts new one on change
# --debounce 1000: wait 1 second after last change before rebuilding
# --stop-signal SIGTERM: gracefully stop the backend server
#
# Note: watchexec respects .gitignore by default, so generated/ and wasm/ are excluded.
watchexec \
    --watch backend/src \
    --watch frontend/src \
    --watch field-validator/src \
    --watch translator/src \
    --watch api-types/src \
    --exts rs,ts,vue,css,scss,yml,yaml \
    --debounce 1000 \
    --on-busy-update restart \
    --stop-signal SIGTERM \
    -- bash -c './build.sh --skip-docs && cd backend && cargo run -- --dev'
