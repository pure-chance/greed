//! # A Policy Optimization of Greed
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
//! use greed::{PolicyOptimizer, Ruleset};
//!
//! let ruleset = Ruleset::new(100, 6);
//! let optimizer = PolicyOptimizer::optimize(ruleset);
//!
//! let policy = optimizer.policy();
//! policy.stdout();
//! ```

mod greed;
mod optimizer;
mod pmf;

pub use greed::{Action, Policy, Ruleset, State};
pub use optimizer::PolicyOptimizer;
pub use pmf::PMFLookup;
