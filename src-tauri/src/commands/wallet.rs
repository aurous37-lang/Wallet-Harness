//! Wallet connection and signer management commands.

use serde::{Deserialize, Serialize};
use signer::{SignerCapabilities, SignerKind};

#[derive(Serialize)]
pub struct SignerOption {
    pub kind: SignerKind,
    pub label: &'static str,
    pub capabilities: SignerCapabilities,
    pub platform_supported: bool,
}

#[tauri::command]
pub async fn list_signer_types() -> Result<Vec<SignerOption>, String> {
    // Platform gating is done here so the UI can render only what works on
    // the current OS without making the frontend OS-aware.
    let is_desktop = cfg!(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    ));
    let is_android = cfg!(target_os = "android");

    let mut out = Vec::new();

    out.push(SignerOption {
        kind: SignerKind::WalletConnectV2,
        label: "WalletConnect v2 (MetaMask, Rainbow, Rabby, etc.)",
        capabilities: SignerCapabilities::walletconnect(),
        platform_supported: true,
    });

    out.push(SignerOption {
        kind: SignerKind::LedgerUsb,
        label: "Ledger (USB)",
        capabilities: SignerCapabilities::hardware(),
        platform_supported: is_desktop || is_android,
    });

    out.push(SignerOption {
        kind: SignerKind::LedgerBluetooth,
        label: "Ledger (Bluetooth)",
        capabilities: SignerCapabilities::hardware(),
        platform_supported: is_desktop || is_android,
    });

    out.push(SignerOption {
        kind: SignerKind::KeystoneQr,
        label: "Keystone / air-gapped QR",
        capabilities: SignerCapabilities::air_gapped(),
        platform_supported: true,
    });

    // Hot-wallet signer is v2-only, gated behind the 73% win-rate threshold
    // for autonomous strategies. Reserved in the enum but not offered here.
    let _ = SignerKind::HotEncryptedKeystore;

    Ok(out)
}

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub kind: SignerKind,
    /// Optional transport-specific hint (e.g. WC pairing URI, BLE device id).
    pub transport_hint: Option<String>,
}

#[derive(Serialize)]
pub struct ConnectedAccount {
    pub address: String,
    pub chain_id: u64,
    pub signer_kind: SignerKind,
}

#[tauri::command]
pub async fn connect_wallet(_req: ConnectRequest) -> Result<ConnectedAccount, String> {
    // TODO: dispatch to signer::SignerRegistry based on kind
    Err("not implemented — signer registry wiring pending".into())
}

#[tauri::command]
pub async fn active_account() -> Result<Option<ConnectedAccount>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn disconnect_wallet() -> Result<(), String> {
    Ok(())
}
