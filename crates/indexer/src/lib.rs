//! indexer — local event indexer and OHLCV candle builder.
//!
//! Pulls swap events from DEX pools (Uniswap V2/V3-style) and transfer
//! events from ERC-20 tokens. Writes them to SQLite. Derives OHLCV candles
//! on demand from swap events for any time window + resolution.
//!
//! This is the "private, no-external-analytics" backbone: the app can be
//! fully functional without ever calling CoinGecko or DexScreener, as long
//! as the user has RPC access.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("rpc error: {0}")]
    Rpc(#[from] chain_rpc::RpcError),
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("no pool registered for {chain_id}:{pair}")]
    UnknownPool { chain_id: u64, pair: String },
}

pub type IndexerResult<T> = Result<T, IndexerError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolRef {
    pub chain_id: u64,
    pub address: String,
    pub token0: String,
    pub token1: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub dex_kind: DexKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DexKind {
    UniswapV2,
    UniswapV3,
    // Monad DEXes as they ship — Kuru, Ambient, etc.
    MonadNative,
}

pub struct Indexer {
    #[allow(dead_code)]
    db: SqlitePool,
    #[allow(dead_code)]
    rpc: chain_rpc::RpcRegistry,
}

impl Indexer {
    pub fn new(db: SqlitePool, rpc: chain_rpc::RpcRegistry) -> Self {
        Self { db, rpc }
    }

    /// Run a backfill from `from_block` to latest for a pool.
    pub async fn backfill_pool(&self, _pool: &PoolRef, _from_block: u64) -> IndexerResult<u64> {
        // TODO: chunked eth_getLogs, decode Swap events, insert into swaps table
        Ok(0)
    }

    /// Run a backfill of ERC-20 Transfer events involving `address`, for P&L.
    pub async fn backfill_transfers_for(
        &self,
        _chain_id: u64,
        _address: &str,
        _from_block: u64,
    ) -> IndexerResult<u64> {
        Ok(0)
    }

    /// Build OHLCV candles from stored swap events.
    pub async fn candles(
        &self,
        _pool: &PoolRef,
        _timeframe_seconds: u32,
        _from: i64,
        _to: i64,
    ) -> IndexerResult<Vec<Candle>> {
        Ok(vec![])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub t: i64,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
}
