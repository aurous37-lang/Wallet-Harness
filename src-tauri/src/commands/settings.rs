//! Settings: RPC endpoints and API keys.
//!
//! API keys are never persisted in the SQLite DB. They live in the OS
//! keychain (`keyring` crate) under service name `wallet-harness`. The DB
//! only tracks *which slots are populated*, never the values.

use serde::{Deserialize, Serialize};

use crate::state::AppState;

const KEYRING_SERVICE: &str = "wallet-harness";

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcEndpoint {
    pub chain_id: u64,
    pub label: String,
    pub url: String,
    pub priority: i32,
}

#[tauri::command]
pub async fn set_rpc_endpoint(
    state: tauri::State<'_, AppState>,
    endpoint: RpcEndpoint,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO rpc_endpoints (chain_id, label, url, priority, enabled, created_at) \
         VALUES (?1, ?2, ?3, ?4, 1, ?5) \
         ON CONFLICT(chain_id, label) DO UPDATE SET \
           url = excluded.url, \
           priority = excluded.priority, \
           enabled = 1",
    )
    .bind(endpoint.chain_id as i64)
    .bind(&endpoint.label)
    .bind(&endpoint.url)
    .bind(endpoint.priority as i64)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| format!("persist rpc endpoint: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn get_rpc_endpoints(
    state: tauri::State<'_, AppState>,
    chain_id: Option<u64>,
) -> Result<Vec<RpcEndpoint>, String> {
    let rows: Vec<(i64, String, String, i64)> = match chain_id {
        Some(cid) => sqlx::query_as(
            "SELECT chain_id, label, url, priority FROM rpc_endpoints \
             WHERE enabled = 1 AND chain_id = ?1 \
             ORDER BY priority ASC, id ASC",
        )
        .bind(cid as i64)
        .fetch_all(&state.db)
        .await
        .map_err(|e| format!("query rpc_endpoints: {e}"))?,
        None => sqlx::query_as(
            "SELECT chain_id, label, url, priority FROM rpc_endpoints \
             WHERE enabled = 1 ORDER BY chain_id ASC, priority ASC, id ASC",
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| format!("query rpc_endpoints: {e}"))?,
    };

    Ok(rows
        .into_iter()
        .map(|(chain_id, label, url, priority)| RpcEndpoint {
            chain_id: chain_id as u64,
            label,
            url,
            priority: priority as i32,
        })
        .collect())
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ApiKeySlot {
    Alchemy,
    Infura,
    QuickNode,
    CoinGecko,
    DexScreener,
    OneInch,
    Paraswap,
}

#[tauri::command]
pub async fn set_api_key(slot: ApiKeySlot, value: String) -> Result<(), String> {
    let entry_name = format!("{:?}", slot);
    let entry = keyring::Entry::new(KEYRING_SERVICE, &entry_name)
        .map_err(|e| format!("keyring init: {e}"))?;
    entry
        .set_password(&value)
        .map_err(|e| format!("keyring set: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn list_api_key_slots() -> Result<Vec<String>, String> {
    let candidates = [
        "Alchemy",
        "Infura",
        "QuickNode",
        "CoinGecko",
        "DexScreener",
        "OneInch",
        "Paraswap",
    ];
    let mut populated = Vec::new();
    for name in candidates {
        if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, name) {
            if entry.get_password().is_ok() {
                populated.push(name.to_string());
            }
        }
    }
    Ok(populated)
}
