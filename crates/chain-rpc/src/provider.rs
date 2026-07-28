//! RPC provider abstraction.
//!
//! The `RpcProvider` trait is what the registry holds. One provider = one
//! endpoint. Failover between providers happens at the registry level, not
//! inside a single provider.

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{RpcError, RpcResult};

#[async_trait]
pub trait RpcProvider: Send + Sync {
    fn label(&self) -> &str;
    fn endpoint(&self) -> &str;
    async fn raw_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> RpcResult<serde_json::Value>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpProviderConfig {
    pub label: String,
    pub endpoint: String,
    pub api_key_header: Option<String>, // e.g. "X-API-Key"
    pub timeout_ms: u64,
}

pub struct HttpProvider {
    config: HttpProviderConfig,
    client: reqwest::Client,
}

impl HttpProvider {
    pub fn new(config: HttpProviderConfig) -> Self {
        let timeout = if config.timeout_ms == 0 {
            Duration::from_secs(10)
        } else {
            Duration::from_millis(config.timeout_ms)
        };
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { config, client }
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    #[serde(default)]
    jsonrpc: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    id: Option<serde_json::Value>,
    #[serde(default)]
    result: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<JsonRpcErrorObj>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcErrorObj {
    code: i64,
    message: String,
}

#[async_trait]
impl RpcProvider for HttpProvider {
    fn label(&self) -> &str {
        &self.config.label
    }

    fn endpoint(&self) -> &str {
        &self.config.endpoint
    }

    async fn raw_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> RpcResult<serde_json::Value> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?;

        let status = resp.status();
        if status.is_server_error() {
            return Err(RpcError::Transport(format!("http {}", status.as_u16())));
        }

        let parsed: JsonRpcResponse = resp
            .json()
            .await
            .map_err(|e| RpcError::Decode(e.to_string()))?;

        if let Some(err) = parsed.error {
            return Err(RpcError::Rpc {
                code: err.code,
                message: err.message,
            });
        }

        parsed
            .result
            .ok_or_else(|| RpcError::Decode("missing result in json-rpc response".into()))
    }
}
