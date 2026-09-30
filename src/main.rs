mod auth;
mod config;
mod db;
mod email;
mod error;
mod middleware;
mod models;
mod ratelimit;
mod routes;
mod state;

use axum::{
    middleware as axum_mw,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use auth::jwt::JwtIssuer;
use config::Config;
use ratelimit::RateLimiter;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aaas_v2=info,tower_http=info".into()),
        )
        .with_target(false)
        .json()
        .init();

    let config = Config::from_env();
    let config_arc = Arc::new(config.clone());

    if let Some(parent) = std::path::Path::new(&config.database_url).parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
    }

    let db_url = if config.database_url.starts_with("postgres") {
        anyhow::bail!("PostgreSQL not wired in this build; use SQLite DATABASE_URL");
    } else {
        format!("sqlite://{}?mode=rwc", config.database_url)
    };

    let db = SqlitePoolOptions::new()
        .max_connections(config.db_pool_size)
        .connect(&db_url)
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL").execute(&db).await.ok();
    sqlx::query("PRAGMA foreign_keys=ON").execute(&db).await.ok();

    let migration_sql = include_str!("../migrations/0001_init.sql");
    for stmt in migration_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(stmt).execute(&db).await?;
    }
    tracing::info!("Database migrated");

    let jwt = Arc::new(JwtIssuer::from_config(&config)?);
    let limiter = Arc::new(RateLimiter::new());

    let state = AppState {
        db: db.clone(),
        config: config_arc.clone(),
        jwt,
        limiter,
    };

    let cors = if config.cors_allowed_origins.contains(&"*".to_string()) {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        use axum::http::HeaderValue;
        let origins: Vec<HeaderValue> = config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let public = Router::new()
        .route("/health", get(routes::health::health))
        .route("/auth/register", post(routes::register::register))
        .route("/auth/login", post(routes::login::login))
        .route("/auth/login/2fa", post(routes::login::login_2fa))
        .route("/auth/refresh", post(routes::refresh::refresh))
        .route("/auth/verify", get(routes::verify::verify))
        .route(
            "/auth/password/reset",
            post(routes::password::request_reset),
        )
        .route(
            "/auth/password/reset/confirm",
            post(routes::password::confirm_reset),
        )
        .route("/auth/email/verify", post(routes::email::verify_email))
        .route(
            "/auth/email/resend",
            post(routes::email::resend_verification),
        );

    let authenticated = Router::new()
        .route("/me", get(routes::me::me))
        .route("/me", put(routes::me::update_me))
        .route("/me/password", post(routes::password::change_password))
        .route("/me/email", post(routes::email::change_email))
        .route("/me/sessions", get(routes::sessions::list))
        .route("/me/sessions/:id", delete(routes::sessions::revoke))
        .route("/me/2fa/enroll", post(routes::two_factor::enroll))
        .route("/me/2fa/verify", post(routes::two_factor::verify))
        .route("/me/2fa/disable", post(routes::two_factor::disable))
        .route("/me/audit-log", get(routes::me::audit_log))
        .route("/auth/logout", post(routes::logout::logout))
        .layer(axum_mw::from_fn_with_state(
            state.clone(),
            middleware::auth::require_auth,
        ));

    let app = Router::new()
        .merge(public)
        .merge(authenticated)
        .layer(cors)
        .layer(axum_mw::from_fn(middleware::auth::request_id))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(address = %addr, "AaaS v2 listening");

    axum::serve(listener, app).await?;
    Ok(())
}
