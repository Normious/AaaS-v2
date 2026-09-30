# Deep Dive: Operations (`config`, Docker, testing)

## Configuration

`src/config.rs` reads env (via `.env`) with safe defaults; every security knob is an env var, so you retune without code changes:

| Group | Keys |
|---|---|
| Server | `PORT` (3000), `ENV`, `LOG_LEVEL` |
| DB | `DATABASE_URL` (`./data/aaas-v2.db`), `DB_POOL_SIZE` (20) |
| JWT | `JWT_PRIVATE_KEY_PATH`, `JWT_PUBLIC_KEY_PATH`, `JWT_ISSUER`, `JWT_AUDIENCE`, `JWT_ACCESS_TOKEN_TTL_SECONDS` (900), `JWT_REFRESH_TOKEN_TTL_SECONDS` (2592000) |
| Argon2id | `ARGON2_MEMORY_KIB` (19456), `ARGON2_ITERATIONS` (2), `ARGON2_PARALLELISM` (1) |
| Sessions | `SESSION_MAX_DEVICES` (10), `SESSION_IDLE_TIMEOUT_DAYS` (90) |
| Lockout | `MAX_FAILED_LOGINS` (5), `LOCKOUT_DURATION_MINUTES` (15) |
| Rate limits | `RL_LOGIN_PER_IP_PER_MIN` (20), `RL_LOGIN_PER_EMAIL_PER_MIN` (5), `RL_REGISTER_PER_IP_PER_HOUR` (10), `RL_PASSWORD_RESET_PER_EMAIL_PER_HOUR` (3) |
| SMTP | `SMTP_ENABLED` (false), `SMTP_HOST/PORT/USERNAME/PASSWORD`, `SMTP_FROM_*` |
| Links/CORS | `APP_BASE_URL`, `EMAIL_VERIFY_PATH`, `PASSWORD_RESET_PATH`, `CORS_ALLOWED_ORIGINS` (`*`) |

`.env.example` documents all of the above. Never commit `.env` or `secrets/*.pem` (both gitignored).

## Running

```bash
cargo run                      # dev (SQLite file auto-created + migrated)
cargo build --release          # 6 MB binary, LTO + strip + panic=abort
docker compose up --build -d   # 31 MB distroless image, :3000
```

Boot sequence: tracing → config → `create_dir_all` for DB parent → sqlite pool → `PRAGMA journal_mode=WAL` + `foreign_keys=ON` → apply `migrations/0001_init.sql` (split on `;`, idempotent) → load Ed25519 keys → serve. Missing/unreadable keys are a **startup abort**, by design.

## Docker notes (verified 2026-09-30)

- Builder must be ≥ Rust 1.85: the lockfile contains `edition2024`
crates that Cargo 1.80 cannot parse (`Dockerfile` pins `rust:1.98-slim`, matching the local toolchain).
- No dummy-`main` cache hack: it once shipped a 301 KB no-op binary that
exited 0 silently. Single honest `cargo build --release` + `.dockerignore`.
- Distroless, not `scratch`: SQLite needs libc.
- Windows Git Bash: use `docker compose`: MSYS mangles `docker run -v`
host paths (results in an empty `/secrets` + `Read JWT private key` abort). With compose, `./secrets:/secrets:ro` resolves correctly.

## Testing

```bash
cargo test        # 4/4: argon2 roundtrip, TOTP roundtrip, token shape, limiter
```

Live E2E (also re-run inside the container, port 3000): register → 409 dup → 403 unverified → verify → login → verify/me → profile update → refresh rotation → replay-old-refresh 401 (session dead) → sessions list/revoke → password change (old 401, new works) → reset request (generic for unknown emails too) → reset confirm → 2FA enroll/verify/challenge/backup-use-once/disable → audit-log → email change+swap → logout → lockout 5×401 then 423 → register-limit 429 with `retry_after_seconds` → 400 validation shapes.

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `Read JWT private key` at boot | Wrong `JWT_*_PATH` or (Docker/Windows) mangled bind mount: use compose |
| `Email not verified` (403) | Expected until `/auth/email/verify`; with SMTP off, stage tokens via DB |
| 423 on correct password | Lockout active: wait `LOCKOUT_DURATION_MINUTES` or clear `locked_until` |
| 429 with `retry_after_seconds` | Limiter working: back off; raise `RL_*` for load tests |
| Container exits 0 silently | Stale dummy binary: rebuild image from current `Dockerfile` |
| Host-side SQLite writes invisible to server | Docker Desktop share staleness: write while the server is stopped |
