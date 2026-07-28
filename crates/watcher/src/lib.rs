//! watcher — the armed-alert engine.
//!
//! v1 behavior: polls price feeds on a tick, evaluates trigger conditions,
//! moves orders from `Armed → Triggered → AwaitingSignature`, and emits a
//! Tauri event so the UI can surface the "ready to sign" queue. The user
//! then approves through their chosen signer.
//!
//! The state machine is crash-safe. On startup the watcher reconciles
//! `Submitted` orders against chain state (receipt lookup) and transitions
//! them to `Confirmed` or `Failed`. Orders in `AwaitingSignature` stay in
//! that state across restarts — the UI just shows them again.
//!
//! Post-73%-gate v2: orders with `autonomous_allowed: true` attached to a
//! `HotEncryptedKeystore` signer skip `AwaitingSignature` and sign+submit
//! inline.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatcherError {
    #[error("no price feed available for {0}")]
    NoPriceFeed(String),
    #[error("signer error: {0}")]
    Signer(#[from] signer::SignerError),
    #[error("rpc error: {0}")]
    Rpc(#[from] chain_rpc::RpcError),
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
}

pub type WatcherResult<T> = Result<T, WatcherError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderState {
    Armed,
    Triggered,
    AwaitingSignature,
    Submitted,
    Confirmed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderKind {
    StopLoss,
    LimitBuy,
    LimitSell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmedOrder {
    pub id: String,
    pub chain_id: u64,
    pub kind: OrderKind,
    pub token_in: String,
    pub token_out: String,
    pub size_token: f64,
    pub trigger_price_usd: f64,
    pub slippage_bps: u32,
    /// v2-only. Ignored in v1.
    pub autonomous_allowed: bool,
    pub state: OrderState,
    pub created_at: i64,
}

/// Price feed consensus config — require agreement from N of M sources
/// before triggering. Guards against single-source flash-crashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub sources_required: u8,
    pub max_dispersion_bps: u32,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            sources_required: 2,
            max_dispersion_bps: 150,
        } // 1.5% dispersion max
    }
}

pub struct Watcher {
    pub tick_interval_ms: u64,
    pub consensus: ConsensusConfig,
    // TODO: SqlitePool, RpcRegistry, SignerRegistry handles, event emitter
}

impl Watcher {
    pub fn new() -> Self {
        Self {
            tick_interval_ms: 2000,
            consensus: ConsensusConfig::default(),
        }
    }

    /// Main loop — polls, evaluates triggers, transitions states.
    pub async fn run(&self) -> WatcherResult<()> {
        // TODO: select! over tick timer and a shutdown signal
        Ok(())
    }

    /// Reconcile pending orders against chain state on startup.
    pub async fn reconcile_on_startup(&self) -> WatcherResult<()> {
        Ok(())
    }
}

impl Default for Watcher {
    fn default() -> Self {
        Self::new()
    }
}
