use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::{audit, users};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthContext;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: String,
    pub email: String,
    pub email_verified: bool,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub totp_enabled: bool,
    pub last_login_at: Option<i64>,
    pub created_at: i64,
}

pub async fn me(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
) -> AppResult<Json<serde_json::Value>> {
    let user = users::get_user_by_id(&state.db, &ctx.user_id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "user": MeResponse {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified == 1,
            first_name: user.first_name,
            last_name: user.last_name,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            phone: user.phone,
            totp_enabled: user.totp_enabled == 1,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
        },
    })))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMeRequest {
    #[validate(length(max = 100))]
    pub first_name: Option<String>,
    #[validate(length(max = 100))]
    pub last_name: Option<String>,
    #[validate(length(max = 100))]
    pub display_name: Option<String>,
    // ponytail: length check only — strict URL validation rejects empty-string clears from clients.
    #[validate(length(max = 2048))]
    pub avatar_url: Option<String>,
    #[validate(length(max = 30))]
    pub phone: Option<String>,
}

pub async fn update_me(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
    Json(req): Json<UpdateMeRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "UPDATE users SET
            first_name = COALESCE(?, first_name),
            last_name = COALESCE(?, last_name),
            display_name = COALESCE(?, display_name),
            avatar_url = COALESCE(?, avatar_url),
            phone = COALESCE(?, phone),
            updated_at = ?
         WHERE id = ?",
    )
    .bind(req.first_name.as_deref())
    .bind(req.last_name.as_deref())
    .bind(req.display_name.as_deref())
    .bind(req.avatar_url.as_deref())
    .bind(req.phone.as_deref())
    .bind(now)
    .bind(&ctx.user_id)
    .execute(&state.db)
    .await?;

    audit::log_event(
        &state.db,
        Some(&ctx.user_id),
        "profile_updated",
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

pub async fn audit_log(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
) -> AppResult<Json<serde_json::Value>> {
    let entries = audit::list_for_user(&state.db, &ctx.user_id, 100).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "events": entries,
    })))
}
