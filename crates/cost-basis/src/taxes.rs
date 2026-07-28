//! Tax estimation helpers. Pure math — no DB, no IO.
//!
//! These are *estimates*, intentionally coarse. Federal marginal brackets and
//! NIIT are deliberately not modeled; exact-owed amounts are out of scope for a
//! wallet harness. The caller should always present outputs as "estimated" and
//! never as "owed".

use serde::{Deserialize, Serialize};

use crate::{RealizedGain, LONG_TERM_DAYS};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TaxRates {
    /// Configurable jurisdiction tax rate applied to positive net gains.
    pub jurisdiction_rate: f64,
    /// Federal long-term capital gains estimate. Default 15% picks the middle
    /// bracket most of our users will land in.
    pub federal_long_term_rate: f64,
    /// Federal short-term marginal rate estimate.
    pub federal_short_term_rate: f64,
}

impl Default for TaxRates {
    fn default() -> Self {
        Self {
            jurisdiction_rate: 0.044,
            federal_long_term_rate: 0.15,
            federal_short_term_rate: 0.24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxEstimate {
    pub short_term_gain_usd: f64,
    pub long_term_gain_usd: f64,
    pub net_gain_usd: f64,
    pub jurisdiction_tax_estimate_usd: f64,
    pub federal_estimate_usd: f64,
    pub rates: TaxRates,
    /// Human-readable disclaimer. Callers should render this verbatim.
    pub disclaimer: &'static str,
}

pub fn estimate(gains: &[RealizedGain], rates: TaxRates) -> TaxEstimate {
    let (mut short, mut long) = (0.0, 0.0);
    for g in gains {
        if g.holding_days >= LONG_TERM_DAYS {
            long += g.gain_usd;
        } else {
            short += g.gain_usd;
        }
    }
    let net = short + long;

    // Apply the configured jurisdiction rate to positive net gains. A net loss
    // offsets the estimate to zero; loss carryforward is not modeled.
    let jurisdiction = (net.max(0.0)) * rates.jurisdiction_rate;

    // Federal estimate: tax each bucket independently when positive; losses in
    // one bucket offset gains in the other for the purposes of this estimate.
    let fed_short = short.max(0.0) * rates.federal_short_term_rate;
    let fed_long = long.max(0.0) * rates.federal_long_term_rate;
    let fed = (fed_short + fed_long).max(0.0);

    TaxEstimate {
        short_term_gain_usd: short,
        long_term_gain_usd: long,
        net_gain_usd: net,
        jurisdiction_tax_estimate_usd: jurisdiction,
        federal_estimate_usd: fed,
        rates,
        disclaimer: "ESTIMATE ONLY — not tax advice. Federal figures ignore brackets, NIIT, \
             and loss carryforwards. Consult a CPA before filing.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(gain: f64, days: i64) -> RealizedGain {
        RealizedGain {
            lot_id: "l".into(),
            account: "a".into(),
            chain_id: 1,
            token_address: "t".into(),
            quantity: 1.0,
            cost_basis_usd: 0.0,
            proceeds_usd: gain,
            gain_usd: gain,
            holding_days: days,
            acquired_at: 0,
            disposed_at: 0,
            source_tx_hash: "x".into(),
        }
    }

    #[test]
    fn split_short_and_long() {
        let gs = vec![mk(100.0, 30), mk(200.0, 500)];
        let est = estimate(&gs, TaxRates::default());
        assert!((est.short_term_gain_usd - 100.0).abs() < 1e-9);
        assert!((est.long_term_gain_usd - 200.0).abs() < 1e-9);
        assert!((est.net_gain_usd - 300.0).abs() < 1e-9);
        assert!((est.jurisdiction_tax_estimate_usd - 300.0 * 0.044).abs() < 1e-9);
        assert!((est.federal_estimate_usd - (100.0 * 0.24 + 200.0 * 0.15)).abs() < 1e-9);
    }

    #[test]
    fn net_loss_zeroes_state_tax() {
        let gs = vec![mk(-500.0, 30), mk(100.0, 500)];
        let est = estimate(&gs, TaxRates::default());
        assert!(est.net_gain_usd < 0.0);
        assert!((est.jurisdiction_tax_estimate_usd - 0.0).abs() < 1e-9);
    }
}
