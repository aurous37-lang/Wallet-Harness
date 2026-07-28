//! Market data commands — OHLCV candles for charts, spot price queries.

use serde::Serialize;

#[derive(Serialize)]
pub struct Candle {
    pub t: i64, // unix seconds
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
}

#[tauri::command]
pub async fn candles(
    _chain_id: u64,
    _pool_address: String,
    _timeframe_seconds: u32,
    _limit: u32,
) -> Result<Vec<Candle>, String> {
    // TODO: indexer-derived candles; fallback to DexScreener with user API key
    Ok(vec![])
}

#[derive(Serialize)]
pub struct SpotPrice {
    pub chain_id: u64,
    pub token_address: String,
    pub price_usd: f64,
    pub source: String, // "indexer:pool" | "dexscreener" | "coingecko" | "consensus"
    pub staleness_ms: u64,
}

#[tauri::command]
pub async fn spot_price(_chain_id: u64, _token_address: String) -> Result<SpotPrice, String> {
    Err("not implemented".into())
}
