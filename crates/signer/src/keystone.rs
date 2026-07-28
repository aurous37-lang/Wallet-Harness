//! Keystone / air-gapped QR signer.
//!
//! Protocol: UR (Uniform Resource) encoded payloads over QR.
//!   1. App renders an animated QR of the unsigned tx (UR-encoded).
//!   2. User scans with the Keystone / GridPlus Lattice1 device.
//!   3. Device displays the decoded tx for review, user approves on-device.
//!   4. Device shows animated QR of the signature.
//!   5. App's webcam (desktop) or native camera (Android) scans it back.
//!
//! This is the most privacy-maximalist path — no network, no Bluetooth, no
//! USB between signer and app. Works identically on desktop and Android.

use async_trait::async_trait;

use crate::{
    Signer, SignerCapabilities, SignerError, SignerKind, SignerResult, UnsignedTransaction,
};

pub struct KeystoneSigner {
    address: String,
    xpub: Option<String>,
}

impl KeystoneSigner {
    /// Initiate account discovery — user scans their Keystone's account-export QR.
    pub async fn import_from_qr(_ur_payload: &str) -> SignerResult<Self> {
        Err(SignerError::Internal(
            "Keystone import not implemented".into(),
        ))
    }
}

#[async_trait]
impl Signer for KeystoneSigner {
    fn kind(&self) -> SignerKind {
        SignerKind::KeystoneQr
    }
    fn capabilities(&self) -> SignerCapabilities {
        SignerCapabilities::air_gapped()
    }
    async fn address(&self) -> SignerResult<String> {
        Ok(self.address.clone())
    }
    async fn sign_transaction(&self, _unsigned: UnsignedTransaction) -> SignerResult<String> {
        // In real impl this would be async with the UI: emit event to display
        // QR, await scanned response event from frontend.
        let _ = &self.xpub;
        Err(SignerError::Internal(
            "Keystone sign_transaction not implemented".into(),
        ))
    }
    async fn sign_typed_data(&self, _typed_data_json: &str) -> SignerResult<String> {
        Err(SignerError::Internal(
            "Keystone sign_typed_data not implemented".into(),
        ))
    }
    async fn sign_personal_message(&self, _message: &[u8]) -> SignerResult<String> {
        Err(SignerError::Internal(
            "Keystone sign_personal_message not implemented".into(),
        ))
    }
}
