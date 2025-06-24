//! Greed: Optimal Policy Solver for a Two-Player Dice Game
//!
//! This crate implements a complete solution for the game of Greed, including both
//! an interactive game runner and a dynamic programming solver that computes
//! mathematically optimal strategies.
//!
//! # Game Overview
//!
//! Greed is a risk-management dice game where players accumulate points by rolling
//! dice, trying to achieve the highest score without "busting" (exceeding the maximum).
//!
//! # Usage Examples
//!
//! ...

pub mod dp;
pub mod play;
pub mod solver;

pub use dp::DpSolver;
pub use play::Greed;
pub use solver::{Policy, Solver};

/// Game configuration parameters for Greed.
///
/// Defines the maximum allowable score and the number of sides on each die.
/// The standard ruleset is (100, 6) representing a maximum score of 100 with 6-sided dice.
#[derive(Debug, Copy, Clone)]
pub struct Ruleset {
    /// Maximum score allowed before busting (typically 100).
    max: u32,
    /// The number of sides on each die (typically 6).
    sides: u32,
}

impl Default for Ruleset {
    fn default() -> Self {
        Self { max: 100, sides: 6 }
    }
}

impl Ruleset {
    /// Create a new ruleset.
    #[must_use]
    pub fn new(max: u32, sides: u32) -> Self {
        Self { max, sides }
    }
    /// Get the maximum score allowed before busting.
    #[must_use]
    pub fn max(&self) -> u32 {
        self.max
    }
    /// Get the number of sides on each die.
    #[must_use]
    pub fn sides(&self) -> u32 {
        self.sides
    }
}

/// A game state in Greed, representing scores and turn information.
///
/// States are represented from the perspective of the current player:
/// - `active`: Current player's score
/// - `queued`: Next player's score
/// - `last`: Whether we're in the final round (triggered when a player stands)
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State {
    /// The score of the player whose turn it is.
    active: u32,
    /// The score of the player whose turn is up next.
    queued: u32,
    /// Whether this is the final round of the game.
    last: bool,
}

impl State {
    /// Create a new state.
    #[must_use]
    pub fn new(active: u32, queued: u32, last: bool) -> Self {
        State {
            active,
            queued,
            last,
        }
    }
    /// Get the score of the player whose turn it is.
    #[must_use]
    pub fn active(&self) -> u32 {
        self.active
    }
    /// Get the score of the player whose turn is up next.
    #[must_use]
    pub fn queued(&self) -> u32 {
        self.queued
    }
    #[must_use]
    pub fn last(&self) -> bool {
        self.last
    }
}

/// An (optimal) action for a given game state, containing the number of dice to roll and expected payoff.
///
/// The meaning of `payoff` is dependent on the type of solver.
///
/// For the `DpSolver`, the payoff represents the expected value (probability of winning minus probability of losing) when following the optimal strategy from this state. Values range from -1.0 (certain loss) to 1.0 (certain win), with 0.0 representing equal chances.
///
/// For the `RlSolver`, the payoff represents the expected reward when following the optimal strategy from this state. Values range from -1.0 (certain loss) to 1.0 (certain win), with 0.0 representing equal chances.
#[derive(Debug, Copy, Clone, Default)]
pub struct Action {
    /// The number of dice to roll (0 means stand/pass).
    n: u32,
    /// The expected payoff when following optimal strategy (-1.0 to 1.0).
    payoff: f64,
}

impl Action {
    /// Create a new optimal action with a given number of dice and expected payoff.
    #[must_use]
    pub fn new(n: u32, payoff: f64) -> Self {
        Self { n, payoff }
    }
    /// Get the number of dice to roll.
    #[must_use]
    pub fn n(&self) -> u32 {
        self.n
    }
    /// Get the expected payoff.
    #[must_use]
    pub fn payoff(&self) -> f64 {
        self.payoff
    }
}
