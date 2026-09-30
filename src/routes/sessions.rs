use axum::{
    extract::{Path, State},
    Extension, Json,
};

use crate::db::{audit, sessions};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthContext;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
) -> AppResult<Json<serde_json::Value>> {
    let list = sessions::list_sessions_for_user(&state.db, &ctx.user_id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "sessions": list,
        "current_session_id": ctx.session_id,
    })))
}

pub async fn revoke(
    State(state): State<AppState>,
    Extension(ctx): Extension<AuthContext>,
    Path(session_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let session = sessions::get_session(&state.db, &session_id).await?;
    if session.user_id != ctx.user_id {
        return Err(AppError::Forbidden("Not your session".into()));
    }

    sessions::revoke_session(&state.db, &session_id, "user_revoked").await?;
    audit::log_event(
        &state.db,
        Some(&ctx.user_id),
        "session_revoked",
        None,
        None,
        Some(serde_json::json!({ "session_id": session_id })),
        true,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({ "success": true })))
}
