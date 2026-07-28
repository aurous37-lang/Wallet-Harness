//! WalletConnect v2 signer.
//!
//! Desktop: QR code pairing (user scans with their mobile wallet).
//! Android: deep-link pairing (user taps, their wallet app opens, approves,
//! control returns via URL scheme).
//!
//! All traffic goes through the WalletConnect relay network, end-to-end
//! encrypted. The relay cannot see payloads.

use async_trait::async_trait;

use crate::{
    Signer, SignerCapabilities, SignerError, SignerKind, SignerResult, UnsignedTransaction,
};

pub struct WalletConnectSigner {
    // TODO: hold the session handle, topic, peer metadata, etc.
    address: String,
    chain_id: u64,
}

impl WalletConnectSigner {
    pub async fn pair_qr(
        _project_id: &str,
    ) -> SignerResult<(Self, String /* pairing uri for QR */)> {
        Err(SignerError::Internal("not implemented".into()))
    }

    pub async fn pair_deeplink(_project_id: &str) -> SignerResult<(Self, String /* deep link */)> {
        Err(SignerError::Internal("not implemented".into()))
    }
}

#[async_trait]
impl Signer for WalletConnectSigner {
    fn kind(&self) -> SignerKind {
        SignerKind::WalletConnectV2
    }
    fn capabilities(&self) -> SignerCapabilities {
        SignerCapabilities::walletconnect()
    }
    async fn address(&self) -> SignerResult<String> {
        Ok(self.address.clone())
    }
    async fn sign_transaction(&self, _unsigned: UnsignedTransaction) -> SignerResult<String> {
        let _ = self.chain_id;
        Err(SignerError::Internal(
            "WC sign_transaction not implemented".into(),
        ))
    }
    async fn sign_typed_data(&self, _typed_data_json: &str) -> SignerResult<String> {
        Err(SignerError::Internal(
            "WC sign_typed_data not implemented".into(),
        ))
    }
    async fn sign_personal_message(&self, _message: &[u8]) -> SignerResult<String> {
        Err(SignerError::Internal(
            "WC sign_personal_message not implemented".into(),
        ))
    }
}
