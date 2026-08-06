#!/usr/bin/env bash
# ==============================================================================
# WebX Metrics Pro - Safe Test Service Startup Script
# ==============================================================================
# This script configures and launches the Rust service with all required secure
# environment variables, ensuring fail-closed controls are satisfied.
# ==============================================================================
set -euo pipefail

# --- Configuration ---
export APP_ENV="development"
export DATABASE_URL="sqlite://data.db?mode=rwc"
export JWT_SECRET="test_jwt_secret_must_be_extremely_long_minimum_32_chars"
export ADMIN_PASS="admin_secret_password_999"
export VIEWER_PASS="viewer_secret_password_999"
export PROM_TOKEN="test_prom_token_999"

PORT=3000

echo "=== WebX Metrics Pro: Launching secured test service ==="
echo "[i] Environment: $APP_ENV"
echo "[i] Database:    $DATABASE_URL"

# Check if port is already in use
if lsof -i :$PORT >/dev/null 2>&1; then
    echo "[-] Error: Port $PORT is already in use. Attempting to terminate existing process..."
    kill $(lsof -t -i :$PORT) 2>/dev/null || true
    sleep 1
fi

echo "[+] Port $PORT is free. Launching cargo run..."
cargo run
