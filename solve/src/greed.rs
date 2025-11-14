use std::ops::{Index, IndexMut};

/// Game rules for Greed.
///
/// Defines the maximum allowable score and the number of sides on each die.
/// The standard ruleset is (100, 6) representing a maximum score of 100 with
/// 6-sided dice.
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
    pub const fn new(max: u32, sides: u32) -> Self {
        Self { max, sides }
    }

    /// Get the maximum score allowed before busting.
    #[must_use]
    pub const fn max(&self) -> u32 {
        self.max
    }

    /// Get the number of sides on each die.
    #[must_use]
    pub const fn sides(&self) -> u32 {
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
    pub const fn new(active: u32, queued: u32, last: bool) -> Self {
        Self {
            active,
            queued,
            last,
        }
    }

    /// Return the score of actively rolling player.
    #[must_use]
    pub const fn active(&self) -> u32 {
        self.active
    }

    /// Return the score queued player.
    #[must_use]
    pub const fn queued(&self) -> u32 {
        self.queued
    }

    /// Return the flag indicating whether this is the final round of the game.
    #[must_use]
    pub const fn last(&self) -> bool {
        self.last
    }
}

/// An (optimal) action for a given game state, containing the number of dice to
/// roll and expected payoff.
///
/// The payoff represents the expected value (probability of winning minus
/// probability of losing) when following the optimal strategy from this state.
/// Values range from -1.0 (certain loss) to 1.0 (certain win), with 0.0
/// representing equal chances.
#[derive(Debug, Copy, Clone, Default)]
pub struct Action {
    /// The number of dice to roll (0 means stand/pass).
    n: u32,
    /// The expected payoff when following optimal strategy (-1.0 to 1.0).
    payoff: f64,
}

impl Action {
    /// Create a new optimal action with a given number of dice and expected
    /// payoff.
    #[must_use]
    pub const fn new(n: u32, payoff: f64) -> Self {
        Self { n, payoff }
    }

    /// Get the number of dice to roll.
    #[must_use]
    pub const fn n(&self) -> u32 {
        self.n
    }

    /// Get the expected payoff.
    #[must_use]
    pub const fn payoff(&self) -> f64 {
        self.payoff
    }
}

/// Stores the policy for a Greed game as a lookup table.
///
/// Maps every possible game state to its optimal action. The policy covers both
/// terminal states (final round) and normal states, storing them in a
/// cache-efficient flat array structure.
///
/// # Memory Layout
///
/// States are stored in a single contiguous array indexed by: active + (max+1)
/// * queued + (max+1)^2 * last
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

    /// Returns an iterator over all state-action pairs in the policy.
    pub fn iter(&self) -> impl Iterator<Item = (State, Action)> + '_ {
        (0..=self.max).flat_map(move |active| {
            (0..=self.max).flat_map(move |queued| {
                [false, true].into_iter().map(move |last| {
                    let state = State::new(active, queued, last);
                    let action = self[state];
                    (state, action)
                })
            })
        })
    }

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
    ///
    /// # Panics
    ///
    /// Panics if file path does not have a parent directory.
    pub fn csv(&self, path: &str) -> Result<(), csv::Error> {
        std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap())?;
        let mut writer = csv::Writer::from_path(path)?;
        writer.serialize(("active", "queued", "last", "n", "payoff"))?; // serialize headers
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
}

impl Index<State> for Policy {
    type Output = Action;
    fn index(&self, s: State) -> &Self::Output {
        let stride = self.max + 1;
        let index = s.active() + stride * s.queued() + stride * stride * u32::from(s.last());
        &self.policy[index as usize]
    }
}

impl IndexMut<State> for Policy {
    fn index_mut(&mut self, s: State) -> &mut Self::Output {
        let stride = self.max + 1;
        let index = s.active() + stride * s.queued() + stride * stride * u32::from(s.last());
        &mut self.policy[index as usize]
    }
}
