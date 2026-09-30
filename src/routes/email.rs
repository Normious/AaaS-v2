use axum::{extract::State, http::HeaderMap, Extension, Json};
use serde::Deserialize;
use validator::Validate;

use crate::auth::tokens as token_util;
use crate::db::{audit, tokens as token_db, users};
use crate::email;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthContext;
use crate::routes::register::client_ip;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

pub async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let (user_id, email) =
        token_db::consume_email_verification_token(&state.db, &req.token).await?;

    // If the token was issued for an email change, apply the new email now.
    let user = users::get_user_by_id(&state.db, &user_id).await?;
    if user.email != email {
        let now = chrono::Utc::now().timestamp_millis();
        let updated = sqlx::query(
            "UPDATE users SET email = ?, email_verified = 1, updated_at = ? WHERE id = ?",
        )
        .bind(&email)
        .bind(now)
        .bind(&user_id)
        .execute(&state.db)
        .await;
        if let Err(e) = updated {
            if e.to_string().contains("UNIQUE") {
                return Err(AppError::Conflict("Email already registered".into()));
            }
            return Err(AppError::Database(e));
        }
    } else {
        users::mark_email_verified(&state.db, &user_id).await?;
    }

    audit::log_event(
        &state.db,
        Some(&user_id),
        "email_verified",
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
        "message": "Email verified.",
    })))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResendVerificationRequest {
    #[validate(email)]
    pub email: String,
}

pub async fn resend_verification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ResendVerificationRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let ip = client_ip(&headers).unwrap_or_else(|| "unknown".into());

    state
        .limiter
        .check(&format!("resend_verify:ip:{ip}"), 5, 3600)
        .map_err(|r| AppError::RateLimited {
            retry_after_seconds: r,
        })?;

    if let Some(user) = users::get_user_by_email(&state.db, &req.email).await? {
        if user.email_verified == 0 {
            let raw_token = token_util::generate_token();
            token_db::issue_email_verification_token(
                &state.db,
                &user.id,
                &user.email,
                &raw_token,
            )
            .await?;

            if state.config.smtp_enabled {
                let url = format!(
                    "{}{}?token={}",
                    state.config.app_base_url.trim_end_matches('/'),
                    state.config.email_verify_path,
                    raw_token
                );
                let sender = email::EmailSender::from_config(&state.config);
                let to = user.email.clone();
                let name = user.first_name.clone();
                tokio::spawn(async move {
                    let _ = sender.send_verification(&to, name.as_deref(), &url).await;
                });
            }
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "If the email is registered and unverified, a new link has been sent.",
    })))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangeEmailRequest {
    #[validate(email)]
    pub new_email: String,
}

/// POST /me/email — change email, requires re-verification.
/// The email is only swapped after the new address is verified via /auth/email/verify.
pub async fn change_email(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
    Json(req): Json<ChangeEmailRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let new_email = req.new_email.trim().to_lowercase();
    let user = users::get_user_by_id(&state.db, &ctx.user_id).await?;

    if user.email == new_email {
        return Err(AppError::Validation("New email is the same as current".into()));
    }
    if users::get_user_by_email(&state.db, &new_email).await?.is_some() {
        return Err(AppError::Conflict("Email already registered".into()));
    }

    let raw_token = token_util::generate_token();
    token_db::issue_email_verification_token(&state.db, &user.id, &new_email, &raw_token).await?;

    if state.config.smtp_enabled {
        let url = format!(
            "{}{}?token={}",
            state.config.app_base_url.trim_end_matches('/'),
            state.config.email_verify_path,
            raw_token
        );
        let sender = email::EmailSender::from_config(&state.config);
        let name = user.first_name.clone();
        tokio::spawn(async move {
            let _ = sender.send_email_changed(&new_email, name.as_deref(), &url).await;
        });
    }

    audit::log_event(
        &state.db,
        Some(&user.id),
        "email_change_requested",
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
        "message": "Verification link sent to your new email. Your email changes after verification.",
    })))
}
