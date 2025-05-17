#![warn(clippy::pedantic)]

pub mod pmf;
pub mod solver;

pub use pmf::pmf;
pub use solver::{Action, GreedSolver, State};
