use sqlx::SqlitePool;
use uuid::Uuid;

use crate::auth::{lockout, password};
use crate::config::Config;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub email_verified: i64,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub is_active: i64,
    pub is_locked: i64,
    pub locked_until: Option<i64>,
    pub failed_login_attempts: i64,
    pub totp_secret: Option<String>,
    pub totp_enabled: i64,
    pub last_login_at: Option<i64>,
    pub last_login_ip: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub async fn create_user(
    pool: &SqlitePool,
    email: &str,
    raw_password: &str,
    first_name: Option<&str>,
    last_name: Option<&str>,
    config: &Config,
) -> AppResult<User> {
    let email = email.trim().to_lowercase();
    let password_hash = password::hash_password(raw_password, config)?;

    let id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, first_name, last_name, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&email)
    .bind(&password_hash)
    .bind(first_name)
    .bind(last_name)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict("Email already registered".into())
        } else {
            AppError::Database(e)
        }
    })?;

    get_user_by_id(pool, &id).await
}

pub async fn get_user_by_id(pool: &SqlitePool, id: &str) -> AppResult<User> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))
}

pub async fn get_user_by_email(pool: &SqlitePool, email: &str) -> AppResult<Option<User>> {
    let email = email.trim().to_lowercase();
    Ok(sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await?)
}

pub async fn record_failed_login(
    pool: &SqlitePool,
    user_id: &str,
    config: &Config,
) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE users SET failed_login_attempts = failed_login_attempts + 1, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await?;

    let user = get_user_by_id(pool, user_id).await?;
    if lockout::should_lock_now(user.failed_login_attempts, config) {
        let locked_until = lockout::compute_lockout_until(config);
        sqlx::query("UPDATE users SET is_locked = 1, locked_until = ?, updated_at = ? WHERE id = ?")
            .bind(locked_until)
            .bind(now)
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn record_successful_login(
    pool: &SqlitePool,
    user_id: &str,
    ip: &str,
) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        r#"
        UPDATE users
        SET failed_login_attempts = 0, is_locked = 0, locked_until = NULL,
            last_login_at = ?, last_login_ip = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(now)
    .bind(ip)
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_password(
    pool: &SqlitePool,
    user_id: &str,
    new_password: &str,
    config: &Config,
) -> AppResult<()> {
    let hash = password::hash_password(new_password, config)?;
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(&hash)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_email_verified(pool: &SqlitePool, user_id: &str) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("UPDATE users SET email_verified = 1, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn enable_totp(pool: &SqlitePool, user_id: &str, secret: &str) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("UPDATE users SET totp_secret = ?, totp_enabled = 1, updated_at = ? WHERE id = ?")
        .bind(secret)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn disable_totp(pool: &SqlitePool, user_id: &str) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("UPDATE users SET totp_secret = NULL, totp_enabled = 0, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM totp_backup_codes WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
