//! signer — the wallet-agnostic signing abstraction.
//!
//! Everything that needs a signature goes through the [`Signer`] trait. This
//! lets the rest of the app be completely unaware of whether the user is
//! signing via WalletConnect, a Ledger over Bluetooth, an air-gapped
//! Keystone, or (post-v1, behind the win-rate gate) a local encrypted
//! keystore.
//!
//! The [`SignerCapabilities`] descriptor lets the order engine and UI decide
//! which features are offerable for the currently-connected signer. For
//! example, stop-losses that require autonomous execution are only shown
//! when `capabilities.autonomous == true`.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;

pub mod keystone;
pub mod ledger;
pub mod walletconnect;

#[derive(Debug, Error)]
pub enum SignerError {
    #[error("signer not connected")]
    NotConnected,
    #[error("user rejected signing request")]
    UserRejected,
    #[error("transport error: {0}")]
    Transport(String),
    #[error("unsupported operation for this signer: {0}")]
    Unsupported(&'static str),
    #[error("internal error: {0}")]
    Internal(String),
}

pub type SignerResult<T> = Result<T, SignerError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignerKind {
    WalletConnectV2,
    LedgerUsb,
    LedgerBluetooth,
    KeystoneQr,
    /// v2-only. Not offered in v1 signer listings. Gated behind a documented
    /// win-rate threshold for autonomous execution.
    HotEncryptedKeystore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    /// No transport — signing happens in-process (hot keystore).
    Local,
    /// USB HID (Ledger desktop, Ledger Nano S Plus on Android via USB-C).
    Usb,
    /// Bluetooth Low Energy (Ledger Nano X / Stax / Flex).
    Bluetooth,
    /// Air-gapped QR code exchange (Keystone, GridPlus Lattice1).
    Qr,
    /// WalletConnect relay (encrypted, end-to-end, relay sees no payload).
    Relay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerCapabilities {
    /// Can sign without per-tx user interaction. Only hot keystores.
    pub autonomous: bool,
    /// Requires explicit user approval per transaction.
    pub requires_user_interaction: bool,
    /// Supports EIP-1559 (type-2) transactions.
    pub supports_eip1559: bool,
    /// Supports EIP-712 typed data signing (needed for CowSwap, 1inch Limit).
    pub supports_eip712: bool,
    /// Supports personal_sign / SIWE.
    pub supports_personal_sign: bool,
    /// Transport mechanism — drives UI prompt copy.
    pub transport: Transport,
}

impl SignerCapabilities {
    pub const fn walletconnect() -> Self {
        Self {
            autonomous: false,
            requires_user_interaction: true,
            supports_eip1559: true,
            supports_eip712: true,
            supports_personal_sign: true,
            transport: Transport::Relay,
        }
    }
    pub const fn hardware() -> Self {
        Self {
            autonomous: false,
            requires_user_interaction: true,
            supports_eip1559: true,
            supports_eip712: true, // modern Ledger firmware
            supports_personal_sign: true,
            transport: Transport::Usb, // callers override to Bluetooth where relevant
        }
    }
    pub const fn air_gapped() -> Self {
        Self {
            autonomous: false,
            requires_user_interaction: true,
            supports_eip1559: true,
            supports_eip712: true,
            supports_personal_sign: true,
            transport: Transport::Qr,
        }
    }
    pub const fn hot_keystore() -> Self {
        Self {
            autonomous: true,
            requires_user_interaction: false,
            supports_eip1559: true,
            supports_eip712: true,
            supports_personal_sign: true,
            transport: Transport::Local,
        }
    }
}

/// The canonical signer interface.
#[async_trait]
pub trait Signer: Send + Sync {
    fn kind(&self) -> SignerKind;
    fn capabilities(&self) -> SignerCapabilities;
    async fn address(&self) -> SignerResult<String>;
    async fn sign_transaction(&self, unsigned: UnsignedTransaction) -> SignerResult<String>; // 0x-prefixed signed raw tx
    async fn sign_typed_data(&self, typed_data_json: &str) -> SignerResult<String>;
    async fn sign_personal_message(&self, message: &[u8]) -> SignerResult<String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    pub chain_id: u64,
    pub from: String,
    pub to: String,
    pub value: String, // hex
    pub data: String,  // hex
    pub gas_limit: u64,
    pub nonce: u64,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
}

/// Registry of instantiated signers keyed by a user-chosen slot name.
///
/// Cheap to clone. The registry itself holds no key material; individual
/// `Signer` impls manage their own transport state.
#[derive(Clone, Default)]
pub struct SignerRegistry {
    inner: Arc<RwLock<HashMap<String, Arc<dyn Signer>>>>,
}

impl SignerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn install(&self, slot: impl Into<String>, signer: Arc<dyn Signer>) {
        self.inner.write().await.insert(slot.into(), signer);
    }

    pub async fn get(&self, slot: &str) -> Option<Arc<dyn Signer>> {
        self.inner.read().await.get(slot).cloned()
    }

    pub async fn remove(&self, slot: &str) -> Option<Arc<dyn Signer>> {
        self.inner.write().await.remove(slot)
    }

    pub async fn slots(&self) -> Vec<String> {
        self.inner.read().await.keys().cloned().collect()
    }
}
