use std::cmp::Ordering;
use std::process::Command;

use csv::Writer;
use rayon::prelude::*;
use tempfile::NamedTempFile;

use crate::greed::{Ruleset, State};
use crate::pmf::fft_convolve;

/// The optimal action to perform, with its corresponding payoff
#[derive(Debug, Copy, Clone, Default)]
pub struct OptimalAction {
    /// Dice to roll
    n: u32,
    /// Payoff given a roll of `n` dice
    payoff: f64,
}

impl OptimalAction {
    /// Create a new optimal action with a given number of dice and payoff
    #[must_use]
    pub fn new(n: u32, payoff: f64) -> Self {
        OptimalAction { n, payoff }
    }
}

/// The optimal policy of Greed for the given ruleset
///
/// # Optimizations
///
/// - Store all policies in a single vector, with states with similar active score next to each other. This improves lookup speed because it reduces the number of cache misses.
#[derive(Debug, Clone, Default)]
pub struct Policy {
    policy: Box<[OptimalAction]>,
    max: u32,
}

impl Policy {
    /// Creates a new unoptimized policy for the given ruleset
    #[must_use]
    pub fn new(max: u32) -> Self {
        let size = ((max + 1) * (max + 1) * 2) as usize;
        let policy = vec![OptimalAction::default(); size].into_boxed_slice();
        Self { policy, max }
    }
    /// Get the optimal action for a given state
    #[must_use]
    pub fn get(&self, state: &State) -> OptimalAction {
        let placement = state.active() + (self.max + 1) * state.queued();
        let last_offset = (self.max + 1) * (self.max + 1) * u32::from(state.last());
        self.policy[(placement + last_offset) as usize]
    }
    /// Set the optimal action for a given state
    pub fn set(&mut self, state: &State, action: OptimalAction) {
        let placement = state.active() + (self.max + 1) * state.queued();
        let last_offset = (self.max + 1) * (self.max + 1) * u32::from(state.last());
        self.policy[(placement + last_offset) as usize] = action;
    }
    /// Iterate over all state-action pairs
    ///
    /// # Panics
    ///
    /// Panics if the state space is too big to fit in a `u32`.
    pub fn iter(&self) -> impl Iterator<Item = (State, OptimalAction)> + '_ {
        self.policy
            .iter()
            .enumerate()
            .map(move |(placement, action)| {
                let placement = u32::try_from(placement).expect("state space too big");
                let last_offset = (self.max + 1) * (self.max + 1);
                let (active, queued, last) = if placement >= last_offset {
                    let adjusted_placement = placement - last_offset;
                    let active = adjusted_placement % (self.max + 1);
                    let queued = adjusted_placement / (self.max + 1);
                    (active, queued, true)
                } else {
                    let active = placement % (self.max + 1);
                    let queued = placement / (self.max + 1);
                    (active, queued, false)
                };
                (State::new(active, queued, last), *action)
            })
    }
}

/// A lookup table for the probability mass function of the sum of dice rolls
#[derive(Debug, Clone, Default)]
pub struct PMFLookup(Vec<Vec<f64>>);

impl PMFLookup {
    /// Creates a new `PMFLookup` table for the given maximum score and number of sides on the dice
    #[must_use]
    pub fn precompute(max: u32, sides: u32) -> Self {
        let max_n = (2 * max / (sides + 1) + 1).max(max + 1);

        let mut pmfs: Vec<Vec<f64>> = Vec::with_capacity(max as usize + 1);
        let dice_pmf = vec![1.0 / f64::from(sides); sides as usize];

        pmfs.push(vec![1.0]);
        for n in 1..=max_n {
            pmfs.push(fft_convolve(&pmfs[(n - 1) as usize], &dice_pmf));
        }
        Self(pmfs)
    }
    /// Lookup the pmf for a given # of dice `n` and sum `total`
    #[must_use]
    pub fn lookup(&self, n: u32, total: u32) -> f64 {
        self.0[n as usize][(total - n) as usize]
    }
}

/// A solver for Greed
///
/// A game of Greed is a two-player dice game where players take turns rolling dice and accumulating points. The goal is to have a higher end score than the opponent without exceeding the maximum score (going bust).
///
/// # Rules
///
/// In each turn, a player can choose to roll 0+ dice. If they decide to roll a non-zero # of dice, they will roll, and the sum of their dice will be added to their score. If their score ever exceeds the maximum score, they go bust and lose the game.
///
/// If a player decides to roll 0 dice, they will not roll and their score will remain unchanged. This triggers the last round. The other player has one more opportunity to roll.
///
/// Whichever player has the highest (non-bust) score wins. If both players have the same score, the game is a draw.
///
/// # Solver
///
/// This solver operates in two stages.
///
/// In the first stage, the solver calculates the optimal moves moves in the last-round (terminal states). For each state `(turn, next, last = true)`, the solver goes through all possibly-optimal # dice, and finds the optimal action.
///
/// In the second stage, the solver calculates the optimal moves moves in the rest of the states (normal states). This is done via dynamic programming.
///
/// Starting with the maximum `turn + next` score, the only option (other than going bust) is to roll 0, thus it's possible to calculate the optimal action by looking up the action for the corresponding terminal state.
///
/// Moving to the second maximum `turn + next` score, the only options are to either end up in the previously computed normal state, or the corresponding terminal state.
///
/// This pattern continues until we reach the minimum `turn + next` score. All normal states are now fully computed.
#[derive(Debug, Clone, Default)]
pub struct GreedSolver {
    /// A ruleset of Greed (max, sides)
    ruleset: Ruleset,
    /// The solvers policy (state-action pairs)
    policy: Policy,
    /// Precomputed PMFs
    pmfs: PMFLookup,
}

impl GreedSolver {
    /// Create a new `GreedSolver` instance
    #[must_use]
    pub fn new(max: u32, sides: u32) -> Self {
        GreedSolver {
            ruleset: Ruleset::new(max, sides),
            policy: Policy::new(max),
            pmfs: PMFLookup::default(),
        }
    }
    /// Precompute all PMFs
    ///
    /// # Optimizations
    ///
    /// - For a given ruleset, there is an upper bound on the number of dice needed to check. So compute the PMF's for each n, and store them in the solver for O(1) lookup.
    pub fn precompute_pmfs(&mut self) {
        self.pmfs = PMFLookup::precompute(self.max(), self.sides());
    }
    /// Solve the game
    pub fn solve(&mut self) {
        // Precompute all PMFs
        self.precompute_pmfs();
        // Solve all the terminal states (this must be done first).
        self.solve_terminal_states();
        // Solve all the normal states (in the correct order).
        self.solve_normal_states();
    }
    /// Get the max score for the given ruleset
    #[must_use]
    pub fn max(&self) -> u32 {
        self.ruleset.max()
    }
    /// Get the sides on each die for the given ruleset
    #[must_use]
    pub fn sides(&self) -> u32 {
        self.ruleset.sides()
    }
}

impl GreedSolver {
    /// Solve terminal states
    pub fn solve_terminal_states(&mut self) {
        let states: Vec<_> = (0..=self.max())
            .flat_map(|turn| (0..=self.max()).map(move |next| State::new(turn, next, true)))
            .collect();

        let actions: Vec<_> = states
            .par_iter()
            .map(|state| (*state, self.find_optimal_terminal_action(*state)))
            .collect();

        for (state, action) in actions {
            self.policy.set(&state, action);
        }
    }
    /// Find the optimal terminal action for a given state
    ///
    /// # Optimizations
    ///
    /// - Because the optimal action is defined as having the highest probability of having `total` fall between `queued` and `max`, the distribution of `payoff` with respect to `n` is unimodal. This means that when the active player is behind we can search from `n = min_non-zero_payoff` up until the payoff starts decreasing, and then stop. This is guaranteed to have found the optimal action.
    fn find_optimal_terminal_action(&self, state: State) -> OptimalAction {
        if state.active() > state.queued() {
            // If already ahead, doing nothing wins 100% of the time.
            return OptimalAction { n: 0, payoff: 1.0 };
        }
        if self.sides() * (state.queued() - state.active() + 1) <= self.max() - state.active() {
            // If there is some action A where the minimum sum > queued - active AND the maximum sum is < max score - active, then that action wins 100% of the time.
            return OptimalAction::new(state.queued() - state.active() + 1, 1.0);
        }

        let mut optimal_action = OptimalAction::new(0, -1.0);
        let mut dice_rolled = (state.queued() - state.active()) / self.sides(); // Start at min non-zero payoff.

        loop {
            let current_payoff = self.calc_terminal_payoff(state, dice_rolled);
            if optimal_action.payoff - current_payoff >= 10e-2
                || dice_rolled >= (2 * self.max() / (self.sides() + 1) + 1).max(self.max() + 1)
            {
                break;
            }
            if current_payoff > optimal_action.payoff {
                optimal_action = OptimalAction::new(dice_rolled, current_payoff);
            }
            dice_rolled += 1;
        }

        optimal_action
    }
    /// Calculate the payoff when in state `state` and rolling `dice_rolled` # of dice
    fn calc_terminal_payoff(&self, state: State, dice_rolled: u32) -> f64 {
        if dice_rolled == 0 {
            return match state.active().cmp(&state.queued()) {
                Ordering::Less => -1.0,
                Ordering::Equal => 0.0,
                Ordering::Greater => 1.0,
            };
        }

        (dice_rolled..=self.sides() * dice_rolled).fold(0.0, |acc, dice_total| {
            let probability = self.pmfs.lookup(dice_rolled, dice_total);
            match (state.active() + dice_total).cmp(&state.queued()) {
                Ordering::Greater if state.active() + dice_total <= self.max() => acc + probability, // higher valid score
                Ordering::Less | Ordering::Greater => acc - probability, // lower score or bust
                Ordering::Equal => acc,                                  // tie
            }
        })
    }
}

impl GreedSolver {
    /// Solve normal states
    ///
    /// As previous payoffs rely on knowing later payoffs, the payoffs must be computed in reverse. This means starting at (M, M, F) and working backwards. This occurs visually as computing along the top-left to bottom-right diagonal, moving towards the bottom-left.
    ///
    /// # Invariants
    ///
    /// This presupposes that the all possible futures states (normal and terminal) have already been solved.
    pub fn solve_normal_states(&mut self) {
        // Process each order sequentially (constraint of the dynamic programming).
        for order in (0..=2 * self.max()).rev() {
            // For each order, process places in parallel.
            let states_actions: Vec<(State, OptimalAction)> = (0..=order
                .min(2 * self.max() - order))
                .into_par_iter() // Parallelize only within each order.
                .map(|place| {
                    // Calculate the player and opponent score for this order and place.
                    let (turn, next) = if order < self.max() {
                        (order - place, place)
                    } else {
                        (self.max() - place, (order - self.max()) + place)
                    };
                    let state = State::new(turn, next, false);
                    let action = self.find_optimal_normal_action(state);
                    (state, action)
                })
                .collect();

            // Insert the results for this order into the policy.
            for (state, action) in states_actions {
                self.policy.set(&state, action);
            }
        }
    }
    /// Find the optimal normal action for a given state
    ///
    /// # Invariants
    ///
    /// This presupposes that the all possible futures states (normal and terminal) have already been solved.
    fn find_optimal_normal_action(&self, state: State) -> OptimalAction {
        let max_reasonable_n = 2 * (self.max() - state.active()) / (self.sides() + 1) + 3; // +1 for safety of checking high enough
        let (optimal_roll, optimal_payoff) = (0..=max_reasonable_n)
            .map(|dice_rolled| (dice_rolled, self.calc_normal_payoff(state, dice_rolled)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        OptimalAction::new(optimal_roll, optimal_payoff)
    }
    /// Calculate the payoff when in state `state` and rolling `dice_rolled` # of dice
    ///
    /// # Invariants
    ///
    /// This presupposes that the all possible futures states (normal and terminal) have already been solved.
    #[must_use]
    pub fn calc_normal_payoff(&self, state: State, dice_rolled: u32) -> f64 {
        if dice_rolled == 0 {
            let terminal_state = State::new(state.queued(), state.active(), true);
            return -self.policy.get(&terminal_state).payoff;
        }
        (dice_rolled..=self.sides() * dice_rolled).fold(0.0, |acc, dice_total| {
            if state.active() + dice_total < self.max() {
                let probability: f64 = self.pmfs.lookup(dice_rolled, dice_total);
                let state = State::new(state.queued(), state.active() + dice_total, false);
                let payoff: f64 = -self.policy.get(&state).payoff;
                acc + probability * payoff
            } else {
                acc
            }
        })
    }
}

impl GreedSolver {
    /// Write the solver's policy to a human-readable format
    pub fn stdout(&self) {
        let mut state_action_pairs: Vec<_> = self.policy.clone().iter().collect();
        state_action_pairs.sort_by_key(|(state, _)| (state.last(), state.active(), state.queued()));

        let (terminal_states, normal_states): (Vec<_>, Vec<_>) = state_action_pairs
            .into_iter()
            .partition(|(state, _)| state.last());

        // terminal states
        for (state, action) in terminal_states {
            println!(
                "({}, {}, terminal) => (dice: #{}, payoff: {})",
                state.active(),
                state.queued(),
                action.n,
                action.payoff
            );
        }
        println!();
        // normal states
        for (state, action) in normal_states {
            println!(
                "({}, {}, normal) => (dice: #{}, payoff: {})",
                state.active(),
                state.queued(),
                action.n,
                action.payoff
            );
        }
    }
    /// Write the solver's policy to a CSV file
    ///
    /// # Errors
    ///
    /// Returns an error if the CSV file cannot be written to.
    pub fn csv(&self, path: &str) -> Result<(), csv::Error> {
        let mut writer = Writer::from_path(path)?;

        // Write headers
        writer.serialize(("active", "queued", "last", "n", "payoff"))?;
        for (state, action) in self.policy.iter() {
            writer.serialize((
                state.active(),
                state.queued(),
                state.last(),
                action.n,
                action.payoff,
            ))?;
        }
        writer.flush()?;
        Ok(())
    }
    /// Generate PNG visualizations using R script
    ///
    /// # Errors
    ///
    /// Returns an error if the R script execution fails.
    pub fn png(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary CSV file
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path();

        // Write CSV data to temporary file
        let mut writer = Writer::from_path(temp_path)?;
        writer.serialize(("active", "queued", "last", "n", "payoff"))?;
        for (state, action) in self.policy.iter() {
            writer.serialize((
                state.active(),
                state.queued(),
                state.last(),
                action.n,
                action.payoff,
            ))?;
        }
        writer.flush()?;

        let output = Command::new("Rscript")
            .arg("optimal_policies.R")
            .arg(temp_path)
            .current_dir("visualize")
            .output()?;

        if output.status.success() {
            if !output.stdout.is_empty() {
                println!("R output: {}", String::from_utf8_lossy(&output.stdout));
            }
        } else {
            eprintln!("R script failed with exit code: {:?}", output.status.code());
            if !output.stderr.is_empty() {
                eprintln!("R error: {}", String::from_utf8_lossy(&output.stderr));
            }
            return Err("R script execution failed".into());
        }

        Ok(())
    }
}
