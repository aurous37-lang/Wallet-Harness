//! CSV seeder — replays a transaction history file through the [`Ledger`]
//! so the cost-basis panel has something to show before the on-chain indexer
//! (step 04) is wired up.
//!
//! CSV columns: `timestamp,chain_id,token_address,side,quantity,unit_price_usd,tx_hash`
//! where `side` is `buy` or `sell`. Rows are applied in the order given; the
//! caller is responsible for ensuring the file is sorted by timestamp if that
//! matters.

use std::path::Path;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::{CostBasisError, CostBasisResult, Ledger};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct SeedSummary {
    pub acquisitions: usize,
    pub disposals: usize,
    pub rows: usize,
}

#[derive(Debug, Deserialize)]
struct Row {
    timestamp: i64,
    chain_id: i64,
    token_address: String,
    side: String,
    quantity: f64,
    unit_price_usd: f64,
    tx_hash: String,
}

pub async fn seed_from_csv(
    pool: &SqlitePool,
    account: &str,
    path: &Path,
) -> CostBasisResult<SeedSummary> {
    let mut rdr = csv::Reader::from_path(path)
        .map_err(|e| CostBasisError::Invalid(format!("csv open {path:?}: {e}")))?;

    let ledger = Ledger::new(pool.clone());
    let mut summary = SeedSummary::default();

    for res in rdr.deserialize::<Row>() {
        let row = res.map_err(|e| CostBasisError::Invalid(format!("csv parse: {e}")))?;
        summary.rows += 1;
        match row.side.to_ascii_lowercase().as_str() {
            "buy" => {
                ledger
                    .apply_acquisition(
                        account,
                        row.chain_id,
                        &row.token_address,
                        row.quantity,
                        row.unit_price_usd,
                        row.timestamp,
                        &row.tx_hash,
                    )
                    .await?;
                summary.acquisitions += 1;
            }
            "sell" => {
                let proceeds = row.quantity * row.unit_price_usd;
                ledger
                    .apply_disposal(
                        account,
                        row.chain_id,
                        &row.token_address,
                        row.quantity,
                        proceeds,
                        row.timestamp,
                        &row.tx_hash,
                    )
                    .await?;
                summary.disposals += 1;
            }
            other => {
                return Err(CostBasisError::Invalid(format!(
                    "unknown side '{other}' (expected buy|sell)"
                )))
            }
        }
    }
    Ok(summary)
}
