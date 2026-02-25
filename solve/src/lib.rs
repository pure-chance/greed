//! # Greed—Optimal Policy Solver
//!
//! ## Background
//!
//! Greed is a dice-based two-player game where players try to get as close to
//! the maximum score as possible without going bust. The player whose score is
//! higher at the end of play wins. This project implements a dynamic
//! programming solution to determine the optimal policy for any game state.
//!
//! ## Game Rules
//!
//! In this game, players alternate turns, each choosing to roll as many dice as
//! they like. Each die is numbered from 1 to n, typically 6. The total rolled
//! on a turn is added to that player’s score. However, if a player’s score ever
//! exceeds the maximum threshold, typically 100, they bust and immediately lose
//! the game.
//!
//! Play continues back and forth until one player decides to roll 0 dice,
//! signaling the beginning of the last round. The opposing player then has one
//! final opportunity to roll, following the same rules. Once this last turn is
//! completed, the game ends. The player with the higher score wins; if both
//! players have the same score, the game is declared a draw.

mod greed;
mod pmf;
mod solver;

pub use greed::{Action, Policy, Ruleset, State};
pub use pmf::PMFLookup;
pub use solver::Solver;
