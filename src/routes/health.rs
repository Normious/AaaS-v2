use axum::Json;

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "AaaS v2 — Hardened Identity Provider",
        "version": "2.0.0",
        "language": "Rust",
        "status": "ok",
        "features": [
            "argon2id",
            "ed25519-jwt",
            "refresh-rotation",
            "totp-2fa",
            "email-verification",
            "password-reset",
            "account-lockout",
            "rate-limiting",
            "audit-log",
            "multi-device-sessions",
        ],
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
