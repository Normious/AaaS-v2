use sqlx::SqlitePool;
use std::sync::Arc;

use crate::auth::jwt::JwtIssuer;
use crate::config::Config;
use crate::ratelimit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
    pub jwt: Arc<JwtIssuer>,
    pub limiter: Arc<RateLimiter>,
}
