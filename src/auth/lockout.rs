use crate::config::Config;

pub fn is_locked(locked_until: Option<i64>) -> bool {
    match locked_until {
        Some(t) => t > chrono::Utc::now().timestamp_millis(),
        None => false,
    }
}

pub fn compute_lockout_until(config: &Config) -> i64 {
    chrono::Utc::now().timestamp_millis() + config.lockout_duration_minutes * 60 * 1000
}

/// Called AFTER the counter was already incremented, so `failed_attempts` is current.
pub fn should_lock_now(failed_attempts: i64, config: &Config) -> bool {
    failed_attempts >= config.max_failed_logins
}
