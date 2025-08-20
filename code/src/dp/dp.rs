use std::cmp::Ordering;
use std::process::Command;

use rayon::prelude::*;

use super::pmf::fft_convolve;
use crate::{Action, Policy, Ruleset, Solver, State};

/// Optimized lookup table for dice roll probability mass functions.
///
/// Precomputes and stores PMFs for all dice counts up to a maximum, enabling
/// O(1) lookup of P(sum = k | n dice). This is the performance-critical
/// component of the solver, as PMF lookups occur millions of times during
/// policy computation.
#[derive(Debug, Clone)]
pub struct PMFLookup {
    /// Flat array containing all PMF data.
    data: Box<[f64]>,
    /// Starting offsets for each n-dice PMF.
    offsets: Box<[usize]>,
    /// Maximum number of dice.
    max_n: u32,
}

impl Default for PMFLookup {
    fn default() -> Self {
        Self {
            data: Box::new([]),
            offsets: Box::new([]),
            max_n: 0,
        }
    }
}

impl PMFLookup {
    /// Precompute all required PMFs for the given game parameters.
    ///
    /// Generates PMFs for 0 to max_n dice, where max_n is determined by the
    /// largest number of dice that could be strategically relevant. Uses FFT
    /// convolution for efficient computation and creates optimized lookup
    /// tables.
    ///
    /// # Time Complexity
    ///
    /// O(max_n × sides × log(sides)) due to FFT operations.
    #[must_use]
    pub fn precompute(max: u32, sides: u32) -> Self {
        let max_n = (2 * (max + sides) / (sides + 1)).max(max + 1);
        let dice_pmf = vec![1.0 / f64::from(sides); sides as usize];

        // First pass: compute individual PMFs to determine total size
        let mut temp_pmfs: Vec<Vec<f64>> = Vec::with_capacity((max_n + 1) as usize);
        temp_pmfs.push(vec![1.0]); // n=0 case

        for n in 1..=max_n {
            temp_pmfs.push(fft_convolve(&temp_pmfs[(n - 1) as usize], &dice_pmf));
        }

        // Validate PMFs sum to 1.0
        for (n, pmf) in temp_pmfs.iter().enumerate() {
            if n > 0 {
                let sum: f64 = pmf.iter().sum();
                debug_assert!(
                    (sum - 1.0).abs() < 1e-10,
                    "PMF for {} dice doesn't sum to 1.0: {}",
                    n,
                    sum
                );
            }
        }

        // Second pass: flatten into single array with offset table
        let total_size: usize = temp_pmfs.iter().map(|v| v.len()).sum();
        let mut data = Vec::with_capacity(total_size);
        let mut offsets = Vec::with_capacity((max_n + 1) as usize);

        for pmf in &temp_pmfs {
            offsets.push(data.len());
            data.extend_from_slice(pmf);
        }

        Self {
            data: data.into_boxed_slice(),
            offsets: offsets.into_boxed_slice(),
            max_n,
        }
    }
    /// Fast lookup of PMF value P(sum = total | n dice).
    ///
    /// Optimized for hot path usage with caching for small n values and unsafe
    /// memory access. Use this in performance-critical code where bounds are
    /// guaranteed.
    ///
    /// # Safety
    ///
    /// Caller must ensure n ≤ max_n and total ≥ n.
    #[must_use]
    #[inline]
    pub fn lookup(&self, n: u32, total: u32) -> f64 {
        debug_assert!(n <= self.max_n, "n={} exceeds max_n={}", n, self.max_n);
        debug_assert!(total >= n, "total={} less than n={}", total, n);

        unsafe {
            let offset = *self.offsets.get_unchecked(n as usize);
            let index = offset + (total - n) as usize;
            *self.data.get_unchecked(index)
        }
    }
    /// Bounds-checked version of PMF lookup that returns 0.0 for invalid
    /// inputs.
    ///
    /// Use this when input bounds are uncertain or in non-performance-critical
    /// code. Slightly slower than `lookup()` due to bounds checking.
    #[must_use]
    #[inline]
    pub fn lookup_safe(&self, n: u32, total: u32) -> f64 {
        if n > self.max_n || total < n {
            return 0.0;
        }

        let offset = self.offsets[n as usize];
        let index = offset + (total - n) as usize;

        if index < self.data.len() {
            self.data[index]
        } else {
            0.0
        }
    }
    /// Returns memory usage statistics for the PMF lookup table.
    #[must_use]
    pub fn memory_usage(&self) -> (usize, usize) {
        let data_bytes = self.data.len() * std::mem::size_of::<f64>();
        let offset_bytes = self.offsets.len() * std::mem::size_of::<usize>();
        (data_bytes, offset_bytes)
    }
}

/// Computes optimal strategies for Greed using dynamic programming.
///
/// The solver determines the best action (number of dice to roll) for every
/// possible game state by working backwards from terminal positions. This
/// approach guarantees mathematically optimal play under the assumption that
/// both players play perfectly.
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
/// let mut solver = GreedSolver::new(100, 6);
/// solver.solve();
/// let action = solver.policy.get(&State::new(50, 45, false));
/// println!("Optimal: roll {} dice (payoff: {:.3})", action.n, action.payoff);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DpSolver {
    /// Game configuration (maximum score and die sides).
    ruleset: Ruleset,
    /// Computed optimal policy mapping states to actions.
    policy: Policy,
    /// Precomputed probability mass functions for dice rolls.
    pmfs: PMFLookup,
}

impl DpSolver {
    /// Create a new solver for the specified game parameters.
    #[must_use]
    pub fn new(max: u32, sides: u32) -> Self {
        DpSolver {
            ruleset: Ruleset::new(max, sides),
            policy: Policy::new(max),
            pmfs: PMFLookup::default(),
        }
    }
    /// Precompute probability mass functions for all strategically relevant
    /// dice counts.
    ///
    /// Calculates an upper bound on the maximum dice needed and generates PMFs
    /// up to that limit. This is done once per solver and enables O(1)
    /// probability lookups during policy computation.
    ///
    /// # Performance Impact
    ///
    /// This is a one-time cost that dramatically speeds up the subsequent solve
    /// operations.
    pub fn precompute_pmfs(&mut self) {
        self.pmfs = PMFLookup::precompute(self.max(), self.sides());
    }
    /// Compute the complete optimal policy for this game configuration.
    ///
    /// Performs the full two-stage solve: terminal states first, then normal
    /// states. After completion, the policy can be queried for any valid game
    /// state.
    pub fn solve(&mut self) {
        // Precompute all PMFs
        self.precompute_pmfs();
        // Solve all the terminal states (this must be done first).
        self.solve_terminal_states();
        // Solve all the normal states (in the correct order).
        self.solve_normal_states();
    }
    /// Returns the maximum score for this game configuration.
    #[must_use]
    pub fn max(&self) -> u32 {
        self.ruleset.max()
    }
    /// Returns the number of sides on each die for this game configuration.
    #[must_use]
    pub fn sides(&self) -> u32 {
        self.ruleset.sides()
    }
}

impl DpSolver {
    /// Compute optimal actions for all terminal (final round) states.
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
            self.policy.set(&state, action);
        }
    }
    /// Find the optimal number of dice to roll in a terminal state.
    ///
    /// Uses the mathematical property that terminal payoff functions are
    /// unimodal (single peak) to enable early termination when payoffs start
    /// decreasing.
    ///
    /// # Algorithm
    ///
    /// + Handle obvious cases (already winning, guaranteed win scenarios)
    /// + Search from minimum viable dice count upward
    /// + Stop when payoff decreases consistently or search limit reached
    pub fn find_optimal_terminal_action(&self, state: State) -> Action {
        if state.active() > state.queued() {
            // If already ahead, doing nothing wins 100% of the time.
            return Action { n: 0, payoff: 1.0 };
        }
        if self.sides() * (state.queued() - state.active() + 1) <= self.max() - state.active() {
            // If there is some action A where the minimum sum > queued - active AND the
            // maximum sum is < max score - active, then that action wins 100% of the time.
            return Action::new(state.queued() - state.active() + 1, 1.0);
        }

        let mut optimal_action = Action::new(0, -1.0);
        let mut dice_rolled = (state.queued() - state.active()) / self.sides(); // Start at min non-zero payoff.

        loop {
            let current_payoff = self.calc_terminal_payoff(state, dice_rolled);
            if optimal_action.payoff - current_payoff >= 10e-2
                || dice_rolled >= (2 * self.max() / (self.sides() + 1) + 1).max(self.max() + 1)
            {
                break;
            }
            if current_payoff > optimal_action.payoff {
                optimal_action = Action::new(dice_rolled, current_payoff);
            }
            dice_rolled += 1;
        }

        optimal_action
    }
    /// Calculate expected payoff for rolling a specific number of dice in a
    /// terminal state.
    ///
    /// Computes the probability-weighted outcome considering all possible dice
    /// sums:
    /// - Win: final score > opponent's score and ≤ max
    /// - Lose: final score < opponent's score or > max (bust)
    /// - Tie: final score = opponent's score
    pub fn calc_terminal_payoff(&self, state: State, dice_rolled: u32) -> f64 {
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

impl DpSolver {
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
    /// Find the optimal number of dice to roll in a normal (non-terminal)
    /// state.
    ///
    /// Considers all possible dice counts up to a mathematically derived upper
    /// bound, computing expected payoffs that account for all possible future
    /// game states.
    ///
    /// # Prerequisites
    ///
    /// All reachable future states (both normal and terminal) must already be
    /// solved.
    pub fn find_optimal_normal_action(&self, state: State) -> Action {
        // The mean is $(n)(s + 1) / 2$, thus the $n$ for which the mean next score is
        // greater than the max score is $ceil(2 * (MAX - a) / (s + 1))$. This is the
        // same as $2 * (MAX - a + s) / (s + 1)$. This is how `max_optimal_n` is
        // calculated.
        let max_optimal_n = 2 * (self.max() - state.active() + self.sides()) / (self.sides() + 1);
        let (optimal_roll, optimal_payoff) = (0..=max_optimal_n)
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
    /// # Prerequisites
    ///
    /// All reachable future states must already be solved for correct payoff
    /// lookup.
    #[must_use]
    pub fn calc_normal_payoff(&self, state: State, dice_rolled: u32) -> f64 {
        if dice_rolled == 0 {
            let terminal_state = State::new(state.queued(), state.active(), true);
            return -self.policy.get(&terminal_state).payoff;
        }
        (dice_rolled..=self.sides() * dice_rolled).fold(0.0, |acc, dice_total| {
            let probability: f64 = self.pmfs.lookup(dice_rolled, dice_total);
            let payoff = if state.active() + dice_total <= self.max() {
                let state = State::new(state.queued(), state.active() + dice_total, false);
                -self.policy.get(&state).payoff
            } else {
                -1.0
            };
            acc + probability * payoff
        })
    }
}

impl DpSolver {
    /// Output the complete policy in human-readable format to stdout.
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
    /// Export the policy to a CSV file for external analysis or visualization.
    ///
    /// Creates a CSV with columns: active, queued, last, n, payoff
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written to.
    pub fn csv(&self, path: &str) -> Result<(), csv::Error> {
        let mut writer = csv::Writer::from_path(path)?;

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
    /// Generate SVG visualizations of the optimal policy using R scripts.
    ///
    /// Creates temporary CSV data and executes the R visualization script to
    /// produce policy heatmaps and strategy visualizations. Requires R and
    /// necessary packages.
    ///
    /// # Errors
    ///
    /// Returns an error if R is not available, the script fails, or file I/O
    /// fails.
    pub fn svg(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary CSV file
        let temp_file = tempfile::NamedTempFile::new()?;
        let temp_path = temp_file.path();

        // Write CSV data to temporary file
        let mut writer = csv::Writer::from_path(temp_path)?;
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

impl Solver for DpSolver {
    /// Returns the ruleset used by the solver.
    fn ruleset(&self) -> Ruleset {
        self.ruleset.clone()
    }
    /// Returns the policy computed by the solver.
    fn policy(&mut self) -> Policy {
        self.solve();
        self.policy.clone()
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_solver_vs_known_optimal_strategies() {
        // Test solver against known optimal strategies for simple cases
        let mut solver = DpSolver::new(10, 2);
        solver.solve();

        // At max score, should never roll
        let max_state = State::new(10, 5, false);
        let action = solver.policy.get(&max_state);
        assert_eq!(action.n, 0, "At max score, should never roll");

        // When opponent is at max and we're behind in terminal state, must roll
        let must_roll_state = State::new(8, 10, true);
        let action = solver.policy.get(&must_roll_state);
        assert!(action.n > 0, "Must roll when behind in terminal state");
    }

    #[test]
    fn test_game_symmetry() {
        // Test that the game exhibits expected symmetry properties
        let mut solver = DpSolver::new(15, 3);
        solver.solve();

        // Test symmetry in normal states
        let state1 = State::new(8, 6, false);
        let state2 = State::new(6, 8, false);

        let action1 = solver.policy.get(&state1);
        let action2 = solver.policy.get(&state2);

        // While not perfectly symmetric due to turn order, payoffs should be roughly
        // opposite
        assert!(
            (action1.payoff + action2.payoff).abs() < 0.5,
            "Symmetric states should have roughly opposite payoffs"
        );
    }

    #[test]
    fn test_end_game_behavior() {
        let mut solver = DpSolver::new(30, 6);
        solver.solve();

        // Test behavior near end game
        let close_states = vec![
            State::new(25, 28, false), // Behind but close
            State::new(28, 25, false), // Ahead but close
            State::new(29, 29, false), // Tied near max
            State::new(30, 25, false), // At max, ahead
        ];

        for state in close_states {
            let action = solver.policy.get(&state);

            // All actions should be valid
            assert!(action.n <= 20, "End game actions should be reasonable");
            assert!(action.payoff >= -1.0 - 1e-10, "Payoffs should be valid");
            assert!(action.payoff <= 1.0 + 1e-10, "Payoffs should be valid");

            // At max score, should never roll
            if state.active() == 30 {
                assert_eq!(action.n, 0, "At max score, should never roll");
            }
        }
    }
}
