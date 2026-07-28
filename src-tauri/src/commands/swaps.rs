//! Manual swap commands.
//!
//! Flow:
//!   1. `quote` — ask dex-router for best route + expected output
//!   2. `prepare_swap` — build the unsigned calldata and tx envelope
//!   3. frontend hands the envelope to the active signer (WC/Ledger/Keystone)
//!   4. `submit_signed_swap` — broadcast the signed raw tx via chain-rpc
//!
//! The backend never has access to the signature flow itself — that belongs
//! to the signer. This keeps the surface small and auditable.

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct QuoteRequest {
    pub chain_id: u64,
    pub token_in: String,
    pub token_out: String,
    pub amount_in_raw: String, // u256 as string
    pub slippage_bps: u32,
}

#[derive(Serialize)]
pub struct QuoteResponse {
    pub router: String, // "1inch" | "paraswap" | "direct:uniswap-v2" | ...
    pub amount_out_raw: String,
    pub price_impact_bps: u32,
    pub estimated_gas: u64,
    pub route_summary: String,
}

#[tauri::command]
pub async fn quote(_req: QuoteRequest) -> Result<QuoteResponse, String> {
    Err("not implemented".into())
}

#[derive(Deserialize)]
pub struct PrepareSwapRequest {
    pub chain_id: u64,
    pub from_address: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in_raw: String,
    pub slippage_bps: u32,
    pub router: String,
    /// Whether to prefer a private mempool (e.g. Flashbots Protect) when available.
    pub private_mempool: bool,
}

#[derive(Serialize)]
pub struct UnsignedTx {
    pub chain_id: u64,
    pub to: String,
    pub value: String, // hex
    pub data: String,  // hex
    pub gas_limit: u64,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: u64,
    /// Opaque token the frontend passes back to `submit_signed_swap` so the
    /// backend can associate a signature with the prepared intent.
    pub intent_id: String,
}

#[tauri::command]
pub async fn prepare_swap(_req: PrepareSwapRequest) -> Result<UnsignedTx, String> {
    Err("not implemented".into())
}

#[derive(Deserialize)]
pub struct SubmitSignedRequest {
    pub intent_id: String,
    pub signed_raw_tx: String, // 0x-prefixed hex
}

#[derive(Serialize)]
pub struct SubmitResponse {
    pub tx_hash: String,
    pub submitted_via: String, // "public_rpc" | "flashbots_protect" | "private_rpc"
}

#[tauri::command]
pub async fn submit_signed_swap(_req: SubmitSignedRequest) -> Result<SubmitResponse, String> {
    Err("not implemented".into())
}
