# AGENTS

## Purpose
This repository contains a secured Rust metrics dashboard implementation named `webx-metrics-pro-secured`.

## What AI agents should know
- The project is a complete secured application with JWT authentication, Argon2id password hashing, CSRF protection, role-based access control, SQLite persistence, and protected Prometheus metrics.
- The repository includes packaging support, security documentation, and CI workflow definitions for dependency and supply chain validation.
- Do not assume the workspace is a placeholder; it is already implemented and should be treated as a production-grade Rust service.

## Recommended behavior
- Preserve the existing security invariants when modifying code:
  - do not weaken CSP or security headers
  - keep `/metrics` protected by `PROM_TOKEN`
  - keep export routes admin-only
  - use prepared SQL queries and avoid raw string interpolation
- Prefer Rust standard tooling for changes:
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Use the provided `.env.example` and `SECURITY.md` for secure deployment guidance.

## Important invariants
- `/metrics` must remain protected by the bearer token set in `PROM_TOKEN`.
- Do not add unauthenticated export or metrics endpoints.
- Preserve strict security headers and CSP policies.
- Keep secrets out of source control; use environment variables such as `JWT_SECRET`, `PROM_TOKEN`, and `ADMIN_PASS`.
- Use `APP_ENV=production` in production to enable secure cookie behavior.

## Build and validation
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --release`
- `docker build -t webx-metrics-pro-secured .`

## Key docs
- `ARCHITECTURE.md` — component boundaries and deployment topology.
- `SECURITY.md` — security design, threat model, and invariants.
- `DEPLOYMENT.md` — production deployment guidance.
- `OPERATIONS.md` — operational verification and runbook practices.

## Important files
- `src/main.rs` — server bootstrap and routing.
- `src/auth.rs` — JWT, password hashing, and auth extraction.
- `src/security.rs` — headers, CSRF, and rate limiting.
- `src/db.rs` — SQLite persistence and queries.
- `src/handlers.rs` — HTTP endpoint implementations.
- `src/metrics.rs` — Prometheus metrics and ingestion.
- `src/export.rs` — admin-only export handlers.
- `templates/`, `static/` — UI templates and static assets.
- `Dockerfile`, `Caddyfile` — packaging and hardened proxy deployment.

## Notes
- The repo now includes `.github/workflows/security-ci.yml` for dependency audit, license checks, SBOM generation, and Cosign signing.
- There is a `Dockerfile` for distroless runtime packaging.
- There is a `Caddyfile` and security guide for hardened reverse proxy deployment.
