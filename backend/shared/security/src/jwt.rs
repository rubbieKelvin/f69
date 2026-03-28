use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::{DecodePrivateKey, EncodePublicKey};
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const KID: &str = "f69-key-1";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub aud: String,
}

#[derive(Clone)]
pub struct JwtSigner {
    encoding_key: Arc<EncodingKey>,
    kid: String,
    issuer: String,
    audience: String,
    ttl: std::time::Duration,
}

impl JwtSigner {
    pub fn from_pem(
        pem: &str,
        issuer: impl Into<String>,
        audience: impl Into<String>,
        ttl: std::time::Duration,
    ) -> Result<Self, JwtError> {
        let key = RsaPrivateKey::from_pkcs1_pem(pem)
            .or_else(|_| RsaPrivateKey::from_pkcs8_pem(pem))
            .map_err(|e| JwtError::Pem(e.to_string()))?;
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

    /// Build JWKS document for this signer (single key).
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

#[derive(Clone, Deserialize)]
pub struct JwksDoc {
    pub keys: Vec<Jwk>,
}

#[derive(Clone)]
pub struct JwtValidator {
    issuer: String,
    audience: String,
}

impl JwtValidator {
    pub fn new(issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            audience: audience.into(),
        }
    }

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
