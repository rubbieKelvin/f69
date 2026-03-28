//! Structured logging via the `tracing` ecosystem (`tracing-subscriber`).
//!
//! Uses `RUST_LOG` when set (same semantics as `env_logger`).

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Installs the process-global tracing subscriber. Call exactly once at startup.
///
/// When `json` is true, logs are JSON lines (for production aggregators); otherwise
/// logs use the human-oriented "pretty" format.
pub fn init(service_name: &'static str, json: bool) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    if json {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_current_span(true)
                    .flatten_event(true),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .pretty()
                    .with_target(true),
            )
            .init();
    }

    tracing::info!(service = service_name, "telemetry initialized");
}
