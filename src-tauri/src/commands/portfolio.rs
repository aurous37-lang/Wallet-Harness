//! Portfolio commands — balances, cost-basis-tracked positions, tx history.

use serde::Serialize;

#[derive(Serialize)]
pub struct TokenBalance {
    pub chain_id: u64,
    pub token_address: String, // "native" for gas token
    pub symbol: String,
    pub decimals: u8,
    pub raw_balance: String, // stringified u256
    pub ui_balance: f64,
}

#[tauri::command]
pub async fn balances(_address: String) -> Result<Vec<TokenBalance>, String> {
    // TODO: RpcRegistry per-chain parallel fetch
    Ok(vec![])
}

#[derive(Serialize)]
pub struct Position {
    pub chain_id: u64,
    pub token_address: String,
    pub symbol: String,
    pub quantity: f64,
    pub avg_cost_usd: f64,
    pub market_price_usd: f64,
    pub unrealized_pnl_usd: f64,
    pub realized_pnl_usd: f64,
    pub lot_count: u32,
}

#[tauri::command]
pub async fn positions_with_cost_basis(_address: String) -> Result<Vec<Position>, String> {
    // TODO: join indexer transactions with cost-basis FIFO lot tracker
    Ok(vec![])
}

#[derive(Serialize)]
pub struct TxRecord {
    pub tx_hash: String,
    pub chain_id: u64,
    pub block_number: u64,
    pub timestamp: i64,
    pub kind: String, // "swap" | "transfer_in" | "transfer_out" | "approval" | ...
    pub token_symbol: Option<String>,
    pub amount_ui: Option<f64>,
    pub counterparty: Option<String>,
}

#[tauri::command]
pub async fn transaction_history(
    _address: String,
    _chain_id: Option<u64>,
    _limit: Option<u32>,
) -> Result<Vec<TxRecord>, String> {
    Ok(vec![])
}
