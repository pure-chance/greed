//! The interface for a Greed `Solver`.
//!
//! The greed solver computes the optimal policy for a game of Greed with some
//! ruleset (m, s). It has two implementations, a dynamic programming solver
//! (dp), and a Reinforcement Learning solver (rl).

use crate::greed::{Policy, Ruleset};

/// A solver for the game of Greed.
///
/// The solver will find some "optimal" policy for greed with the given ruleset.
/// The term "optimal" is defined in context of the solver's design.
pub trait Solver {
    fn ruleset(&self) -> Ruleset;
    fn policy(&mut self) -> Policy;
}
