use std::time::Duration;

use anyhow::Context;
use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderValue, Method};
use axum::routing::get;
use axum::Router;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use shared_config::{Environment, FlagServiceConfig};
use shared_security::JwtValidator;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use flag_service::jwks_cache::JwksCache;
use flag_service::routes::{health, v1};
use flag_service::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = FlagServiceConfig::from_env().context("load FLAG_* config")?;

    let json_logs = matches!(config.environment, Environment::Production);
    shared_telemetry::init("flag-service", json_logs);

    let db = Database::connect(&config.database_url)
        .await
        .context("database connect")?;

    if config.run_migrations_on_startup {
        flag_migration::Migrator::up(&db, None)
            .await
            .context("flag migrations")?;
    }

    let jwks = JwksCache::new(&config.auth_base_url, Duration::from_secs(300));
    let jwt = JwtValidator::new(&config.jwt_issuer, &config.jwt_audience);

    let state = AppState {
        db: db.clone(),
        jwks,
        jwt,
    };

    let cors = build_cors(&config);

    let app = Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .route("/v1/projects", get(v1::list_projects).post(v1::create_project))
        .route(
            "/v1/projects/{project_id}/flags",
            get(v1::list_flags).post(v1::create_flag),
        )
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
    tracing::info!(%addr, "flag-service listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_cors(config: &FlagServiceConfig) -> CorsLayer {
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
