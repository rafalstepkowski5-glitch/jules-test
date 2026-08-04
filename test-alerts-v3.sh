#!/bin/bash
set -e

TEST_DB="test_alert_data.db"
BACKUP_DIR="./backups"

echo "=== STARTING ALERTS & RECOVERY AUTOMATION TEST v3 ==="
rm -f "$TEST_DB"
rm -f "$TEST_DB.pre_restore_"*
mkdir -p "$BACKUP_DIR"

echo "[i] Step 1: Initializing test database with mock schema..."
sqlite3 "$TEST_DB" <<SQL
CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT, role TEXT);
INSERT INTO users (id, username, role) VALUES (1, 'admin', 'admin');
SQL

echo "[i] Step 2: Running automated online backup (SUCCESS SCENARIO)..."
./backup-sqlite-v3.sh "$TEST_DB" "$BACKUP_DIR" 7
echo "[+] Success scenario passed without alerts."
echo

echo "[i] Step 3: Running backup on non-existent database (FAILURE/ALERT SCENARIO)..."
export DISCORD_WEBHOOK_URL="https://discord.com/api/webhooks/mock_test_123/mock"
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/T0000/B0000/MOCK"
export SMTP_HOST="smtp.mock-mail.com"
export SMTP_PORT="587"
export NOTIFICATION_EMAIL="admin@mock.com"
export HOSTNAME="Test-Environment"
export DATABASE_PATH="/mock/data.db"

# Oczekujemy, że to zakończy się błędem, wyłączamy set -e
set +e
./backup-sqlite-v3.sh "NON_EXISTENT_DB.db" "$BACKUP_DIR" 7
set -e

echo "[+] Alert testing scenario completed."
echo "🎉 ALL ALERTS TESTS EXECUTED!"
