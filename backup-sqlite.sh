#!/bin/bash
set -e

DB_FILE=${1:-"data.db"}
BACKUP_DIR=${2:-"./backups"}
MAX_BACKUPS=${3:-7}

echo "=== WebX Metrics Pro - SQLite Online Backup ==="
echo "[i] DB Source: $DB_FILE"

if [ ! -f "$DB_FILE" ]; then
  echo "[-] Error: Database file '$DB_FILE' does not exist."
  exit 1
fi

mkdir -p "$BACKUP_DIR"

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_FILE="$BACKUP_DIR/webx_backup_$TIMESTAMP.sqlite"

echo "[i] Creating safe backup at: $BACKUP_FILE"
sqlite3 "$DB_FILE" ".backup '$BACKUP_FILE'"

if [ -f "$BACKUP_FILE" ]; then
  echo "[+] Backup created successfully: $BACKUP_FILE"
  
  INTEGRITY=$(sqlite3 "$BACKUP_FILE" "PRAGMA integrity_check;")
  if [ "$INTEGRITY" = "ok" ]; then
    echo "[+] Integrity check passed: OK"
  else
    echo "[-] Warning: Integrity check failed for backup file: $INTEGRITY"
  fi
  
  # Rotation
  BACKUPS_COUNT=$(ls -1q $BACKUP_DIR/webx_backup_*.sqlite 2>/dev/null | wc -l)
  if [ "$BACKUPS_COUNT" -gt "$MAX_BACKUPS" ]; then
    echo "[i] Rotating old backups (keeping last $MAX_BACKUPS)..."
    ls -1t $BACKUP_DIR/webx_backup_*.sqlite | tail -n +$((MAX_BACKUPS + 1)) | xargs rm -f
  fi
else
  echo "[-] Backup failed."
  exit 1
fi
