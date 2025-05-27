use std::cmp::Ordering;
use std::io::{stdin, Write};

use colored::Colorize;
use rand::{distr::Uniform, prelude::*};

const WIDTH: usize = 41; // based on banner width
const BANNER: &str = r#"
 ██████╗ ██████╗ ███████╗███████╗██████╗
██╔════╝ ██╔══██╗██╔════╝██╔════╝██╔══██╗
██║  ███╗██████╔╝█████╗  █████╗  ██║  ██║
██║   ██║██╔══██╗██╔══╝  ██╔══╝  ██║  ██║
╚██████╔╝██║  ██║███████╗███████╗██████╔╝
 ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝╚═════╝"#;

#[derive(Debug, Copy, Clone, Default)]
pub struct Configuration {
    /// Maximum score allowed.
    max: u16,
    /// The # of sides on each dice.
    sides: u16,
}

impl Configuration {
    #[must_use]
    pub fn new(max: u16, sides: u16) -> Self {
        Self { max, sides }
    }
    #[must_use]
    pub fn max(&self) -> u16 {
        self.max
    }
    #[must_use]
    pub fn sides(&self) -> u16 {
        self.sides
    }
}

/// A state of Greed.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State {
    /// The score of the player whose turn it is.
    active: u16,
    /// The score of the player whose turn is up next.
    queued: u16,
    /// Whether this is the last turn of the game.
    last: bool,
}

impl State {
    #[must_use]
    pub fn new(active: u16, queued: u16, last: bool) -> Self {
        State {
            active,
            queued,
            last,
        }
    }
    #[must_use]
    pub fn active(&self) -> u16 {
        self.active
    }
    #[must_use]
    pub fn queued(&self) -> u16 {
        self.queued
    }
    #[must_use]
    pub fn last(&self) -> bool {
        self.last
    }
}

/// A game of Greed
///
/// A game of Greed is a two-player dice game where players take turns rolling dice and accumulating points. The goal is to have a higher end score than the opponent without exceeding the maximum score (going bust).
///
/// # Rules
///
/// In each turn, a player can choose to roll 0+ dice. If they decide to roll a non-zero # of dice, they will roll, and the sum of their dice will be added to their score. If their score ever exceeds the maximum score, they go bust and lose the game.
///
/// If a player decides to roll 0 dice, they will not roll and their score will remain unchanged. This triggers the last round. The other player has one more opportunity to roll.
///
/// Whichever player has the highest (non-bust) score wins. If both players have the same score, the game is a draw.
pub struct Greed {
    rng: ThreadRng,
    configuration: Configuration,
    players: (String, String),
    state: State,
    turn: u16,
}

impl Greed {
    #[must_use]
    pub fn new(max: u16, sides: u16, players: (&str, &str)) -> Self {
        Self::banner(max, sides);

        Self {
            rng: ThreadRng::default(),
            configuration: Configuration::new(max, sides),
            players: (players.0.to_string(), players.1.to_string()),
            state: State::new(0, 0, false),
            turn: 0,
        }
    }
    fn banner(max: u16, sides: u16) {
        let ruleset = format!("max score: {max}, sides: {sides}");
        let padding = (WIDTH.saturating_sub(ruleset.len())) / 2;

        println!("{}", BANNER);
        println!("{pad}{ruleset}", pad = " ".repeat(padding));
    }
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
    fn results(&self) {
        println!();
        println!("{}", "=".repeat(WIDTH));
        println!("{pad}final results", pad = " ".repeat((WIDTH - 13) / 2));
        println!("{}", "=".repeat(WIDTH));

        let winners: &[&String] = if self.state.queued() > self.configuration.max {
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
    fn active_player(&self) -> &str {
        if self.turn % 2 == 0 {
            &self.players.0
        } else {
            &self.players.1
        }
    }
    fn queued_player(&self) -> &str {
        if self.turn % 2 == 0 {
            &self.players.1
        } else {
            &self.players.0
        }
    }
    fn player_0(&self) -> u16 {
        if self.turn % 2 == 0 {
            self.state.active()
        } else {
            self.state.queued()
        }
    }
    fn player_1(&self) -> u16 {
        if self.turn % 2 == 0 {
            self.state.queued()
        } else {
            self.state.active()
        }
    }
    fn roll(&mut self, n: u16) -> bool {
        let sum = (0..n).fold(0, |acc, _| {
            acc + self
                .rng
                .sample(Uniform::new(1, self.configuration.sides).unwrap())
        });
        self.turn += 1;
        if self.state.last {
            self.state = State::new(self.state.queued(), self.state.active() + sum, true);
            self.results();
            return true;
        }
        self.state = State::new(self.state.queued(), self.state.active() + sum, n == 0);
        if self.state.queued() > self.configuration.max() {
            self.results();
            return true;
        }
        false
    }
    pub fn play(max: u16, sides: u16, players: (&str, &str)) {
        let mut greed = Greed::new(max, sides, players);

        loop {
            println!();
            greed.game_state();

            // Get number of dice
            let mut input = String::new();
            print!("{} rolls: ", greed.active_player().green());
            std::io::stdout().flush().unwrap();
            stdin().read_line(&mut input).unwrap();
            let n = input.trim().parse::<u16>().unwrap();

            // Roll dice
            if greed.roll(n) {
                break;
            }
        }
    }
}
