# Progress for WebX Metrics Pro v2.0

## Completed implementation

- Added a new Rust project `webx-metrics-pro` with `Cargo.toml` and `src/` modules.
- Implemented JWT-based authentication with access tokens (15 min) and refresh tokens (7 days).
- Added Argon2id password hashing for stored users.
- Added SQLite persistence via `sqlx` and automatic database initialization.
- Added metrics ingestion simulator saving one sample every 5 seconds.
- Added Prometheus metrics with a protected `/metrics` endpoint.
- Added role-based access control: `admin` and `viewer` roles.
- Added zero-JavaScript inline SVG charts with `charts::svg_line_chart`.
- Added strong security headers, CSP, rate limiting, CSRF double submit, body size limit.
- Added export endpoints for PDF, Markdown, and plain text, restricted to `admin` role.

## New files

- `Cargo.toml`
- `src/main.rs`
- `src/auth.rs`
- `src/db.rs`
- `src/handlers.rs`
- `src/metrics.rs`
- `src/export.rs`
- `src/charts.rs`
- `src/security.rs`
- `static/login.js`
- `AGENTS.md`
- `progress.md`

## Notes

- The request to keep zero inline JS is honored by serving `static/login.js`.
- `/metrics` now requires a bearer token matching `PROM_TOKEN`.
- Export endpoints are blocked for `admin` only and are visible only on the dashboard for admin users.
- The database bootstrap creates default `admin` and `viewer` accounts if none exist.
- The generated `Dockerfile` builds a distroless runtime image and includes templates and static assets.

## Environment variables

- `JWT_SECRET`: must be at least 32 characters, or the app falls back to a hardcoded development secret.
- `DATABASE_URL`: optional SQLite connection string, defaults to `sqlite://data.db?mode=rwc`.
- `ADMIN_PASS`: optional admin password for first-time startup, defaults to `admin123`.
- `PROM_TOKEN`: bearer token required to access the `/metrics` endpoint.
- `ALLOWED_ORIGINS`: optional comma-separated CORS allow list for browser requests.
