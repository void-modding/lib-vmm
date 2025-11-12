use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("network: {0}")]
    Network(String),
    #[error("parse json: {0}")]
    Parse(String),
    #[error("schema mismatch: {0}")]
    Schema(String),
    #[error("internal error: {0}")]
    Internal(String)
}

#[async_trait]
pub trait ProviderHttpClient: Send + Sync {
    async fn get_json(&self, url: &str) -> Result<Value, HttpError>;
}

/// Extension trait providing typed deserialization
/// Its like this so ProviderHttpClient is dyn compatible
#[async_trait]
pub trait ProviderHttpClientTypedExt {
    async fn get_typed<T: DeserializeOwned>(&self, url: &str) -> Result<T, HttpError>;
}

#[async_trait]
impl<C: ProviderHttpClient + ?Sized> ProviderHttpClientTypedExt for C {
    async fn get_typed<T: DeserializeOwned>(&self, url: &str) -> Result<T, HttpError> {
        let v = self.get_json(url).await?;
        serde_json::from_value(v).map_err(|e| HttpError::Parse(e.to_string()))
    }
}



/// This should also be behind the defualt implementation flag
pub struct ReqwestProviderHttpClient {
    client: reqwest::Client,
}

impl ReqwestProviderHttpClient {
    pub fn new () -> Arc<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("client");
        Arc::new(Self { client })
    }
}

#[async_trait]
impl ProviderHttpClient for ReqwestProviderHttpClient {
    async fn get_json(&self, url: &str) -> Result<Value, HttpError> {
        let resp = self.client
            .get(url)
            .header(USER_AGENT, "VoidModManager/0.1.0 (+https://github.com/void-mod-manager/app)")
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(|e| HttpError::Network(e.to_string()))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| HttpError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(HttpError::Network(format!("status {} | body = {}", status, text)));
        }

        serde_json::from_str(&text).map_err(|e| HttpError::Parse(e.to_string()))
    }
}
