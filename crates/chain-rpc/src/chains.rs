//! Canonical chain metadata for the v1 EVM set.
//!
//! This is the only place where chain ids are hardcoded. Everything else
//! should accept `ChainId` values and look up metadata here.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u64)]
pub enum ChainId {
    Ethereum = 1,
    Arbitrum = 42161,
    Base = 8453,
    Bnb = 56,
    Monad = 10143, // Monad testnet id; mainnet id updated at launch
}

impl ChainId {
    pub fn as_u64(self) -> u64 {
        self as u64
    }

    pub fn from_u64(v: u64) -> Option<Self> {
        match v {
            1 => Some(Self::Ethereum),
            42161 => Some(Self::Arbitrum),
            8453 => Some(Self::Base),
            56 => Some(Self::Bnb),
            10143 => Some(Self::Monad),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMeta {
    pub chain_id: u64,
    pub name: &'static str,
    pub short_name: &'static str,
    pub native_symbol: &'static str,
    pub native_decimals: u8,
    pub explorer_base: &'static str,
}

pub fn meta(chain: ChainId) -> ChainMeta {
    match chain {
        ChainId::Ethereum => ChainMeta {
            chain_id: 1,
            name: "Ethereum",
            short_name: "ETH",
            native_symbol: "ETH",
            native_decimals: 18,
            explorer_base: "https://etherscan.io",
        },
        ChainId::Arbitrum => ChainMeta {
            chain_id: 42161,
            name: "Arbitrum One",
            short_name: "ARB",
            native_symbol: "ETH",
            native_decimals: 18,
            explorer_base: "https://arbiscan.io",
        },
        ChainId::Base => ChainMeta {
            chain_id: 8453,
            name: "Base",
            short_name: "BASE",
            native_symbol: "ETH",
            native_decimals: 18,
            explorer_base: "https://basescan.org",
        },
        ChainId::Bnb => ChainMeta {
            chain_id: 56,
            name: "BNB Chain",
            short_name: "BNB",
            native_symbol: "BNB",
            native_decimals: 18,
            explorer_base: "https://bscscan.com",
        },
        ChainId::Monad => ChainMeta {
            chain_id: 10143,
            name: "Monad",
            short_name: "MON",
            native_symbol: "MON",
            native_decimals: 18,
            explorer_base: "https://explorer.monad.xyz",
        },
    }
}
