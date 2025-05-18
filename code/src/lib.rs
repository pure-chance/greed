#![warn(clippy::pedantic)]

pub mod pmf;
pub mod solver;

pub use pmf::fft_convolve;
pub use solver::{Action, GreedSolver, State};
