use std::cmp::Ordering;

use csv::Writer;
use rayon::prelude::*;

use crate::greed::{Configuration, State};
use crate::pmf::fft_convolve;

/// An action to perform, with its corresponding rating.
#[derive(Debug, Copy, Clone, Default)]
pub struct OptimalAction {
    /// Dice to roll
    n: u16,
    /// Rating given a roll of `n` dice.
    payoff: f64,
}

impl OptimalAction {
    #[must_use]
    pub fn new(n: u16, rating: f64) -> Self {
        OptimalAction { n, payoff: rating }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Policy {
    terminal: Vec<Vec<OptimalAction>>,
    normal: Vec<Vec<OptimalAction>>,
}

impl Policy {
    #[must_use]
    pub fn new(max: u16) -> Self {
        let size = usize::from(max + 1);
        let terminal = vec![vec![OptimalAction::default(); size]; size];
        let normal = vec![vec![OptimalAction::default(); size]; size];
        Self { terminal, normal }
    }
    #[must_use]
    pub fn get(&self, state: &State) -> OptimalAction {
        if state.last() {
            self.terminal[state.active() as usize][state.queued() as usize]
        } else {
            self.normal[state.active() as usize][state.queued() as usize]
        }
    }
    pub fn set(&mut self, state: &State, action: OptimalAction) {
        if state.last() {
            self.terminal[state.active() as usize][state.queued() as usize] = action;
        } else {
            self.normal[state.active() as usize][state.queued() as usize] = action;
        }
    }
}

impl Policy {
    pub fn iter(&self) -> impl Iterator<Item = (State, OptimalAction)> + '_ {
        let terminal_iter = self.terminal.iter().enumerate().flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .map(move |(j, &action)| (State::new(i as u16, j as u16, true), action))
        });

        let normal_iter = self.normal.iter().enumerate().flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .map(move |(j, &action)| (State::new(i as u16, j as u16, false), action))
        });

        terminal_iter.chain(normal_iter)
    }
}

impl IntoIterator for Policy {
    type Item = (State, OptimalAction);
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        let terminal_iter = self.terminal.into_iter().enumerate().flat_map(|(i, row)| {
            row.into_iter()
                .enumerate()
                .map(move |(j, action)| (State::new(i as u16, j as u16, true), action))
        });

        let normal_iter = self.normal.into_iter().enumerate().flat_map(|(i, row)| {
            row.into_iter()
                .enumerate()
                .map(move |(j, action)| (State::new(i as u16, j as u16, false), action))
        });

        Box::new(terminal_iter.chain(normal_iter))
    }
}

/// A solver for Greed
///
/// A game of Greed is a two-player dice game where players take turns rolling dice and accumulating points.
/// The goal is to have a higher end score than the opponent without exceeding the maximum score (going bust).
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
/// Starting with the maximum `turn + next` score, the only option (other than going bust) is to roll 0, thus its possible to calculate the optimal action by looking up the action for the corresponding terminal state.
///
/// Moving to the second maximum `turn + next` score, the only options are to either end up in the previously computed normal state, or the corresponding terminal state.
///
/// This pattern continues until we reach the minimum `turn + next` score. All normal states are now fully computed.
#[derive(Debug, Clone, Default)]
pub struct GreedSolver {
    /// A configuration of Greed (max, sides)
    configuration: Configuration,
    /// The solvers policy (state-action pairs)
    policy: Policy,
    /// Precomputed PMFs
    pmfs: Vec<Vec<f64>>,
}

impl GreedSolver {
    /// Create a new `GreedSolver` instance
    #[must_use]
    pub fn new(max: u16, sides: u16) -> Self {
        GreedSolver {
            configuration: Configuration::new(max, sides),
            policy: Policy::new(max),
            pmfs: Self::precompute_pmfs(max, sides),
        }
    }
    #[must_use]
    pub fn precompute_pmfs(max: u16, sides: u16) -> Vec<Vec<f64>> {
        let max_n = (2 * max / (sides + 1) + 1).max(max + 1);

        let mut pmfs: Vec<Vec<f64>> = Vec::with_capacity(max as usize + 1);
        let dice_pmf = vec![1.0 / f64::from(sides); sides as usize];

        pmfs.push(vec![1.0]);
        for n in 1..=max_n {
            pmfs.push(fft_convolve(&pmfs[(n - 1) as usize], &dice_pmf));
        }
        pmfs
    }
    /// Solve the game
    pub fn solve(&mut self) {
        // Solve all the terminal states (this must be done first).
        self.solve_terminal_states();
        // Solve all the normal states (in the correct order).
        self.solve_normal_states();
    }
    #[must_use]
    pub fn max(&self) -> u16 {
        self.configuration.max()
    }
    #[must_use]
    pub fn sides(&self) -> u16 {
        self.configuration.sides()
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
    /// Because the optimal action is defined as having the highest probability of having `total` fall between `queued` and `max`, the distribution of `rating` with respect to `n` is unimodal. This means that when the active player is behind we can search from `n = min_non-zero_rating` up until the rating starts decreasing, and then stop. This is guaranteed to have found the optimal action.
    fn find_optimal_terminal_action(&self, state: State) -> OptimalAction {
        if state.active() > state.queued() {
            // If already ahead, doing nothing wins 100% of the time.
            return OptimalAction { n: 0, payoff: 1.0 };
        }

        let mut optimal_action = OptimalAction::new(0, -1.0);
        let mut dice_rolled = (state.queued() - state.active()) / self.sides(); // Start at min non-zero rating.

        loop {
            let current_payoff = self.calc_terminal_rating(state, dice_rolled);
            if optimal_action.payoff - current_payoff >= 10e-4 {
                break;
            }
            if current_payoff > optimal_action.payoff {
                optimal_action = OptimalAction::new(dice_rolled, current_payoff);
            }
            dice_rolled += 1;
        }

        optimal_action
    }
    /// Calculate the rating when in state `state` and rolling `dice_rolled` # of dice
    fn calc_terminal_rating(&self, state: State, dice_rolled: u16) -> f64 {
        if dice_rolled == 0 {
            return match state.active().cmp(&state.queued()) {
                Ordering::Less => -1.0,
                Ordering::Equal => 0.0,
                Ordering::Greater => 1.0,
            };
        }

        (dice_rolled..=self.sides() * dice_rolled).fold(0.0, |acc, dice_total| {
            let p_for_total = self.pmfs[dice_rolled as usize][(dice_total - dice_rolled) as usize];
            match (state.active() + dice_total).cmp(&state.queued()) {
                Ordering::Greater if state.active() + dice_total <= self.max() => acc + p_for_total,
                Ordering::Less | Ordering::Greater => acc - p_for_total, // lower score or bust
                Ordering::Equal => acc,                                  // tie
            }
        })
    }
}

impl GreedSolver {
    /// Solve normal states
    ///
    /// # Panics
    ///
    /// This presupposes that the terminal states have already been solved. Will panic if this invariant is not met.
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
    /// # Panics
    ///
    /// This presupposes that the terminal states have already been solved, and that all ratings with a higher order have already been calculated. Will panic if this invariant is not met.
    fn find_optimal_normal_action(&self, state: State) -> OptimalAction {
        let max_reasonable_n = 2 * (self.max() - state.active()) / (self.sides() + 1) + 3; // +1 for safety of checking high enough
        let (optimal_roll, optimal_rating) = (0..=max_reasonable_n)
            .map(|dice_rolled| (dice_rolled, self.calc_normal_rating(state, dice_rolled)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        OptimalAction::new(optimal_roll, optimal_rating)
    }
    /// Calculate the rating when in state `state` and rolling `dice_rolled` # of dice
    ///
    /// # Panics
    ///
    /// This presupposes that the terminal states have already been solved, and that all ratings with a higher order have already been calculated. Will panic if this invariant is not met.
    #[must_use]
    pub fn calc_normal_rating(&self, state: State, dice_rolled: u16) -> f64 {
        if dice_rolled == 0 {
            let terminal_state = State::new(state.queued(), state.active(), true);
            return -self.policy.get(&terminal_state).payoff;
        }
        (dice_rolled..=self.sides() * dice_rolled).fold(0.0, |acc, dice_total| {
            if state.active() + dice_total < self.max() {
                let probability: f64 =
                    self.pmfs[dice_rolled as usize][(dice_total - dice_rolled) as usize];
                let state = State::new(state.queued(), state.active() + dice_total, false);
                let rating: f64 = -self.policy.get(&state).payoff;
                acc + probability * rating
            } else {
                acc
            }
        })
    }
}

impl GreedSolver {
    /// Write the solver's policy to a CSV file
    ///
    /// # Errors
    ///
    /// Returns an error if the CSV file cannot be written to.
    pub fn csv(&self, path: &str) -> Result<(), csv::Error> {
        let mut writer = Writer::from_path(path)?;

        // Write headers
        writer.serialize(("active", "queued", "last", "n", "rating"))?;
        for (state, action) in self.policy.clone() {
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
    /// Write the solver's policy to a human-readable format
    pub fn display(&self) {
        let mut all_states: Vec<_> = self.policy.iter().collect();
        all_states.sort_by_key(|(state, _)| (state.last(), state.active(), state.queued()));

        let (terminal_states, normal_states): (Vec<_>, Vec<_>) =
            all_states.into_iter().partition(|(state, _)| state.last());

        // terminal states
        for (state, action) in terminal_states {
            println!(
                "({}, {}, terminal) => (dice: #{}, rating: {})",
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
                "({}, {}, normal) => (dice: #{}, rating: {})",
                state.active(),
                state.queued(),
                action.n,
                action.payoff
            );
        }
    }
}
