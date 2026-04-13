//! Shared statistical primitives for Monte Carlo verification.
//!
//! All hypothesis tests in this crate reduce to z-scores derived from sample
//! means of bounded random variables, with multiple-comparison correction via
//! the Benjamini–Yekutieli procedure.  This module collects those primitives
//! so they are implemented and tested exactly once.

// ---------------------------------------------------------------------------
// Normal distribution
// ---------------------------------------------------------------------------

/// Normal CDF via the Abramowitz & Stegun rational approximation (26.2.17).
/// Maximum absolute error < 7.5×10⁻⁸.
pub fn normal_cdf(x: f64) -> f64 {
    let sign = x.signum();
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.231_641_9 * x);
    let poly = t
        * (0.319_381_530
            + t * (-0.356_563_782
                + t * (1.781_477_937 + t * (-1.821_255_978 + t * 1.330_274_429))));
    let pdf = (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let p = 1.0 - pdf * poly;
    0.5 + sign * (p - 0.5)
}

/// Two-tailed p-value for a standard normal z-statistic.
pub fn two_tailed_p(z: f64) -> f64 {
    2.0 * normal_cdf(-z.abs())
}

/// Right-tailed p-value P(Z > z) for a standard normal z-statistic.
pub fn right_tailed_p(z: f64) -> f64 {
    1.0 - normal_cdf(z)
}

// ---------------------------------------------------------------------------
// Record sheet
// ---------------------------------------------------------------------------

/// Accumulates outcomes from one player's perspective at a single state or
/// (state, action) pair.  Outcomes are drawn from {−1, 0, +1} (loss/tie/win).
#[derive(Default, Clone)]
pub struct RecordSheet {
    wins: u64,
    ties: u64,
    losses: u64,
}

impl RecordSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn record_win(&mut self) {
        self.wins += 1;
    }

    pub const fn record_tie(&mut self) {
        self.ties += 1;
    }

    pub const fn record_loss(&mut self) {
        self.losses += 1;
    }

    pub fn n(&self) -> u64 {
        self.wins + self.ties + self.losses
    }

    /// Empirical mean payoff: (wins − losses) / n.
    pub fn mean(&self) -> f64 {
        let n = self.n();
        if n == 0 {
            return 0.0;
        }
        (self.wins as f64 - self.losses as f64) / n as f64
    }

    /// Standard error of the mean.
    ///
    /// For X ∈ {−1, 0, +1}: E[X²] = (wins + losses) / n, so
    ///   Var(X) = E[X²] − mean²,  SE = √(Var / n).
    pub fn standard_error(&self) -> f64 {
        let n = self.n();
        if n < 2 {
            return f64::INFINITY;
        }
        let nf = n as f64;
        let mean = (self.wins as f64 - self.losses as f64) / nf;
        let e_x2 = (self.wins + self.losses) as f64 / nf;
        let var = (e_x2 - mean * mean).max(0.0);
        (var / nf).sqrt()
    }
}

// ---------------------------------------------------------------------------
// Sample accumulator (continuous samples in [−1, +1])
// ---------------------------------------------------------------------------

/// Accumulates continuous samples from [−1, +1] and computes their mean and
/// standard error.  Used by policy verification, where each sample is a
/// looked-up V(s') value rather than a raw {−1, 0, +1} outcome.
#[derive(Default, Clone)]
pub struct SampleAccumulator {
    n: u64,
    sum: f64,
    sum_sq: f64,
}

impl SampleAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, x: f64) {
        self.n += 1;
        self.sum += x;
        self.sum_sq += x * x;
    }

    pub fn n(&self) -> u64 {
        self.n
    }

    pub fn mean(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            self.sum / self.n as f64
        }
    }

    pub fn standard_error(&self) -> f64 {
        if self.n < 2 {
            return f64::INFINITY;
        }
        let nf = self.n as f64;
        let mean = self.sum / nf;
        let var = (self.sum_sq / nf - mean * mean).max(0.0);
        (var / nf).sqrt()
    }
}

// ---------------------------------------------------------------------------
// BH-BY multiple testing correction
// ---------------------------------------------------------------------------

/// Applies the Benjamini–Yekutieli (BY) procedure to a list of p-values and
/// returns a boolean mask indicating which hypotheses are rejected.
///
/// # Why BY and not BH?
///
/// Standard BH controls the FDR only under independence or positive regression
/// dependence (PRDS).  BY controls it under *arbitrary* dependence by
/// deflating the BH threshold by the harmonic number c_K = Σ_{j=1}^{K} 1/j.
/// Both verification modules harvest observations from shared trajectories,
/// inducing positive correlation between test statistics — BY is therefore the
/// appropriate choice.
///
/// # Arguments
///
/// * `p_values` – p-value for each hypothesis, in any order.
/// * `fdr_q`    – desired false-discovery rate level (e.g. 0.05).
///
/// # Returns
///
/// A `Vec<bool>` of the same length as `p_values`, with `true` wherever H₀
/// is rejected.
pub fn bhby_reject(p_values: &[f64], fdr_q: f64) -> Vec<bool> {
    let k = p_values.len();
    if k == 0 {
        return vec![];
    }

    let c_k: f64 = (1..=k).map(|j| 1.0 / j as f64).sum();

    // Sort indices by p-value ascending.
    let mut order: Vec<usize> = (0..k).collect();
    order.sort_by(|&a, &b| p_values[a].partial_cmp(&p_values[b]).unwrap());

    // Find the largest rank i (1-indexed) such that p_(i) ≤ (i/K)·q/c_K.
    // All hypotheses up to and including that rank are rejected.
    let mut last_reject: Option<usize> = None;
    for (i0, &idx) in order.iter().enumerate() {
        let threshold = ((i0 + 1) as f64 / k as f64) * fdr_q / c_k;
        if p_values[idx] <= threshold {
            last_reject = Some(i0);
        }
    }

    let mut rejected = vec![false; k];
    if let Some(last) = last_reject {
        for &idx in order.iter().take(last + 1) {
            rejected[idx] = true;
        }
    }
    rejected
}

// ---------------------------------------------------------------------------
// Global chi-squared omnibus test
// ---------------------------------------------------------------------------

/// Computes a global omnibus p-value from a collection of z-statistics.
///
/// Under H₀, each zᵢ ~ N(0,1), so Σ zᵢ² ~ χ²(K).  Under positive
/// dependence (as holds here), the effective degrees of freedom are less than
/// K, making this test *conservative*: a significant result is robust, and a
/// non-significant result is an even stronger null finding.
///
/// Returns `(chi2_statistic, degrees_of_freedom, p_value)`.
/// The p-value is one-tailed: P(χ²(K) > observed), so large values of the
/// statistic yield small p-values.
pub fn global_chi2_test(z_stats: &[f64]) -> (f64, usize, f64) {
    let k = z_stats.len();
    if k == 0 {
        return (0.0, 0, 1.0);
    }
    let chi2: f64 = z_stats.iter().map(|z| z * z).sum();
    // Normal approximation to χ²(K): standardise by mean K and std √(2K).
    let z_approx = (chi2 - k as f64) / (2.0 * k as f64).sqrt();
    let p = right_tailed_p(z_approx);
    (chi2, k, p)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_cdf_spot_checks() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-6);
        assert!((normal_cdf(1.96) - 0.975).abs() < 1e-3);
        assert!((normal_cdf(-1.96) - 0.025).abs() < 1e-3);
    }

    #[test]
    fn two_tailed_p_symmetry() {
        assert!((two_tailed_p(1.96) - two_tailed_p(-1.96)).abs() < 1e-10);
        assert!(two_tailed_p(0.0) < 1.0 + 1e-10);
        assert!(two_tailed_p(10.0) < 1e-6);
    }

    #[test]
    fn record_sheet_mean_and_se() {
        let mut s = RecordSheet::new();
        for _ in 0..80 {
            s.record_win();
        }
        for _ in 0..20 {
            s.record_loss();
        }
        let mean = s.mean();
        assert!((mean - 0.6).abs() < 1e-10);
        assert!(s.standard_error() > 0.0);
        assert!(s.standard_error().is_finite());
    }

    #[test]
    fn record_sheet_all_ties_has_zero_se() {
        let mut s = RecordSheet::new();
        for _ in 0..100 {
            s.record_tie();
        }
        assert_eq!(s.mean(), 0.0);
        // Var = E[X²] - mean² = 0 - 0 = 0 → SE = 0.
        assert_eq!(s.standard_error(), 0.0);
    }

    #[test]
    fn sample_accumulator_matches_record_sheet_on_ternary() {
        // When all samples are in {-1, 0, +1}, SampleAccumulator and
        // RecordSheet should agree on mean and SE to floating-point precision.
        let mut acc = SampleAccumulator::new();
        let mut sheet = RecordSheet::new();
        let samples = [1.0_f64, -1.0, 0.0, 1.0, 1.0, -1.0, 0.0];
        for &x in &samples {
            acc.push(x);
            match x as i32 {
                1 => sheet.record_win(),
                -1 => sheet.record_loss(),
                _ => sheet.record_tie(),
            }
        }
        assert!((acc.mean() - sheet.mean()).abs() < 1e-12);
        assert!((acc.standard_error() - sheet.standard_error()).abs() < 1e-12);
    }

    #[test]
    fn bhby_rejects_none_when_all_p_large() {
        let p = vec![0.5, 0.6, 0.7, 0.8];
        let rejected = bhby_reject(&p, 0.05);
        assert!(rejected.iter().all(|&r| !r));
    }

    #[test]
    fn bhby_rejects_clear_signal() {
        // One extremely small p-value among noise should be rejected.
        let mut p = vec![0.5_f64; 99];
        p.push(1e-15);
        let rejected = bhby_reject(&p, 0.05);
        assert!(rejected[99]);
    }

    #[test]
    fn bhby_empty_input() {
        assert!(bhby_reject(&[], 0.05).is_empty());
    }

    #[test]
    fn global_chi2_uniform_z_near_null() {
        // z-stats drawn near zero: chi2 ≈ K, p should be near 0.5.
        let z: Vec<f64> = (0..100).map(|i| (i as f64 * 0.01) - 0.5).collect();
        let (_, df, p) = global_chi2_test(&z);
        assert_eq!(df, 100);
        assert!(p > 0.01, "expected p near null, got {p}");
    }

    #[test]
    fn global_chi2_large_z_small_p() {
        let z = vec![5.0_f64; 100];
        let (chi2, _, p) = global_chi2_test(&z);
        assert!(chi2 > 2000.0);
        assert!(p < 1e-10);
    }
}
