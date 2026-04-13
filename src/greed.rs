use std::fmt;
use std::ops::{Index, IndexMut};

/// The ruleset for a game of Greed.
///
/// The standard ruleset is `(100, 6)`.
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
    /// Create a new ruleset with the given maximum score and die size.
    ///
    /// Both values should be non-zero. A `sides` of 0 will cause division
    /// by zero in the optimizer; a `max` of 0 makes every roll a bust.
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

/// A game state in Greed, viewed from the perspective of the active player.
#[derive(Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State {
    /// The score of the player whose turn it is.
    active: u32,
    /// The score of the player whose turn is up next.
    queued: u32,
    /// Whether this is the last round of the game.
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

    /// Return the score of active player.
    #[must_use]
    pub const fn active(&self) -> u32 {
        self.active
    }

    /// Return the score of the queued player.
    #[must_use]
    pub const fn queued(&self) -> u32 {
        self.queued
    }

    /// Return whether this is the last round.
    #[must_use]
    pub const fn last(&self) -> bool {
        self.last
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "State({}, {}, {})",
            self.active,
            self.queued,
            if self.last { "terminal" } else { "normal" }
        )
    }
}

/// An (optimal) action for a single game state.
///
/// An action consists of a number of dice to roll (`n`) and a payoff
/// (`payoff`).
///
/// A payoff is a scaler in \[−1, 1\], a uniform scale where:
/// - `-1.0` = certain loss
/// - `0.0` = balanced (or guaranteed tie).
/// - `1.0` = certain victory
#[derive(Copy, Clone, Default)]
pub struct Action {
    /// The number of dice to roll, with 0 meaning stand/pass.
    n: u32,
    /// The expected payoff when following optimal strategy (-1.0 to 1.0).
    payoff: f64,
}

impl Action {
    /// Create a new optimal action with a given number of dice and expected
    /// payoff.
    #[must_use]
    pub const fn new(n: u32, payoff: f64) -> Self {
        let payoff = if payoff == 0.0 { 0.0 } else { payoff };
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

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.n == other.n && (self.payoff - other.payoff).abs() < 1e-12
    }
}

impl Eq for Action {}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Action(n: {}, payoff: {})", self.n, self.payoff)
    }
}

/// A Greed policy.
///
/// A *policy* is a mapping from game states to actions. An *optimal policy* is
/// a policy that maximizes the expected payoff for each game state.
///
/// # Memory layout
///
/// States are packed into a contiguous `Box<[Action]>` indexed by `active +
/// (max + 1) * queued + (max + 1)² * last`
///
/// This keeps states that share the same `queued` and `last` values
/// adjacent, which is friendly to the optimizer's inner loops (they iterate
/// over possible `active` outcomes for a fixed opponent score).
#[derive(Clone, Default, PartialEq, Eq)]
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
    /// Allocates space for all `2(max+1)²` possible states.
    #[must_use]
    pub fn new(max: u32) -> Self {
        let size = ((max + 1) * (max + 1) * 2) as usize;
        let policy = vec![Action::default(); size].into_boxed_slice();
        Self { policy, max }
    }

    /// Iterate over every `(State, Action)` pair in the table.
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

    /// Print the full policy to stdout in a human-readable format.
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

    /// Export the policy to a CSV file.
    ///
    /// The CSV has columns `active, queued, last, n, payoff` and one row per
    /// state. Parent directories are created automatically.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written to, or if
    /// the path has no parent directory component.
    ///
    /// # Panics
    ///
    /// Panics if the path does not have a parent.
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

impl fmt::Debug for Policy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for active in 0..=self.max {
            for queued in 0..=self.max {
                for last in [false, true] {
                    let state = State::new(active, queued, last);
                    map.entry(&state, &self[state]);
                }
            }
        }
        map.finish()
    }
}
