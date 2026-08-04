# --- SEKCJA BAZY DANYCH (SQLITE) ---

.PHONY: backup restore test-backup

# Wykonanie bezpiecznej kopii zapasowej online działającej bazy danych
backup:
	@echo "[i] Uruchamianie automatycznej kopii zapasowej bazy danych..."
	@chmod +x ./backup-sqlite-v3.sh
	@./backup-sqlite-v3.sh

# Przywrócenie bazy danych z określonego pliku kopii zapasowej
# Użycie: make restore BACKUP=backups/nazwa_pliku.sqlite
restore:
	@if [ -z "$(BACKUP)" ]; then \
		echo "[!] BŁĄD: Wskaż plik kopii zapasowej za pomocą zmiennej BACKUP."; \
		echo "    Przykład: make restore BACKUP=backups/webx_backup_20260803_191958.sqlite"; \
		exit 1; \
	fi
	@chmod +x ./restore-sqlite.sh
	@./restore-sqlite.sh $(BACKUP)

# Uruchomienie pełnego testu integracyjnego backup/restore w środowisku testowym
test-backup:
	@chmod +x ./test-backup-restore.sh
	@./test-backup-restore.sh
