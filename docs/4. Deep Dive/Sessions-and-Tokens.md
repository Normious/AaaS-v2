# Deep Dive: Sessions, Tokens & Audit (`src/db/` + `src/routes/`)

## Sessions (`db/sessions.rs`)

- `create_session` enforces `SESSION_MAX_DEVICES` by revoking the oldest active session first (never fails the login).
- `device_name` derives from User-Agent (`"Chrome on macOS"`); handlers store raw UA + IP for the device list.
- `revoke_session` / `revoke_all_user_sessions` stamp reason (`logout`, `password_changed`, `password_reset`, `theft_detected`, `user_revoked`, `max_devices_exceeded`).
- Listing filters `is_revoked=0 AND expires_at > now`, newest-first.

## Refresh rotation (`db/tokens.rs`)

`rotate_refresh_token` is the security heart: order matters below.

1. Find by hash → none: 401.
2. `used_at` set → **theft**: revoke session, warn-log, 401.
3. Expired token or dead session → 401.
4. Insert successor, mark predecessor `used + replaced_by`, bump
`sessions.last_used_at` → return `(new_raw, session_id, user_id)`.

Email (24 h) and password-reset (1 h) tokens share the file: issue invalidates prior unused tokens for the user; consume is single-use (`used_at` set in the same call that returns the user id).

## Users (`db/users.rs`)

- Handlers lowercase and trim emails at the boundary; UNIQUE violations map to 409 (register); reset/resend paths stay generic.
- `record_failed_login` increments then evaluates lockout; the success path resets everything including `locked_until`.
- `enable/disable_totp` also manage the backup-code rows.

## Audit (`db/audit.rs` + `GET /me/audit-log`)

- `log_event` is best-effort (`.await.ok()` at most call sites): audit must never break auth. `user_id` is nullable so failed/anonymous events (`failed_login`, `password_reset_requested`) are still recorded.
- Covered event types: register, verify, login (+`_2fa`, `+_backup`), lock, refresh, logout(+`_all`), password change/reset, email change, 2FA on/off, session revoke, profile update.
- `list_for_user` caps at 100, newest first.

## Route notes worth knowing

- `login.rs`: rate limits run **before** the DB lookup; `dummy_verify` runs on unknown users; unverified emails stop before any session exists.
- `password.rs`: change requires current password and revokes **all** sessions including the caller's (client must re-login).
- `email.rs`: the new address rides on the verification token and is applied on confirm, with a UNIQUE guard at swap time.
- `logout.rs`: `{"all_devices": true}` returns the revoked-session count.
