#![warn(clippy::pedantic)]

pub mod greed;
pub mod pmf;
pub mod solver;

pub use greed::{Greed, Ruleset, State};
pub use pmf::fft_convolve;
pub use solver::{GreedSolver, OptimalAction};
