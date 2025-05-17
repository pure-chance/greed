use std::cmp::Ordering;

use csv::Writer;
use rayon::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::pmf::pmf;

/// A state of Greed.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State {
    /// The score of the player whose turn it is.
    active: u16,
    /// The score of the player whose turn is up next.
    queued: u16,
    /// Whether this is the last turn of the game.
    last: bool,
}

impl State {
    #[must_use]
    pub fn new(active: u16, queued: u16, last: bool) -> Self {
        State {
            active,
            queued,
            last,
        }
    }
}

/// An action to perform, with its corresponding rating.
#[derive(Debug, Copy, Clone, Default)]
pub struct Action {
    /// Dice to roll
    n: u16,
    /// Rating given a roll of `n` dice.
    rating: f64,
}

impl Action {
    #[must_use]
    pub fn new(n: u16, rating: f64) -> Self {
        Action { n, rating }
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
    /// Maximum score allowed.
    max: u16,
    /// The # of sides on each dice.
    sides: u16,
    /// Table of state-action pairs
    table: FxHashMap<State, Action>,
}

impl GreedSolver {
    /// Create a new `GreedSolver` instance
    #[must_use]
    pub fn new(max: u16, sides: u16) -> Self {
        GreedSolver {
            max,
            sides,
            table: FxHashMap::default(),
        }
    }
    /// Solve the game
    pub fn solve(&mut self) {
        // Reserve capacity for all possible states upfront
        self.table = FxHashMap::with_capacity_and_hasher(
            (self.max as usize + 1) * (self.max as usize + 1) * 2,
            FxBuildHasher,
        );
        // Solve all the terminal states (this must be done first).
        self.solve_terminal_states();
        // Solve all the normal states (in the correct order).
        self.solve_normal_states();
    }
}

impl GreedSolver {
    /// Solve terminal states
    pub fn solve_terminal_states(&mut self) {
        let states: Vec<_> = (0..=self.max)
            .flat_map(|turn| (0..=self.max).map(move |next| State::new(turn, next, true)))
            .collect();

        let actions: Vec<_> = states
            .par_iter()
            .map(|state| (*state, self.find_optimal_terminal_action(*state)))
            .collect();

        for (state, action) in actions {
            self.table.insert(state, action);
        }
    }
    /// Find the optimal terminal action for a given state
    ///
    /// Because the optimal action is defined as having the highest probability of having `total` fall between `queued` and `max`, the distribution of `rating` with respect to `n` is unimodal. This means that when the active player is behind we can search from `n = min_non-zero_rating` up until the rating starts decreasing, and then stop. This is guaranteed to have found the optimal action.
    fn find_optimal_terminal_action(&self, state: State) -> Action {
        if state.active > state.queued {
            // If already ahead, doing nothing wins 100% of the time.
            return Action { n: 0, rating: 1.0 };
        }

        let mut optimal_action = Action::new(0, 0.0);
        let mut dice_rolled = (state.queued - state.active) / self.sides; // Start at min non-zero rating.

        loop {
            let current_rating = self.calc_terminal_rating(state, dice_rolled);
            if optimal_action.rating - current_rating >= 10e-4 {
                break;
            }
            if current_rating > optimal_action.rating {
                optimal_action = Action::new(dice_rolled, current_rating);
            }
            dice_rolled += 1;
        }

        optimal_action
    }
    /// Calculate the rating when in state `state` and rolling `dice_rolled` # of dice
    fn calc_terminal_rating(&self, state: State, dice_rolled: u16) -> f64 {
        if dice_rolled == 0 {
            return match state.active.cmp(&state.queued) {
                Ordering::Less => 0.0,
                Ordering::Equal => 0.5,
                Ordering::Greater => 1.0,
            };
        }
        println!("...");
        (dice_rolled..=self.sides * dice_rolled).fold(0.0, |acc, dice_total| {
            match (state.active + dice_total).cmp(&state.queued) {
                Ordering::Greater if state.active + dice_total <= self.max => {
                    acc + pmf(dice_total, dice_rolled, self.sides)
                }
                Ordering::Equal => acc + 0.5 * pmf(dice_total, dice_rolled, self.sides),
                _ => acc,
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
        for order in (0..=2 * self.max).rev() {
            // For each order, process places in parallel.
            let states_actions: Vec<(State, Action)> = (0..=order.min(2 * self.max - order))
                .into_par_iter() // Parallelize only within each order.
                .map(|place| {
                    // Calculate the player and opponent score for this order and place.
                    let (turn, next) = if order < self.max {
                        (order - place, place)
                    } else {
                        (self.max - place, (order - self.max) + place)
                    };
                    let state = State::new(turn, next, false);
                    let action = self.find_optimal_normal_action(state);
                    (state, action)
                })
                .collect();

            // Insert the results for this order into the table.
            for (state, action) in states_actions {
                self.table.insert(state, action);
            }
        }
    }
    /// Find the optimal normal action for a given state
    ///
    /// # Panics
    ///
    /// This presupposes that the terminal states have already been solved, and that all ratings with a higher order have already been calculated. Will panic if this invariant is not met.
    fn find_optimal_normal_action(&self, state: State) -> Action {
        let max_reasonable_n = 2 * (self.max - state.active) / (self.sides + 1) + 2; // +2 for safety of checking high enough
        let (optimal_roll, optimal_rating) = (0..=max_reasonable_n)
            .map(|dice_rolled| (dice_rolled, self.calc_normal_rating(state, dice_rolled)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        Action::new(optimal_roll, optimal_rating)
    }
    /// Calculate the rating when in state `state` and rolling `dice_rolled` # of dice
    ///
    /// # Panics
    ///
    /// This presupposes that the terminal states have already been solved, and that all ratings with a higher order have already been calculated. Will panic if this invariant is not met.
    #[must_use]
    pub fn calc_normal_rating(&self, state: State, dice_rolled: u16) -> f64 {
        if dice_rolled == 0 {
            let terminal_state = State::new(state.queued, state.active, true);
            return 1.0 - self.table.get(&terminal_state).unwrap().rating;
        }
        (dice_rolled..=self.sides * dice_rolled).fold(0.0, |acc, dice_total| {
            if state.active + dice_total < self.max {
                let probability: f64 = pmf(dice_total, dice_rolled, self.sides);
                let state = State::new(state.queued, state.active + dice_total, false);
                let rating: f64 = 1.0 - self.table.get(&state).unwrap().rating;
                acc + probability * rating
            } else {
                acc
            }
        })
    }
}

impl GreedSolver {
    /// Write the solver's table to a CSV file
    ///
    /// # Errors
    ///
    /// Returns an error if the CSV file cannot be written to.
    pub fn csv(&self, path: &str) -> Result<(), csv::Error> {
        let mut writer = Writer::from_path(path)?;

        // Write headers
        writer.serialize(("active", "queued", "last", "n", "rating"))?;
        for (state, action) in self.table.clone() {
            writer.serialize((
                state.active,
                state.queued,
                state.last,
                action.n,
                action.rating,
            ))?;
        }
        writer.flush()?;
        Ok(())
    }
    /// Write the solver's table to a human-readable format
    pub fn display(&self) {
        let mut terminal_states: Vec<_> =
            self.table.iter().filter(|(state, _)| state.last).collect();
        terminal_states.sort_by_key(|(state, _)| (state.active, state.queued));

        let mut normal_states: Vec<_> =
            self.table.iter().filter(|(state, _)| !state.last).collect();
        normal_states.sort_by_key(|(state, _)| (state.active, state.queued));

        // terminal states
        for (state, action) in terminal_states {
            println!(
                "({}, {}, terminal) => (dice: #{}, rating: {})",
                state.active, state.queued, action.n, action.rating
            );
        }
        println!();
        // normal states
        for (state, action) in normal_states {
            println!(
                "({}, {}, normal) => (dice: #{}, rating: {})",
                state.active, state.queued, action.n, action.rating
            );
        }
    }
}
