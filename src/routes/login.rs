use axum::{extract::State, http::HeaderMap, Json};
use serde::Deserialize;
use validator::Validate;

use crate::auth::{lockout, password, totp};
use crate::db::{audit, sessions, tokens as token_db, users};
use crate::error::{AppError, AppResult};
use crate::routes::register::client_ip;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Login2FARequest {
    pub challenge_token: String,
    pub code: String,
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let ip = client_ip(&headers).unwrap_or_else(|| "unknown".into());
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok());

    state
        .limiter
        .check(&format!("login:ip:{ip}"), state.config.rl_login_per_ip_per_min, 60)
        .map_err(|r| AppError::RateLimited {
            retry_after_seconds: r,
        })?;
    state
        .limiter
        .check(
            &format!("login:email:{}", req.email.to_lowercase()),
            state.config.rl_login_per_email_per_min,
            60,
        )
        .map_err(|r| AppError::RateLimited {
            retry_after_seconds: r,
        })?;

    let user = match users::get_user_by_email(&state.db, &req.email).await? {
        Some(u) => u,
        None => {
            password::dummy_verify(&req.password);
            audit::log_event(
                &state.db,
                None,
                "failed_login",
                Some(&ip),
                ua,
                None,
                false,
                Some("user_not_found"),
            )
            .await
            .ok();
            return Err(AppError::InvalidCredentials);
        }
    };

    if user.is_locked == 1 && lockout::is_locked(user.locked_until) {
        audit::log_event(
            &state.db,
            Some(&user.id),
            "login_locked",
            Some(&ip),
            ua,
            None,
            false,
            None,
        )
        .await
        .ok();
        return Err(AppError::AccountLocked(user.locked_until.unwrap_or(0)));
    }

    if !password::verify_password(&req.password, &user.password_hash)? {
        users::record_failed_login(&state.db, &user.id, &state.config).await?;
        audit::log_event(
            &state.db,
            Some(&user.id),
            "failed_login",
            Some(&ip),
            ua,
            None,
            false,
            Some("wrong_password"),
        )
        .await
        .ok();
        return Err(AppError::InvalidCredentials);
    }

    if user.is_active == 0 {
        return Err(AppError::Forbidden("Account disabled".into()));
    }

    if user.email_verified == 0 {
        return Err(AppError::EmailNotVerified);
    }

    if user.totp_enabled == 1 {
        let challenge = state.jwt.issue_access_token(&user.id, "2fa-pending", vec!["2fa".into()])?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "requires_2fa": true,
            "challenge_token": challenge,
        })));
    }

    let session = sessions::create_session(&state.db, &user.id, ua, Some(&ip), &state.config).await?;
    let access = state.jwt.issue_access_token(&user.id, &session.id, vec!["user".into()])?;
    let refresh = token_db::issue_refresh_token(&state.db, &session.id, &user.id, &state.config).await?;

    users::record_successful_login(&state.db, &user.id, &ip).await?;
    audit::log_event(&state.db, Some(&user.id), "login", Some(&ip), ua, None, true, None).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "access_token": access,
        "refresh_token": refresh,
        "token_type": "Bearer",
        "expires_in": state.config.jwt_access_ttl,
        "session_id": session.id,
    })))
}

pub async fn login_2fa(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<Login2FARequest>,
) -> AppResult<Json<serde_json::Value>> {
    let ip = client_ip(&headers).unwrap_or_else(|| "unknown".into());
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok());

    let claims = state.jwt.verify_access_token(&req.challenge_token)?;
    if !claims.scope.contains(&"2fa".to_string()) {
        return Err(AppError::Unauthorized);
    }

    let user = users::get_user_by_id(&state.db, &claims.sub).await?;
    let secret = user.totp_secret.ok_or(AppError::Unauthorized)?;

    // Try TOTP first, then single-use backup codes.
    let totp_ok = totp::verify_code(&secret, &user.email, &req.code)?;
    let mut backup_used = false;
    if !totp_ok {
        backup_used = consume_backup_code(&state.db, &user.id, &req.code).await?;
        if !backup_used {
            audit::log_event(
                &state.db,
                Some(&user.id),
                "failed_2fa",
                Some(&ip),
                ua,
                None,
                false,
                None,
            )
            .await
            .ok();
            return Err(AppError::Unauthorized);
        }
    }

    let session = sessions::create_session(&state.db, &user.id, ua, Some(&ip), &state.config).await?;
    let access = state.jwt.issue_access_token(&user.id, &session.id, vec!["user".into()])?;
    let refresh = token_db::issue_refresh_token(&state.db, &session.id, &user.id, &state.config).await?;

    users::record_successful_login(&state.db, &user.id, &ip).await?;
    audit::log_event(
        &state.db,
        Some(&user.id),
        if backup_used { "login_2fa_backup" } else { "login_2fa" },
        Some(&ip),
        ua,
        None,
        true,
        None,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "access_token": access,
        "refresh_token": refresh,
        "token_type": "Bearer",
        "expires_in": state.config.jwt_access_ttl,
        "session_id": session.id,
    })))
}

async fn consume_backup_code(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    code: &str,
) -> Result<bool, AppError> {
    let hash = crate::auth::tokens::hash_token(code);
    let now = chrono::Utc::now().timestamp_millis();
    let res = sqlx::query(
        "UPDATE totp_backup_codes SET used_at = ? WHERE user_id = ? AND code_hash = ? AND used_at IS NULL",
    )
    .bind(now)
    .bind(user_id)
    .bind(&hash)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}
