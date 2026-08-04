# WebX Metrics Pro Secured — Operations

## Purpose
This document provides operational playbooks for running, monitoring, and maintaining the secured WebX Metrics Pro service.

## Startup

1. Copy `.env.example` to `.env`.
2. Verify required environment variables are present:
   - `JWT_SECRET`
   - `PROM_TOKEN`
   - `ADMIN_PASS`
   - `DATABASE_URL`
   - `ALLOWED_ORIGINS`
   - `APP_ENV=production`
3. Start the application:
   ```bash
   cargo run --release
   ```
4. Verify the service is listening:
   ```bash
   curl -I http://127.0.0.1:3000/login
   ```

## Health checks

- `GET /login` should return the login page.
- `GET /` requires authentication and should redirect or return 401 if unauthorized.
- `GET /metrics` requires `Authorization: Bearer <PROM_TOKEN>`.
- `POST /api/auth/login` should verify credentials and return session cookies.

## Logs

- Application logs are JSON-formatted via `tracing-subscriber`.
- Capture logs from stdout/stderr when running in containers.
- Monitor for repeated authentication failures, CSRF rejections, rate limit hits, and token refresh errors.

## Secrets management

- Keep `JWT_SECRET` and `PROM_TOKEN` in a secrets manager or environment-only configuration.
- Do not commit secrets to git.
- Rotate secrets periodically and deploy the new values carefully.

## Backup and persistence

- SQLite is a local file-based data store. Back up `data.db` regularly.
- Retain metrics exports and refresh token state according to your retention policy.
- If using a container, mount the SQLite data directory on persistent storage.

## Incident response

### Suspicious authentication activity
- Check application logs for repeated failed login attempts.
- If brute-force is suspected, temporarily reduce the login rate limit or block offending IPs.
- Rotate `JWT_SECRET` and `PROM_TOKEN` if credential leakage is suspected.

### Token misuse
- Revoke refresh tokens by invalidating stored `jti` entries in the database.
- Force logout by clearing client cookies and rotating tokens.

## Maintenance

- Run `cargo fmt -- --check` and `cargo clippy --all-targets -- -D warnings` before deploying.
- Keep dependencies updated and verify with the established CI workflow.
- Periodically audit the `Caddyfile`, `SECURITY.md`, and `ARCHITECTURE.md` to ensure operational practices remain aligned with security requirements.

## Recovery

- If the database is corrupted, restore from the most recent backup.
- If the service fails to start, inspect logs for initialization errors and missing environment variables.
- Verify `DATABASE_URL` points to a writable SQLite file.

## Rolling upgrades

1. Deploy the new binary or container image.
2. Verify the service starts successfully.
3. Run smoke tests against `/login`, `/metrics`, and authenticated endpoints.
4. Monitor authentication and refresh token behavior for regressions.

## Add-ons

- Use an external process supervisor or container orchestrator to restart the service automatically on failures.
- Integrate with centralized logging and metrics ingestion for production monitoring.
