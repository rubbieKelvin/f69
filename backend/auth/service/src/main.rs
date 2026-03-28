//! Identity service: registration, login, RS256 JWT issuance, and JWKS for other services.

use std::time::Duration;

use anyhow::Context;
use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderValue, Method};
use axum::routing::{get, post};
use axum::Router;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use shared_config::{AuthServiceConfig, Environment};
use shared_security::JwtSigner;
use shared_telemetry;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use auth_service::routes::{health, jwks, v1};
use auth_service::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = AuthServiceConfig::from_env().context("load AUTH_* config")?;

    let json_logs = matches!(config.environment, Environment::Production);
    shared_telemetry::init("auth-service", json_logs);

    let pem = load_jwt_pem(&config)?;
    let jwt = JwtSigner::from_pem(
        &pem,
        &config.jwt_issuer,
        &config.jwt_audience,
        Duration::from_secs(config.jwt_access_ttl_secs),
    )
    .context("JWT signer init")?;

    let db = Database::connect(&config.database_url)
        .await
        .context("database connect")?;

    if config.run_migrations_on_startup {
        auth_migration::Migrator::up(&db, None)
            .await
            .context("auth migrations")?;
    }

    let state = AppState {
        db: db.clone(),
        jwt,
        jwt_private_pem: pem,
        jwt_ttl_secs: config.jwt_access_ttl_secs,
    };

    let cors = build_cors(&config);

    let app = Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .route(
            "/.well-known/jwks.json",
            get(jwks::jwks),
        )
        .route("/v1/register", post(v1::register))
        .route("/v1/login", post(v1::login))
        // Axum default body limit is small; we set an explicit cap below instead.
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(256 * 1024))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(cors)
        .with_state(state);

    let addr = config.socket_addr();
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "auth-service listening");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Read PEM from inline config or from disk.
fn load_jwt_pem(config: &AuthServiceConfig) -> anyhow::Result<String> {
    if let Some(pem) = &config.jwt_private_key_pem {
        return Ok(pem.clone());
    }
    if let Some(path) = &config.jwt_private_key_path {
        return std::fs::read_to_string(path).with_context(|| format!("read {path}"));
    }
    anyhow::bail!("no JWT private key configured")
}

/// CORS: explicit allowlist in production; permissive in development when unset.
fn build_cors(config: &AuthServiceConfig) -> CorsLayer {
    let mut layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            axum::http::HeaderName::from_static("x-request-id"),
        ]);

    if let Some(origins) = &config.cors_allowed_origins {
        let list: Vec<_> = origins
            .split(',')
            .filter_map(|s| s.trim().parse::<HeaderValue>().ok())
            .collect();
        if !list.is_empty() {
            layer = layer.allow_origin(AllowOrigin::list(list));
        }
    } else if !config.environment.is_production() {
        layer = layer.allow_origin(tower_http::cors::Any);
    }

    layer
}
