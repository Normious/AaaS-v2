use axum::{extract::State, Extension, Json};
use serde::Deserialize;

use crate::db::{audit, sessions};
use crate::error::AppResult;
use crate::middleware::auth::AuthContext;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    #[serde(default)]
    pub all_devices: bool,
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
    Json(req): Json<LogoutRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if req.all_devices {
        let count = sessions::revoke_all_user_sessions(&state.db, &ctx.user_id, "logout").await?;
        audit::log_event(
            &state.db,
            Some(&ctx.user_id),
            "logout_all",
            None,
            None,
            None,
            true,
            None,
        )
        .await
        .ok();
        return Ok(Json(serde_json::json!({
            "success": true,
            "revoked_sessions": count,
        })));
    }

    sessions::revoke_session(&state.db, &ctx.session_id, "logout").await?;
    audit::log_event(
        &state.db,
        Some(&ctx.user_id),
        "logout",
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
