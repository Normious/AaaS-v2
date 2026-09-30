use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user_id: String,
    pub session_id: String,
    pub scope: Vec<String>,
}

/// Middleware that validates the Bearer access token.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer(&req).ok_or(AppError::Unauthorized)?;
    let claims = state.jwt.verify_access_token(token)?;

    // Rejected 2fa-pending challenge tokens on normal routes.
    if claims.scope == vec!["2fa".to_string()] {
        return Err(AppError::Unauthorized);
    }

    let ctx = AuthContext {
        user_id: claims.sub,
        session_id: claims.sid,
        scope: claims.scope,
    };

    req.extensions_mut().insert(ctx);
    Ok(next.run(req).await)
}

pub async fn request_id(mut req: Request, next: Next) -> Response {
    let id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    if let Ok(v) = id.parse() {
        req.headers_mut().insert("x-request-id", v);
    }

    let mut response = next.run(req).await;
    if let Ok(v) = id.parse() {
        response.headers_mut().insert("x-request-id", v);
    }
    response
}

fn extract_bearer(req: &Request) -> Option<&str> {
    req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
}

pub fn context(req: &Request) -> Result<AuthContext, AppError> {
    req.extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(AppError::Unauthorized)
}
