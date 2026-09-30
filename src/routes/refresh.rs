use axum::{extract::State, Json};
use serde::Deserialize;

use crate::db::{audit, tokens as token_db};
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let (new_refresh, session_id, user_id) =
        token_db::rotate_refresh_token(&state.db, &req.refresh_token, &state.config).await?;

    let access = state.jwt.issue_access_token(&user_id, &session_id, vec!["user".into()])?;

    audit::log_event(
        &state.db,
        Some(&user_id),
        "token_refreshed",
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
        "access_token": access,
        "refresh_token": new_refresh,
        "token_type": "Bearer",
        "expires_in": state.config.jwt_access_ttl,
    })))
}
