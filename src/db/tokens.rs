use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::auth::tokens as token_util;
use crate::config::Config;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: i64,
    pub used_at: Option<i64>,
    pub replaced_by: Option<String>,
}

/// Issue a fresh refresh token for a session.
pub async fn issue_refresh_token(
    pool: &SqlitePool,
    session_id: &str,
    user_id: &str,
    config: &Config,
) -> AppResult<String> {
    let id = Uuid::now_v7().to_string();
    let raw_token = token_util::generate_token();
    let token_hash = token_util::hash_token(&raw_token);
    let now = chrono::Utc::now().timestamp_millis();
    let expires = now + config.jwt_refresh_ttl * 1000;

    sqlx::query(
        "INSERT INTO refresh_tokens (id, session_id, user_id, token_hash, expires_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(session_id)
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(raw_token)
}

/// Rotate a refresh token. Returns (new_raw_token, session_id, user_id).
pub async fn rotate_refresh_token(
    pool: &SqlitePool,
    raw_token: &str,
    config: &Config,
) -> AppResult<(String, String, String)> {
    let token_hash = token_util::hash_token(raw_token);

    let existing = sqlx::query_as::<_, RefreshToken>(
        "SELECT * FROM refresh_tokens WHERE token_hash = ?",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?;

    let existing = match existing {
        Some(t) => t,
        None => return Err(AppError::Unauthorized),
    };

    // THEFT DETECTION: already-used token reused => revoke entire session.
    if existing.used_at.is_some() {
        super::sessions::revoke_session(pool, &existing.session_id, "theft_detected").await?;
        tracing::warn!(
            user_id = %existing.user_id,
            session_id = %existing.session_id,
            "Refresh token reuse detected — session revoked"
        );
        return Err(AppError::Unauthorized);
    }

    let now = chrono::Utc::now().timestamp_millis();
    if existing.expires_at < now {
        return Err(AppError::Unauthorized);
    }

    let session = super::sessions::get_session(pool, &existing.session_id).await?;
    if session.is_revoked == 1 || session.expires_at < now {
        return Err(AppError::Unauthorized);
    }

    let new_token_id = Uuid::now_v7().to_string();
    let new_raw = token_util::generate_token();
    let new_hash = token_util::hash_token(&new_raw);
    let new_expires = now + config.jwt_refresh_ttl * 1000;

    sqlx::query(
        "INSERT INTO refresh_tokens (id, session_id, user_id, token_hash, expires_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&new_token_id)
    .bind(&existing.session_id)
    .bind(&existing.user_id)
    .bind(&new_hash)
    .bind(new_expires)
    .bind(now)
    .execute(pool)
    .await?;

    sqlx::query("UPDATE refresh_tokens SET used_at = ?, replaced_by = ? WHERE id = ?")
        .bind(now)
        .bind(&new_token_id)
        .bind(&existing.id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE sessions SET last_used_at = ? WHERE id = ?")
        .bind(now)
        .bind(&existing.session_id)
        .execute(pool)
        .await?;

    Ok((new_raw, existing.session_id, existing.user_id))
}

// ─── Email Verification Tokens ────────────────────────────────

pub async fn issue_email_verification_token(
    pool: &SqlitePool,
    user_id: &str,
    email: &str,
    raw_token: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE email_verification_tokens SET used_at = ? WHERE user_id = ? AND used_at IS NULL",
    )
    .bind(chrono::Utc::now().timestamp_millis())
    .bind(user_id)
    .execute(pool)
    .await?;

    let id = Uuid::now_v7().to_string();
    let token_hash = token_util::hash_token(raw_token);
    let now = chrono::Utc::now().timestamp_millis();
    let expires = now + 24 * 60 * 60 * 1000;

    sqlx::query(
        "INSERT INTO email_verification_tokens (id, user_id, token_hash, email, expires_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&token_hash)
    .bind(email)
    .bind(expires)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn consume_email_verification_token(
    pool: &SqlitePool,
    raw_token: &str,
) -> AppResult<(String, String)> {
    let token_hash = token_util::hash_token(raw_token);
    let now = chrono::Utc::now().timestamp_millis();

    let row = sqlx::query(
        "SELECT id, user_id, email FROM email_verification_tokens
         WHERE token_hash = ? AND used_at IS NULL AND expires_at > ?",
    )
    .bind(&token_hash)
    .bind(now)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let token_id: String = row.get("id");
    let user_id: String = row.get("user_id");
    let email: String = row.get("email");

    sqlx::query("UPDATE email_verification_tokens SET used_at = ? WHERE id = ?")
        .bind(now)
        .bind(&token_id)
        .execute(pool)
        .await?;

    Ok((user_id, email))
}

// ─── Password Reset Tokens ────────────────────────────────────

pub async fn issue_password_reset_token(
    pool: &SqlitePool,
    user_id: &str,
    raw_token: &str,
) -> AppResult<()> {
    sqlx::query("UPDATE password_reset_tokens SET used_at = ? WHERE user_id = ? AND used_at IS NULL")
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(user_id)
        .execute(pool)
        .await?;

    let id = Uuid::now_v7().to_string();
    let token_hash = token_util::hash_token(raw_token);
    let now = chrono::Utc::now().timestamp_millis();
    let expires = now + 60 * 60 * 1000;

    sqlx::query(
        "INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn consume_password_reset_token(
    pool: &SqlitePool,
    raw_token: &str,
) -> AppResult<String> {
    let token_hash = token_util::hash_token(raw_token);
    let now = chrono::Utc::now().timestamp_millis();

    let row = sqlx::query(
        "SELECT id, user_id FROM password_reset_tokens
         WHERE token_hash = ? AND used_at IS NULL AND expires_at > ?",
    )
    .bind(&token_hash)
    .bind(now)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let token_id: String = row.get("id");
    let user_id: String = row.get("user_id");

    sqlx::query("UPDATE password_reset_tokens SET used_at = ? WHERE id = ?")
        .bind(now)
        .bind(&token_id)
        .execute(pool)
        .await?;

    Ok(user_id)
}
