# WebX Metrics Pro Secured — Deployment

## Purpose
This document describes recommended deployment practices for the secured `webx-metrics-pro` application.

## Deployment model

The recommended production architecture is:

- Public HTTPS edge: Caddy reverse proxy
- Internal application: Rust binary running on localhost or private internal port
- Local SQLite persistence: mounted on durable storage

## Recommended topology

```text
Internet --> Caddy HTTPS proxy --> Rust app (127.0.0.1:3000)
                         |
                         --> static assets / health checks
```

## Environment variables

- `JWT_SECRET`: secure secret, minimum 32 bytes.
- `PROM_TOKEN`: bearer token for `/metrics`.
- `ADMIN_PASS`: initial admin password.
- `DATABASE_URL`: SQLite connection string.
- `ALLOWED_ORIGINS`: comma-separated allowed origins for CORS.
- `APP_ENV=production`: enables `Secure` cookie flag.

## Container build

Build with Docker:

```bash
docker build -t webx-metrics-pro-secured .
```

For distroless deployment, the `Dockerfile` already packages the release binary and runtime assets.

## Reverse proxy configuration

Use the provided `Caddyfile` as the recommended proxy configuration.

### Key proxy behaviors
- TLS termination at the proxy layer
- Header hardening with HSTS, X-Frame-Options, Referrer-Policy, and Permissions-Policy
- Optional user-agent blocking for common scanners and fuzzers

## Security hardening

- Do not expose the Rust process directly to the public internet.
- Use the `Caddyfile` as the front door and keep the backend bound to `127.0.0.1`.
- Ensure `APP_ENV=production` in the runtime environment.
- Mount the SQLite file on persistent storage, not ephemeral container storage.

## CI/CD and build verification

- Use `.github/workflows/security-ci.yml` for automated dependency and supply chain validation.
- Build and sign artifacts as part of the release pipeline.
- Maintain a signed SBOM for each production release.

## Rollout procedure

1. Build the container image.
2. Deploy to a staging environment.
3. Run smoke tests for login, dashboard, and `/metrics` authorization.
4. Promote the image to production once all checks pass.

## Monitoring

- Monitor application logs for authentication failures, CSRF failures, rate-limit events, and token refresh errors.
- Monitor the proxy for TLS and HTTP anomalies.
- Verify `/metrics` access is only successful with the correct `PROM_TOKEN`.

## Secrets

- Manage `JWT_SECRET`, `PROM_TOKEN`, and `ADMIN_PASS` with a secret manager.
- Do not store secrets in git or in build artifacts.

## Notes

- This deployment model is optimized for a single-instance service with a hardened reverse proxy.
- For multi-instance deployments, add a centralized session store and replace SQLite with a managed database.
