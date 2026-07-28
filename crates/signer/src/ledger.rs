//! Ledger signer (USB HID + Bluetooth LE).
//!
//! Device interaction uses the Ethereum app APDU protocol. User must approve
//! every transaction on the device itself — no autonomous signing ever, by
//! hardware design.
//!
//! Transport selection is explicit: callers pick `LedgerTransport::Usb` or
//! `LedgerTransport::Bluetooth` based on the platform and what the user
//! chose in the UI.

use async_trait::async_trait;

use crate::{
    Signer, SignerCapabilities, SignerError, SignerKind, SignerResult, Transport,
    UnsignedTransaction,
};

#[derive(Debug, Clone, Copy)]
pub enum LedgerTransport {
    Usb,
    Bluetooth,
}

pub struct LedgerSigner {
    transport: LedgerTransport,
    derivation_path: String,
    address: String,
}

impl LedgerSigner {
    pub async fn connect(
        _transport: LedgerTransport,
        _derivation_path: &str,
    ) -> SignerResult<Self> {
        Err(SignerError::Internal(
            "Ledger connect not implemented".into(),
        ))
    }
}

#[async_trait]
impl Signer for LedgerSigner {
    fn kind(&self) -> SignerKind {
        match self.transport {
            LedgerTransport::Usb => SignerKind::LedgerUsb,
            LedgerTransport::Bluetooth => SignerKind::LedgerBluetooth,
        }
    }
    fn capabilities(&self) -> SignerCapabilities {
        let mut caps = SignerCapabilities::hardware();
        caps.transport = match self.transport {
            LedgerTransport::Usb => Transport::Usb,
            LedgerTransport::Bluetooth => Transport::Bluetooth,
        };
        caps
    }
    async fn address(&self) -> SignerResult<String> {
        Ok(self.address.clone())
    }
    async fn sign_transaction(&self, _unsigned: UnsignedTransaction) -> SignerResult<String> {
        let _ = &self.derivation_path;
        Err(SignerError::Internal(
            "Ledger sign_transaction not implemented".into(),
        ))
    }
    async fn sign_typed_data(&self, _typed_data_json: &str) -> SignerResult<String> {
        Err(SignerError::Internal(
            "Ledger sign_typed_data not implemented".into(),
        ))
    }
    async fn sign_personal_message(&self, _message: &[u8]) -> SignerResult<String> {
        Err(SignerError::Internal(
            "Ledger sign_personal_message not implemented".into(),
        ))
    }
}
