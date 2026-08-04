#!/usr/bin/env bash
set -euo pipefail

echo "=== Installing system deps for SQLCipher + musl ==="
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev musl-tools sqlcipher openssl

echo "=== Installing Rust targets and tools ==="
rustup target add x86_64-unknown-linux-musl
cargo install cargo-audit cargo-deny cargo-cyclonedx cargo-auditable --locked || true

echo "=== Creating secrets and data folders ==="
mkdir -p data secrets

if [ ! -f secrets/db_key.txt ]; then
  openssl rand -base64 48 > secrets/db_key.txt
fi

if [ ! -f .env ]; then
  echo "JWT_SECRET=$(openssl rand -base64 48)" >>.env
  echo "PROM_TOKEN=$(openssl rand -base64 32)" >>.env
  echo "DATABASE_URL=sqlite://./data/metrics.db" >>.env
  echo "DB_ENCRYPTION_KEY=$(cat secrets/db_key.txt)" >>.env
fi

echo "=== Setup complete. Run: source .env && cargo run ==="
#!/usr/bin/env bash
set -e

echo "=== Installing system deps for SQLCipher + musl ==="
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev musl-tools sqlcipher

echo "=== Installing Rust targets and tools ==="
rustup target add x86_64-unknown-linux-musl
cargo install cargo-audit cargo-deny cargo-cyclonedx cargo-auditable

echo "=== Creating secrets and data folders ==="
mkdir -p data secrets

if [ ! -f secrets/db_key.txt ]; then
  openssl rand -base64 48 > secrets/db_key.txt
  echo "DB_ENCRYPTION_KEY=$(cat secrets/db_key.txt)" >> .env
fi

if ! grep -q "JWT_SECRET" .env 2>/dev/null; then
  echo "JWT_SECRET=$(openssl rand -base64 48)" >> .env
  echo "PROM_TOKEN=$(openssl rand -base64 32)" >> .env
  echo "DATABASE_URL=sqlite://./data/metrics.db" >> .env
fi

echo "=== Done. Run 'source .env && cargo run' ==="
