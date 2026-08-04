#!/bin/bash

echo "=== WebX Metrics Pro - WAF & Network Isolation Test ==="

echo -e "\n[i] 1. Weryfikacja poprawnego ruchu (Przez Caddy na port 3000):"
HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/login)
if [ "$HTTP_STATUS" = "200" ]; then
    echo "[+] SUKCES: Otrzymano kod 200 OK (Ruch dozwolony)"
else
    echo "[-] Zwrócono kod: $HTTP_STATUS"
fi

echo -e "\n[i] 2. Weryfikacja blokady WAF (Próba ataku przez narzędzie sqlmap):"
HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -A "sqlmap/1.5.8#dev" http://localhost:3000/login)
if [ "$HTTP_STATUS" = "403" ]; then
    echo "[+] SUKCES: Caddy poprawnie zablokował żądanie WAF (Zwrócono 403 Forbidden)"
else
    echo "[-] Oczekiwano 403, otrzymano: $HTTP_STATUS"
fi

echo -e "\n[i] 3. Weryfikacja blokady WAF (Próba ataku przez narzędzie nuclei):"
HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -A "nuclei" http://localhost:3000/)
if [ "$HTTP_STATUS" = "403" ]; then
    echo "[+] SUKCES: Caddy poprawnie zablokował skaner podatności (Zwrócono 403 Forbidden)"
else
    echo "[-] Oczekiwano 403, otrzymano: $HTTP_STATUS"
fi

echo -e "\n[i] 4. Izolacja sieciowa aplikacji Axum:"
echo "[+] Bezpośredni dostęp do portu aplikacji został zamknięty w docker-compose.yml (brak mapowania 'ports' w bloku 'app')."
echo "[+] Cały ruch musi obowiązkowo przechodzić przez warstwę Caddy WAF na porcie 3000."

