# Raport Techniczny i Playbook Scenariuszy Testowych: WebX Metrics Pro v2.0 SECURED

## 1. Wprowadzenie i Modelowanie Zagrożeń (Threat Modeling)
System WebX Metrics Pro v2.0 SECURED, rozwijany pod kryptonimem „Twierdza”, stanowi rygorystyczną odpowiedź na współczesne zagrożenia wymierzone w infrastrukturę monitorującą. W dobie zautomatyzowanych ataków napędzanych przez LLM oraz zaawansowane narzędzia do fuzzingu API, przejście z wersji v1.0 na v2.0 nie było jedynie inkrementalną zmianą – to strategiczna redefinicja bezpieczeństwa. Jako Główny Architekt, wymusiłem odejście od niebezpiecznych wzorców typowych dla agentów pisanych w C++, zastępując je gwarancjami bezpieczeństwa pamięci (Memory Safety) języka Rust. Eliminacja całych klas podatności, takich jak buffer overflows czy use-after-free, stanowi fundament, na którym zbudowaliśmy wielowarstwową strukturę obronną.

### 1.1. Charakterystyka Architektury Obronnej
Nasza architektura opiera się na zasadzie ograniczonego zaufania i izolacji komponentów:
- **Zaufane Jądro (Rust Binary):** Wykorzystujemy silne typowanie i system własności Rusta, aby zapewnić bezpieczeństwo operacji na danych. Logika biznesowa jest odizolowana od warstwy prezentacji, co w połączeniu z brakiem panik (eliminacja unwrap()) gwarantuje ciągłość działania.
- **Magazyn Danych (SQLite):** Wykorzystujemy parametryzowane zapytania poprzez sqlx. Co krytyczne dla odporności na ataki typu DoS nakierowane na wyczerpanie zasobów (Storage Exhaustion), system wymusza rygorystyczny limit 10 000 rekordów w tabeli metryk, automatycznie czyszcząc najstarsze wpisy.
- **Hartowane Proxy (Caddy):** Pierwsza linia obrony realizująca funkcje WAF. Caddy odpowiada za rygorystyczną terminację TLS 1.3 oraz filtrowanie żądań na podstawie sygnatur narzędzi skanujących i podejrzanych wzorców URL.
- **Granice Zaufania (Trust Boundaries):** Każde żądanie przekraczające granicę pomiędzy siecią publiczną a serwerem Caddy jest poddawane inspekcji. Backend traktuje każde wejście (headers, body, cookies) jako potencjalnie złośliwe, dopóki nie zostanie ono zwalidowane przez middleware bezpieczeństwa.

### 1.2. Mapowanie Wektorów Ataku na Mechanizmy Obronne
Poniższa tabela przedstawia synergię pomiędzy zidentyfikowanymi zagrożeniami a wdrożonymi mechanizmami obronnymi (Defense in Depth):

| Wektor Ataku | Implementacja Obronna | Technologia / Mechanizm |
|---|---|---|
| XSS (Cross-Site Scripting) | Całkowita eliminacja inline JS i bezpieczne renderowanie | Content Security Policy (CSP) & Server-Side SVG |
| CSRF (Cross-Site Request Forgery) | Mechanizm Double Submit Cookie | Weryfikacja X-CSRF-Token + SameSite=Strict |
| Brute-Force & Fuzzing | Wielopoziomowe limitowanie żądań | DashMap Rate Limiting (Login: 5/min, Global: 100/min) |
| SQL Injection | Pełna parametryzacja zapytań | sqlx Prepared Statements |
| DoS (Resource Exhaustion) | Limity rozmiaru body i retencja danych | Body Limit (1MB) & 10k Record Retention Policy |
| Information Leakage | Niestandardowa obsługa błędów | Eliminacja unwrap() i ukrywanie szczegółów technicznych |
| Supply Chain Attack | Rygorystyczny audyt zależności | cargo-audit, cargo-deny & SLSA Level 3 |

Podsumowując, wielowarstwowość systemu sprawia, że kompromitacja pojedynczego mechanizmu nie prowadzi do upadku „Twierdzy”. Każda warstwa posiada niezależny mechanizm kontrolny.

## 2. Playbook Scenariuszy Testowych (Red Teaming Playbook)
Playbook ten stanowi fundament procesu Continuous Security Validation. Każdy test ma na celu weryfikację specyficznego mechanizmu obronnego poprzez symulację realnych prób eskalacji i nadużyć.

### 2.1. Weryfikacja Rate Limitingu i Odporności Brute-Force
- **Red Team Insight:** Atakujący będzie próbował nie tylko przełamać auth, ale także zdestabilizować API poprzez masowe żądania. Weryfikujemy dwa limity: login (5 prób/min) oraz globalny (100 req/min).
- **Scenariusz:** Wywołanie blokady 429 Too Many Requests.
```bash
# Symulacja ataku brute-force na login
for i in {1..7}; do curl -i -X POST http://localhost:3000/api/auth/login \
-H "Content-Type: application/json" \
-d '{"username":"admin", "password":"wrong_password"}'; done
```
- **Oczekiwana reakcja:** Szóste żądanie musi zwrócić kod 429. Należy zweryfikować, czy body odpowiedzi nie zawiera żadnych informacji o wersji systemu ani o strukturze backendu (brak info-leak).

### 2.2. Testy Zabezpieczeń CSRF (Cross-Site Request Forgery)
Wykorzystujemy wzorzec Double Submit Cookie jako uzupełnienie dla ciasteczek SameSite=Strict. Jest to niezbędne, ponieważ aplikacja łączy API z renderowanymi szablonami.
- **Testy porównawcze (POST /api/export/pdf):**
  - **Brak nagłówka:** `curl -i -X POST http://localhost:3000/api/export/pdf` -> 403 Forbidden.
  - **Mismatch tokenów:** Wysłanie poprawnego ciasteczka csrf_token, ale błędnej wartości w nagłówku X-CSRF-Token -> 403 Forbidden.
  - **Poprawne żądanie:** Zgodność tokenów w obu miejscach -> 200 OK.

### 2.3. Autoryzacja Metryk Prometheusa
Endpoint `/metrics` jest całkowicie odizolowany od standardowej sesji użytkownika. Dostęp jest możliwy wyłącznie dla scraperów posiadających statyczny token techniczny.
- **Komendy weryfikacyjne:**
```bash
curl -i http://localhost:3000/metrics -> 401 Unauthorized.
curl -i -H "Authorization: Bearer <PROM_TOKEN>" http://localhost:3000/metrics -> 200 OK.
```

### 2.4. Weryfikacja Blokad WAF (Caddy)
Nasze proxy Caddy implementuje filtry oparte na wyrażeniach regularnych (regex), aby odrzucić złośliwe ładunki przed ich przetworzeniem przez backend.
- **Próba ataku (Oczekiwany kod 403 z poziomu proxy):**
  - **Path Traversal:** `curl -i http://localhost:3000/../../etc/passwd`
  - **SQLi Payload:** `curl -i "http://localhost:3000/api/metrics?id=1%20union%20select%20null"`
  - **Zablokowany User-Agent (nuclei/sqlmap):** `curl -i -A "nuclei" http://localhost:3000/`

### 2.5. Kontrola Dostępu Oparta na Rolach (RBAC)
Testowanie szczelności ekstraktora AuthUser.
- **Scenariusz eskalacji:** Użytkownik viewer próbuje uzyskać dostęp do funkcji administracyjnej eksportu PDF.
```bash
curl -i -X POST http://localhost:3000/api/export/pdf -H "Authorization: Bearer <VIEWER_TOKEN>"
```
- **Oczekiwana reakcja:** 403 Forbidden. Ekstraktor ról w Rust musi poprawnie zidentyfikować brak uprawnienia admin w roszczeniach (claims) tokena JWT.

## 3. Głęboka Analiza Kodu i Architektury
Bezpieczeństwo WebX Metrics Pro nie jest „doklejone” na końcu – jest ono integralną częścią logiki binarnej.

### 3.1. Kryptografia i Zarządzanie Hasłami: Argon2id
W `src/auth.rs` stosujemy Argon2id – zwycięzcę Password Hashing Competition. Wybór ten jest podyktowany jego bezprecedensową odpornością na ataki przy użyciu GPU oraz side-channel attacks. W przeciwieństwie do przestarzałych algorytmów jak BCrypt, Argon2id pozwala nam na precyzyjne sterowanie zużyciem pamięci i czasu procesora podczas hashowania, co czyni próby łamania haseł nieekonomicznymi dla napastnika. Sól generowana przez OsRng gwarantuje unikalność każdego hasza.

### 3.2. Cykl Życia Sesji: JWT, Revocation i DB Limits
Implementacja JWT opiera się na krótkotrwałych (15 min) Access Tokens oraz długotrwałych (7 dni) Refresh Tokens.
- **Revocation Blacklist:** Pole jti (JWT ID) nie służy tylko do unikalności. Każdy Refresh Token jest rejestrowany w bazie SQLite. W przypadku wykrycia anomalii lub wylogowania, wpis w bazie jest oznaczany jako revoked. Nasza architektura wymusza sprawdzenie stanu jti przy każdym odświeżeniu, co pozwala na natychmiastowe „zabicie” sesji mimo jej bezstanowego charakteru.
- **Operational Safety:** Aby zapobiec atakom DoS na bazę danych, `db.rs` wymusza limit 10 tysięcy rekordów metryk. To gwarantuje stałą wydajność i przewidywalne zużycie dysku.

### 3.3. Zero-JS Logic: Server-Side SVG i CSP
Najsilniejszą obroną przed XSS w WebX Metrics Pro jest rezygnacja z logiki renderowania wykresów po stronie klienta. Wykresy są generowane w całości na backendzie (`src/charts.rs`) jako czyste tagi `<svg>`.
- **Eliminacja Ataku:** Ponieważ frontend nie pobiera danych JSON do budowy wykresów, nie istnieje wektor ataku polegający na wstrzyknięciu złośliwego kodu do parsera JS.
- **CSP:** Restrykcyjna polityka `script-src 'self'` całkowicie blokuje `unsafe-inline`. Brak skryptów osadzonych bezpośrednio w HTML sprawia, że nawet w przypadku znalezienia luki w renderowaniu tekstu, napastnik nie może wykonać złośliwego kodu JS w kontekście przeglądarki użytkownika.

## 4. Bezpieczny Łańcuch Dostaw i Środowisko Uruchomieniowe
Współczesne włamania często omijają kod aplikacji, atakując jej zależności lub proces budowania. Dlatego WebX Metrics Pro v2.0 implementuje standardy SLSA Level 3.

### 4.1. Supply Chain Hardening: Cargo-Audit i Cargo-Deny
Nasze potoki CI/CD są rygorystycznie chronione:
- **Cargo-Audit:** Każdy build jest skanowany pod kątem znanych CVE w bibliotekach. Wykrycie podatności o krytyczności "Medium" lub wyższej natychmiast przerywa proces budowania.
- **Cargo-Deny:** Automatycznie blokujemy biblioteki na niebezpiecznych licencjach (np. GPL-3.0) oraz duplikaty zależności, które mogłyby zostać wykorzystane do ataków typu dependency confusion.

### 4.2. Atestacja i SBOM (Software Bill of Materials)
Każda binarka produkcyjna posiada cyfrowy paszport:
- **CycloneDX SBOM:** Generujemy pełną listę komponentów, co umożliwia błyskawiczną reakcję w przypadku ogłoszenia nowych luk (np. typu Log4Shell).
- **Cosign (Sigstore/Rekor):** Wykorzystujemy technologię Keyless Signing. Artefakty są podpisywane w oparciu o tożsamość OIDC z GitHub Actions, a certyfikat podpisu jest publikowany w logu transparentności Rekor. Pozwala to operatorowi na weryfikację, czy binarka nie została podmieniona na serwerze dystrybucyjnym.

### 4.3. Izolacja Środowiska: Obrazy Distroless
Aplikacja jest dystrybuowana w obrazach `gcr.io/distroless/cc-debian12:nonroot`. Jest to kluczowy element hartowania runtime'u:
- **Redukcja Attack Surface:** Z obrazu usunięto wszystkie zbędne binarki, w tym powłokę (sh, bash), menedżery pakietów (apt) oraz podstawowe narzędzia (ls, cat, mkdir).
- **Ograniczenie "Living off the Land":** Jeśli napastnik uzyskałby teoretyczną możliwość wykonania kodu (RCE), nie znajdzie wewnątrz kontenera narzędzi do eskalacji uprawnień, skanowania sieci czy exfiltracji danych. Praca na uprawnieniach użytkownika `nonroot` domyka system izolacji.

---
WebX Metrics Pro v2.0 SECURED to nie tylko aplikacja – to hartowany ekosystem, gotowy na najbardziej rygorystyczne audyty bezpieczeństwa i wdrożenia w sektorach o wysokim profilu ryzyka.

## 5. Przewodnik Operacyjny: Prometheus, Trivy i Cosign

### 5.1. Konfiguracja lokalnego Prometheusa z tokenem PROM_TOKEN
Ścieżka `/metrics` w aplikacji jest chroniona i wymaga uwierzytelnienia za pomocą statycznego tokena Bearer (`PROM_TOKEN`), co zapobiega nieautoryzowanemu dostępowi do metryk systemu.
Aby Prometheus mógł pomyślnie pobierać dane, musisz przekazać ten token w nagłówku autoryzacji. Konfiguruje się to bezpośrednio w pliku `prometheus.yml` wewnątrz definicji zadania pobierania danych (`scrape_config`) za pomocą parametru `bearer_token`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'webx-metrics-pro'
    static_configs:
      - targets: ['app:3000'] # Adres kontenera/aplikacji Axum
    
    # Konfiguracja autoryzacji z użyciem tokenu PROM_TOKEN
    bearer_token: 'secret_prom_token' # Podmień na wartość z pliku .env
```
Po uruchomieniu Prometheusa z taką konfiguracją, będzie on automatycznie dołączał nagłówek `Authorization: Bearer secret_prom_token` do każdego zapytania kierowanego na `/metrics`.

### 5.2. Integracja Trivy z GitHub Actions krok po kroku
Integracja narzędzia Trivy w potoku CI/CD (np. w pliku `.github/workflows/release.yml`) pozwala na automatyczne wykrywanie podatności zarówno w kodzie źródłowym (bibliotekach Rust), jak i w ostatecznym obrazie kontenera.

**Krok A: Skanowanie systemu plików (FS Scan) przed kompilacją**
Ten krok sprawdza pliki konfiguracyjne oraz zależności projektu pod kątem znanych podatności (CVE):
```yaml
- name: Run Trivy Static FS Scan
  uses: aquasecurity/trivy-action@0.20.0
  with:
    scan-type: 'fs'          # Skanowanie struktury plików projektu
    format: 'sarif'          # Format wyjściowy kompatybilny z kartą Security na GitHubie
    output: 'trivy-results.sarif'
    severity: 'HIGH,CRITICAL' # Analiza tylko wysokich i krytycznych podatności
```

**Krok B: Skanowanie zbudowanego obrazu kontenera Docker (Image Scan)**
Ten krok weryfikuje ostateczny obraz (w tym pakiety systemu operacyjnego wewnątrz kontenera) przed przesłaniem go do rejestru:
```yaml
- name: Run Trivy Image Scan
  uses: aquasecurity/trivy-action@0.20.0
  with:
    image-ref: 'webx:${{ github.sha }}' # Referencja do lokalnie zbudowanego obrazu
    scan-type: 'image'
    severity: 'HIGH,CRITICAL'
    exit-code: '1' # Przerwij potok (zwróć błąd), jeśli zostaną znalezione krytyczne luki
```

### 5.3. Weryfikacja podpisu binarki i kontenera za pomocą Cosign
Dzięki wdrożeniu podpisywania kryptograficznego Cosign, możesz w łatwy sposób zweryfikować integralność pobranych plików i upewnić się, że nie zostały one podmienione.

**A. Weryfikacja lokalnej binarki (pliku wykonywalnego)**
Gdy pobierasz skompilowaną binarkę systemu, powinieneś znaleźć pliki sygnatur (`.sig`), certyfikatu publicznego (`.pem`) oraz paczkę Rekor (`.bundle`). Aby potwierdzić ich autentyczność, uruchom komendę:
```bash
cosign verify-blob \
  --certificate webx-metrics-pro.pem \
  --signature webx-metrics-pro.sig \
  --bundle webx-metrics-pro.bundle \
  ./webx-metrics-pro
```

**B. Weryfikacja obrazu kontenera w rejestrze (np. GHCR)**
Dla obrazów opublikowanych w rejestrze kontenerów, weryfikację bezkluczykową (Keyless) przeprowadza się poprzez powiązanie tożsamości wydawcy z jego oficjalnym przepływem pracy w GitHub Actions:
```bash
cosign verify ghcr.io/twoj-user/webx-metrics-pro:latest \
  --certificate-identity-regexp "https://github.com/twoj-user/webx-metrics-pro/.github/workflows/release.yml@.*" \
  --issuer https://token.actions.githubusercontent.com
```
