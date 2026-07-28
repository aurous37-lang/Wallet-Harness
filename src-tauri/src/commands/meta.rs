use std::time::Instant;

use serde::Serialize;

use chain_rpc::ChainId;

use crate::state::AppState;

#[tauri::command]
pub async fn ping() -> Result<&'static str, String> {
    Ok("pong")
}

#[derive(Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub commit: Option<&'static str>,
    pub build_mode: &'static str,
}

#[tauri::command]
pub async fn app_info() -> Result<AppInfo, String> {
    Ok(AppInfo {
        name: "Wallet Harness",
        version: env!("CARGO_PKG_VERSION"),
        commit: option_env!("GIT_COMMIT"),
        build_mode: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
    })
}

#[derive(Serialize)]
pub struct ChainStatus {
    pub chain_id: u64,
    pub block_number: Option<u64>,
    pub latency_ms: Option<u64>,
    pub endpoint_label: Option<String>,
}

#[tauri::command]
pub async fn meta_chain_status(
    state: tauri::State<'_, AppState>,
    chain_id: u64,
) -> Result<ChainStatus, String> {
    let unreachable = ChainStatus {
        chain_id,
        block_number: None,
        latency_ms: None,
        endpoint_label: None,
    };

    let chain = match ChainId::from_u64(chain_id) {
        Some(c) => c,
        None => return Ok(unreachable),
    };

    let endpoint_label = match state.rpc.providers_for(chain_id).await {
        Ok(providers) => providers.first().map(|p| p.label().to_string()),
        Err(_) => return Ok(unreachable),
    };

    let start = Instant::now();
    match state.rpc.block_number(chain).await {
        Ok(block_number) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            Ok(ChainStatus {
                chain_id,
                block_number: Some(block_number),
                latency_ms: Some(latency_ms),
                endpoint_label,
            })
        }
        Err(_) => Ok(unreachable),
    }
}
