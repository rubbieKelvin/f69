//! In-memory JWKS cache with TTL to avoid hitting the auth service on every request.

use std::sync::Arc;
use std::time::{Duration, Instant};

use shared_security::JwksDoc;
use tokio::sync::RwLock;

type CachedJwks = Option<(Instant, JwksDoc)>;

/// Fetches `GET {auth}/.well-known/jwks.json` and caches the parsed document.
pub struct JwksCache {
    client: reqwest::Client,
    jwks_url: String,
    inner: RwLock<CachedJwks>,
    ttl: Duration,
}

impl JwksCache {
    /// `ttl` controls how long cached keys are reused before a refresh.
    pub fn new(auth_base_url: impl Into<String>, ttl: Duration) -> Arc<Self> {
        let base = auth_base_url.into().trim_end_matches('/').to_string();
        let jwks_url = format!("{}/.well-known/jwks.json", base);
        Arc::new(Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            jwks_url,
            inner: RwLock::new(None),
            ttl,
        })
    }

    /// Returns cached JWKS if fresh; otherwise downloads and stores a new copy.
    pub async fn get(&self) -> Result<JwksDoc, JwksFetchError> {
        {
            let guard = self.inner.read().await;
            if let Some((at, doc)) = guard.as_ref() {
                if at.elapsed() < self.ttl {
                    return Ok(doc.clone());
                }
            }
        }

        let doc = self.fetch().await?;
        *self.inner.write().await = Some((Instant::now(), doc.clone()));
        Ok(doc)
    }

    async fn fetch(&self) -> Result<JwksDoc, JwksFetchError> {
        let res = self
            .client
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(JwksFetchError::Http)?;
        if !res.status().is_success() {
            return Err(JwksFetchError::Status(res.status().as_u16()));
        }
        let doc: JwksDoc = res.json().await.map_err(JwksFetchError::Http)?;
        Ok(doc)
    }
}

/// Network or HTTP failures while loading JWKS.
#[derive(Debug, thiserror::Error)]
pub enum JwksFetchError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("jwks status {0}")]
    Status(u16),
}
