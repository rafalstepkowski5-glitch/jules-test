#!/bin/bash
set -e

BACKUP_FILE=$1
TARGET_DB=${2:-"data.db"}
ASSUME_YES=${ASSUME_YES:-0}

echo "=== WebX Metrics Pro - SQLite Safe Recovery ==="

if [ -z "$BACKUP_FILE" ] || [ ! -f "$BACKUP_FILE" ]; then
  echo "[-] Error: Backup file not specified or does not exist."
  echo "Usage: $0 <path_to_backup.sqlite> [target_db]"
  exit 1
fi

echo "[i] Verifying backup file integrity..."
INTEGRITY=$(sqlite3 "$BACKUP_FILE" "PRAGMA integrity_check;")
if [ "$INTEGRITY" != "ok" ]; then
  echo "[-] Error: Backup file is corrupted (integrity_check: $INTEGRITY). Aborting."
  exit 1
fi
echo "[+] Backup file integrity: OK"

if [ "$ASSUME_YES" -eq 1 ]; then
  echo "[i] ASSUME_YES is set. Skipping manual confirmation prompt."
else
  read -p "Are you sure you want to overwrite $TARGET_DB with $BACKUP_FILE? (y/n): " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "[i] Recovery aborted."
    exit 0
  fi
fi

if [ -f "$TARGET_DB" ]; then
  TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
  PRE_RESTORE="$TARGET_DB.pre_restore_$TIMESTAMP"
  echo "[i] Backing up active database to $PRE_RESTORE..."
  cp "$TARGET_DB" "$PRE_RESTORE"
fi

echo "[i] Restoring backup..."
cp "$BACKUP_FILE" "$TARGET_DB"

echo "[i] Verifying restored database integrity..."
FINAL_INTEGRITY=$(sqlite3 "$TARGET_DB" "PRAGMA integrity_check;")
if [ "$FINAL_INTEGRITY" = "ok" ]; then
  echo "[+] Database restored successfully!"
else
  echo "[-] Error: Restored database integrity check failed."
  exit 1
fi
