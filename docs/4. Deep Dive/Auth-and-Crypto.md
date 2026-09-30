# Deep Dive: Auth & Crypto (`src/auth/`)

Five pure modules: no HTTP, no SQL. All take plain inputs and return `AppResult`.

## `password.rs`: Argon2id

- `hash_password` builds params from env (`ARGON2_MEMORY_KIB=19456`,
`ITERATIONS=2`, `PARALLELISM=1`: OWASP 2024) and returns the PHC-encoded string (salt + params embedded, single `users.password_hash` column).
- `verify_password` parses + verifies; returns `bool`, never the hash.
- `dummy_verify` hashes against a fixed string when the user doesn't exist: login timing stays roughly constant (user-enumeration mitigation).
- Unit test: `hash_and_verify` (reduced memory for speed).

## `jwt.rs`: Ed25519 access tokens

- `JwtIssuer::from_config` reads both PEMs at boot and validates the
Ed25519 private key; any key problem is a startup failure, not a runtime surprise.
- Claims: `sub` (user), `sid` (session), `iss/aud/exp/iat`, `jti`
(per-token id), `scope` (`["user"]` or `["2fa"]`), header `kid: aaas-v2-ed25519`.
- `verify_access_token` enforces issuer + audience + 5 s leeway.
- Why asymmetric: siblings verify with the public key only
(`docs` in spec §10): compromise of a consumer leaks nothing that can *mint* tokens.

## `tokens.rs`: opaque token utilities

- `generate_token`: 32 CSPRNG bytes, base64url-no-pad (43 chars).
- `hash_token`: SHA-256 hex. Raw values travel only over TLS (email links, JSON bodies).
- `generate_numeric_code`: 10-digit TOTP backup codes.

## `totp.rs`: 2FA (RFC 6238)

- Store base32 secrets for time-based one-time passwords (TOTP); `otpauth_url` renders the QR payload (`issuer=AaaS`).
- Params: SHA1 / 6 digits / 30 s / ±1 step skew.
- Gotcha (fixed during build): `Secret::generate_secret()` returns
`Secret::Raw` whose `to_string()` is **hex, not base32**: call `.to_encoded().to_string()` before persisting, else every code fails.
- Enrollment is two-phase: secret saved on `enroll`, `totp_enabled=1` only
after `verify` with a live code. Fresh backup codes per enroll; disabling wipes secret + codes.

## `lockout.rs`: brute-force brake

- `is_locked`: `locked_until` in the future → 423.
- `should_lock_now` runs **after** the counter increment, so it compares
`failed >= MAX_FAILED_LOGINS` (an earlier `+1` version locked one attempt early; fixed, covered by live test: 5×401 then 423).
- Success resets counter + clears lock (`record_successful_login`).
