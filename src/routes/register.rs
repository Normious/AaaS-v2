use axum::{extract::State, http::HeaderMap, Json};
use serde::Deserialize;
use validator::Validate;

use crate::db::{audit, tokens as token_db, users};
use crate::email;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(max = 100))]
    pub first_name: Option<String>,
    #[validate(length(max = 100))]
    pub last_name: Option<String>,
}

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let ip = client_ip(&headers);
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok());

    let key = format!("register:ip:{}", ip.as_deref().unwrap_or("unknown"));
    state
        .limiter
        .check(&key, state.config.rl_register_per_ip_per_hour, 3600)
        .map_err(|retry| AppError::RateLimited {
            retry_after_seconds: retry,
        })?;

    let user = users::create_user(
        &state.db,
        &req.email,
        &req.password,
        req.first_name.as_deref(),
        req.last_name.as_deref(),
        &state.config,
    )
    .await?;

    let raw_token = crate::auth::tokens::generate_token();
    token_db::issue_email_verification_token(&state.db, &user.id, &user.email, &raw_token)
        .await?;

    if state.config.smtp_enabled {
        let verify_url = format!(
            "{}{}?token={}",
            state.config.app_base_url.trim_end_matches('/'),
            state.config.email_verify_path,
            raw_token
        );
        let sender = email::EmailSender::from_config(&state.config);
        let to = user.email.clone();
        let name = user.first_name.clone();
        tokio::spawn(async move {
            let _ = sender.send_verification(&to, name.as_deref(), &verify_url).await;
        });
    }

    audit::log_event(
        &state.db,
        Some(&user.id),
        "user_registered",
        ip.as_deref(),
        ua,
        None,
        true,
        None,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "user_id": user.id,
        "email": user.email,
        "email_verified": user.email_verified == 1,
        "message": "Account created. Check your email to verify your address.",
    })))
}

pub fn client_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("cf-connecting-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
}
