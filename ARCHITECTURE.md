# WebX Metrics Pro Secured — Architecture

## Purpose
This document describes the architecture, trust boundaries, and deployment topology for `webx-metrics-pro-secured`.
It is intended for security reviewers, operators, and engineers maintaining the hardened Rust service.

## System components

### 1. Rust application
- `src/main.rs`: HTTP server bootstrap and middleware pipeline.
- `src/auth.rs`: JWT issuance and validation, password hashing, token extraction.
- `src/db.rs`: SQLite persistence for metrics, users, and refresh tokens.
- `src/handlers.rs`: HTTP endpoints for login, dashboard, refresh/logout, metrics APIs, and export.
- `src/metrics.rs`: Simulated ingestion, Prometheus metrics registration, and persistence scheduler.
- `src/export.rs`: Admin-only exports to PDF, Markdown, and plain text.
- `src/charts.rs`: Server-side SVG chart generation with zero inline JavaScript.
- `src/security.rs`: Security headers, CSRF defense, global and login rate limiting.

### 2. Static assets
- `templates/`: Askama templates for login and dashboard pages.
- `static/`: static frontend assets including login JS, CSS, and security disclosure files.

### 3. Deployment support
- `Dockerfile`: builds a release binary and packages it into a distroless runtime image.
- `Caddyfile`: hardened HTTPS reverse proxy configuration for production.
- `.github/workflows/security-ci.yml`: CI workflow for security audits, license checks, SBOM generation, and Cosign signing.
- `SECURITY.md`: security controls, threat model, and deployment guidance.

## Data flow

1. Client requests `GET /login`.
2. Server renders login page and sets a `csrf_token` cookie.
3. Client submits credentials to `/api/auth/login` with `X-CSRF-Token` and credentials payload.
4. Server validates CSRF token, login rate limit, and credentials against SQLite.
5. On success, server issues `access_token` and `refresh_token` cookies and keeps `csrf_token` for ongoing CSRF protection.
6. Authenticated users request `/` and dashboard endpoints using `access_token` from HttpOnly cookie.
7. Prometheus scraping uses `Authorization: Bearer <PROM_TOKEN>` to access `/metrics`.
8. Admins may use export endpoints; viewer role is blocked from export actions.

## Trust boundaries

### Trusted components
- Rust application binary: implements core business logic, auth, and persistence.
- SQLite database: local data store for metrics, users, and refresh token state.
- Caddy reverse proxy: optional production boundary enforcing HTTPS, request filtering, and header hardening.

### Untrusted inputs
- HTTP request headers, bodies, and cookies from external clients.
- Origin headers and browser-supplied tokens.
- Any external requests to `/metrics`.

## Security boundaries

### Authentication
- Access tokens: JWT bearer tokens valid for 15 minutes, issued to authenticated sessions.
- Refresh tokens: JWT bearer tokens valid for 7 days, stored in database with revocation support.
- Passwords: hashed with Argon2id and salted, no plaintext storage.

### Authorization
- `admin` role: full dashboard plus export endpoints.
- `viewer` role: dashboard and metrics history only.
- `/metrics`: protected by `PROM_TOKEN` bearer token, separate from user auth.

### Session and CSRF protection
- Double-submit cookie pattern for CSRF.
- `X-CSRF-Token` required for state-changing requests.
- `SameSite=Strict` cookies for access and refresh tokens.

### Network and transport
- `Strict-Transport-Security` with preload and long max-age.
- CSP locked to `self` and only trusted source patterns.
- `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and `Permissions-Policy` set.

## Deployment topology

### Recommended production deployment
- Public edge: Caddy HTTPS reverse proxy.
- Internal service: Rust binary listening on localhost or an internal port.
- Database: SQLite file mounted inside the service container or host filesystem.

### Suggested network layout

- External client -> HTTPS -> Caddy
- Caddy -> HTTP -> Rust application
- Rust application -> local SQLite database

### Environment configuration
- `JWT_SECRET`: strong secret, minimum 32 bytes.
- `DATABASE_URL`: SQLite connection string.
- `ADMIN_PASS`: initial admin password.
- `PROM_TOKEN`: required token for `/metrics`.
- `ALLOWED_ORIGINS`: optional CORS allow list.
- `APP_ENV`: set to `production` for Secure cookie enforcement.

## Hardening notes

- Do not expose the Rust app directly to the internet without the Caddy reverse proxy.
- Use `APP_ENV=production` in production so cookies get the `Secure` attribute.
- Keep `JWT_SECRET` and `PROM_TOKEN` out of source control.
- Rotate secrets regularly and enforce strong random values.
- Monitor login failures, refresh token revocations, and global rate limit violations.

## Operational guidance

- Build with `cargo build --release`.
- Verify security checks with the CI workflow before merging.
- Use `docker build -t webx-metrics-pro-secured .` for packaging.
- Use `cosign` or your chosen code-signing solution for verified release artifacts.

## Notes

This architecture is designed for a hardened, single-instance Rust application with clear security boundaries and minimal runtime attack surface. For multi-node or cloud deployments, add a secrets manager, external auth service, and a more scalable persistent datastore.
