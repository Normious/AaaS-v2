use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub env: String,
    pub log_level: String,
    pub database_url: String,
    pub db_pool_size: u32,

    pub jwt_private_key_path: String,
    pub jwt_public_key_path: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub jwt_access_ttl: i64,
    pub jwt_refresh_ttl: i64,

    pub argon2_memory_kib: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,

    pub session_max_devices: i64,
    pub session_idle_timeout_days: i64,

    pub max_failed_logins: i64,
    pub lockout_duration_minutes: i64,

    pub rl_login_per_ip_per_min: u32,
    pub rl_login_per_email_per_min: u32,
    pub rl_register_per_ip_per_hour: u32,
    pub rl_password_reset_per_email_per_hour: u32,

    pub smtp_enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,

    pub app_base_url: String,
    pub email_verify_path: String,
    pub password_reset_path: String,

    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            port: get_int("PORT", 3000) as u16,
            env: get("ENV", "production"),
            log_level: get("LOG_LEVEL", "info"),
            database_url: get("DATABASE_URL", "./data/aaas-v2.db"),
            db_pool_size: get_int("DB_POOL_SIZE", 20) as u32,

            jwt_private_key_path: get("JWT_PRIVATE_KEY_PATH", "./secrets/jwt-private.pem"),
            jwt_public_key_path: get("JWT_PUBLIC_KEY_PATH", "./secrets/jwt-public.pem"),
            jwt_issuer: get("JWT_ISSUER", "https://auth.example.com"),
            jwt_audience: get("JWT_AUDIENCE", "api.example.com"),
            jwt_access_ttl: get_int("JWT_ACCESS_TOKEN_TTL_SECONDS", 900),
            jwt_refresh_ttl: get_int("JWT_REFRESH_TOKEN_TTL_SECONDS", 30 * 24 * 3600),

            argon2_memory_kib: get_int("ARGON2_MEMORY_KIB", 19456) as u32,
            argon2_iterations: get_int("ARGON2_ITERATIONS", 2) as u32,
            argon2_parallelism: get_int("ARGON2_PARALLELISM", 1) as u32,

            session_max_devices: get_int("SESSION_MAX_DEVICES", 10),
            session_idle_timeout_days: get_int("SESSION_IDLE_TIMEOUT_DAYS", 90),

            max_failed_logins: get_int("MAX_FAILED_LOGINS", 5),
            lockout_duration_minutes: get_int("LOCKOUT_DURATION_MINUTES", 15),

            rl_login_per_ip_per_min: get_int("RL_LOGIN_PER_IP_PER_MIN", 20) as u32,
            rl_login_per_email_per_min: get_int("RL_LOGIN_PER_EMAIL_PER_MIN", 5) as u32,
            rl_register_per_ip_per_hour: get_int("RL_REGISTER_PER_IP_PER_HOUR", 10) as u32,
            rl_password_reset_per_email_per_hour: get_int("RL_PASSWORD_RESET_PER_EMAIL_PER_HOUR", 3)
                as u32,

            smtp_enabled: get_bool("SMTP_ENABLED", false),
            smtp_host: get("SMTP_HOST", ""),
            smtp_port: get_int("SMTP_PORT", 587) as u16,
            smtp_username: get("SMTP_USERNAME", ""),
            smtp_password: get("SMTP_PASSWORD", ""),
            smtp_from_email: get("SMTP_FROM_EMAIL", "noreply@example.com"),
            smtp_from_name: get("SMTP_FROM_NAME", "AaaS"),

            app_base_url: get("APP_BASE_URL", "https://example.com"),
            email_verify_path: get("EMAIL_VERIFY_PATH", "/verify-email"),
            password_reset_path: get("PASSWORD_RESET_PATH", "/reset-password"),

            cors_allowed_origins: get("CORS_ALLOWED_ORIGINS", "*")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        }
    }
}

fn get(k: &str, d: &str) -> String {
    env::var(k).unwrap_or_else(|_| d.to_string())
}

fn get_int(k: &str, d: i64) -> i64 {
    env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}

fn get_bool(k: &str, d: bool) -> bool {
    env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}
