use axum::{extract::State, Extension, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::totp;
use crate::db::{audit, users};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthContext;
use crate::state::AppState;

pub async fn enroll(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
) -> AppResult<Json<serde_json::Value>> {
    let user = users::get_user_by_id(&state.db, &ctx.user_id).await?;

    if user.totp_enabled == 1 {
        return Err(AppError::Conflict("2FA already enabled".into()));
    }

    let secret = totp::generate_secret();
    let otpauth_url = totp::otpauth_url(&secret, &user.email)?;

    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("UPDATE users SET totp_secret = ?, updated_at = ? WHERE id = ?")
        .bind(&secret)
        .bind(now)
        .bind(&user.id)
        .execute(&state.db)
        .await?;

    // Fresh backup codes on every enroll.
    sqlx::query("DELETE FROM totp_backup_codes WHERE user_id = ?")
        .bind(&user.id)
        .execute(&state.db)
        .await?;

    let mut codes = Vec::new();
    for _ in 0..10 {
        let code = crate::auth::tokens::generate_numeric_code(10);
        let code_hash = crate::auth::tokens::hash_token(&code);
        let code_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO totp_backup_codes (id, user_id, code_hash, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&code_id)
        .bind(&user.id)
        .bind(&code_hash)
        .bind(now)
        .execute(&state.db)
        .await?;
        codes.push(code);
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "secret": secret,
        "otpauth_url": otpauth_url,
        "backup_codes": codes,
        "message": "Scan the QR code, then confirm with a valid TOTP code via /me/2fa/verify.",
    })))
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub code: String,
}

pub async fn verify(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
    Json(req): Json<VerifyRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user = users::get_user_by_id(&state.db, &ctx.user_id).await?;
    let secret = user
        .totp_secret
        .ok_or_else(|| AppError::Validation("No TOTP enrollment in progress".into()))?;

    if !totp::verify_code(&secret, &user.email, &req.code)? {
        return Err(AppError::Validation("Invalid TOTP code".into()));
    }

    users::enable_totp(&state.db, &user.id, &secret).await?;
    audit::log_event(
        &state.db,
        Some(&user.id),
        "2fa_enabled",
        None,
        None,
        None,
        true,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "2FA enabled.",
    })))
}

#[derive(Debug, Deserialize)]
pub struct DisableRequest {
    pub password: String,
}

pub async fn disable(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
    Json(req): Json<DisableRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user = users::get_user_by_id(&state.db, &ctx.user_id).await?;

    if !crate::auth::password::verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    users::disable_totp(&state.db, &user.id).await?;
    audit::log_event(
        &state.db,
        Some(&user.id),
        "2fa_disabled",
        None,
        None,
        None,
        true,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({ "success": true })))
}
