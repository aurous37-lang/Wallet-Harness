//! chain-rpc — multi-provider EVM RPC client with failover.
//!
//! The registry owns a set of providers per chain, ordered by user priority.
//! Every RPC call tries providers in order, failing over on transport errors
//! and rate-limit responses. Individual providers are treated as replaceable
//! — Alchemy, Infura, QuickNode, public endpoints, or the user's own node
//! all satisfy the same trait.
//!
//! This crate deliberately does *not* own chain metadata like DEX pool
//! addresses or token registries. Those belong to `indexer` and higher
//! layers. This crate is strictly the transport.

use std::collections::HashMap;
use std::sync::Arc;

use alloy_primitives::{Address, Bytes, B256, U256};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tokio::sync::RwLock;

pub mod chains;
pub mod provider;

pub use chains::{ChainId, ChainMeta};
pub use provider::{HttpProvider, RpcProvider};

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("no providers configured for chain {0}")]
    NoProviders(u64),
    #[error("all providers failed for chain {0}")]
    AllProvidersFailed(u64),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("rpc error {code}: {message}")]
    Rpc { code: i64, message: String },
    #[error("decode error: {0}")]
    Decode(String),
}

pub type RpcResult<T> = Result<T, RpcError>;

/// Registry of RPC providers, keyed by chain id.
///
/// Cheap to clone (internally arc'd). Safe to share across async tasks.
#[derive(Clone, Default)]
pub struct RpcRegistry {
    inner: Arc<RwLock<HashMap<u64, Vec<Arc<dyn RpcProvider>>>>>,
}

impl RpcRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a provider for a chain. Providers are tried in insertion order
    /// within their priority tier.
    pub async fn register(&self, chain_id: u64, provider: Arc<dyn RpcProvider>) {
        let mut guard = self.inner.write().await;
        guard.entry(chain_id).or_default().push(provider);
    }

    pub async fn providers_for(&self, chain_id: u64) -> RpcResult<Vec<Arc<dyn RpcProvider>>> {
        let guard = self.inner.read().await;
        let list = guard
            .get(&chain_id)
            .ok_or(RpcError::NoProviders(chain_id))?
            .clone();
        if list.is_empty() {
            return Err(RpcError::NoProviders(chain_id));
        }
        Ok(list)
    }

    /// Execute a closure against each provider in priority order until one
    /// succeeds or all fail. Transport / decode / server errors advance to the
    /// next provider; a well-formed `eth_*` error short-circuits and is
    /// returned immediately (the node answered, the request was the problem).
    pub async fn failover<F, Fut, T>(&self, chain_id: u64, mut f: F) -> RpcResult<T>
    where
        F: FnMut(Arc<dyn RpcProvider>) -> Fut,
        Fut: std::future::Future<Output = RpcResult<T>>,
    {
        let providers = self.providers_for(chain_id).await?;
        let mut last_err: Option<RpcError> = None;
        for p in providers {
            let label = p.label().to_string();
            match f(p).await {
                Ok(v) => return Ok(v),
                Err(e @ RpcError::Rpc { .. }) => return Err(e),
                Err(e) => {
                    tracing::warn!(chain_id, provider = %label, error = %e, "provider failed, failing over");
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or(RpcError::AllProvidersFailed(chain_id)))
    }

    // ---- high-level reads ---------------------------------------------------

    /// `eth_chainId`
    pub async fn chain_id(&self, chain: ChainId) -> RpcResult<u64> {
        let cid = chain.as_u64();
        self.failover(cid, |p| async move {
            let v = p.raw_request("eth_chainId", json!([])).await?;
            parse_hex_u64(&v)
        })
        .await
    }

    /// `eth_blockNumber`
    pub async fn block_number(&self, chain: ChainId) -> RpcResult<u64> {
        let cid = chain.as_u64();
        self.failover(cid, |p| async move {
            let v = p.raw_request("eth_blockNumber", json!([])).await?;
            parse_hex_u64(&v)
        })
        .await
    }

    /// `eth_getBalance` at "latest".
    pub async fn native_balance(&self, chain: ChainId, address: Address) -> RpcResult<U256> {
        let cid = chain.as_u64();
        let addr_hex = format!("{address:#x}");
        self.failover(cid, |p| {
            let addr_hex = addr_hex.clone();
            async move {
                let v = p
                    .raw_request("eth_getBalance", json!([addr_hex, "latest"]))
                    .await?;
                parse_hex_u256(&v)
            }
        })
        .await
    }

    /// `eth_call` to `balanceOf(address)` at "latest".
    pub async fn erc20_balance(
        &self,
        chain: ChainId,
        token: Address,
        holder: Address,
    ) -> RpcResult<U256> {
        let cid = chain.as_u64();
        // selector = keccak256("balanceOf(address)")[0..4] = 0x70a08231
        let mut data = Vec::with_capacity(4 + 32);
        data.extend_from_slice(&[0x70, 0xa0, 0x82, 0x31]);
        data.extend_from_slice(&[0u8; 12]);
        data.extend_from_slice(holder.as_slice());
        let calldata = format!("0x{}", hex_encode(&data));
        let to_hex = format!("{token:#x}");

        self.failover(cid, |p| {
            let calldata = calldata.clone();
            let to_hex = to_hex.clone();
            async move {
                let v = p
                    .raw_request(
                        "eth_call",
                        json!([{ "to": to_hex, "data": calldata }, "latest"]),
                    )
                    .await?;
                let s = v
                    .as_str()
                    .ok_or_else(|| RpcError::Decode("eth_call result not a string".into()))?;
                let bytes = hex_decode_0x(s)?;
                if bytes.len() < 32 {
                    return Err(RpcError::Decode(format!(
                        "balanceOf returned {} bytes",
                        bytes.len()
                    )));
                }
                Ok(U256::from_be_slice(&bytes[bytes.len() - 32..]))
            }
        })
        .await
    }

    /// `eth_getLogs`
    pub async fn get_logs(&self, chain: ChainId, filter: LogFilter) -> RpcResult<Vec<RpcLog>> {
        let cid = chain.as_u64();
        let filter_json = filter.to_rpc_json();
        self.failover(cid, |p| {
            let filter_json = filter_json.clone();
            async move {
                let v = p.raw_request("eth_getLogs", json!([filter_json])).await?;
                serde_json::from_value::<Vec<RpcLogRaw>>(v)
                    .map_err(|e| RpcError::Decode(e.to_string()))
                    .and_then(|raw| raw.into_iter().map(RpcLog::try_from_raw).collect())
            }
        })
        .await
    }

    /// `eth_sendRawTransaction`
    pub async fn send_raw_transaction(&self, chain: ChainId, raw: Bytes) -> RpcResult<B256> {
        let cid = chain.as_u64();
        let hex_tx = format!("0x{}", hex_encode(raw.as_ref()));
        self.failover(cid, |p| {
            let hex_tx = hex_tx.clone();
            async move {
                let v = p
                    .raw_request("eth_sendRawTransaction", json!([hex_tx]))
                    .await?;
                let s = v.as_str().ok_or_else(|| {
                    RpcError::Decode("sendRawTransaction result not a string".into())
                })?;
                let bytes = hex_decode_0x(s)?;
                if bytes.len() != 32 {
                    return Err(RpcError::Decode(format!(
                        "expected 32-byte tx hash, got {}",
                        bytes.len()
                    )));
                }
                Ok(B256::from_slice(&bytes))
            }
        })
        .await
    }
}

/// Minimal top-level RPC operations the rest of the app uses.
///
/// Concrete RPC JSON encoding lives in `provider`. This trait is the narrow
/// surface that callers depend on.
#[async_trait]
pub trait RpcClient: Send + Sync {
    async fn chain_id(&self) -> RpcResult<u64>;
    async fn block_number(&self) -> RpcResult<u64>;
    async fn native_balance(&self, address: &str) -> RpcResult<String>; // u256 as decimal string
    async fn get_logs(&self, filter: LogFilter) -> RpcResult<Vec<RpcLog>>;
    async fn send_raw_transaction(&self, signed_tx_hex: &str) -> RpcResult<String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    pub from_block: u64,
    pub to_block: u64,
    pub address: Option<String>,
    pub topics: Vec<Option<String>>,
}

impl LogFilter {
    fn to_rpc_json(&self) -> serde_json::Value {
        let topics: Vec<serde_json::Value> = self
            .topics
            .iter()
            .map(|t| match t {
                Some(s) => serde_json::Value::String(s.clone()),
                None => serde_json::Value::Null,
            })
            .collect();
        let mut obj = serde_json::Map::new();
        obj.insert(
            "fromBlock".into(),
            serde_json::Value::String(format!("0x{:x}", self.from_block)),
        );
        obj.insert(
            "toBlock".into(),
            serde_json::Value::String(format!("0x{:x}", self.to_block)),
        );
        if let Some(a) = &self.address {
            obj.insert("address".into(), serde_json::Value::String(a.clone()));
        }
        obj.insert("topics".into(), serde_json::Value::Array(topics));
        serde_json::Value::Object(obj)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub block_number: u64,
    pub tx_hash: String,
    pub log_index: u32,
}

#[derive(Debug, Deserialize)]
struct RpcLogRaw {
    address: String,
    #[serde(default)]
    topics: Vec<String>,
    #[serde(default)]
    data: String,
    #[serde(rename = "blockNumber", default)]
    block_number: Option<String>,
    #[serde(rename = "transactionHash", default)]
    transaction_hash: Option<String>,
    #[serde(rename = "logIndex", default)]
    log_index: Option<String>,
}

impl RpcLog {
    fn try_from_raw(raw: RpcLogRaw) -> RpcResult<Self> {
        let block_number = raw
            .block_number
            .as_deref()
            .map(parse_hex_u64_str)
            .transpose()?
            .unwrap_or(0);
        let log_index = raw
            .log_index
            .as_deref()
            .map(parse_hex_u64_str)
            .transpose()?
            .unwrap_or(0) as u32;
        Ok(Self {
            address: raw.address,
            topics: raw.topics,
            data: raw.data,
            block_number,
            tx_hash: raw.transaction_hash.unwrap_or_default(),
            log_index,
        })
    }
}

// ---- hex helpers -----------------------------------------------------------

fn parse_hex_u64(v: &serde_json::Value) -> RpcResult<u64> {
    let s = v
        .as_str()
        .ok_or_else(|| RpcError::Decode("expected hex string".into()))?;
    parse_hex_u64_str(s)
}

fn parse_hex_u64_str(s: &str) -> RpcResult<u64> {
    let trimmed = s.trim_start_matches("0x");
    if trimmed.is_empty() {
        return Ok(0);
    }
    u64::from_str_radix(trimmed, 16).map_err(|e| RpcError::Decode(e.to_string()))
}

fn parse_hex_u256(v: &serde_json::Value) -> RpcResult<U256> {
    let s = v
        .as_str()
        .ok_or_else(|| RpcError::Decode("expected hex string".into()))?;
    let trimmed = s.trim_start_matches("0x");
    U256::from_str_radix(trimmed, 16).map_err(|e| RpcError::Decode(e.to_string()))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode_0x(s: &str) -> RpcResult<Vec<u8>> {
    let trimmed = s.trim_start_matches("0x");
    if trimmed.len() % 2 != 0 {
        return Err(RpcError::Decode("odd-length hex".into()));
    }
    let mut out = Vec::with_capacity(trimmed.len() / 2);
    let bytes = trimmed.as_bytes();
    let from_nib = |n: u8| -> RpcResult<u8> {
        match n {
            b'0'..=b'9' => Ok(n - b'0'),
            b'a'..=b'f' => Ok(n - b'a' + 10),
            b'A'..=b'F' => Ok(n - b'A' + 10),
            _ => Err(RpcError::Decode("invalid hex char".into())),
        }
    };
    for pair in bytes.chunks_exact(2) {
        out.push((from_nib(pair[0])? << 4) | from_nib(pair[1])?);
    }
    Ok(out)
}
