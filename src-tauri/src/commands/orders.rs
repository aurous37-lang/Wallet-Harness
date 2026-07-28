//! Order commands.
//!
//! v1 behavior: orders are *armed alerts*. When the trigger condition fires,
//! the watcher emits a Tauri event; the UI surfaces a "ready to sign" queue;
//! the user approves through their chosen signer; the swap is submitted.
//!
//! v2 behavior (behind 73% win-rate gate): users with a `HotEncryptedKeystore`
//! signer can opt into autonomous execution on a per-order basis.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum OrderStatus {
    Armed,
    Triggered,
    AwaitingSignature,
    Submitted,
    Confirmed,
    Cancelled,
    Failed,
}

#[derive(Serialize, Clone)]
pub struct Order {
    pub id: String,
    pub chain_id: u64,
    pub pair: String, // e.g. "MON/USDC"
    pub kind: String, // "stop_loss" | "limit_buy" | "limit_sell"
    pub trigger_price_usd: f64,
    pub size_token: f64,
    pub slippage_bps: u32,
    pub status: OrderStatus,
    pub created_at: i64,
}

#[tauri::command]
pub async fn list_open_orders(_address: String) -> Result<Vec<Order>, String> {
    Ok(vec![])
}

#[derive(Deserialize)]
pub struct ArmStopLossRequest {
    pub chain_id: u64,
    pub token_in: String,
    pub token_out: String,
    pub trigger_price_usd: f64,
    pub size_token: f64,
    pub slippage_bps: u32,
}

#[tauri::command]
pub async fn arm_stop_loss(_req: ArmStopLossRequest) -> Result<String, String> {
    Err("not implemented".into())
}

#[derive(Deserialize)]
pub struct ArmLimitRequest {
    pub chain_id: u64,
    pub token_in: String,
    pub token_out: String,
    pub side: String, // "buy" | "sell"
    pub trigger_price_usd: f64,
    pub size_token: f64,
    pub slippage_bps: u32,
    /// If true and protocol supports it (1inch/CowSwap), route via on-chain
    /// limit order protocol instead of local watcher.
    pub prefer_protocol_order: bool,
}

#[tauri::command]
pub async fn arm_limit(_req: ArmLimitRequest) -> Result<String, String> {
    Err("not implemented".into())
}

#[tauri::command]
pub async fn cancel_order(_order_id: String) -> Result<(), String> {
    Err("not implemented".into())
}
