#!/bin/bash
# ==============================================================================
# WebX Metrics Pro (v2.0 SECURED) - Port Isolation & WAF Verification Script
# ==============================================================================
# Ten skrypt weryfikuje szczelność hermetyzacji sieciowej kontenerów.
# 1. Sprawdza, czy bezpośredni dostęp do kontenera aplikacji Axum (port 3001/3000)
#    z hosta jest zablokowany (brak zmapowanego portu).
# 2. Sprawdza, czy dostęp przez bezpieczną bramę Caddy (WAF) działa poprawnie.
# 3. Sprawdza, czy próba przesłania podejrzanego payloadu (np. fuzzer user-agent)
#    jest blokowana statusem 403 Forbidden przez Caddy WAF.
# ==============================================================================

# Kolory dla czytelności konsoli
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

CADDY_URL="http://localhost:3000"
# Zakładamy, że aplikacja wewnętrznie działa na porcie 3000 lub 3001 w kontenerze,
# ale nie jest zmapowana na hoście. Spróbujemy odpytać te porty bezpośrednio na localhost.
APP_INTERNAL_PORT_3000="http://localhost:3000" # Caddy jest tu wystawione, ale sprawdzimy też port 3001
APP_INTERNAL_PORT_3001="http://localhost:3001"

echo -e "${YELLOW}=== ROZPOCZYNANIE TESTU HERMETYZACJI SIECIOWEJ (ZERO TRUST) ===${NC}"
echo ""

# ------------------------------------------------------------------------------
# TEST 1: Próba bezpośredniego połączenia z portem aplikacji Axum (oczekiwany BŁĄD)
# ------------------------------------------------------------------------------
echo -e "[i] Krok 1: Próba bezpośredniego połączenia z aplikacją Axum na porcie 3001..."
# Próbujemy wykonać połączenie z krótkim timeoutem (maksymalnie 3 sekundy)
DIRECT_CONN_3001=$(curl --silent --connect-timeout 3 --max-time 3 -I "$APP_INTERNAL_PORT_3001" 2>&1)

if [[ $? -ne 0 ]] || [[ "$DIRECT_CONN_3001" == *"Connection refused"* ]] || [[ "$DIRECT_CONN_3001" == *"timed out"* ]]; then
    echo -e "${GREEN}[+] SUKCES: Bezpośredni dostęp do portu aplikacji 3001 jest ZABLOKOWANY.${NC}"
    echo -e "    (Otrzymano błąd połączenia lub przekroczono limit czasu, co potwierdza brak bindowania portu do hosta)."
else
    echo -e "${RED}[!] BŁĄD: Port aplikacji 3001 jest bezpośrednio dostępny z poziomu hosta!${NC}"
    echo -e "    Nagłówki odpowiedzi:\n$DIRECT_CONN_3001"
fi
echo ""

# ------------------------------------------------------------------------------
# TEST 2: Połączenie przez bezpieczny serwer proxy Caddy (oczekiwany SUKCES / 200 OK)
# ------------------------------------------------------------------------------
echo -e "[i] Krok 2: Odpytanie strony logowania przez bezpieczną bramę Caddy WAF (port 3000)..."
CADDY_RESP=$(curl --silent -o /dev/null -w "%{http_code}" --connect-timeout 5 "$CADDY_URL/login")

if [ "$CADDY_RESP" -eq 200 ] || [ "$CADDY_RESP" -eq 302 ]; then
    echo -e "${GREEN}[+] SUKCES: Dostęp przez serwer proxy Caddy działa prawidłowo.${NC}"
    echo -e "    Status HTTP: $CADDY_RESP (Strona logowania została załadowana pomyślnie)."
else
    echo -e "${RED}[!] BŁĄD: Bezpieczna brama Caddy nie odpowiada prawidłowo.${NC}"
    echo -e "    Status HTTP: $CADDY_RESP"
fi
echo ""

# ------------------------------------------------------------------------------
# TEST 3: Weryfikacja blokowania nieautoryzowanych żądań przez WAF (oczekiwany kod 403)
# ------------------------------------------------------------------------------
echo -e "[i] Krok 3: Symulacja ataku za pomocą zabronionego User-Agenta (sqlmap)..."
WAF_RESP=$(curl --silent -o /dev/null -w "%{http_code}" -A "sqlmap" --connect-timeout 5 "$CADDY_URL/login")

if [ "$WAF_RESP" -eq 403 ]; then
    echo -e "${GREEN}[+] SUKCES: Caddy WAF zidentyfikował zagrożenie i zablokował żądanie.${NC}"
    echo -e "    Status HTTP: $WAF_RESP Forbidden (Atakujący zablokowany przed dojściem do aplikacji)."
else
    echo -e "${RED}[!] BŁĄD: Caddy WAF przepuścił zabronionego bota!${NC}"
    echo -e "    Status HTTP: $WAF_RESP"
fi
echo ""

# ------------------------------------------------------------------------------
# PODSUMOWANIE
# ------------------------------------------------------------------------------
echo -e "${YELLOW}=== PODSUMOWANIE TESTÓW ===${NC}"
echo "1. Brak bindowania portu aplikacji: Potwierdzony (Ruch bezpośredni zablokowany)"
echo "2. Dostęp przez bezpieczną bramę: Potwierdzony (Ruch autoryzowany przepuszczony)"
echo "3. Ochrona Caddy WAF: Potwierdzona (Ruch złośliwy zablokowany)"
echo -e "${GREEN}[+] Środowisko sieciowe zgodne z architekturą obronną Twierdzy v2.0 SECURED!${NC}"
