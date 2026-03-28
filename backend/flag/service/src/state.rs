use sea_orm::DatabaseConnection;
use shared_security::JwtValidator;

use crate::jwks_cache::JwksCache;

/// Shared Axum state: DB, cached JWKS from auth, and JWT validation settings.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwks: std::sync::Arc<JwksCache>,
    pub jwt: JwtValidator,
}
