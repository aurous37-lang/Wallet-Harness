//! cost-basis — FIFO lot tracking with configurable jurisdiction and federal tax estimates.
//!
//! Public entry point is [`Ledger`], a thin wrapper over a `SqlitePool` that
//! implements the core write paths (`apply_acquisition`, `apply_disposal`) plus
//! read helpers (`position_snapshot`, `realized_gains`). Tax math lives in
//! [`taxes`], CSV backfill in [`seeder`].

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

pub mod seeder;
pub mod taxes;

pub use seeder::{seed_from_csv, SeedSummary};
pub use taxes::{TaxEstimate, TaxRates};

/// Long-term holding threshold. IRS treats >1 year as long-term; we use the
/// conservative strict-greater boundary expressed in whole days.
pub const LONG_TERM_DAYS: i64 = 366;

#[derive(Debug, Error)]
pub enum CostBasisError {
    #[error("insufficient open lots: requested {requested} but only {available} available")]
    InsufficientLots { requested: f64, available: f64 },
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
}

pub type CostBasisResult<T> = Result<T, CostBasisError>;

pub type LotId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lot {
    pub id: LotId,
    pub account: String,
    pub chain_id: i64,
    pub token_address: String,
    pub quantity: f64,
    pub quantity_original: f64,
    pub unit_cost_usd: f64,
    pub acquired_at: i64,
    pub source_tx_hash: String,
    pub closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealizedGain {
    pub lot_id: LotId,
    pub account: String,
    pub chain_id: i64,
    pub token_address: String,
    pub quantity: f64,
    pub cost_basis_usd: f64,
    pub proceeds_usd: f64,
    pub gain_usd: f64,
    pub holding_days: i64,
    pub acquired_at: i64,
    pub disposed_at: i64,
    pub source_tx_hash: String,
}

impl RealizedGain {
    pub fn is_long_term(&self) -> bool {
        self.holding_days >= LONG_TERM_DAYS
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub account: String,
    pub chain_id: i64,
    pub token_address: String,
    pub quantity: f64,
    pub cost_basis_usd: f64,
    pub avg_unit_cost_usd: f64,
    pub lot_count: u32,
}

pub struct Ledger {
    pool: SqlitePool,
}

impl Ledger {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn apply_acquisition(
        &self,
        account: &str,
        chain_id: i64,
        token: &str,
        quantity: f64,
        unit_cost_usd: f64,
        acquired_at: i64,
        tx_hash: &str,
    ) -> CostBasisResult<LotId> {
        if quantity <= 0.0 {
            return Err(CostBasisError::Invalid(
                "acquisition quantity must be > 0".into(),
            ));
        }
        if unit_cost_usd < 0.0 {
            return Err(CostBasisError::Invalid("unit_cost_usd must be >= 0".into()));
        }

        let id = Uuid::new_v4().to_string();
        let token_norm = token.to_lowercase();

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"INSERT INTO lots
                (id, account, chain_id, token_address, quantity, quantity_original,
                 unit_cost_usd, acquired_at, source_tx_hash, closed)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0)"#,
        )
        .bind(&id)
        .bind(account)
        .bind(chain_id)
        .bind(&token_norm)
        .bind(quantity)
        .bind(quantity)
        .bind(unit_cost_usd)
        .bind(acquired_at)
        .bind(tx_hash)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(id)
    }

    pub async fn apply_disposal(
        &self,
        account: &str,
        chain_id: i64,
        token: &str,
        quantity: f64,
        proceeds_usd_total: f64,
        disposed_at: i64,
        tx_hash: &str,
    ) -> CostBasisResult<Vec<RealizedGain>> {
        if quantity <= 0.0 {
            return Err(CostBasisError::Invalid(
                "disposal quantity must be > 0".into(),
            ));
        }
        if proceeds_usd_total < 0.0 {
            return Err(CostBasisError::Invalid("proceeds must be >= 0".into()));
        }

        let token_norm = token.to_lowercase();
        let mut tx = self.pool.begin().await?;

        let open_lots: Vec<(String, f64, f64, i64, String)> = sqlx::query_as(
            r#"SELECT id, quantity, unit_cost_usd, acquired_at, source_tx_hash
               FROM lots
               WHERE account = ? AND chain_id = ? AND token_address = ? AND closed = 0
               ORDER BY acquired_at ASC, id ASC"#,
        )
        .bind(account)
        .bind(chain_id)
        .bind(&token_norm)
        .fetch_all(&mut *tx)
        .await?;

        let available: f64 = open_lots.iter().map(|r| r.1).sum();
        if quantity > available + f64::EPSILON {
            return Err(CostBasisError::InsufficientLots {
                requested: quantity,
                available,
            });
        }

        let unit_proceeds = proceeds_usd_total / quantity;
        let mut remaining = quantity;
        let mut gains: Vec<RealizedGain> = Vec::new();

        for (lot_id, lot_qty, unit_cost, acquired_at, _lot_tx_hash) in open_lots {
            if remaining <= 0.0 {
                break;
            }
            let take = remaining.min(lot_qty);
            let new_lot_qty = lot_qty - take;
            let fully_closed = new_lot_qty <= f64::EPSILON;

            let cost_basis = take * unit_cost;
            let proceeds = take * unit_proceeds;
            let gain = proceeds - cost_basis;
            let holding_days = ((disposed_at - acquired_at).max(0)) / 86_400;

            if fully_closed {
                sqlx::query("UPDATE lots SET quantity = 0, closed = 1 WHERE id = ?")
                    .bind(&lot_id)
                    .execute(&mut *tx)
                    .await?;
            } else {
                sqlx::query("UPDATE lots SET quantity = ? WHERE id = ?")
                    .bind(new_lot_qty)
                    .bind(&lot_id)
                    .execute(&mut *tx)
                    .await?;
            }

            sqlx::query(
                r#"INSERT INTO realized_gains
                    (lot_id, account, chain_id, token_address, quantity,
                     cost_basis_usd, proceeds_usd, gain_usd, holding_days,
                     disposed_at, source_tx_hash)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(&lot_id)
            .bind(account)
            .bind(chain_id)
            .bind(&token_norm)
            .bind(take)
            .bind(cost_basis)
            .bind(proceeds)
            .bind(gain)
            .bind(holding_days)
            .bind(disposed_at)
            .bind(tx_hash)
            .execute(&mut *tx)
            .await?;

            gains.push(RealizedGain {
                lot_id: lot_id.clone(),
                account: account.to_string(),
                chain_id,
                token_address: token_norm.clone(),
                quantity: take,
                cost_basis_usd: cost_basis,
                proceeds_usd: proceeds,
                gain_usd: gain,
                holding_days,
                acquired_at,
                disposed_at,
                source_tx_hash: tx_hash.to_string(),
            });

            remaining -= take;
        }

        tx.commit().await?;
        Ok(gains)
    }

    pub async fn position_snapshot(
        &self,
        account: &str,
        chain_id: i64,
        token: &str,
    ) -> CostBasisResult<Position> {
        let token_norm = token.to_lowercase();
        let rows: Vec<(f64, f64)> = sqlx::query_as(
            r#"SELECT quantity, unit_cost_usd FROM lots
               WHERE account = ? AND chain_id = ? AND token_address = ? AND closed = 0"#,
        )
        .bind(account)
        .bind(chain_id)
        .bind(&token_norm)
        .fetch_all(&self.pool)
        .await?;

        let mut qty = 0.0;
        let mut cost = 0.0;
        for (q, u) in &rows {
            qty += q;
            cost += q * u;
        }
        let avg = if qty > 0.0 { cost / qty } else { 0.0 };

        Ok(Position {
            account: account.to_string(),
            chain_id,
            token_address: token_norm,
            quantity: qty,
            cost_basis_usd: cost,
            avg_unit_cost_usd: avg,
            lot_count: rows.len() as u32,
        })
    }

    pub async fn realized_gains(
        &self,
        account: &str,
        from_ts: i64,
        to_ts: i64,
    ) -> CostBasisResult<Vec<RealizedGain>> {
        let rows: Vec<(String, i64, String, f64, f64, f64, f64, i64, i64, String)> =
            sqlx::query_as(
                r#"SELECT rg.lot_id, rg.chain_id, rg.token_address, rg.quantity,
                      rg.cost_basis_usd, rg.proceeds_usd, rg.gain_usd,
                      rg.holding_days, rg.disposed_at, rg.source_tx_hash
               FROM realized_gains rg
               WHERE rg.account = ? AND rg.disposed_at >= ? AND rg.disposed_at <= ?
               ORDER BY rg.disposed_at ASC, rg.id ASC"#,
            )
            .bind(account)
            .bind(from_ts)
            .bind(to_ts)
            .fetch_all(&self.pool)
            .await?;

        // Join acquired_at from lots for accurate holding-period reporting.
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let acquired_at: i64 = sqlx::query_scalar("SELECT acquired_at FROM lots WHERE id = ?")
                .bind(&r.0)
                .fetch_one(&self.pool)
                .await?;
            out.push(RealizedGain {
                lot_id: r.0,
                account: account.to_string(),
                chain_id: r.1,
                token_address: r.2,
                quantity: r.3,
                cost_basis_usd: r.4,
                proceeds_usd: r.5,
                gain_usd: r.6,
                holding_days: r.7,
                disposed_at: r.8,
                source_tx_hash: r.9,
                acquired_at,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    const MIGRATION: &str = include_str!("../../../migrations/20260416000000_init.sql");

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        // Strip SQL line comments (anything from `--` to end-of-line) so that
        // a stray `;` inside a comment doesn't corrupt the statement split.
        let stripped: String = MIGRATION
            .lines()
            .map(|l| match l.find("--") {
                Some(i) => &l[..i],
                None => l,
            })
            .collect::<Vec<_>>()
            .join("\n");
        for stmt in stripped.split(';') {
            let s = stmt.trim();
            if s.is_empty() {
                continue;
            }
            sqlx::query(s).execute(&pool).await.unwrap();
        }
        pool
    }

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[tokio::test]
    async fn acquisition_then_snapshot() {
        let pool = setup_pool().await;
        let ledger = Ledger::new(pool);
        ledger
            .apply_acquisition("acct", 1, "0xabc", 10.0, 2.0, 1_700_000_000, "0x1")
            .await
            .unwrap();
        ledger
            .apply_acquisition("acct", 1, "0xabc", 5.0, 4.0, 1_700_100_000, "0x2")
            .await
            .unwrap();
        let pos = ledger.position_snapshot("acct", 1, "0xabc").await.unwrap();
        assert!(approx_eq(pos.quantity, 15.0, 1e-9));
        assert!(approx_eq(pos.cost_basis_usd, 40.0, 1e-9));
        // avg = 40/15
        assert!(approx_eq(pos.avg_unit_cost_usd, 40.0 / 15.0, 1e-9));
        assert_eq!(pos.lot_count, 2);
    }

    #[tokio::test]
    async fn disposal_consumes_fifo_and_realizes_gain() {
        let pool = setup_pool().await;
        let ledger = Ledger::new(pool);
        // Lot A: 10 @ $2, acquired at t0
        ledger
            .apply_acquisition("acct", 1, "0xabc", 10.0, 2.0, 1_700_000_000, "0xA")
            .await
            .unwrap();
        // Lot B: 5 @ $4, acquired a day later
        ledger
            .apply_acquisition("acct", 1, "0xabc", 5.0, 4.0, 1_700_086_400, "0xB")
            .await
            .unwrap();

        // Sell 12 @ $5/unit -> 12 * 5 = $60 proceeds. Should consume all of A (10)
        // then 2 of B. Cost basis = 10*2 + 2*4 = 28. Gain = 60 - 28 = 32.
        let gains = ledger
            .apply_disposal("acct", 1, "0xabc", 12.0, 60.0, 1_700_200_000, "0xSELL")
            .await
            .unwrap();
        assert_eq!(gains.len(), 2);
        let total_gain: f64 = gains.iter().map(|g| g.gain_usd).sum();
        assert!(approx_eq(total_gain, 32.0, 1e-9));

        let pos = ledger.position_snapshot("acct", 1, "0xabc").await.unwrap();
        assert!(approx_eq(pos.quantity, 3.0, 1e-9));
        assert!(approx_eq(pos.cost_basis_usd, 3.0 * 4.0, 1e-9));
        assert_eq!(pos.lot_count, 1);
    }

    #[tokio::test]
    async fn disposal_exceeding_open_quantity_errors() {
        let pool = setup_pool().await;
        let ledger = Ledger::new(pool);
        ledger
            .apply_acquisition("acct", 1, "0xabc", 5.0, 1.0, 1_700_000_000, "0xA")
            .await
            .unwrap();
        let err = ledger
            .apply_disposal("acct", 1, "0xabc", 10.0, 50.0, 1_700_100_000, "0xSELL")
            .await
            .unwrap_err();
        match err {
            CostBasisError::InsufficientLots {
                requested,
                available,
            } => {
                assert!(approx_eq(requested, 10.0, 1e-9));
                assert!(approx_eq(available, 5.0, 1e-9));
            }
            other => panic!("expected InsufficientLots, got {other:?}"),
        }
        // Ensure no realized_gains rows were written (transaction rolled back).
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM realized_gains")
            .fetch_one(ledger.pool())
            .await
            .unwrap();
        assert_eq!(n, 0);
        // And lot quantity is untouched.
        let q: f64 = sqlx::query_scalar("SELECT quantity FROM lots WHERE source_tx_hash = '0xA'")
            .fetch_one(ledger.pool())
            .await
            .unwrap();
        assert!(approx_eq(q, 5.0, 1e-9));
    }

    #[tokio::test]
    async fn csv_seeder_produces_expected_state() {
        let pool = setup_pool().await;
        let summary = seed_from_csv(
            &pool,
            "acct",
            std::path::Path::new("fixtures/sample_history.csv"),
        )
        .await
        .unwrap();
        assert_eq!(summary.acquisitions, 4);
        assert_eq!(summary.disposals, 2);

        let ledger = Ledger::new(pool);
        // The fixture ends with a final position; see fixture for math.
        let pos = ledger
            .position_snapshot("acct", 10143, "0xtokena")
            .await
            .unwrap();
        // After: buy 10@1, buy 10@2 (t+2y), sell 5 (partial fifo -> 5 of lot1),
        // sell 10 (full lot1 5 remaining + 5 of lot2). Remaining: 5 of lot2 @ $2.
        assert!(approx_eq(pos.quantity, 5.0, 1e-9));
        assert!(approx_eq(pos.cost_basis_usd, 10.0, 1e-9));

        // Second token untouched by disposals.
        let pos2 = ledger
            .position_snapshot("acct", 10143, "0xtokenb")
            .await
            .unwrap();
        assert!(approx_eq(pos2.quantity, 100.0, 1e-9));

        // Realized gains: 2 disposals producing 3 realized_gain rows total.
        let gains = ledger.realized_gains("acct", 0, i64::MAX).await.unwrap();
        assert_eq!(gains.len(), 3);

        // Tax estimate sanity.
        let est = taxes::estimate(&gains, taxes::TaxRates::default());
        // Net gain = (5*3 - 5*1) + (5*4 - 5*1) + (5*4 - 5*2) = 10 + 15 + 10 = 35.
        assert!(approx_eq(
            est.short_term_gain_usd + est.long_term_gain_usd,
            35.0,
            1e-9
        ));
        // Default jurisdiction estimate = 0.044 * 35 = 1.54.
        assert!(approx_eq(est.jurisdiction_tax_estimate_usd, 1.54, 1e-9));
    }
}
