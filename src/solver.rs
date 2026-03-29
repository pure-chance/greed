//! A policy optimizer for the game of Greed.
//!
//! The solver computes the optimal policy for a game of Greed with some ruleset
//! `(M, s)`.

use std::cmp::Ordering;

use rayon::prelude::*;

use crate::greed::{Action, Policy, Ruleset, State};
use crate::pmf::PMFLookup;

/// Computes optimal strategies for Greed using dynamic programming.
///
/// The solver determines the best action (number of dice to roll) for every
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
/// use greed::Solver;
/// let mut solver = Solver::new(100, 6);
/// solver.solve();
/// ```
#[derive(Debug, Clone, Default)]
pub struct Solver {
    /// Ruleset for the game.
    ruleset: Ruleset,
    /// Computed optimal policy.
    policy: Policy,
    /// Precomputed probability mass functions for dice rolls.
    pmfs: PMFLookup,
}

impl Solver {
    /// Create a new solver for the specified game parameters.
    #[must_use]
    pub fn new(max: u32, sides: u32) -> Self {
        Self {
            ruleset: Ruleset::new(max, sides),
            policy: Policy::new(max),
            pmfs: PMFLookup::default(),
        }
    }

    /// Precompute probability mass functions for all strategically relevant
    /// dice counts.
    ///
    /// Calculates an upper bound on the maximum dice needed by the solver
    /// and generates PMFs up to that limit. This is done once and enables O(1)
    /// pmf lookups during policy computation.
    pub fn precompute_pmfs(&mut self) {
        self.pmfs = PMFLookup::precompute(self.max(), self.sides());
    }

    /// Compute the complete optimal policy for the given ruleset.
    ///
    /// Performs the full two-stage solve: terminal states first, then normal
    /// states. After completion, the policy can be queried for any valid game
    /// state.
    pub fn solve(&mut self) {
        self.precompute_pmfs();
        self.solve_terminal_states();
        self.solve_normal_states();
    }

    /// Returns the computed policy.
    ///
    /// The policy is only correct (and non-empty) after the `solve` method is called.
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

impl Solver {
    /// Compute optimal actions for all terminal (last round) states.
    ///
    /// Terminal states occur when one player has stood, triggering the final
    /// round. These states can be solved independently since there are no
    /// future rounds to consider.
    pub fn solve_terminal_states(&mut self) {
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
            let probability = self.pmfs[(dice_rolled, dice_total)];
            let outcome = match (state.active() + dice_total).cmp(&state.queued()) {
                Ordering::Greater if state.active() + dice_total <= self.max() => 1.0,
                Ordering::Equal => 0.0,
                Ordering::Less | Ordering::Greater => -1.0,
            };
            acc + outcome * probability
        })
    }
}

impl Solver {
    /// Compute optimal actions for all normal (non-terminal) game states.
    ///
    /// Uses dynamic programming with a specific ordering constraint: states
    /// must be processed in decreasing order of (active + queued) score to
    /// ensure all reachable future states have already been computed.
    ///
    /// # Ordering Requirement
    ///
    /// Normal states reference other normal states and terminal states, so they
    /// must be solved after terminal states and in the correct dependency
    /// order.
    ///
    /// # Parallelization
    ///
    /// States within each order can be computed in parallel since they don't
    /// depend on each other.
    pub fn solve_normal_states(&mut self) {
        // Process each order sequentially (constraint of the dynamic programming).
        for order in (0..=2 * self.max()).rev() {
            // For each order, process places in parallel.
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

            // Insert the results for this order into the policy.
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
    /// All reachable future states must already be solved for correct payoff
    /// lookup.
    #[must_use]
    pub fn calc_normal_payoff(&self, state: State, dice_rolled: u32) -> f64 {
        if dice_rolled == 0 {
            let terminal_state = State::new(state.queued(), state.active(), true);
            return -self.policy[terminal_state].payoff();
        }
        (dice_rolled..=self.sides() * dice_rolled).fold(0.0, |acc, dice_total| {
            let probability: f64 = self.pmfs[(dice_rolled, dice_total)];
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

    #[test]
    fn test_solver_vs_known_optimal_strategies() {
        // Test solver against known optimal strategies for simple cases
        let mut solver = Solver::new(10, 2);
        solver.solve();

        // At max score, should never roll
        let max_state = State::new(10, 5, false);
        let action = solver.policy[max_state];
        assert_eq!(action.n(), 0, "At max score, should never roll");

        // When opponent is at max and we're behind, must roll
        let must_roll_state = State::new(8, 10, true);
        let action = solver.policy[must_roll_state];
        assert!(action.n() > 0, "Must roll when behind in terminal state");
    }

    #[test]
    fn test_game_symmetry() {
        // Test that the game exhibits expected symmetry properties
        let mut solver = Solver::new(15, 3);
        solver.solve();

        // Test symmetry in normal states
        let state1 = State::new(8, 6, false);
        let state2 = State::new(6, 8, false);

        let action1 = solver.policy[state1];
        let action2 = solver.policy[state2];

        // While not perfectly symmetric due to turn order, payoffs should be roughly
        // opposite
        assert!(
            (action1.payoff() + action2.payoff()).abs() < 0.5,
            "Symmetric states should have roughly opposite payoffs"
        );
    }

    #[test]
    fn test_end_game_behavior() {
        let mut solver = Solver::new(30, 6);
        solver.solve();

        // Test behavior near end game
        let close_states = vec![
            State::new(25, 28, false), // Behind but close
            State::new(28, 25, false), // Ahead but close
            State::new(29, 29, false), // Tied near max
            State::new(30, 25, false), // At max, ahead
        ];

        for state in close_states {
            let action = solver.policy[state];

            // All actions should be valid
            assert!(action.n() <= 20, "End game actions should be reasonable");
            assert!(action.payoff() >= -1.0 - 1e-10, "payoffs should be valid");
            assert!(action.payoff() <= 1.0 + 1e-10, "payoffs should be valid");

            // At max score, should never roll
            if state.active() == 30 {
                assert_eq!(action.n(), 0, "At max score, should never roll");
            }
        }
    }
}
