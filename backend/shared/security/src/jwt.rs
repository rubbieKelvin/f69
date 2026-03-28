//! RS256 JWT signing (auth) and validation (consumers) plus minimal JWKS types.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::{DecodePrivateKey, EncodePublicKey};
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Key id embedded in JWT headers and JWKS; rotate by changing this and supporting multiple JWKs.
const KID: &str = "f69-key-1";

/// Errors from PEM loading, RSA operations, or `jsonwebtoken`.
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("invalid PEM: {0}")]
    Pem(String),
    #[error("token error: {0}")]
    Token(#[from] jsonwebtoken::errors::Error),
    #[error("RSA: {0}")]
    Rsa(#[from] rsa::Error),
    #[error("invalid JWK")]
    InvalidJwk,
    #[error("kid not found: {0}")]
    KidNotFound(String),
}

/// Standard access-token claims; `sub` holds the authenticated user id (UUID string).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub aud: String,
}

/// Signs access JWTs with an RSA private key (`RS256`).
#[derive(Clone)]
pub struct JwtSigner {
    encoding_key: Arc<EncodingKey>,
    kid: String,
    issuer: String,
    audience: String,
    ttl: std::time::Duration,
}

impl JwtSigner {
    /// Load a PKCS#1 or PKCS#8 PEM private key and configure issuer, audience, and TTL.
    pub fn from_pem(
        pem: &str,
        issuer: impl Into<String>,
        audience: impl Into<String>,
        ttl: std::time::Duration,
    ) -> Result<Self, JwtError> {
        let key = RsaPrivateKey::from_pkcs1_pem(pem)
            .or_else(|_| RsaPrivateKey::from_pkcs8_pem(pem))
            .map_err(|e| JwtError::Pem(e.to_string()))?;
        // `jsonwebtoken::EncodingKey` expects PKCS#1 RSA PEM for signing.
        let pem_out = key
            .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .map_err(|e| JwtError::Pem(e.to_string()))?;
        let encoding_key = Arc::new(
            EncodingKey::from_rsa_pem(pem_out.as_bytes())
                .map_err(|e| JwtError::Pem(e.to_string()))?,
        );
        Ok(Self {
            encoding_key,
            kid: KID.into(),
            issuer: issuer.into(),
            audience: audience.into(),
            ttl,
        })
    }

    /// Mint a signed JWT with `sub` set to `user_id` and `exp` derived from the configured TTL.
    pub fn sign_user(&self, user_id: &uuid::Uuid) -> Result<String, JwtError> {
        let now = chrono::Utc::now().timestamp();
        let exp = now + self.ttl.as_secs() as i64;
        let claims = AccessClaims {
            sub: user_id.to_string(),
            exp,
            iat: now,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.kid.clone());
        jsonwebtoken::encode(&header, &claims, self.encoding_key.as_ref()).map_err(Into::into)
    }

    /// Serialize a JWKS JSON document for the public half of `pem` (must match the signing key).
    pub fn jwks_json(&self, pem: &str) -> Result<String, JwtError> {
        let key = RsaPrivateKey::from_pkcs1_pem(pem)
            .or_else(|_| RsaPrivateKey::from_pkcs8_pem(pem))
            .map_err(|e| JwtError::Pem(e.to_string()))?;
        let pub_key = key.to_public_key();
        let n = pub_key.n().to_bytes_be();
        let e = pub_key.e().to_bytes_be();
        let jwk = Jwk {
            kty: "RSA".into(),
            kid: self.kid.clone(),
            n: URL_SAFE_NO_PAD.encode(n),
            e: URL_SAFE_NO_PAD.encode(e),
            alg: "RS256".into(),
            use_: "sig".into(),
        };
        let jwks = Jwks { keys: vec![jwk] };
        serde_json::to_string(&jwks).map_err(|e| JwtError::Pem(e.to_string()))
    }
}

#[derive(Serialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

/// One RSA JWK entry as used in [`JwksDoc`].
#[derive(Serialize, Deserialize, Clone)]
pub struct Jwk {
    pub kty: String,
    pub kid: String,
    pub n: String,
    pub e: String,
    pub alg: String,
    #[serde(rename = "use")]
    pub use_: String,
}

/// Parsed `/.well-known/jwks.json` body.
#[derive(Clone, Deserialize)]
pub struct JwksDoc {
    pub keys: Vec<Jwk>,
}

/// Validates RS256 JWTs against a JWKS document and fixed `iss` / `aud`.
#[derive(Clone)]
pub struct JwtValidator {
    issuer: String,
    audience: String,
}

impl JwtValidator {
    /// Expected `iss` and `aud` claim values.
    pub fn new(issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            audience: audience.into(),
        }
    }

    /// Decode and validate `token` using the JWK whose `kid` matches the JWT header.
    pub fn validate(&self, token: &str, jwks: &JwksDoc) -> Result<AccessClaims, JwtError> {
        let header = jsonwebtoken::decode_header(token)?;
        let kid = header.kid.ok_or(JwtError::InvalidJwk)?;
        let jwk = jwks
            .keys
            .iter()
            .find(|k| k.kid == kid)
            .ok_or_else(|| JwtError::KidNotFound(kid.clone()))?;
        let decoding_key = jwk_to_decoding_key(jwk)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        let data = jsonwebtoken::decode::<AccessClaims>(token, &decoding_key, &validation)?;
        Ok(data.claims)
    }
}

fn jwk_to_decoding_key(jwk: &Jwk) -> Result<DecodingKey, JwtError> {
    // Build an RSA public key from URL-safe base64 `n` and `e`, then PEM-encode for jsonwebtoken.
    let n = URL_SAFE_NO_PAD
        .decode(jwk.n.as_bytes())
        .map_err(|_| JwtError::InvalidJwk)?;
    let e = URL_SAFE_NO_PAD
        .decode(jwk.e.as_bytes())
        .map_err(|_| JwtError::InvalidJwk)?;
    let rsa_pub = rsa::RsaPublicKey::new(
        rsa::BigUint::from_bytes_be(&n),
        rsa::BigUint::from_bytes_be(&e),
    )?;
    let pem = rsa_pub
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .map_err(|e| JwtError::Pem(e.to_string()))?;
    DecodingKey::from_rsa_pem(pem.as_bytes()).map_err(Into::into)
}
