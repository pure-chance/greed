//! # Greed—Optimal Policy Solver
//!
//! Greed is a dice-based two-player game where players try to get as close to
//! the maximum score as possible without going bust. The player whose score is
//! higher at the end of play wins. This crate determines the optimal policy for
//! any game state via dynamic programming.
//!
//! ## Game Rules
//!
//! Players alternate turns, each choosing how many dice to roll. Each die is
//! numbered from 1 to *s* (typically 6). The total rolled on a turn is added to
//! that player's score. If a player's score exceeds the maximum threshold *M*
//! (typically 100), they bust (i.e., lose).
//!
//! Play continues until one player rolls 0 dice (stands), triggering the final
//! round: the opponent takes exactly one more turn. The player with the higher
//! score wins; equal scores result in a draw.
//!
//! ## Quick Start
//!
//! ```rust
//! use greed_solve::Solver;
//!
//! let mut solver = Solver::new(20, 4);
//! solver.solve();
//!
//! let policy = solver.policy();
//! policy.stdout();
//! ```

mod greed;
mod pmf;
mod solver;

pub use greed::{Action, Policy, Ruleset, State};
pub use pmf::PMFLookup;
pub use solver::Solver;
