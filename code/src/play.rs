//! Interactive game runner for Greed.
//!
//! Allows two players to play the game interactively via a cli game.

use std::cmp::Ordering;
use std::io::{Write, stdin};

use colored::Colorize;
use rand::{distr::Uniform, prelude::*};

use crate::{Ruleset, State};

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
        let active = format!("{}: {}", self.active_player().white(), self.state.active());
        let queued = format!("{}: {}", self.queued_player().black(), self.state.queued());
        println!(
            "round {round}: {active}, {queued}, last: {last}",
            round = self.turn,
            active = active.bold(),
            queued = queued.italic(),
            last = self.state.last
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
                    self.player_0().to_string().yellow(),
                    self.players.1,
                    self.player_1().to_string().red()
                );
            } else {
                println!(
                    "{}: {}, {}: {}",
                    self.players.0,
                    self.player_0().to_string().red(),
                    self.players.1,
                    self.player_1().to_string().yellow()
                );
            }
            if self.turn % 2 == 0 {
                &[&self.players.0]
            } else {
                &[&self.players.1]
            }
        } else {
            match self.player_0().cmp(&self.player_1()) {
                Ordering::Greater => {
                    println!(
                        "{}: {}, {}: {}",
                        self.players.0,
                        self.player_0().to_string().yellow(),
                        self.players.1,
                        self.player_1().to_string().white()
                    );
                    &[&self.players.0]
                }
                Ordering::Less => {
                    println!(
                        "{}: {}, {}: {}",
                        self.players.0,
                        self.player_0().to_string().white(),
                        self.players.1,
                        self.player_1().to_string().yellow()
                    );
                    &[&self.players.1]
                }
                Ordering::Equal => {
                    println!(
                        "{}: {}, {}: {}",
                        self.players.0,
                        self.player_0().to_string().yellow(),
                        self.players.1,
                        self.player_1().to_string().yellow()
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
    fn player_0(&self) -> u32 {
        if self.turn % 2 == 0 {
            self.state.active()
        } else {
            self.state.queued()
        }
    }
    /// Get the queued player's score.
    fn player_1(&self) -> u32 {
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
