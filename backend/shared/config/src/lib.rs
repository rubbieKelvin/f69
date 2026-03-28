//! Service configuration from the environment.
//!
//! Variables use the `AUTH_` or `FLAG_` prefix (see [`AuthServiceConfig::from_env`],
//! [`FlagServiceConfig::from_env`]). Nested keys can use `__`, e.g. `AUTH__FOO__BAR`.
//! Docker and shells pass booleans and numbers as strings; deserializers in `serde_env`
//! accept both native JSON values and string forms.

use figment::{providers::Env, Figment};
use serde::Deserialize;
use std::fmt;
use std::net::SocketAddr;

/// Serde helpers so Figment/env strings deserialize into `bool`, `u64`, and `u16`.
mod serde_env {
    use super::*;
    use serde::de::{self, Deserializer, Visitor};

    pub fn bool_from_env<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct B;
        impl<'de> Visitor<'de> for B {
            type Value = bool;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a boolean or string true/false")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<bool, E> {
                Ok(v)
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<bool, E> {
                match v.trim().to_lowercase().as_str() {
                    "true" | "1" | "yes" => Ok(true),
                    "false" | "0" | "no" => Ok(false),
                    _ => Err(E::custom("invalid boolean env value")),
                }
            }
        }
        deserializer.deserialize_any(B)
    }

    pub fn u64_from_env<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct U;
        impl<'de> Visitor<'de> for U {
            type Value = u64;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a u64 integer or decimal string")
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<u64, E> {
                Ok(v)
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<u64, E> {
                Ok(v as u64)
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<u64, E> {
                v.trim()
                    .parse()
                    .map_err(|_| E::custom("invalid u64 env value"))
            }
        }
        deserializer.deserialize_any(U)
    }

    pub fn u16_from_env<'de, D>(deserializer: D) -> Result<u16, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct U;
        impl<'de> Visitor<'de> for U {
            type Value = u16;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a u16 integer or decimal string")
            }
            fn visit_u16<E: de::Error>(self, v: u16) -> Result<u16, E> {
                Ok(v)
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<u16, E> {
                u16::try_from(v).map_err(|_| E::custom("port out of range"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<u16, E> {
                v.trim()
                    .parse()
                    .map_err(|_| E::custom("invalid u16 env value"))
            }
        }
        deserializer.deserialize_any(U)
    }
}

/// Auth HTTP service: bind address, database, JWT signing, and runtime toggles.
#[derive(Debug, Clone, Deserialize)]
pub struct AuthServiceConfig {
    /// Listen address (typically `0.0.0.0` in containers).
    #[serde(default = "default_bind_host")]
    pub host: String,
    #[serde(default = "default_auth_port", deserialize_with = "serde_env::u16_from_env")]
    pub port: u16,
    /// PostgreSQL URL for the auth database only.
    #[serde(default)]
    pub database_url: String,
    /// RSA private key PEM (inline or path via `jwt_private_key_path`)
    pub jwt_private_key_pem: Option<String>,
    /// Filesystem path to an RSA private key PEM (alternative to `jwt_private_key_pem`).
    pub jwt_private_key_path: Option<String>,
    /// `iss` claim on issued JWTs; the flag service must expect the same value.
    #[serde(default = "default_jwt_issuer")]
    pub jwt_issuer: String,
    /// `aud` claim on issued JWTs.
    #[serde(default = "default_jwt_audience")]
    pub jwt_audience: String,
    /// Access token lifetime in seconds
    #[serde(
        default = "default_access_ttl",
        deserialize_with = "serde_env::u64_from_env"
    )]
    pub jwt_access_ttl_secs: u64,
    /// When `production`, JSON logs and stricter CORS defaults apply.
    #[serde(default)]
    pub environment: Environment,
    /// Comma-separated allowed `Origin` headers; unset in dev allows any origin.
    pub cors_allowed_origins: Option<String>,
    /// Run embedded SeaORM migrations before accepting traffic.
    #[serde(
        default = "default_run_migrations",
        deserialize_with = "serde_env::bool_from_env"
    )]
    pub run_migrations_on_startup: bool,
}

fn default_bind_host() -> String {
    "0.0.0.0".into()
}

fn default_auth_port() -> u16 {
    3000
}

fn default_jwt_issuer() -> String {
    "f69-auth".into()
}

fn default_jwt_audience() -> String {
    "f69-api".into()
}

fn default_access_ttl() -> u64 {
    3600
}

fn default_run_migrations() -> bool {
    true
}

/// Feature-flag HTTP service: bind address, flags database, and auth service URL for JWKS.
#[derive(Debug, Clone, Deserialize)]
pub struct FlagServiceConfig {
    /// Listen address (typically `0.0.0.0` in containers).
    #[serde(default = "default_bind_host")]
    pub host: String,
    #[serde(default = "default_flag_port", deserialize_with = "serde_env::u16_from_env")]
    pub port: u16,
    /// PostgreSQL URL for the flags database only.
    #[serde(default)]
    pub database_url: String,
    /// Base URL of the auth service (e.g. `http://auth-service:3000`) used to fetch JWKS.
    #[serde(default)]
    pub auth_base_url: String,
    /// Expected JWT `iss`; must match what the auth service puts on tokens.
    #[serde(default = "default_jwt_issuer")]
    pub jwt_issuer: String,
    /// Expected JWT `aud`.
    #[serde(default = "default_jwt_audience")]
    pub jwt_audience: String,
    /// When `production`, JSON logs and stricter CORS defaults apply.
    #[serde(default)]
    pub environment: Environment,
    /// Comma-separated allowed `Origin` headers; unset in dev allows any origin.
    pub cors_allowed_origins: Option<String>,
    /// Run embedded SeaORM migrations before accepting traffic.
    #[serde(
        default = "default_run_migrations",
        deserialize_with = "serde_env::bool_from_env"
    )]
    pub run_migrations_on_startup: bool,
}

fn default_flag_port() -> u16 {
    3001
}

/// Log format and some default security-related behavior.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    /// Whether this deployment should use production-oriented defaults.
    pub fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

/// Failure loading or validating configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("configuration error: {0}")]
    Figment(#[from] figment::Error),
    #[error("AUTH: {0}")]
    Auth(&'static str),
    #[error("FLAG: {0}")]
    Flag(&'static str),
}

fn auth_figment() -> Figment {
    Figment::from(Env::prefixed("AUTH_").split("__"))
}

fn flag_figment() -> Figment {
    Figment::from(Env::prefixed("FLAG_").split("__"))
}

impl AuthServiceConfig {
    /// Load from `AUTH_*` environment variables (via Figment).
    pub fn from_env() -> Result<Self, ConfigError> {
        let c: Self = auth_figment().extract()?;
        if c.jwt_private_key_pem.is_none() && c.jwt_private_key_path.is_none() {
            return Err(ConfigError::Auth(
                "set AUTH_JWT_PRIVATE_KEY_PEM or AUTH_JWT_PRIVATE_KEY_PATH",
            ));
        }
        if c.database_url.is_empty() {
            return Err(ConfigError::Auth("AUTH_DATABASE_URL is required"));
        }
        Ok(c)
    }

    /// TCP listen address for the HTTP server.
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("invalid AUTH_HOST or AUTH_PORT")
    }
}

impl FlagServiceConfig {
    /// Load from `FLAG_*` environment variables (via Figment).
    pub fn from_env() -> Result<Self, ConfigError> {
        let c: Self = flag_figment().extract()?;
        if c.database_url.is_empty() {
            return Err(ConfigError::Flag("FLAG_DATABASE_URL is required"));
        }
        if c.auth_base_url.is_empty() {
            return Err(ConfigError::Flag("FLAG_AUTH_BASE_URL is required"));
        }
        Ok(c)
    }

    /// TCP listen address for the HTTP server.
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("invalid FLAG_HOST or FLAG_PORT")
    }
}
