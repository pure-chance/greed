//! A policy optimizer for the game of Greed.
//!
//! The policy optimizer computes the optimal policy for a game of Greed with some ruleset
//! `(M, s)`.

use std::cmp::Ordering;

use rayon::prelude::*;

use crate::greed::{Action, Policy, Ruleset, State};
use crate::pmf::PMFLookup;

/// Computes optimal strategies for Greed using dynamic programming.
///
/// The policy optimizer determines the best action (number of dice to roll) for every
/// possible game state by working backwards from terminal positions. This
/// approach guarantees mathematically optimal play.
///
/// # Algorithm Overview
///
/// ## Stage 1: Terminal States
///
/// Computes optimal actions for final-round states where one player has already
/// stood. Uses optimization to find the dice count that maximizes win
/// probability.
///
/// ## Stage 2: Normal States
///
/// Uses dynamic programming to compute optimal actions for regular game states.
/// States are processed in reverse order of total score (active + queued) to
/// ensure all future states are already computed when needed.
///
/// # Example
///
/// ```rust
/// use greed::{PolicyOptimizer, Ruleset};
/// let ruleset = Ruleset::new(100, 6);
/// let optimizer = PolicyOptimizer::optimize(ruleset);
/// ```
#[derive(Debug, Clone, Default)]
pub struct PolicyOptimizer {
    /// Ruleset for the game.
    ruleset: Ruleset,
    /// Computed optimal policy.
    policy: Policy,
    /// Precomputed probability mass functions for dice rolls.
    pmfs: PMFLookup,
}

impl PolicyOptimizer {
    /// Constructs a new policy optimizer, where the optimal policy has not been computed yet.
    ///
    /// This is used for certain benchmarking and testing scenarios.
    pub fn new(ruleset: Ruleset) -> Self {
        Self {
            ruleset,
            policy: Policy::new(ruleset.max()),
            pmfs: PMFLookup::precompute(ruleset.max(), ruleset.sides()),
        }
    }

    /// Compute the complete optimal policy for the given ruleset.
    ///
    /// Performs the full two-stage optimization: terminal states first, then normal
    /// states. After completion, the policy can be queried for any valid game
    /// state.
    pub fn optimize(ruleset: Ruleset) -> Self {
        let mut optimizer = PolicyOptimizer::new(ruleset);
        optimizer.optimize_terminal_states();
        optimizer.optimize_normal_states();
        optimizer
    }

    /// Returns the computed policy.
    ///
    /// The policy is only correct (and non-empty) after the `optimize` method is
    /// called.
    #[must_use]
    pub const fn policy(&self) -> &Policy {
        &self.policy
    }

    /// Returns the ruleset of this Greed game.
    #[must_use]
    pub const fn ruleset(&self) -> &Ruleset {
        &self.ruleset
    }

    /// Returns the maximum score for this ruleset.
    #[must_use]
    pub const fn max(&self) -> u32 {
        self.ruleset.max()
    }

    /// Returns the number of sides on each die for this ruleset.
    #[must_use]
    pub const fn sides(&self) -> u32 {
        self.ruleset.sides()
    }
}

impl PolicyOptimizer {
    /// Compute optimal actions for all terminal (last round) states.
    ///
    /// Terminal states occur when one player has stood, triggering the final
    /// round. These states can be optimized independently since there are no
    /// future rounds to consider.
    pub fn optimize_terminal_states(&mut self) {
        let states: Vec<_> = (0..=self.max())
            .flat_map(|turn| (0..=self.max()).map(move |next| State::new(turn, next, true)))
            .collect();

        let actions: Vec<_> = states
            .par_iter()
            .map(|state| (*state, self.find_optimal_terminal_action(*state)))
            .collect();

        for (state, action) in actions {
            self.policy[state] = action;
        }
    }

    /// Find the optimal number of dice to roll in a terminal state.
    ///
    /// # Algorithm
    ///
    /// We can split the possible terminal states into 3 categories:
    /// - If already ahead, doing nothing wins 100% of the time.
    /// - If there is some action A where the minimum sum > queued - active AND
    ///   the maximum sum is < max score - active, then that action wins 100% of
    ///   the time.
    /// - Otherwise, we need to calculate the optimal action by finding the max
    ///   in `range`, with boundaries that guarantee that the optimal action is
    ///   within the range.
    ///
    /// # Panics
    ///
    /// Panics if the either (1) one of the `f64` from
    /// `self.calc_terminal_payoff(state, dice_rolled)` is not comparable (i.e.,
    /// it's `NaN`), or (2) the `range` is empty. **Both of these conditions are
    /// impossible.**
    #[must_use]
    pub fn find_optimal_terminal_action(&self, state: State) -> Action {
        if state.active() > state.queued() {
            return Action::new(0, 1.0); // Ahead
        }
        if self.sides() * (state.queued() - state.active() + 1) <= self.max() - state.active() {
            return Action::new(state.queued() - state.active() + 1, 1.0); // Guaranteed win
        }

        // `range` starts at the min non-zero payoff, and ends at the maximum
        // optimal payoff (the point where the mean of the sum + active exceeds
        // the max score).
        let range = (state.queued() - state.active()) / self.sides()
            ..=2 * (self.max() - state.active() + self.sides()) / (self.sides() + 1).max(1);

        range
            .rev() // prefer conservative rolls
            .map(|dice_rolled| (dice_rolled, self.calc_terminal_payoff(state, dice_rolled)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(dice_rolled, payoff)| Action::new(dice_rolled, payoff))
            .unwrap() // Possible win
    }

    /// Calculate expected payoff for rolling a specific number of dice in a
    /// terminal state.
    ///
    /// Computes the probability-weighted outcome considering all possible dice
    /// sums:
    /// - Win: final score > opponent's score and ≤ max
    /// - Tie: final score = opponent's score
    /// - Lose: final score < opponent's score or > max (bust)
    #[must_use]
    pub fn calc_terminal_payoff(&self, state: State, dice_rolled: u32) -> f64 {
        if dice_rolled == 0 {
            return match state.active().cmp(&state.queued()) {
                Ordering::Less => -1.0,
                Ordering::Equal => 0.0,
                Ordering::Greater => 1.0,
            };
        }

        (dice_rolled..=self.sides() * dice_rolled).fold(0.0, |acc, dice_total| {
            let probability = self.pmfs.pmf(dice_rolled, dice_total);
            let outcome = match (state.active() + dice_total).cmp(&state.queued()) {
                Ordering::Greater if state.active() + dice_total <= self.max() => 1.0,
                Ordering::Equal => 0.0,
                Ordering::Less | Ordering::Greater => -1.0,
            };
            acc + outcome * probability
        })
    }
}

impl PolicyOptimizer {
    /// Compute optimal actions for all normal (non-terminal) game states.
    ///
    /// Uses dynamic programming with a specific ordering constraint: states
    /// must be processed in decreasing order of (active + queued) score to
    /// ensure all reachable future states have already been computed.
    ///
    /// # Ordering Requirement
    ///
    /// Normal states reference other normal states and terminal states, so they
    /// must be optimized after terminal states and in the correct dependency
    /// order.
    ///
    /// # Parallelization
    ///
    /// States within each order can be computed in parallel since they don't
    /// depend on each other.
    pub fn optimize_normal_states(&mut self) {
        for order in (0..=2 * self.max()).rev() {
            let states_actions: Vec<(State, Action)> = (0..=order.min(2 * self.max() - order))
                .into_par_iter() // Parallelize only within each order.
                .map(|place| {
                    let turn = order.min(self.max()) - place;
                    let next = order.max(self.max()) - self.max() + place;
                    let state = State::new(turn, next, false);
                    let action = self.find_optimal_normal_action(state);
                    (state, action)
                })
                .collect();

            for (state, action) in states_actions {
                self.policy[state] = action;
            }
        }
    }

    /// Find the optimal number of dice to roll in a normal (non-terminal)
    /// state.
    ///
    /// Considers all possible dice counts up to a mathematically derived upper
    /// bound, computing expected payoffs that account for all possible future
    /// game states.
    ///
    /// # Panics
    ///
    /// Panics if potential future states have not already been computed.
    #[must_use]
    pub fn find_optimal_normal_action(&self, state: State) -> Action {
        // The mean is $(n)(s + 1) / 2$, thus the $n$ for which the mean next score is
        // greater than the max score is $ceil(2 * (MAX - a) / (s + 1))$. This is the
        // same as $2 * (MAX - a + s) / (s + 1)$. This is how `limit` is calculated.
        let limit = 2 * (self.max() - state.active() + self.sides()) / (self.sides() + 1).max(1);
        let (optimal_roll, optimal_payoff) = (0..=limit)
            .rev() // If equal, the less aggressive move is taken.
            .map(|dice_rolled| (dice_rolled, self.calc_normal_payoff(state, dice_rolled)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        Action::new(optimal_roll, optimal_payoff)
    }

    /// Calculate expected payoff for rolling a specific number of dice in a
    /// normal state.
    ///
    /// For each possible dice outcome, looks up the optimal payoff from the
    /// resulting state and computes the probability-weighted expected value.
    /// Rolling 0 dice triggers the terminal round with swapped player
    /// positions.
    ///
    /// # Panics
    ///
    /// All reachable future states must already be optimized for correct payoff
    /// lookup.
    #[must_use]
    pub fn calc_normal_payoff(&self, state: State, dice_rolled: u32) -> f64 {
        if dice_rolled == 0 {
            let terminal_state = State::new(state.queued(), state.active(), true);
            return -self.policy[terminal_state].payoff();
        }
        (dice_rolled..=self.sides() * dice_rolled).fold(0.0, |acc, dice_total| {
            let probability = self.pmfs.pmf(dice_rolled, dice_total);
            let payoff = if state.active() + dice_total <= self.max() {
                let state = State::new(state.queued(), state.active() + dice_total, false);
                -self.policy[state].payoff()
            } else {
                -1.0
            };
            probability.mul_add(payoff, acc)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn make_policy(entries: &[(State, Action)], max: u32) -> Policy {
        let mut policy = Policy::new(max);
        for &(state, action) in entries {
            policy[state] = action;
        }
        policy
    }

    #[test]
    fn test_bellman_optimality() {
        const EPSILON: f64 = 1e-12;

        let optimizer = PolicyOptimizer::optimize(Ruleset::new(100, 6));

        for (state, action) in optimizer.policy().iter() {
            // Maximum possible optimal action.
            let n_max = 2 * (optimizer.max() - state.active() + optimizer.sides())
                / (optimizer.sides() + 1);
            let stored_payoff = action.payoff();

            for n in 0..=n_max {
                let computed = if state.last() {
                    optimizer.calc_terminal_payoff(state, n)
                } else {
                    optimizer.calc_normal_payoff(state, n)
                };

                assert!(
                    computed <= stored_payoff + EPSILON,
                    "Bellman violation at {state:?}: \
                     action n={n} yields payoff {computed:.10} \
                     which exceeds stored optimum {stored_payoff:.10} \
                     (diff: {:.2e})",
                    computed - stored_payoff,
                );
            }
        }
    }

    #[test]
    fn test_1_2_policy() {
        let optimizer = PolicyOptimizer::optimize(Ruleset::new(1, 2));

        let expected = make_policy(
            &[
                // terminal states
                (State::new(0, 0, true), Action::new(0, 0.0)),
                (State::new(0, 1, true), Action::new(1, -0.5)),
                (State::new(1, 0, true), Action::new(0, 1.0)),
                (State::new(1, 1, true), Action::new(0, 0.0)),
                // normal states
                (State::new(0, 0, false), Action::new(0, 0.0)),
                (State::new(0, 1, false), Action::new(1, -0.5)),
                (State::new(1, 0, false), Action::new(0, 0.5)),
                (State::new(1, 1, false), Action::new(0, 0.0)),
            ],
            1,
        );

        assert_eq!(optimizer.policy(), &expected);
    }

    #[test]
    fn test_2_2_policy() {
        let optimizer = PolicyOptimizer::optimize(Ruleset::new(2, 2));

        let expected = make_policy(
            &[
                // terminal states
                (State::new(0, 0, true), Action::new(1, 1.0)),
                (State::new(0, 1, true), Action::new(1, 0.5)),
                (State::new(0, 2, true), Action::new(1, -0.5)),
                (State::new(1, 0, true), Action::new(0, 1.0)),
                (State::new(1, 1, true), Action::new(0, 0.0)),
                (State::new(1, 2, true), Action::new(1, -0.5)),
                (State::new(2, 0, true), Action::new(0, 1.0)),
                (State::new(2, 1, true), Action::new(0, 1.0)),
                (State::new(2, 2, true), Action::new(0, 0.0)),
                // normal states
                (State::new(0, 0, false), Action::new(1, 0.0)),
                (State::new(0, 1, false), Action::new(1, 0.25)),
                (State::new(0, 2, false), Action::new(1, -0.25)),
                (State::new(1, 0, false), Action::new(1, -0.375)),
                (State::new(1, 1, false), Action::new(0, 0.0)),
                (State::new(1, 2, false), Action::new(1, -0.5)),
                (State::new(2, 0, false), Action::new(0, 0.5)),
                (State::new(2, 1, false), Action::new(0, 0.5)),
                (State::new(2, 2, false), Action::new(0, 0.0)),
            ],
            2,
        );

        assert_eq!(optimizer.policy(), &expected);
    }

    #[test]
    fn test_3_2_policy() {
        let optimizer = PolicyOptimizer::optimize(Ruleset::new(3, 2));

        let expected = make_policy(
            &[
                // terminal states
                (State::new(0, 0, true), Action::new(1, 1.0)),
                (State::new(0, 1, true), Action::new(1, 0.5)),
                (State::new(0, 2, true), Action::new(2, 0.25)),
                (State::new(0, 3, true), Action::new(2, -0.5)),
                (State::new(1, 0, true), Action::new(0, 1.0)),
                (State::new(1, 1, true), Action::new(1, 1.0)),
                (State::new(1, 2, true), Action::new(1, 0.5)),
                (State::new(1, 3, true), Action::new(1, -0.5)),
                (State::new(2, 0, true), Action::new(0, 1.0)),
                (State::new(2, 1, true), Action::new(0, 1.0)),
                (State::new(2, 2, true), Action::new(0, 0.0)),
                (State::new(2, 3, true), Action::new(1, -0.5)),
                (State::new(3, 0, true), Action::new(0, 1.0)),
                (State::new(3, 1, true), Action::new(0, 1.0)),
                (State::new(3, 2, true), Action::new(0, 1.0)),
                (State::new(3, 3, true), Action::new(0, 0.0)),
                // normal states
                (State::new(0, 0, false), Action::new(1, -0.03125)),
                (State::new(0, 1, false), Action::new(1, -0.125)),
                (State::new(0, 2, false), Action::new(1, 0.1875)),
                (State::new(0, 3, false), Action::new(2, -0.375)),
                (State::new(1, 0, false), Action::new(1, 0.09375)),
                (State::new(1, 1, false), Action::new(1, 0.0)),
                (State::new(1, 2, false), Action::new(1, 0.25)),
                (State::new(1, 3, false), Action::new(1, -0.25)),
                (State::new(2, 0, false), Action::new(0, -0.25)),
                (State::new(2, 1, false), Action::new(1, -0.375)),
                (State::new(2, 2, false), Action::new(0, 0.0)),
                (State::new(2, 3, false), Action::new(1, -0.5)),
                (State::new(3, 0, false), Action::new(0, 0.5)),
                (State::new(3, 1, false), Action::new(0, 0.5)),
                (State::new(3, 2, false), Action::new(0, 0.5)),
                (State::new(3, 3, false), Action::new(0, 0.0)),
            ],
            3,
        );

        assert_eq!(optimizer.policy(), &expected);
    }

    #[test]
    fn test_4_2_policy() {
        let optimizer = PolicyOptimizer::optimize(Ruleset::new(4, 2));

        let expected = make_policy(
            &[
                // terminal states
                (State::new(0, 0, true), Action::new(1, 1.0)),
                (State::new(0, 1, true), Action::new(2, 1.0)),
                (State::new(0, 2, true), Action::new(2, 0.75)),
                (State::new(0, 3, true), Action::new(2, 0.0)),
                (State::new(0, 4, true), Action::new(3, -0.625)),
                (State::new(1, 0, true), Action::new(0, 1.0)),
                (State::new(1, 1, true), Action::new(1, 1.0)),
                (State::new(1, 2, true), Action::new(1, 0.5)),
                (State::new(1, 3, true), Action::new(2, 0.25)),
                (State::new(1, 4, true), Action::new(2, -0.5)),
                (State::new(2, 0, true), Action::new(0, 1.0)),
                (State::new(2, 1, true), Action::new(0, 1.0)),
                (State::new(2, 2, true), Action::new(1, 1.0)),
                (State::new(2, 3, true), Action::new(1, 0.5)),
                (State::new(2, 4, true), Action::new(1, -0.5)),
                (State::new(3, 0, true), Action::new(0, 1.0)),
                (State::new(3, 1, true), Action::new(0, 1.0)),
                (State::new(3, 2, true), Action::new(0, 1.0)),
                (State::new(3, 3, true), Action::new(0, 0.0)),
                (State::new(3, 4, true), Action::new(1, -0.5)),
                (State::new(3, 0, true), Action::new(0, 1.0)),
                (State::new(3, 1, true), Action::new(0, 1.0)),
                (State::new(3, 2, true), Action::new(0, 1.0)),
                (State::new(4, 0, true), Action::new(0, 1.0)),
                (State::new(4, 1, true), Action::new(0, 1.0)),
                (State::new(4, 2, true), Action::new(0, 1.0)),
                (State::new(4, 3, true), Action::new(0, 1.0)),
                (State::new(4, 4, true), Action::new(0, 0.0)),
                // normal states
                (State::new(0, 0, false), Action::new(1, -0.015625)),
                (State::new(0, 1, false), Action::new(1, 0.078125)),
                (State::new(0, 2, false), Action::new(1, -0.046875)),
                (State::new(0, 3, false), Action::new(1, 0.3125)),
                (State::new(0, 4, false), Action::new(2, -0.375)),
                (State::new(1, 0, false), Action::new(1, -0.1328125)),
                (State::new(1, 1, false), Action::new(1, -0.03125)),
                (State::new(1, 2, false), Action::new(1, -0.125)),
                (State::new(1, 3, false), Action::new(1, 0.1875)),
                (State::new(1, 4, false), Action::new(2, -0.375)),
                (State::new(2, 0, false), Action::new(1, 0.03125)),
                (State::new(2, 1, false), Action::new(1, 0.09375)),
                (State::new(2, 2, false), Action::new(1, 0.0)),
                (State::new(2, 3, false), Action::new(1, 0.25)),
                (State::new(2, 4, false), Action::new(1, -0.25)),
                (State::new(3, 0, false), Action::new(0, 0.0)),
                (State::new(3, 1, false), Action::new(0, -0.25)),
                (State::new(3, 2, false), Action::new(1, -0.375)),
                (State::new(3, 3, false), Action::new(0, 0.0)),
                (State::new(3, 4, false), Action::new(1, -0.5)),
                (State::new(4, 0, false), Action::new(0, 0.625)),
                (State::new(4, 1, false), Action::new(0, 0.5)),
                (State::new(4, 2, false), Action::new(0, 0.5)),
                (State::new(4, 3, false), Action::new(0, 0.5)),
                (State::new(4, 4, false), Action::new(0, 0.0)),
            ],
            4,
        );

        assert_eq!(optimizer.policy(), &expected);
    }
}
