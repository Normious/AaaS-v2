use axum::{extract::State, http::HeaderMap, Extension, Json};
use serde::Deserialize;
use validator::Validate;

use crate::auth::{password, tokens as token_util};
use crate::db::{audit, sessions, tokens as token_db, users};
use crate::email;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthContext;
use crate::routes::register::client_ip;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = users::get_user_by_id(&state.db, &ctx.user_id).await?;

    if !password::verify_password(&req.current_password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    users::update_password(&state.db, &ctx.user_id, &req.new_password, &state.config).await?;
    sessions::revoke_all_user_sessions(&state.db, &ctx.user_id, "password_changed").await?;

    audit::log_event(
        &state.db,
        Some(&ctx.user_id),
        "password_changed",
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
        "message": "Password changed. All other sessions have been logged out.",
    })))
}

#[derive(Debug, Deserialize, Validate)]
pub struct RequestResetRequest {
    #[validate(email)]
    pub email: String,
}

pub async fn request_reset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RequestResetRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let ip = client_ip(&headers).unwrap_or_else(|| "unknown".into());

    state
        .limiter
        .check(
            &format!("pwreset:email:{}", req.email.to_lowercase()),
            state.config.rl_password_reset_per_email_per_hour,
            3600,
        )
        .map_err(|r| AppError::RateLimited {
            retry_after_seconds: r,
        })?;

    if let Some(user) = users::get_user_by_email(&state.db, &req.email).await? {
        let raw_token = token_util::generate_token();
        token_db::issue_password_reset_token(&state.db, &user.id, &raw_token).await?;

        if state.config.smtp_enabled {
            let reset_url = format!(
                "{}{}?token={}",
                state.config.app_base_url.trim_end_matches('/'),
                state.config.password_reset_path,
                raw_token
            );
            let sender = email::EmailSender::from_config(&state.config);
            let to = user.email.clone();
            let name = user.first_name.clone();
            tokio::spawn(async move {
                let _ = sender.send_password_reset(&to, name.as_deref(), &reset_url).await;
            });
        }
    }

    audit::log_event(
        &state.db,
        None,
        "password_reset_requested",
        Some(&ip),
        None,
        None,
        true,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "If that email is registered, you'll receive a reset link shortly.",
    })))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmResetRequest {
    pub token: String,
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

pub async fn confirm_reset(
    State(state): State<AppState>,
    Json(req): Json<ConfirmResetRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = token_db::consume_password_reset_token(&state.db, &req.token).await?;

    users::update_password(&state.db, &user_id, &req.new_password, &state.config).await?;
    sessions::revoke_all_user_sessions(&state.db, &user_id, "password_reset").await?;

    audit::log_event(
        &state.db,
        Some(&user_id),
        "password_reset_completed",
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
        "message": "Password reset. Please log in with your new password.",
    })))
}
