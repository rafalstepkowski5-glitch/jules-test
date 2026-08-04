#!/bin/bash
set -e

TEST_DB="test_data.db"
BACKUP_DIR="./backups"

echo "=== STARTING BACKUP & RECOVERY AUTOMATION TEST ==="
rm -f "$TEST_DB"
rm -f "$TEST_DB.pre_restore_"*
rm -rf "$BACKUP_DIR"

echo "[i] Step 1: Initializing test database with mock schema..."
sqlite3 "$TEST_DB" <<SQL
CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT, role TEXT);
INSERT INTO users (id, username, role) VALUES (1, 'admin', 'admin');
INSERT INTO users (id, username, role) VALUES (2, 'viewer', 'viewer');

CREATE TABLE metrics (id INTEGER PRIMARY KEY, cpu REAL, mem REAL);
INSERT INTO metrics (id, cpu, mem) VALUES (1, 12.5, 256.0);
INSERT INTO metrics (id, cpu, mem) VALUES (2, 33.1, 512.2);
SQL

echo "[+] Test database created successfully. Content:"
sqlite3 "$TEST_DB" "SELECT * FROM users;"
sqlite3 "$TEST_DB" "SELECT * FROM metrics;"
echo

echo "[i] Step 2: Running automated online backup..."
./backup-sqlite.sh "$TEST_DB" "$BACKUP_DIR" 7
echo

echo "[i] Step 3: Simulating critical database corruption..."
# Corrupt the database
dd if=/dev/urandom of="$TEST_DB" bs=1024 count=1 conv=notrunc &>/dev/null
echo "[i] Running database integrity check on corrupted file..."
if sqlite3 "$TEST_DB" "PRAGMA integrity_check;" | grep -q "ok"; then
    echo "[-] Failed to corrupt DB."
    exit 1
else
    echo "[+] SUCCESS: Database is proven to be corrupted/unreadable!"
fi
echo

echo "[i] Step 4: Running recovery restore procedure..."
LATEST_BACKUP=$(ls -1t $BACKUP_DIR/webx_backup_*.sqlite | head -n 1)
export ASSUME_YES=1
./restore-sqlite.sh "$LATEST_BACKUP" "$TEST_DB"
echo

echo "[i] Step 5: Validating restored database data..."
INTEGRITY=$(sqlite3 "$TEST_DB" "PRAGMA integrity_check;")
if [ "$INTEGRITY" = "ok" ]; then
  echo "[+] SUCCESS: Database integrity check passed!"
  echo "[i] Restored users:"
  sqlite3 "$TEST_DB" "SELECT * FROM users;"
  echo "[i] Restored metrics:"
  sqlite3 "$TEST_DB" "SELECT * FROM metrics;"
  echo
  echo "🎉 ALL TESTS PASSED SUCCESSFULLY! BACKUP/RESTORE SYSTEM VALIDATED!"
else
  echo "[-] Failed."
  exit 1
fi
