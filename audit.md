# Raport z Audytu Bezpieczeństwa i SRE (Brutally Critical Review)
**Projekt:** WebX Metrics Pro v2.0 Secured
**Status:** Krytyczny Przegląd Powdrożeniowy

---

## ⚠️ PODSUMOWANIE STATUSU BEZPIECZEŃSTWA
Mimo pomyślnego zamknięcia najbardziej palących podatności P0 (poprawka parsera IP rate limitera, transakcyjna rotacja refresh tokenów oraz eliminacja unwrapów), **architektura systemu nadal wykazuje fundamentalne słabości**, które uniemożliwiają uznanie "Twierdzy v2.0" za w pełni gotową do środowisk o krytycznym poziomie ryzyka.

Poniżej przedstawiono bezwzględną i brutalną analizę pięciu krytycznych wektorów podatności oraz wąskich gardeł wydajnościowych.

---

## 1. Krytyczne luki i wektory ataków (Gaps & Vulnerabilities)

### 🚨 1.1. Plaintext SQLite i brak SQLCipher (Data at Rest Exposure)
* **Status:** **KRYTYCZNY**
* **Opis:** Plik bazy danych `data.db` jest przechowywany na wolumenie kontenera w formacie plaintext. Choć `sqlx` chroni przed SQL Injection, sam nośnik nie jest zaszyfrowany.
* **Wektor Ataku:** Dowolny wyciek kopii zapasowej, nieuprawniony dostęp dewelopera/administratora do hosta Docker, bądź luka Path Traversal/RCE w innym kontenerze współdzielącym sieć/wolumen, umożliwia natychmiastowe pobranie `data.db`.
* **Konsekwencje:** Atakujący uzyskuje dostęp do:
  - Haszy haseł administratorów (Argon2id - podatne na offline brute-force przy użyciu dedykowanych rigów GPU).
  - Wszystkich aktywnych sesji i tokenów `jti` (umożliwia przejmowanie sesji i generowanie tokenów dostępowych).
  - Całej historii telemetrycznej systemu.
* **Rekomendacja:** Natychmiastowa kompilacja binarnego z obsługą `sqlx` i featurem `sqlite-cipher` oraz wymuszenie `PRAGMA key` przy inicjalizacji połączenia DB.

### 🚨 1.2. Brak Rate Limitingu na ciężkich endpointach eksportu (Resource Exhaustion & Denial of Service)
* **Status:** **WYSOKI**
* **Opis:** Endpointy `/api/export/pdf`, `/api/export/md` oraz `/api/export/txt` są chronione tylko uprawnieniem roli `admin`. Nie posiadają jednak żadnego dedykowanego ogranicznika częstotliwości żądań (rate limit).
* **Wektor Ataku:** Przejęcie konta administratora (np. przez kradzież tokenu) lub złośliwy użytkownik wewnętrzny (Insider Threat) może uruchomić skrypt wysyłający setki równoległych żądań generowania PDF.
* **Konsekwencje:** Alokacja pamięci i narzut procesora (CPU) na generowanie plików PDF natychmiastowo blokuje pulę Tokio. Prowadzi to do całkowitego paraliżu (Denial of Service) serwera dla wszystkich pozostałych użytkowników.
* **Rekomendacja:** Wdrożyć dedykowany rate limit na poziomie 2 żądań eksportu na minutę dla każdego konta administratora.

### 🚨 1.3. Permanentny wzrost bazy danych i brak czyszczenia tokenów (Database Bloat / Disk Exhaustion)
* **Status:** **ŚREDNI / SRE BLOCKER**
* **Opis:** Przy każdym pomyślnym logowaniu lub rotacji refresh tokenu w bazie danych zapisywany jest nowy rekord `jti`. W bazie nie istnieje żaden mechanizm usuwania przedawnionych (expired) lub unieważnionych (`revoked = 1`) tokenów.
* **Konsekwencje:** Pod obciążeniem produkcyjnym (tysiące użytkowników rotujących sesje codziennie), tabela `refresh_tokens` urośnie do milionów rekordów. Doprowadzi to do drastycznego spadku wydajności zapytań `SELECT` na krytycznej ścieżce uwierzytelniania, a w skrajnym wypadku do wyczerpania przestrzeni dyskowej (Disk Exhaustion) i awarii bazy.
* **Rekomendacja:** Uruchomić asynchroniczny job w tle (np. raz na dobę), który wykonuje `DELETE FROM refresh_tokens WHERE revoked = 1 OR expires_at < CURRENT_TIMESTAMP`.

### 🚨 1.4. Ograniczenie klucza symetrycznego JWT (Symmetric Key Security Constraint)
* **Status:** **ŚREDNI**
* **Opis:** Aplikacja wykorzystuje algorytm `HS256` (klucz symetryczny). Oznacza to, że ten sam sekret `JWT_SECRET` jest wymagany do podpisywania, jak i do weryfikacji tokenów.
* **Ryzyko:** Jeśli w przyszłości inne usługi (np. bramka API, mikrousługi raportujące) będą musiały niezależnie weryfikować poprawność JWT, będą musiały otrzymać dostęp do `JWT_SECRET`. Kompromitacja jakiejkolwiek z tych pobocznych usług daje napastnikowi możliwość fałszowania dowolnych tokenów (pełne przejęcie tożsamości admina).
* **Rekomendacja:** Przejść na algorytm asymetryczny `RS256` (podpisywanie kluczem prywatnym, dystrybucja klucza publicznego przez JWKS).

### 🚨 1.5. Blokowanie wątków Tokio przez I/O SQLite (SRE Tail Latency Bottleneck)
* **Status:** **SRE WARN**
* **Opis:** Chociaż `sqlx` jest asynchroniczny, pod spodem SQLite jest biblioteką synchroniczną C. Każda operacja zapisu i odczytu wykonuje synchroniczne wywołania systemowe I/O do dysku.
* **Ryzyko:** Przy dużym natężeniu ruchu (szczególnie przy jednoczesnych zapisach telemetrycznych i rotacjach tokenów), SQLite blokuje wątek systemu operacyjnego. Jeśli pula Tokio zostanie wysycana przez operacje dyskowe, opóźnienia żądań HTTP (tail latencies) drastycznie wzrosną, powodując timeouty połączeń u klientów.
* **Rekomendacja:** Rozważyć migrację z SQLite na PostgreSQL dla wdrożeń produkcyjnych wymagających wysokiej dostępności i odporności na obciążenia I/O.

---

## 2. Metryki i wskaźniki ryzyka (SRE Metrics Checklist)
Podczas monitorowania wdrożenia produkcyjnego w Grafanie należy bezwzględnie skonfigurować alerty na następujące anomalie:

1. `webx_auth_failures_total > 20/min` -> Podejrzenie ataku brute-force / credential stuffing.
2. `webx_refresh_token_revocations_total > 50/min` -> Podejrzenie próby użycia skradzionych refresh tokenów (replay attack).
3. `webx_db_latency_seconds_bucket{le="0.5"} < 0.95` -> Opóźnienia bazy danych przekraczają bezpieczne progi, ryzyko wysycenia wątków Tokio.
