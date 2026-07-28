//! Wallet Harness — Tauri backend entry.
//!
//! Wires together the crate ecosystem (`chain-rpc`, `signer`, `indexer`,
//! `cost-basis`, `watcher`, `dex-router`) and exposes a narrow IPC surface to
//! the SvelteKit frontend. Every command is explicit. There is no catch-all
//! "invoke anything" bridge.

use tauri::Manager;
use tracing::{info, Level};

mod commands;
mod state;

use state::AppState;

pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("wallet-harness starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = tauri::async_runtime::block_on(async move {
                AppState::initialize(&handle)
                    .await
                    .expect("failed to initialize app state")
            });
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::meta::ping,
            commands::meta::app_info,
            commands::meta::meta_chain_status,
            commands::settings::set_rpc_endpoint,
            commands::settings::get_rpc_endpoints,
            commands::settings::set_api_key,
            commands::settings::list_api_key_slots,
            commands::wallet::list_signer_types,
            commands::wallet::connect_wallet,
            commands::wallet::active_account,
            commands::wallet::disconnect_wallet,
            commands::portfolio::balances,
            commands::portfolio::positions_with_cost_basis,
            commands::portfolio::transaction_history,
            commands::market::candles,
            commands::market::spot_price,
            commands::orders::list_open_orders,
            commands::orders::arm_stop_loss,
            commands::orders::arm_limit,
            commands::orders::cancel_order,
            commands::swaps::quote,
            commands::swaps::prepare_swap,
            commands::swaps::submit_signed_swap,
        ])
        .run(tauri::generate_context!())
        .expect("error while running wallet-harness");
}
