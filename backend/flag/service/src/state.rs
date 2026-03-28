use sea_orm::DatabaseConnection;
use shared_security::JwtValidator;

use crate::jwks_cache::JwksCache;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwks: std::sync::Arc<JwksCache>,
    pub jwt: JwtValidator,
}
