# Security Design for WebX Metrics Pro

## Overview
WebX Metrics Pro is a security-first Rust web service for metrics visualization and export. It uses Axum, SQLite, JWT authentication, CSRF defense, strict CSP, and a protected Prometheus endpoint.

## Architecture
- `src/main.rs` configures the HTTP server, middleware chain, CORS policy, metrics, and routing.
- `src/auth.rs` handles JWT issuance, validation, password hashing, and token extraction.
- `src/security.rs` enforces headers, CSRF double-submit, and rate limiting.
- `src/db.rs` persists users, refresh tokens, and metrics in SQLite.
- `src/handlers.rs` implements page rendering, login, refresh, logout, metrics APIs, and RBAC.
- `src/metrics.rs` simulates ingestion and exposes Prometheus metrics.
- `src/export.rs` provides admin-only PDF/MD/TXT export.
- `templates/` and `static/` contain the rendering and static assets.

## Threat model
The application protects against:
- Credential theft and replay
- CSRF targeting authenticated sessions
- XSS via strict content security policy and no inline scripts/styles
- Unauthorized access to metrics, exports, and Prometheus data
- SQL injection via parameterized queries
- brute-force login attempts via rate limiting
- network or browser misconfiguration through strict transport headers

## Security controls

### Authentication and session management
- Passwords are hashed with Argon2id and salted.
- Access tokens are JWT bearer tokens valid for 15 minutes.
- Refresh tokens are JWT tokens valid for 7 days with a unique `jti` and revocation support.
- JWT validation explicitly enforces HS256, issuer, and audience.
- Access and refresh tokens are stored in `HttpOnly` cookies.
- Token and CSRF cookies use `SameSite=Strict`.
- `Secure` cookies should be enabled in production behind TLS.

### Authorization
- Two roles are supported: `admin` and `viewer`.
- `viewer` users can read dashboards and metrics history only.
- `admin` users can export data to PDF/Markdown/TXT.
- `/metrics` is protected by a static bearer token (`PROM_TOKEN`) and is not public.

### CSRF and CORS
- CSRF uses a double-submit cookie pattern plus `X-CSRF-Token` header.
- Login and refresh endpoints are exempt from CSRF validation because they are the authentication flow.
- CORS is strict: permitted origins are configured via `ALLOWED_ORIGINS` and not open by default.

### Transport and browser security headers
- `Strict-Transport-Security` enforces HTTPS for browsers.
- `Content-Security-Policy` restricts sources to `self` and disallows inline scripts/styles.
- `X-Content-Type-Options: nosniff` prevents MIME sniffing.
- `X-Frame-Options: DENY` disables clickjacking.
- `Referrer-Policy: no-referrer` prevents leakage of URL data.
- `Permissions-Policy` disables geolocation, microphone, and camera.
- `Cross-Origin-Opener-Policy: same-origin` isolates browsing context.

### Data handling and persistence
- SQLite is used with `sqlx` prepared queries only.
- Metrics retention is limited to the latest 10k records to avoid storage growth.
- Refresh token revocation is persisted in the database.
- Default admin credentials are created only when no users exist.

### Rate limiting
- Global rate limiting is applied per client IP: 100 requests per minute.
- Login rate limiting is recommended for future hardening at the route level.

### Build and deployment hardening
- Production profile uses `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, and `strip = true`.
- Docker is built with a distroless runtime image for minimal attack surface.
- Runtime secrets must come from environment variables, not the repository.

## Deployment recommendations
- Terminate TLS at the edge or load balancer.
- Set `JWT_SECRET` to a strong secret of at least 32 characters.
- Set `PROM_TOKEN` for `/metrics` protection.
- Use an external key/value store or secret manager in production if available.
- Monitor authentication and refresh token failures.

## Review guidance
When changing this repository, preserve the following invariants:
- do not weaken CSP or security headers without explicit justification
- do not expose `/metrics` without bearer authentication
- do not add unauthenticated export routes
- ensure new stateful APIs are protected by CSRF or same-site cookie rules
- ensure any new DB access uses prepared parameter binding
