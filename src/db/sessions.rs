use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config::Config;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub device_name: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    #[serde(skip)]
    pub is_revoked: i64,
    pub revoked_at: Option<i64>,
    #[serde(skip)]
    pub revoked_reason: Option<String>,
    pub created_at: i64,
    pub last_used_at: i64,
    pub expires_at: i64,
}

pub async fn create_session(
    pool: &SqlitePool,
    user_id: &str,
    user_agent: Option<&str>,
    ip: Option<&str>,
    config: &Config,
) -> AppResult<Session> {
    enforce_max_devices(pool, user_id, config).await?;

    let id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let expires = now + config.jwt_refresh_ttl * 1000;

    let device_name = user_agent.and_then(parse_device_name);

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, device_name, user_agent, ip_address, created_at, last_used_at, expires_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(user_id)
    .bind(&device_name)
    .bind(user_agent)
    .bind(ip)
    .bind(now)
    .bind(now)
    .bind(expires)
    .execute(pool)
    .await?;

    get_session(pool, &id).await
}

pub async fn get_session(pool: &SqlitePool, id: &str) -> AppResult<Session> {
    sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Session not found".into()))
}

pub async fn list_sessions_for_user(
    pool: &SqlitePool,
    user_id: &str,
) -> AppResult<Vec<Session>> {
    let now = chrono::Utc::now().timestamp_millis();
    Ok(sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions
         WHERE user_id = ? AND is_revoked = 0 AND expires_at > ?
         ORDER BY last_used_at DESC",
    )
    .bind(user_id)
    .bind(now)
    .fetch_all(pool)
    .await?)
}

pub async fn revoke_session(pool: &SqlitePool, session_id: &str, reason: &str) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "UPDATE sessions SET is_revoked = 1, revoked_at = ?, revoked_reason = ?
         WHERE id = ? AND is_revoked = 0",
    )
    .bind(now)
    .bind(reason)
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn revoke_all_user_sessions(
    pool: &SqlitePool,
    user_id: &str,
    reason: &str,
) -> AppResult<u64> {
    let now = chrono::Utc::now().timestamp_millis();
    let result = sqlx::query(
        "UPDATE sessions SET is_revoked = 1, revoked_at = ?, revoked_reason = ?
         WHERE user_id = ? AND is_revoked = 0",
    )
    .bind(now)
    .bind(reason)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

async fn enforce_max_devices(
    pool: &SqlitePool,
    user_id: &str,
    config: &Config,
) -> AppResult<()> {
    let active = list_sessions_for_user(pool, user_id).await?;

    if active.len() as i64 >= config.session_max_devices {
        if let Some(oldest) = active.last() {
            revoke_session(pool, &oldest.id, "max_devices_exceeded").await?;
        }
    }

    Ok(())
}

fn parse_device_name(ua: &str) -> Option<String> {
    let ua_lower = ua.to_lowercase();

    let browser = if ua_lower.contains("firefox") {
        "Firefox"
    } else if ua_lower.contains("edg/") {
        "Edge"
    } else if ua_lower.contains("chrome") {
        "Chrome"
    } else if ua_lower.contains("safari") {
        "Safari"
    } else {
        "Browser"
    };

    let os = if ua_lower.contains("windows") {
        "Windows"
    } else if ua_lower.contains("mac os") {
        "macOS"
    } else if ua_lower.contains("linux") {
        "Linux"
    } else if ua_lower.contains("android") {
        "Android"
    } else if ua_lower.contains("iphone") || ua_lower.contains("ipad") {
        "iOS"
    } else {
        "Unknown OS"
    };

    Some(format!("{browser} on {os}"))
}
