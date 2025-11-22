use std::cmp::Ordering;
use std::io::{Write, stdin};

use clap::{Arg, Command};
use colored::Colorize;
use rand::{distr::Uniform, prelude::*};

fn main() {
    let cli = Command::new("play")
        .about("An interactive game of Greed")
        .arg(
            Arg::new("max")
                .short('m')
                .long("max")
                .value_name("MAX")
                .help("Maximum score")
                .value_parser(clap::value_parser!(u32))
                .default_value("100"),
        )
        .arg(
            Arg::new("sides")
                .short('s')
                .long("sides")
                .value_name("SIDES")
                .help("Number of sides on each die")
                .value_parser(clap::value_parser!(u32))
                .default_value("6"),
        );

    let args = cli.get_matches();

    let max = *args.get_one::<u32>("max").unwrap();
    let sides = *args.get_one::<u32>("sides").unwrap();

    Greed::play(max, sides, ("P1", "P2"));
}

const WIDTH: usize = 41; // based on banner width
const BANNER: &str = r"
 ██████╗ ██████╗ ███████╗███████╗██████╗
██╔════╝ ██╔══██╗██╔════╝██╔════╝██╔══██╗
██║  ███╗██████╔╝█████╗  █████╗  ██║  ██║
██║   ██║██╔══██╗██╔══╝  ██╔══╝  ██║  ██║
╚██████╔╝██║  ██║███████╗███████╗██████╔╝
 ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝╚═════╝";

/// Interactive game runner for Greed.
pub struct Greed {
    rng: ThreadRng,
    ruleset: Ruleset,
    players: (String, String),
    state: State,
    turn: u32,
}

impl Greed {
    /// Create a new `Greed` game.
    #[must_use]
    pub fn new(max: u32, sides: u32, players: (&str, &str)) -> Self {
        Self::banner(max, sides);

        Self {
            rng: ThreadRng::default(),
            ruleset: Ruleset::new(max, sides),
            players: (players.0.to_string(), players.1.to_string()),
            state: State::new(0, 0, false),
            turn: 0,
        }
    }

    /// Print the game banner.
    fn banner(max: u32, sides: u32) {
        let ruleset = format!("max score: {max}, sides: {sides}");
        let padding = (WIDTH.saturating_sub(ruleset.len())) / 2;

        println!("{BANNER}");
        println!("{pad}{ruleset}", pad = " ".repeat(padding));
    }

    /// Print the game state.
    fn game_state(&self) {
        let final_flag = if self.state.last { " [FINAL]" } else { "" };
        println!(
            "round {}: {}: {}, {}: {}{}",
            self.turn,
            self.active_player().white(),
            self.state.active(),
            self.queued_player().black().italic(),
            self.state.queued(),
            final_flag
        );
    }

    /// Print the game results.
    fn results(&self) {
        println!();
        println!("{}", "=".repeat(WIDTH));
        println!("{pad}final results", pad = " ".repeat((WIDTH - 13) / 2));
        println!("{}", "=".repeat(WIDTH));

        let winners: &[&String] = if self.state.queued() > self.ruleset.max {
            if self.turn % 2 == 0 {
                println!(
                    "{}: {}, {}: {}",
                    self.players.0,
                    self.player_1().to_string().yellow(),
                    self.players.1,
                    self.player_2().to_string().red()
                );
            } else {
                println!(
                    "{}: {}, {}: {}",
                    self.players.0,
                    self.player_1().to_string().red(),
                    self.players.1,
                    self.player_2().to_string().yellow()
                );
            }
            if self.turn % 2 == 0 {
                &[&self.players.0]
            } else {
                &[&self.players.1]
            }
        } else {
            match self.player_1().cmp(&self.player_2()) {
                Ordering::Greater => {
                    println!(
                        "{}: {}, {}: {}",
                        self.players.0,
                        self.player_1().to_string().yellow(),
                        self.players.1,
                        self.player_2().to_string().white()
                    );
                    &[&self.players.0]
                }
                Ordering::Less => {
                    println!(
                        "{}: {}, {}: {}",
                        self.players.0,
                        self.player_1().to_string().white(),
                        self.players.1,
                        self.player_2().to_string().yellow()
                    );
                    &[&self.players.1]
                }
                Ordering::Equal => {
                    println!(
                        "{}: {}, {}: {}",
                        self.players.0,
                        self.player_1().to_string().yellow(),
                        self.players.1,
                        self.player_2().to_string().yellow()
                    );
                    &[&self.players.0, &self.players.1]
                }
            }
        };

        if winners.len() == 1 {
            println!("{} wins!", winners[0]);
        } else {
            println!("{} and {} tie!", winners[0], winners[1]);
        }
    }

    /// Get the active player's name.
    fn active_player(&self) -> &str {
        if self.turn % 2 == 0 {
            &self.players.0
        } else {
            &self.players.1
        }
    }

    /// Get the queued player's name.
    fn queued_player(&self) -> &str {
        if self.turn % 2 == 0 {
            &self.players.1
        } else {
            &self.players.0
        }
    }

    /// Get the active player's score.
    fn player_1(&self) -> u32 {
        if self.turn % 2 == 0 {
            self.state.active()
        } else {
            self.state.queued()
        }
    }

    /// Get the queued player's score.
    fn player_2(&self) -> u32 {
        if self.turn % 2 == 0 {
            self.state.queued()
        } else {
            self.state.active()
        }
    }

    /// Simulate rolling `n` dice.
    fn roll(&mut self, n: u32) -> bool {
        let sum = (0..n).fold(0, |acc, _| {
            acc + self
                .rng
                .sample(Uniform::new(1, self.ruleset.sides).unwrap())
        });
        self.turn += 1;
        if self.state.last {
            self.state = State::new(self.state.queued(), self.state.active() + sum, true);
            self.results();
            return true;
        }
        self.state = State::new(self.state.queued(), self.state.active() + sum, n == 0);
        if self.state.queued() > self.ruleset.max() {
            self.results();
            return true;
        }
        false
    }

    /// Start an interactive game of Greed between two players.
    ///
    /// Players take turns entering the number of dice to roll. The game
    /// continues until one player busts or both players have stood (rolled
    /// 0 dice).
    ///
    /// # Panics
    ///
    /// Panics if stdin input cannot be read or parsed as a valid number.
    pub fn play(max: u32, sides: u32, players: (&str, &str)) {
        let mut greed = Greed::new(max, sides, players);

        loop {
            println!();
            greed.game_state();

            // Get number of dice
            let mut input = String::new();
            print!("{} rolls: ", greed.active_player().green());
            std::io::stdout().flush().unwrap();
            stdin().read_line(&mut input).unwrap();
            let n = input.trim().parse::<u32>().unwrap();

            // Roll dice
            if greed.roll(n) {
                break;
            }
        }
    }
}

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
