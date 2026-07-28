//! dex-router — swap quoting and unsigned-tx construction.
//!
//! The router abstracts away which DEX actually fills the swap. Three
//! strategies, tried in order:
//!
//! 1. **Aggregator API** (1inch, Paraswap, 0x) — best route across many
//!    DEXes. Requires a user-supplied API key (1inch Pro). Returns ready-
//!    to-sign calldata.
//! 2. **Direct pool routing** (Uniswap V2/V3 math in-process) — used when
//!    no aggregator supports the chain (e.g. Monad today) or when the user
//!    turns off all aggregators for privacy.
//! 3. **On-chain limit order protocols** (1inch Limit Order, CowSwap) — for
//!    limit orders specifically, returns a signed-off-chain order intent
//!    that the protocol fills without the app needing to stay online.
//!
//! The router never submits transactions. It only constructs quotes and
//! unsigned calldata. Submission is the caller's responsibility, via a
//! signer and chain-rpc.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("no route found")]
    NoRoute,
    #[error("slippage would exceed {0} bps")]
    SlippageExceeded(u32),
    #[error("unsupported chain: {0}")]
    UnsupportedChain(u64),
    #[error("aggregator error: {0}")]
    Aggregator(String),
    #[error("rpc error: {0}")]
    Rpc(#[from] chain_rpc::RpcError),
}

pub type RouterResult<T> = Result<T, RouterError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub router: String, // "1inch" | "paraswap" | "0x" | "direct:univ2" | "direct:univ3"
    pub chain_id: u64,
    pub token_in: String,
    pub token_out: String,
    pub amount_in_raw: String,
    pub amount_out_raw: String,
    pub price_impact_bps: u32,
    pub estimated_gas: u64,
    pub route_summary: String,
    /// Opaque intent id — pass back to `build_unsigned` to produce the tx.
    pub intent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedSwapTx {
    pub chain_id: u64,
    pub to: String,
    pub value: String,
    pub data: String,
    pub gas_limit: u64,
}

#[async_trait]
pub trait SwapRouter: Send + Sync {
    fn label(&self) -> &str;
    fn supports_chain(&self, chain_id: u64) -> bool;
    async fn quote(
        &self,
        chain_id: u64,
        token_in: &str,
        token_out: &str,
        amount_in_raw: &str,
        slippage_bps: u32,
    ) -> RouterResult<SwapQuote>;
    async fn build_unsigned(&self, intent_id: &str, from: &str) -> RouterResult<UnsignedSwapTx>;
}

/// Router that tries a list of sub-routers in order and returns the best
/// quote (highest `amount_out_raw`). This is the top-level entry the command
/// layer uses.
pub struct BestOfRouters {
    pub routers: Vec<std::sync::Arc<dyn SwapRouter>>,
}

impl BestOfRouters {
    pub fn new(routers: Vec<std::sync::Arc<dyn SwapRouter>>) -> Self {
        Self { routers }
    }

    pub async fn best_quote(
        &self,
        chain_id: u64,
        token_in: &str,
        token_out: &str,
        amount_in_raw: &str,
        slippage_bps: u32,
    ) -> RouterResult<SwapQuote> {
        let mut best: Option<SwapQuote> = None;
        for r in &self.routers {
            if !r.supports_chain(chain_id) {
                continue;
            }
            match r
                .quote(chain_id, token_in, token_out, amount_in_raw, slippage_bps)
                .await
            {
                Ok(q) => {
                    let take = match &best {
                        None => true,
                        Some(b) => {
                            q.amount_out_raw.parse::<u128>().unwrap_or(0)
                                > b.amount_out_raw.parse::<u128>().unwrap_or(0)
                        }
                    };
                    if take {
                        best = Some(q);
                    }
                }
                Err(e) => tracing::debug!(router = r.label(), error = %e, "router quote failed"),
            }
        }
        best.ok_or(RouterError::NoRoute)
    }
}
