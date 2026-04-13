/// Payoff consistency verification.
///
/// Tests whether the empirically observed mean payoff for each game state,
/// obtained by simulating many games with both players following the optimal
/// policy, is consistent with the theoretically computed payoff V(s).
///
/// # Method
///
/// For each state s, we accumulate N(s) independent game outcomes X ∈ {-1,0,+1}.
/// By the Markov property, starting a game directly at state s is equivalent
/// to conditioning on reaching s during normal play, so we use an adaptive
/// scheduler to ensure all states receive adequate coverage.
///
/// The test statistic is a z-score: Z(s) = (X̄(s) - V(s)) / SE(s).
///
/// Multiple comparisons are controlled with the Benjamini-Yekutieli (BY)
/// procedure, which is valid under arbitrary positive dependence — the
/// condition that holds here because trajectory-harvested observations for
/// different states share a terminal outcome.
use std::collections::HashMap;
use std::io::Write;

use rand::prelude::*;

use crate::verification::simulate::{Player, play_game_from_state};
use crate::verification::stats::RecordSheet;
use crate::{Policy, Ruleset, State};

// ---------------------------------------------------------------------------
// Normal CDF
// ---------------------------------------------------------------------------

/// Normal CDF via the Abramowitz & Stegun rational approximation (26.2.17).
/// Maximum absolute error < 7.5×10⁻⁸.
pub(crate) fn normal_cdf(x: f64) -> f64 {
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
pub(crate) fn two_tailed_p(z: f64) -> f64 {
    2.0 * normal_cdf(-z.abs())
}

// ---------------------------------------------------------------------------
// Monte Carlo results
// ---------------------------------------------------------------------------

pub struct MonteCarloResults(pub HashMap<State, RecordSheet>);

impl MonteCarloResults {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn record_game(&mut self, winner: Option<Player>, visited: &[(State, Player)]) {
        for &(state, active_player) in visited {
            let sheet = self.0.entry(state).or_default();
            match winner {
                None => sheet.record_tie(),
                Some(w) if w == active_player => sheet.record_win(),
                Some(_) => sheet.record_loss(),
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&State, &RecordSheet)> {
        self.0.iter()
    }
}

// ---------------------------------------------------------------------------
// Adaptive starting-state scheduler
// ---------------------------------------------------------------------------

/// Tracks per-state observation counts and selects the starting state for the
/// next game batch as the one furthest below its target.
///
/// Because the Markov property makes starting at any state s equivalent to
/// conditioning on reaching s, this purely shifts the sampling distribution
/// without introducing any bias in the per-state estimates.
pub struct AdaptiveScheduler {
    observed: Vec<u64>,
    target: u64,
    max: u32,
}

impl AdaptiveScheduler {
    pub fn new(max: u32, target: u64) -> Self {
        let n = 2 * (max as usize + 1) * (max as usize + 1);
        Self {
            observed: vec![0; n],
            target,
            max,
        }
    }

    const fn idx(max: u32, active: u32, queued: u32, last: bool) -> usize {
        let lr = last as usize;
        let m = max as usize + 1;
        lr * m * m + active as usize * m + queued as usize
    }

    pub fn record(&mut self, state: &State) {
        let i = Self::idx(self.max, state.active(), state.queued(), state.last());
        if i < self.observed.len() {
            self.observed[i] += 1;
        }
    }

    /// Returns the state with the largest deficit below target, or None if all
    /// states have met their target.
    pub fn most_needed(&self) -> Option<State> {
        let max = self.max;
        let mut best_deficit = 0i64;
        let mut best: Option<State> = None;
        for last in [false, true] {
            for active in 0..=max {
                for queued in 0..=max {
                    let deficit = self.target as i64
                        - self.observed[Self::idx(max, active, queued, last)] as i64;
                    if deficit > best_deficit {
                        best_deficit = deficit;
                        best = Some(State::new(active, queued, last));
                    }
                }
            }
        }
        best
    }

    pub fn all_satisfied(&self) -> bool {
        self.observed.iter().all(|&o| o >= self.target)
    }
}

// ---------------------------------------------------------------------------
// Simulation
// ---------------------------------------------------------------------------

/// Runs the adaptive simulation, returning accumulated per-state records.
pub fn simulate_adaptive(
    seed: u64,
    ruleset: Ruleset,
    policy: &Policy,
    target_obs: u64,
    batch_size: u64,
    max_games: u64,
) -> MonteCarloResults {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut results = MonteCarloResults::new();
    let mut scheduler = AdaptiveScheduler::new(ruleset.max(), target_obs);
    let mut start = State::new(0, 0, false);
    let mut total = 0u64;

    loop {
        for _ in 0..batch_size {
            if total >= max_games {
                return results;
            }
            let strategy = |s: State| policy[s].n();
            let (winner, visited) =
                play_game_from_state(start, strategy, strategy, ruleset, &mut rng);
            for (s, _) in &visited {
                scheduler.record(s);
            }
            results.record_game(winner, &visited);
            total += 1;
        }
        if scheduler.all_satisfied() {
            return results;
        }
        match scheduler.most_needed() {
            Some(s) => start = s,
            None => return results,
        }
    }
}

// ---------------------------------------------------------------------------
// Statistical analysis: BH-BY corrected z-tests
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct StateTestResult {
    pub state: State,
    pub theoretical: f64,
    pub empirical: f64,
    pub n_obs: u64,
    pub z_stat: f64,
    pub p_value: f64,
    pub rejected: bool,
}

pub struct PayoffReport {
    pub results: Vec<StateTestResult>,
    pub global_chi2: f64,
    pub global_df: usize,
    /// One-tailed p-value: large chi² = evidence against H0.
    pub global_p: f64,
    pub n_flagged: usize,
    pub n_untested: usize,
    pub fdr_q: f64,
}

impl PayoffReport {
    pub fn print_summary<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        writeln!(w, "=== Payoff Consistency Report ===")?;
        writeln!(w, "States tested:   {}", self.results.len())?;
        writeln!(w, "States untested: {} (< min_obs)", self.n_untested)?;
        writeln!(w)?;
        writeln!(w, "Global chi-squared test")?;
        writeln!(w, "  H0: all empirical payoffs match theoretical")?;
        writeln!(
            w,
            "  chi2 = {:.4}  df = {}  p = {:.6}",
            self.global_chi2, self.global_df, self.global_p
        )?;
        writeln!(w)?;
        writeln!(w, "BH-BY per-state tests  (FDR q = {})", self.fdr_q)?;
        writeln!(w, "  Flagged: {}", self.n_flagged)?;
        writeln!(w)?;

        if self.n_flagged > 0 {
            let mut flagged: Vec<_> = self.results.iter().filter(|r| r.rejected).collect();
            flagged.sort_by(|a, b| b.z_stat.abs().partial_cmp(&a.z_stat.abs()).unwrap());
            writeln!(
                w,
                "  {:>32}  {:>10}  {:>10}  {:>8}  {:>8}  {:>12}",
                "state", "theory", "empirical", "n", "z", "p"
            )?;
            for r in flagged.iter().take(50) {
                writeln!(
                    w,
                    "  {:>32?}  {:>10.4}  {:>10.4}  {:>8}  {:>8.3}  {:>12.2e}",
                    r.state, r.theoretical, r.empirical, r.n_obs, r.z_stat, r.p_value
                )?;
            }
        } else {
            writeln!(w, "  PASS — no states flagged.")?;
            writeln!(w, "  The policy payoffs are consistent with observed play.")?;
        }
        Ok(())
    }
}

/// Runs the BH-BY multiple-testing procedure over all states.
///
/// # Why BH-BY and not standard BH?
///
/// Observations for different states are harvested from shared trajectories,
/// so their test statistics are positively correlated. Benjamini & Yekutieli
/// (2001) prove that BH corrected by the harmonic number `c_K = Σ 1/j` controls
/// the FDR under arbitrary positive dependence. Standard BH requires
/// independence; BH-BY does not.
pub fn analyze(
    results: &MonteCarloResults,
    policy: &Policy,
    fdr_q: f64,
    min_obs: u64,
) -> PayoffReport {
    let mut state_results = Vec::new();
    let mut n_untested = 0;

    for (state, sheet) in results.iter() {
        if sheet.n() < min_obs {
            n_untested += 1;
            continue;
        }
        let se = sheet.standard_error();
        if se == 0.0 || !se.is_finite() {
            continue;
        }
        let theoretical = policy[*state].payoff();
        let empirical = sheet.mean();
        let z = (empirical - theoretical) / se;
        let p = two_tailed_p(z);
        state_results.push(StateTestResult {
            state: *state,
            theoretical,
            empirical,
            n_obs: sheet.n(),
            z_stat: z,
            p_value: p,
            rejected: false,
        });
    }

    let k = state_results.len();

    // Global chi-squared: sum of z² ~ chi²(K) under H0.
    // Conservative under positive dependence (effective df < K), so
    // a significant result is robust; a non-significant result is even cleaner.
    let global_chi2: f64 = state_results.iter().map(|r| r.z_stat * r.z_stat).sum();
    let chi2_z = (global_chi2 - k as f64) / (2.0 * k as f64).sqrt();
    let global_p = 1.0 - normal_cdf(chi2_z);

    // BH-BY: threshold for rank i (1-indexed) is (i/K) * q / c_K.
    let c_k: f64 = (1..=k).map(|j| 1.0 / j as f64).sum();
    let mut order: Vec<usize> = (0..k).collect();
    order.sort_by(|&a, &b| {
        state_results[a]
            .p_value
            .partial_cmp(&state_results[b].p_value)
            .unwrap()
    });

    let mut last_reject = None;
    for (i0, &idx) in order.iter().enumerate() {
        let threshold = ((i0 + 1) as f64 / k as f64) * fdr_q / c_k;
        if state_results[idx].p_value <= threshold {
            last_reject = Some(i0);
        }
    }

    let n_flagged = last_reject.map_or(0, |last| {
        for &idx in order.iter().take(last + 1) {
            state_results[idx].rejected = true;
        }
        last + 1
    });

    PayoffReport {
        results: state_results,
        global_chi2,
        global_df: k,
        global_p,
        n_flagged,
        n_untested,
        fdr_q,
    }
}

/// Full payoff verification pipeline.
pub fn verify_payoffs(
    seed: u64,
    ruleset: Ruleset,
    policy: &Policy,
    target_obs: u64,
    fdr_q: f64,
    batch_size: u64,
    max_games: u64,
) -> PayoffReport {
    eprintln!("[payoff_verify] target_obs={target_obs}, fdr_q={fdr_q}, seed={seed}");
    let mc = simulate_adaptive(seed, ruleset, policy, target_obs, batch_size, max_games);
    let total: u64 = mc.iter().map(|(_, s)| s.n()).sum();
    eprintln!("[payoff_verify] total state-visits: {total}");
    let min_obs = 30.max(target_obs / 10);
    analyze(&mc, policy, fdr_q, min_obs)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::PolicyOptimizer;

    #[test]
    fn correct_policy_passes_payoff_verify() {
        let ruleset = Ruleset::new(10, 6);
        let optimizer = PolicyOptimizer::optimize(ruleset.clone());
        let policy = optimizer.policy();
        let report = verify_payoffs(42, ruleset, &policy, 1_000, 0.05, 500, 5_000_000);
        report.print_summary(std::io::stderr()).unwrap();
        assert_eq!(report.n_flagged, 0);
        assert!(report.global_p > 0.001);
    }

    #[test]
    fn normal_cdf_spot_check() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-6);
        assert!((normal_cdf(1.96) - 0.975).abs() < 1e-3);
        assert!((normal_cdf(-1.96) - 0.025).abs() < 1e-3);
    }
}
