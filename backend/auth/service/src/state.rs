use sea_orm::DatabaseConnection;
use shared_security::JwtSigner;

/// Shared Axum state: DB pool, JWT signer, and PEM copy for JWKS derivation.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt: JwtSigner,
    pub jwt_private_pem: String,
    pub jwt_ttl_secs: u64,
}
