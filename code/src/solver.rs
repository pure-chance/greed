//! The interface for a Greed `Solver`.

use std::process::Command;

use crate::{Action, Ruleset, State};

/// Stores the policy for a Greed game as a lookup table.
///
/// Maps every possible game state to its optimal action. The policy covers both
/// terminal states (final round) and normal states, storing them in a
/// cache-efficient flat array structure.
///
/// # Memory Layout
///
/// States are stored in a single contiguous array indexed by: `active + (max+1)
/// * queued + (max+1)^2 * last`
///
/// This layout improves cache performance by keeping related states close
/// together.
#[derive(Debug, Clone, Default)]
pub struct Policy {
    /// The optimal action for each state.
    policy: Box<[Action]>,
    /// The maximum score.
    ///
    /// This is used for properly indexing the policy table.
    max: u32,
}

impl Policy {
    /// Creates a new empty policy table for the given maximum score.
    ///
    /// Allocates space for all possible states: (max+1)² normal states +
    /// (max+1)² terminal states.
    #[must_use]
    pub fn new(max: u32) -> Self {
        let size = ((max + 1) * (max + 1) * 2) as usize;
        let policy = vec![Action::default(); size].into_boxed_slice();
        Self { policy, max }
    }
    /// Returns the index of a state in the policy table.
    #[inline]
    fn index(&self, state: &State) -> usize {
        let stride = self.max + 1;
        let placement = state.active() + stride * state.queued();
        let last_offset = stride * stride * u32::from(state.last());
        (placement + last_offset) as usize
    }
    /// Retrieve the optimal action for a given game state.
    #[must_use]
    #[inline]
    pub fn get(&self, state: &State) -> Action {
        self.policy[self.index(state)]
    }
    /// Store the optimal action for a given game state.
    #[inline]
    pub fn set(&mut self, state: &State, action: Action) {
        let idx = self.index(state);
        self.policy[idx] = action;
    }
    /// Iterate over all computed state-action pairs in the policy.
    ///
    /// Yields tuples of (state, optimal_action) for every state in the game.
    /// Useful for analysis, visualization, and policy export.
    ///
    /// # Panics
    ///
    /// Panics if the state space is too large to fit in a `u32`.
    pub fn iter(&self) -> impl Iterator<Item = (State, Action)> + '_ {
        self.policy.iter().enumerate().map(|(placement, action)| {
            let placement = u32::try_from(placement).expect("state space too big");
            let last_offset = (self.max + 1) * (self.max + 1);
            let (active, queued, last) = if placement >= last_offset {
                let adjusted_placement = placement - last_offset;
                let active = adjusted_placement % (self.max + 1);
                let queued = adjusted_placement / (self.max + 1);
                (active, queued, true)
            } else {
                let active = placement % (self.max + 1);
                let queued = placement / (self.max + 1);
                (active, queued, false)
            };
            (State::new(active, queued, last), *action)
        })
    }
}

impl Policy {
    /// Output the complete policy in human-readable format to stdout.
    ///
    /// Prints all state-action pairs sorted by state type and scores, useful
    /// for analysis and debugging.
    pub fn stdout(&self) {
        let mut state_action_pairs: Vec<(State, Action)> = self.iter().collect();
        state_action_pairs.sort_by_key(|(state, _)| (state.last(), state.active(), state.queued()));

        let (terminal_states, normal_states): (Vec<_>, Vec<_>) = state_action_pairs
            .into_iter()
            .partition(|(state, _)| state.last());

        // terminal states
        for (state, action) in terminal_states {
            println!(
                "({}, {}, terminal) => (dice: #{}, payoff: {})",
                state.active(),
                state.queued(),
                action.n(),
                action.payoff()
            );
        }
        println!();
        // normal states
        for (state, action) in normal_states {
            println!(
                "({}, {}, normal) => (dice: #{}, payoff: {})",
                state.active(),
                state.queued(),
                action.n(),
                action.payoff()
            );
        }
    }
    /// Export the policy to a CSV file for external analysis or visualization.
    ///
    /// Creates a CSV with columns: active, queued, last, n, payoff
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written to.
    pub fn csv(&self, path: &str) -> Result<(), csv::Error> {
        let mut writer = csv::Writer::from_path(path)?;

        // Write headers
        writer.serialize(("active", "queued", "last", "n", "payoff"))?;
        for (state, action) in self.iter() {
            writer.serialize((
                state.active(),
                state.queued(),
                state.last(),
                action.n(),
                action.payoff(),
            ))?;
        }
        writer.flush()?;
        Ok(())
    }
    /// Generate SVG visualizations of the optimal policy using R scripts.
    ///
    /// Creates temporary CSV data and executes the R visualization script to
    /// produce policy heatmaps and strategy visualizations. Requires R and
    /// necessary packages.
    ///
    /// # Errors
    ///
    /// Returns an error if R is not available, the script fails, or file I/O
    /// fails.
    pub fn svg(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary CSV file
        let temp_file = tempfile::NamedTempFile::new()?;
        let temp_path = temp_file.path();

        // Write CSV data to temporary file
        let mut writer = csv::Writer::from_path(temp_path)?;
        writer.serialize(("active", "queued", "last", "n", "payoff"))?;
        for (state, action) in self.iter() {
            writer.serialize((
                state.active(),
                state.queued(),
                state.last(),
                action.n(),
                action.payoff(),
            ))?;
        }
        writer.flush()?;

        let output = Command::new("Rscript")
            .arg("optimal_policies.R")
            .arg(temp_path)
            .current_dir("visualize")
            .output()?;

        if output.status.success() {
            if !output.stdout.is_empty() {
                println!("R output: {}", String::from_utf8_lossy(&output.stdout));
            }
        } else {
            eprintln!("R script failed with exit code: {:?}", output.status.code());
            if !output.stderr.is_empty() {
                eprintln!("R error: {}", String::from_utf8_lossy(&output.stderr));
            }
            return Err("R script execution failed".into());
        }

        Ok(())
    }
}

/// A solver for the game of Greed.
///
/// The solver will find some "optimal" policy for greed with the given ruleset.
/// The term "optimal" is defined in context of the solver's design.
pub trait Solver {
    fn ruleset(&self) -> Ruleset;
    fn policy(&mut self) -> Policy;
}

pub enum OutputFormat {
    Stdout,
    Csv,
    Svg,
}
