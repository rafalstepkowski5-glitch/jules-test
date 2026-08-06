#!/usr/bin/env bash
set -euo pipefail

# migrate_to_sqlcipher.sh
# Usage: ./migrate_to_sqlcipher.sh /path/to/data.db /path/to/encrypted.db "YOUR_SQLCIPHER_KEY"
# Requires: sqlite3, sqlcipher
#
# What it does:
# 1) creates an offline backup of the original DB
# 2) dumps plaintext DB to SQL
# 3) creates encrypted DB with sqlcipher and loads dump
# 4) verifies encrypted DB is unreadable without key and readable with key
# 5) cleans up temporary files
#
# IMPORTANT: Do not run on production DB in-place. Always operate on a copy.

ORIG_DB="${1:-data.db}"
ENC_DB="${2:-encrypted.db}"
SQLCIPHER_KEY="${3:-}"

TMP_DUMP="$(mktemp --suffix=.sql)"
BACKUP="${ORIG_DB}.bak.$(date +%Y%m%d%H%M%S)"

if [ -z "$SQLCIPHER_KEY" ]; then
  echo "ERROR: Provide SQLCIPHER_KEY as third argument."
  echo "Usage: $0 /path/to/data.db /path/to/encrypted.db 'YOUR_SQLCIPHER_KEY'"
  exit 2
fi

echo "[1/7] Creating offline backup of original DB -> $BACKUP"
cp -- "$ORIG_DB" "$BACKUP"

echo "[2/7] Exporting plaintext DB to SQL dump -> $TMP_DUMP"
sqlite3 "$ORIG_DB" .dump > "$TMP_DUMP"

echo "[3/7] Creating encrypted DB and applying PRAGMA key"
# Create encrypted DB and load dump using sqlcipher
sqlcipher "$ENC_DB" <<SQLCIPHER_CMDS
PRAGMA key = '$SQLCIPHER_KEY';
.read $TMP_DUMP
.exit
SQLCIPHER_CMDS

echo "[4/7] Verifying encrypted DB cannot be read without key (expected failure)"
set +e
if sqlcipher "$ENC_DB" "SELECT count(*) FROM sqlite_master;" >/dev/null 2>&1; then
  echo "WARNING: encrypted DB appears readable without key — verify sqlcipher installation and DB format."
else
  echo "OK: encrypted DB not readable without key."
fi
set -e

echo "[5/7] Verifying encrypted DB can be opened with provided key"
sqlcipher "$ENC_DB" <<SQLCIPHER_VERIFY
PRAGMA key = '$SQLCIPHER_KEY';
SELECT name FROM sqlite_master WHERE type='table' LIMIT 5;
.exit
SQLCIPHER_VERIFY

echo "[6/7] (Optional) Run migrations on encrypted DB"
echo "If you use sqlx migrations, set DATABASE_URL to sqlite://$ENC_DB and run your migration runner."

echo "[7/7] Cleanup temporary dump"
rm -f "$TMP_DUMP"

echo "Migration complete. Encrypted DB: $ENC_DB"
echo "Backup of original DB: $BACKUP"
echo "IMPORTANT: Store SQLCIPHER_KEY in KMS/CI secrets and update app config to use encrypted DB."
