//! Typed configuration loaded from environment (and optional `.env` via dotenvy in binaries).

use figment::{providers::Env, Figment};
use serde::Deserialize;
use std::fmt;
use std::net::SocketAddr;

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

#[derive(Debug, Clone, Deserialize)]
pub struct AuthServiceConfig {
    #[serde(default = "default_bind_host")]
    pub host: String,
    #[serde(default = "default_auth_port", deserialize_with = "serde_env::u16_from_env")]
    pub port: u16,
    #[serde(default)]
    pub database_url: String,
    /// RSA private key PEM (inline or path via `jwt_private_key_path`)
    pub jwt_private_key_pem: Option<String>,
    pub jwt_private_key_path: Option<String>,
    #[serde(default = "default_jwt_issuer")]
    pub jwt_issuer: String,
    #[serde(default = "default_jwt_audience")]
    pub jwt_audience: String,
    /// Access token lifetime in seconds
    #[serde(
        default = "default_access_ttl",
        deserialize_with = "serde_env::u64_from_env"
    )]
    pub jwt_access_ttl_secs: u64,
    #[serde(default)]
    pub environment: Environment,
    pub cors_allowed_origins: Option<String>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct FlagServiceConfig {
    #[serde(default = "default_bind_host")]
    pub host: String,
    #[serde(default = "default_flag_port", deserialize_with = "serde_env::u16_from_env")]
    pub port: u16,
    #[serde(default)]
    pub database_url: String,
    /// Base URL of auth service (e.g. http://auth-service:3000) for JWKS
    #[serde(default)]
    pub auth_base_url: String,
    #[serde(default = "default_jwt_issuer")]
    pub jwt_issuer: String,
    #[serde(default = "default_jwt_audience")]
    pub jwt_audience: String,
    #[serde(default)]
    pub environment: Environment,
    pub cors_allowed_origins: Option<String>,
    #[serde(
        default = "default_run_migrations",
        deserialize_with = "serde_env::bool_from_env"
    )]
    pub run_migrations_on_startup: bool,
}

fn default_flag_port() -> u16 {
    3001
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

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

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("invalid AUTH_HOST or AUTH_PORT")
    }
}

impl FlagServiceConfig {
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

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("invalid FLAG_HOST or FLAG_PORT")
    }
}
