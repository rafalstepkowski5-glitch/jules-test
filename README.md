# WebX Metrics Pro - Secured Edition (Audit & Review Version)

WebX Metrics Pro is a high-performance, security-focused telemetry visualization dashboard built with Rust (Axum, SQLx SQLite/SQLCipher) and designed to collect, process, and display browser metrics.

This repository represents the secured "Twierdza v2.0" deployment version, implementing several industry-grade security practices, but containing multiple latent architectural and production bottlenecks identified during our deep SRE and Security Audit.

---

## ⚠️ Known Issues & Audit Findings (Brutally Critical Review)

Below is the complete list of technical deficiencies, security vulnerabilities, and architectural bottlenecks identified during our review.

### 🔴 P0: Critical - Fix IMMEDIATELY (Security / Crash / Deployment Blockers)

#### 1. Hardcoded Credentials Leak
* **File:** `src/db.rs:17`
* **Code:** `let pass_viewer = "viewer123";`
* **Problem:** The password for the viewer user is hardcoded in the codebase. Any user with access to the source repository or binary can authenticate as a viewer.
* **Risk:** Unauthorized access to dashboard data and metrics history.
* **Fix:** Load the viewer password from an environment variable (e.g., `VIEWER_PASS`) during database initialization.

#### 2. Default Administration Password Vulnerability
* **File:** `src/db.rs:11`
* **Code:** `let pass = std::env::var("ADMIN_PASS").unwrap_or_else(|_| "admin123".into());`
* **Problem:** Fallback to a highly weak default password (`admin123`) if the `ADMIN_PASS` environment variable is not explicitly defined.
* **Risk:** Insecure defaults lead to immediate system compromise on deployment if forgotten.
* **Fix:** Enforce "fail-closed" behavior. If `ADMIN_PASS` is not set or too short, panic on startup with an error message.

#### 3. Absolute Absence of Test Suite (0 Tests)
* **File:** Entire codebase (`cargo test` output)
* **Code:** `running 0 tests; test result: ok. 0 passed; 0 failed`
* **Problem:** Zero automated unit or integration tests exist in the codebase.
* **Risk:** Regression hazard, high risk of code corruption during future refactors, unable to verify security behaviors in CI automatically.
* **Fix:** Implement robust unit tests in `src/auth.rs`, `src/security.rs`, and integration tests in `tests/` using `axum::serve` or mock clients.

#### 4. Broken DevOps Docker Build
* **File:** Repository Root (`Dockerfile.txt`)
* **Code:** The file is named `Dockerfile.txt` rather than `Dockerfile`.
* **Problem:** `docker-compose.yml` expects a file named `Dockerfile` for build. Consequently, `docker compose up --build` fails immediately with `Dockerfile not found`.
* **Risk:** Broken continuous integration, inability to deploy on containerized environments.
* **Fix:** Rename `Dockerfile.txt` to `Dockerfile`.

#### 5. Broken Global Rate Limiter (IP Spoofing & Single IP Collapser)
* **File:** `src/security.rs:46-51`
* **Code:**
  ```rust
  let ip = req.extensions().get::<axum::extract::ConnectInfo<std::net::SocketAddr>>().map(|c| c.0.ip().to_string()).unwrap_or_else(|| "unknown".into());
  ```
* **Problem:** Behind reverse proxies like Caddy, `ConnectInfo` will resolve to Caddy’s internal container/loopback IP (e.g., `127.0.0.1` or `172.18.0.X`).
* **Risk:** If a single external user triggers the 100 requests/min rate limit, Caddy's IP is blocked globally, resulting in a total Denial of Service (DoS) for all legitimate users worldwide.
* **Fix:** Read the real user IP from the `X-Forwarded-For` or `X-Real-IP` headers securely.

---

### 🟡 P1: Logic Bugs & Security Deficiencies

#### 1. Non-existent Token Revocation Checking
* **File:** `src/handlers.rs:77-88` (`refresh_api`)
* **Code:**
  ```rust
  let token = extract_cookie(&headers, "refresh_token").ok_or(StatusCode::UNAUTHORIZED)?;
  let claims = auth::verify_jwt(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
  ```
* **Problem:** The system tracks refresh tokens in the `refresh_tokens` table on login, but the `/api/auth/refresh` handler **never** queries this table to verify if the token is revoked, blacklisted, or expired in the database!
* **Risk:** Users who log out can still use their refresh token indefinitely. Blacklisted or compromised tokens cannot be revoked.
* **Fix:** Add a database query to `/api/auth/refresh` to check if the token's `jti` is present and has `revoked = 0`.

#### 2. False Creation Timestamp for Expiration in Database
* **File:** `src/handlers.rs:59`
* **Code:** `.bind(chrono::Utc::now().to_rfc3339())` (bound to `expires_at`)
* **Problem:** When inserting the refresh token metadata, `chrono::Utc::now().to_rfc3339()` is used for `expires_at`. This sets the expiration to the *creation time*, meaning the token is technically "expired" the second it is written.
* **Risk:** While currently harmless because the validation logic ignores the DB, it will cause issues once DB validation is implemented.
* **Fix:** Set expiration to `(chrono::Utc::now() + chrono::Duration::days(7)).to_rfc3339()`.

#### 3. Broken In-Database Revocation on Logout
* **File:** `src/handlers.rs:91` (`logout_api`)
* **Code:** Clears browser cookies but does not receive `State` and does not write to the DB.
* **Problem:** Logout simply clears cookies on the client side without invalidating the active token in the `refresh_tokens` table.
* **Risk:** If the refresh token is intercepted, it remains valid because it was never revoked.
* **Fix:** Pass `State(state)` into `logout_api`, extract the token `jti`, and set `revoked = 1` in the database.

#### 4. Missing Cookie Security Flags (Secure)
* **File:** `src/handlers.rs:27, 65, 69, 85`
* **Code:** `...SameSite=Strict; HttpOnly`
* **Problem:** None of the authentication/CSRF cookies set the `Secure` attribute.
* **Risk:** In non-HTTPS configurations or mixed environments, cookies can be leaked over unencrypted transport.
* **Fix:** Conditionally append `; Secure` if `APP_ENV=production`.

---

### 🔵 P2: Technical Debt & Scalability Bottlenecks

#### 1. Severe DB Bottleneck: In-Thread Table Sweeps
* **File:** `src/db.rs:22-25` (`save_metric`)
* **Code:**
  ```rust
  sqlx::query("DELETE FROM metrics WHERE id NOT IN (SELECT id FROM metrics ORDER BY id DESC LIMIT 10000)").execute(pool).await?;
  ```
* **Problem:** On **every single metric write** (occurring every 5 seconds), a heavy subquery sweep is performed to trim the table.
* **Risk:** This blocks SQLite writes under load and completely locks up the database, causing a Denial of Service.
* **Fix:** Offload table trimming to a daily/hourly background cron job or run it periodically on a randomized basis rather than on every write.

#### 2. Slow Environment Reads on Critical Path
* **File:** `src/handlers.rs:136` (`protected_metrics`)
* **Code:** `let expected = std::env::var("PROM_TOKEN").map_err(...)`
* **Problem:** Reads `PROM_TOKEN` from the process environment variables on **every request** to `/metrics`.
* **Risk:** Reading environment variables takes a global system lock in Rust, causing massive request contention and latency under heavy scraping.
* **Fix:** Load the token into `AppState` once at startup and extract it from state in the handler.

#### 3. Missing Axum Graceful Shutdown
* **File:** `src/main.rs:71`
* **Code:** `axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;`
* **Problem:** Standard server startup does not specify a shutdown signal.
* **Risk:** Abrupt termination of the container causes database corruption, aborted active requests, and poor operational metrics.
* **Fix:** Implement a shutdown helper using `.with_graceful_shutdown()`.

---

### ⚪ P3: Code Quality, DX & Vulnerabilities

#### 1. Crash Risk via Unchecked Template Rendering
* **File:** `src/handlers.rs:33` & `src/handlers.rs:49`
* **Code:** `Html(template.render().unwrap())`
* **Problem:** If a template fails to parse or render, the server threads will panic and terminate.
* **Risk:** Potential micro-services outage under edge-case template processing.
* **Fix:** Match or map errors to returning `StatusCode::INTERNAL_SERVER_ERROR`.

#### 2. Outdated Dependencies and CVEs
* Run `cargo audit` to see:
  - **`protobuf v2.28.0`**: Under RUSTSEC-2024-0437 (Crash due to uncontrolled recursion).
  - **`rsa v0.9.10`**: Under RUSTSEC-2023-0071 (Marvin Attack timing sidechannels).

---

## 🛠️ Run & Build Instructions

### Prerequisites
- Rust (MSRV 1.75+)
- Docker & Docker Compose (Optional)

### Run Locally
1. Copy and configure env file:
   ```bash
   cp .env.example .env
   ```
2. Build and run:
   ```bash
   cargo run
   ```

### Run with Docker Compose
1. Ensure the DevOps bug is fixed by copying/renaming `Dockerfile.txt`:
   ```bash
   cp Dockerfile.txt Dockerfile
   ```
2. Start the service stack:
   ```bash
   docker compose up --build
   ```
