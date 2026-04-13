//! Policy optimality verification.
//!
//! Tests whether the policy's action π*(s) is optimal at every state by
//! directly estimating the Q-value of each candidate action and confirming
//! that π*(s) achieves the maximum.
//!
//! # Method
//!
//! For each state s and each candidate action n, we perform `trials`
//! independent single-step simulations:
//!
//! 1. Roll n dice.
//! 2. If the roll busts, the sample is −1 (active player loses immediately).
//! 3. If n = 0 (stand) or the roll succeeds, the successor state s' is
//!    determined and the sample is −V(s'), where V(s') is read directly from
//!    the policy table.  The negation reflects the zero-sum role swap: V(s')
//!    is from the *next* active player's perspective.
//!
//! The empirical mean of these samples estimates Q(s, n).
//!
//! # Why this constitutes a complete optimality test
//!
//! By the Markov property, the value of any strategy at state s depends only
//! on the actions taken at s and at the states reachable from s — not on how
//! s was reached.  The one-step deviation principle then states: a policy π
//! is globally optimal if and only if no single-state deviation improves the
//! expected payoff, assuming π is followed everywhere else.
//!
//! This module tests exactly that condition.  By using V(s') from the policy
//! table — which encodes the assumed-correct downstream values — each Q
//! estimate is a clean single-step test with no compounding across the game
//! tree.  Together, the per-state tests are exhaustive: any globally superior
//! strategy must be locally superior at some state, which would be detected.
//!
//! # Statistical design
//!
//! Each Q estimate is the mean of `trials` i.i.d. samples in [−1, +1].  The
//! test statistic for the hypothesis "action n beats the policy action n*" is
//! the one-sided z-score:
//!
//!   Z = (Q̂(s,n) − Q̂(s,n*)) / √(SE(s,n)² + SE(s,n*)²)
//!
//! with p-value P(Z_std > Z).  Multiple comparisons across all (state, action)
//! pairs are controlled with the Benjamini–Yekutieli procedure (see
//! [`stats::bhby_reject`]).

use std::io::Write;

use rand::prelude::*;

use super::stats::{SampleAccumulator, bhby_reject, global_chi2_test, right_tailed_p};
use crate::{Policy, Ruleset, State};

// ---------------------------------------------------------------------------
// Action range
// ---------------------------------------------------------------------------

/// Upper bound on the number of dice worth considering at a state.
///
/// Rolling more than ⌈(M − a) / μ⌉ dice is dominated in expectation: the
/// expected total already exceeds the remaining headroom M − a, so busting
/// becomes more likely than scoring.  Any optimal policy must choose n ≤ this
/// bound, so restricting candidates to [0, n_max] does not weaken the test.
///
/// Proved in the paper; verified by the `n_max_never_below_policy` test below.
fn n_max(active_score: u32, ruleset: Ruleset) -> u32 {
    let gap = ruleset.max().saturating_sub(active_score) as f64;
    let mean_per_die = (ruleset.sides() as f64 + 1.0) / 2.0;
    (gap / mean_per_die).ceil() as u32
}

/// All candidate actions at a state: [0, n_max(a)], inclusive.
fn all_actions(active_score: u32, ruleset: Ruleset) -> Vec<u32> {
    (0..=n_max(active_score, ruleset)).collect()
}

// ---------------------------------------------------------------------------
// Q-value estimation
// ---------------------------------------------------------------------------

/// Estimates Q(s, n) for a **terminal** state (last_round = true) by rolling n
/// dice `trials` times and recording the immediate win/tie/loss outcome.
///
/// At a terminal state, rolling ends the game: the active player's new score
/// is compared against the queued player's score.  No successor lookup is
/// needed.
fn estimate_q_terminal(
    state: State,
    n: u32,
    ruleset: Ruleset,
    rng: &mut SmallRng,
    trials: u64,
) -> SampleAccumulator {
    debug_assert!(state.last());
    let a = state.active();
    let q = state.queued();
    let mut acc = SampleAccumulator::new();

    for _ in 0..trials {
        let sample = if n == 0 {
            // Standing: compare scores directly.
            match a.cmp(&q) {
                std::cmp::Ordering::Greater => 1.0,
                std::cmp::Ordering::Equal => 0.0,
                std::cmp::Ordering::Less => -1.0,
            }
        } else {
            let total: u32 = (0..n).map(|_| rng.random_range(1..=ruleset.sides())).sum();
            let new_score = a + total;
            if new_score > ruleset.max() {
                -1.0 // bust
            } else {
                match new_score.cmp(&q) {
                    std::cmp::Ordering::Greater => 1.0,
                    std::cmp::Ordering::Equal => 0.0,
                    std::cmp::Ordering::Less => -1.0,
                }
            }
        };
        acc.push(sample);
    }
    acc
}

/// Estimates Q(s, n) for a **normal** state (last_round = false) by rolling n
/// dice `trials` times and looking up V(s') from the policy table.
///
/// Each sample is −V(s') because V(s') is from the *next* active player's
/// perspective, and this is a zero-sum game.  A bust contributes −1.
///
/// Standing (n = 0) transitions to (queued, active, last_round = true), the
/// start of the opponent's final turn.
fn estimate_q_normal(
    state: State,
    n: u32,
    policy: &Policy,
    ruleset: Ruleset,
    rng: &mut SmallRng,
    trials: u64,
) -> SampleAccumulator {
    debug_assert!(!state.last());
    let a = state.active();
    let q = state.queued();
    let mut acc = SampleAccumulator::new();

    for _ in 0..trials {
        let sample = if n == 0 {
            // Stand: game enters last_round; opponent becomes active.
            let succ = State::new(q, a, true);
            -policy[succ].payoff()
        } else {
            let total: u32 = (0..n).map(|_| rng.random_range(1..=ruleset.sides())).sum();
            let new_score = a + total;
            if new_score > ruleset.max() {
                -1.0 // bust
            } else {
                // Roles swap; last_round flag is unchanged (still false).
                let succ = State::new(q, new_score, false);
                -policy[succ].payoff()
            }
        };
        acc.push(sample);
    }
    acc
}

// ---------------------------------------------------------------------------
// Results
// ---------------------------------------------------------------------------

/// Test result for one (state, challenger action) comparison.
#[derive(Debug, Clone)]
pub struct ActionTestResult {
    pub state: State,
    /// The policy's chosen action at this state.
    pub n_policy: u32,
    /// The challenger action being tested.
    pub n_challenger: u32,
    /// Estimated Q(s, n_policy).
    pub q_policy: f64,
    pub q_policy_se: f64,
    /// Estimated Q(s, n_challenger).
    pub q_challenger: f64,
    pub q_challenger_se: f64,
    /// Z = (Q̂_challenger − Q̂_policy) / pooled_se.
    /// Positive = challenger appears better.
    pub z_stat: f64,
    /// P(Z_std > z_stat): right-tailed.
    pub p_value: f64,
    pub rejected: bool,
}

pub struct PolicyReport {
    /// One entry per (state, non-policy action) pair.
    pub results: Vec<ActionTestResult>,
    /// Number of (state, action) pairs where the challenger was significantly
    /// better than the policy action after BH-BY correction.
    pub n_flagged: usize,
    pub fdr_q: f64,
    /// Global omnibus test: are *all* z-stats jointly consistent with H₀?
    pub global_chi2: f64,
    pub global_df: usize,
    pub global_p: f64,
    /// States where Q̂(n*) ≥ Q̂(n) for all challengers n.
    pub argmax_matches: usize,
    /// States where some challenger had higher Q̂ but the difference was not
    /// significant after BH-BY correction.
    pub argmax_near_misses: usize,
    /// States where at least one challenger was significantly better (= failures).
    pub argmax_failures: usize,
}

impl PolicyReport {
    pub fn print_summary<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        writeln!(w, "=== Policy Optimality Report ===")?;
        writeln!(w, "Comparisons:  {}", self.results.len())?;
        writeln!(w, "FDR q:        {}", self.fdr_q)?;
        writeln!(w)?;
        writeln!(w, "Global omnibus test")?;
        writeln!(w, "  H0: no challenger beats the policy at any state")?;
        writeln!(
            w,
            "  chi2 = {:.4}  df = {}  p = {:.6}",
            self.global_chi2, self.global_df, self.global_p
        )?;
        writeln!(w)?;
        writeln!(w, "Empirical argmax")?;
        writeln!(w, "  Matches policy:      {}", self.argmax_matches)?;
        writeln!(w, "  Near-misses (n.s.):  {}", self.argmax_near_misses)?;
        writeln!(w, "  Failures (rejected): {}", self.argmax_failures)?;
        writeln!(w)?;
        writeln!(w, "BH-BY flagged: {}", self.n_flagged)?;

        if self.n_flagged == 0 {
            writeln!(w)?;
            writeln!(
                w,
                "PASS — no challenger outperforms the policy at any state."
            )?;
        } else {
            writeln!(w)?;
            writeln!(
                w,
                "FAIL — {} action(s) significantly outperform the policy:",
                self.n_flagged
            )?;
            writeln!(
                w,
                "  {:>34}  {:>5}  {:>6}  {:>8}  {:>8}  {:>8}  {:>12}",
                "state", "n_pol", "n_ch", "q_pol", "q_ch", "z", "p"
            )?;
            let mut flagged: Vec<_> = self.results.iter().filter(|r| r.rejected).collect();
            flagged.sort_by(|a, b| a.p_value.partial_cmp(&b.p_value).unwrap());
            for r in flagged.iter().take(50) {
                writeln!(
                    w,
                    "  {:>34?}  {:>5}  {:>6}  {:>8.4}  {:>8.4}  {:>8.3}  {:>12.2e}",
                    r.state,
                    r.n_policy,
                    r.n_challenger,
                    r.q_policy,
                    r.q_challenger,
                    r.z_stat,
                    r.p_value
                )?;
            }
        }
        Ok(())
    }

    /// All results sorted by ascending p-value.
    pub fn sorted_by_p(&self) -> Vec<&ActionTestResult> {
        let mut v: Vec<_> = self.results.iter().collect();
        v.sort_by(|a, b| a.p_value.partial_cmp(&b.p_value).unwrap());
        v
    }
}

// ---------------------------------------------------------------------------
// Main verification function
// ---------------------------------------------------------------------------

/// Verifies policy optimality via one-step Q-value estimation.
///
/// # Arguments
///
/// * `seed`    – RNG seed for reproducibility.
/// * `ruleset` – game parameters.
/// * `policy`  – the policy to verify.  Both the actions *and* the stored
///               payoff values V(s) are used: actions define what is being
///               tested; payoffs provide the downstream successor values.
/// * `trials`  – number of dice rolls per (state, action) pair.
///               Recommend ≥ 10_000 for reliable SE estimates.
/// * `fdr_q`   – BH-BY FDR level (recommend 0.05 or 0.01).
pub fn verify_policy(
    seed: u64,
    ruleset: Ruleset,
    policy: &Policy,
    trials: u64,
    fdr_q: f64,
) -> PolicyReport {
    eprintln!("[policy_verify] trials={trials}, fdr_q={fdr_q}, seed={seed}");
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut results: Vec<ActionTestResult> = Vec::new();
    let max = ruleset.max();

    // ------------------------------------------------------------------
    // Phase A: terminal states (last_round = true)
    //
    // Q(s, n) is estimated purely from immediate outcomes; no policy
    // table lookups are needed.  All successor states have been
    // "processed" trivially — the game ends after one roll.
    // ------------------------------------------------------------------
    eprintln!("[policy_verify] phase A: terminal states");

    for a in 0..=max {
        for q in 0..=max {
            let state = State::new(a, q, true);
            let actions = all_actions(a, ruleset);
            let estimates: Vec<(u32, SampleAccumulator)> = actions
                .iter()
                .map(|&n| (n, estimate_q_terminal(state, n, ruleset, &mut rng, trials)))
                .collect();
            push_comparisons(&mut results, state, policy[state].n(), &estimates);
        }
    }

    // ------------------------------------------------------------------
    // Phase B: normal states (last_round = false), processed in
    // descending order of total score (active + queued).
    //
    // A normal state s with total score T can only transition to states
    // with total score T' > T (rolling advances one player's score) or
    // to a terminal state (standing).  Processing in descending T order
    // guarantees every successor has already been processed — this
    // mirrors the backward-induction structure of the optimizer, but
    // here serves only to make the processing order explicit.  Since we
    // read V(s') directly from the policy table rather than from our own
    // estimates, the order has no effect on correctness; it merely
    // organises the work.
    // ------------------------------------------------------------------
    eprintln!("[policy_verify] phase B: normal states");

    for total in (0..=(2 * max)).rev() {
        for a in 0..=total.min(max) {
            let q = total - a;
            if q > max {
                continue;
            }
            let state = State::new(a, q, false);
            let actions = all_actions(a, ruleset);
            let estimates: Vec<(u32, SampleAccumulator)> = actions
                .iter()
                .map(|&n| {
                    (
                        n,
                        estimate_q_normal(state, n, policy, ruleset, &mut rng, trials),
                    )
                })
                .collect();
            push_comparisons(&mut results, state, policy[state].n(), &estimates);
        }
    }

    eprintln!("[policy_verify] total comparisons: {}", results.len());

    // ------------------------------------------------------------------
    // BH-BY correction (one-sided right tail).
    //
    // H₀ for each test: Q(s, n_challenger) ≤ Q(s, n_policy).
    // A rejection means the challenger appears *better* than the policy.
    // ------------------------------------------------------------------
    let p_values: Vec<f64> = results.iter().map(|r| r.p_value).collect();
    let rejected = bhby_reject(&p_values, fdr_q);
    let mut n_flagged = 0;
    for (r, rej) in results.iter_mut().zip(rejected.iter()) {
        r.rejected = *rej;
        if *rej {
            n_flagged += 1;
        }
    }

    // ------------------------------------------------------------------
    // Global omnibus test.
    // ------------------------------------------------------------------
    let z_stats: Vec<f64> = results.iter().map(|r| r.z_stat).collect();
    let (global_chi2, global_df, global_p) = global_chi2_test(&z_stats);

    // ------------------------------------------------------------------
    // Argmax summary: per state, find the highest-Q challenger.
    // ------------------------------------------------------------------
    let (argmax_matches, argmax_near_misses, argmax_failures) = argmax_summary(&results);

    PolicyReport {
        results,
        n_flagged,
        fdr_q,
        global_chi2,
        global_df,
        global_p,
        argmax_matches,
        argmax_near_misses,
        argmax_failures,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Appends one `ActionTestResult` per challenger action (every action except
/// n_policy) at the given state.
fn push_comparisons(
    results: &mut Vec<ActionTestResult>,
    state: State,
    n_policy: u32,
    estimates: &[(u32, SampleAccumulator)],
) {
    let (q_pol, se_pol) = estimates
        .iter()
        .find(|(n, _)| *n == n_policy)
        .map(|(_, acc)| (acc.mean(), acc.standard_error()))
        .unwrap_or((0.0, f64::INFINITY));

    for (n, acc) in estimates {
        if *n == n_policy {
            continue;
        }
        let q_ch = acc.mean();
        let se_ch = acc.standard_error();
        let pooled_se = (se_pol * se_pol + se_ch * se_ch).sqrt();
        let (z, p) = if pooled_se < 1e-12 {
            (0.0, 0.5)
        } else {
            let zv = (q_ch - q_pol) / pooled_se;
            (zv, right_tailed_p(zv))
        };
        results.push(ActionTestResult {
            state,
            n_policy,
            n_challenger: *n,
            q_policy: q_pol,
            q_policy_se: se_pol,
            q_challenger: q_ch,
            q_challenger_se: se_ch,
            z_stat: z,
            p_value: p,
            rejected: false,
        });
    }
}

/// Classifies each state by whether the policy action's Q̂ was the empirical
/// maximum, and whether any higher-Q challenger was significant.
///
/// Returns `(matches, near_misses, failures)`.
fn argmax_summary(results: &[ActionTestResult]) -> (usize, usize, usize) {
    // Collect the best challenger per state.
    let mut by_state: std::collections::HashMap<
        crate::State,
        (f64, f64, bool), // (best_challenger_q, policy_q, any_rejected)
    > = std::collections::HashMap::new();

    for r in results {
        let entry = by_state
            .entry(r.state)
            .or_insert((f64::NEG_INFINITY, r.q_policy, false));
        if r.q_challenger > entry.0 {
            entry.0 = r.q_challenger;
        }
        if r.rejected {
            entry.2 = true;
        }
    }

    let mut matches = 0;
    let mut near_misses = 0;
    let mut failures = 0;

    for (best_ch_q, pol_q, any_rejected) in by_state.values() {
        if best_ch_q <= pol_q {
            matches += 1;
        } else if *any_rejected {
            failures += 1;
        } else {
            near_misses += 1;
        }
    }

    (matches, near_misses, failures)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::PolicyOptimizer;

    #[test]
    fn correct_policy_passes_verification() {
        let ruleset = Ruleset::new(10, 6);
        let optimizer = PolicyOptimizer::optimize(ruleset);
        let policy = optimizer.policy();
        let report = verify_policy(42, ruleset, &policy, 2_000, 0.05);
        report.print_summary(std::io::stderr()).unwrap();
        assert_eq!(report.n_flagged, 0);
        assert_eq!(report.argmax_failures, 0);
        assert!(report.global_p > 0.001);
    }

    #[test]
    fn n_max_never_below_policy() {
        // If n_max is too tight, it would silently exclude the policy's own
        // action from the candidate set, making the test vacuous.
        let ruleset = Ruleset::new(20, 6);
        let optimizer = PolicyOptimizer::optimize(ruleset);
        let policy = optimizer.policy();
        let max = ruleset.max();
        for last in [false, true] {
            for a in 0..=max {
                for q in 0..=max {
                    let state = State::new(a, q, last);
                    assert!(
                        policy[state].n() <= n_max(a, ruleset),
                        "n_max too tight at {state:?}: policy={} n_max={}",
                        policy[state].n(),
                        n_max(a, ruleset)
                    );
                }
            }
        }
    }

    #[test]
    fn terminal_q_standing_ahead_is_one() {
        let ruleset = Ruleset::new(20, 6);
        let mut rng = SmallRng::seed_from_u64(0);
        let state = State::new(15, 10, true);
        let acc = estimate_q_terminal(state, 0, ruleset, &mut rng, 10_000);
        assert!(
            (acc.mean() - 1.0).abs() < 3.0 * acc.standard_error(),
            "Q(stand, ahead) should be +1, got {}",
            acc.mean()
        );
    }

    #[test]
    fn terminal_q_rolling_from_max_is_minus_one() {
        let ruleset = Ruleset::new(20, 6);
        let mut rng = SmallRng::seed_from_u64(1);
        // active = max, any roll busts.
        let state = State::new(20, 0, true);
        let acc = estimate_q_terminal(state, 1, ruleset, &mut rng, 10_000);
        assert!(
            (acc.mean() + 1.0).abs() < 3.0 * acc.standard_error(),
            "Q(roll from max) should be -1, got {}",
            acc.mean()
        );
    }
}
