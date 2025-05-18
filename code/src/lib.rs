#![warn(clippy::pedantic)]

pub mod greed;
pub mod pmf;
pub mod solver;

pub use greed::{Configuration, Greed, State};
pub use pmf::fft_convolve;
pub use solver::{GreedSolver, OptimalAction};
