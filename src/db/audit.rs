use sqlx::SqlitePool;

use crate::error::AppResult;

#[allow(clippy::too_many_arguments)]
pub async fn log_event(
    pool: &SqlitePool,
    user_id: Option<&str>,
    event_type: &str,
    ip: Option<&str>,
    user_agent: Option<&str>,
    metadata: Option<serde_json::Value>,
    success: bool,
    error_message: Option<&str>,
) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp_millis();
    let meta_str = metadata.map(|m| m.to_string()).unwrap_or_else(|| "{}".into());

    sqlx::query(
        "INSERT INTO audit_log (user_id, event_type, ip_address, user_agent, metadata, success, error_message, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(event_type)
    .bind(ip)
    .bind(user_agent)
    .bind(&meta_str)
    .bind(i64::from(success))
    .bind(error_message)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub user_id: Option<String>,
    pub event_type: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<String>,
    pub success: i64,
    pub error_message: Option<String>,
    pub created_at: i64,
}

pub async fn list_for_user(
    pool: &SqlitePool,
    user_id: &str,
    limit: i64,
) -> AppResult<Vec<AuditEntry>> {
    Ok(sqlx::query_as::<_, AuditEntry>(
        "SELECT * FROM audit_log WHERE user_id = ? ORDER BY created_at DESC LIMIT ?",
    )
    .bind(user_id)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await?)
}
