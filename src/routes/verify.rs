use axum::{extract::State, http::HeaderMap, Json};

use crate::auth::jwt::AccessClaims;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub async fn verify(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims: AccessClaims = state.jwt.verify_access_token(token)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "valid": true,
        "user_id": claims.sub,
        "session_id": claims.sid,
        "scope": claims.scope,
        "expires_at": claims.exp,
        "issued_at": claims.iat,
    })))
}
