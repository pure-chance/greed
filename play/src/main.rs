use std::io::{Write, stdin, stdout};
use std::thread;
use std::time::Duration;

use clap::{Arg, Command};
use rand::distr::Uniform;
use rand::prelude::*;

fn main() {
    let cli = Command::new("greed")
        .about("Play a game of Greed")
        .arg(
            Arg::new("max")
                .short('m')
                .long("max")
                .value_parser(clap::value_parser!(u32))
                .default_value("100"),
        )
        .arg(
            Arg::new("sides")
                .short('s')
                .long("sides")
                .value_parser(clap::value_parser!(u32))
                .default_value("6"),
        )
        .arg(
            Arg::new("ai")
                .long("ai")
                .action(clap::ArgAction::SetTrue)
                .help("Play against AI"),
        )
        .arg(
            Arg::new("p1")
                .long("p1")
                .default_value("Alice")
                .help("Player 1 name"),
        )
        .arg(
            Arg::new("p2")
                .long("p2")
                .default_value("Blair")
                .help("Player 2 name"),
        );

    let args = cli.get_matches();
    let max = *args.get_one::<u32>("max").unwrap();
    let sides = *args.get_one::<u32>("sides").unwrap();
    let ai = args.get_flag("ai");
    let p1 = args.get_one::<String>("p1").unwrap().clone();
    let p2 = args.get_one::<String>("p2").unwrap().clone();

    let mut game = Game::new(max, sides, [p1, p2], [false, ai]);
    game.play();
}

enum Turn {
    Roll { dice: u32, sum: u32, bust: bool },
    Stand,
}

struct Game {
    names: [String; 2],
    is_ai: [bool; 2],
    scores: [u32; 2],
    cur: usize,
    last_round: bool,
    max: u32,
    sides: u32,
    rng: ThreadRng,
    turns: Vec<Turn>,
}

impl Game {
    fn new(max: u32, sides: u32, names: [String; 2], is_ai: [bool; 2]) -> Self {
        Self {
            names,
            is_ai,
            scores: [0; 2],
            cur: 0,
            last_round: false,
            max,
            sides,
            rng: ThreadRng::default(),
            turns: Vec::new(),
        }
    }

    fn play(&mut self) {
        clear_screen();

        loop {
            let n = if self.is_ai[self.cur] {
                self.ai_turn()
            } else {
                self.human_turn()
            };

            if n == 0 {
                self.turns.push(Turn::Stand);
                set_game_line("s");
                pause(800);

                if self.last_round {
                    break;
                }

                self.last_round = true;
                self.cur = 1 - self.cur;
                continue;
            }

            let sum = self.roll_dice(n);
            let new_score = self.scores[self.cur] + sum;
            let bust = new_score > self.max;

            self.scores[self.cur] = new_score;
            self.turns.push(Turn::Roll { dice: n, sum, bust });

            if bust {
                set_game_line(&format!("{n}d+{sum}! BUST!"));
                pause(1500);
                break;
            }

            set_game_line(&format!("{n}d+{sum}"));
            pause(800);

            if self.last_round {
                break;
            }

            self.cur = 1 - self.cur;
        }

        pause(500);
        self.print_record();
    }

    fn human_turn(&self) -> u32 {
        loop {
            set_game_line(&self.prompt());
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();
            if let Ok(n) = buf.trim().parse::<u32>() {
                return n;
            }
        }
    }

    fn ai_turn(&self) -> u32 {
        set_game_line(&format!(
            "{}:{} {}{}...",
            self.scores[self.cur],
            self.scores[1 - self.cur],
            self.names[self.cur],
            if self.last_round { "*" } else { "" },
        ));
        pause(600);
        self.ai_choose()
    }

    fn prompt(&self) -> String {
        format!(
            "{}:{} {}{}? ",
            self.scores[self.cur],
            self.scores[1 - self.cur],
            self.names[self.cur],
            if self.last_round { "*" } else { "" },
        )
    }

    fn ai_choose(&self) -> u32 {
        let my = self.scores[self.cur];
        let opp = self.scores[1 - self.cur];
        let room = self.max - my;
        let avg = (self.sides + 1) as f64 / 2.0;

        if self.last_round {
            if my >= opp {
                return 0;
            }
            let need = (opp - my) as f64;
            return (need / avg).ceil().max(1.0) as u32;
        }

        let target = (self.max as f64 * 0.85) as u32;
        if my >= target && my > opp {
            return 0;
        }

        let want = if my < target {
            (target - my) as f64
        } else {
            avg
        };

        let n = (want / (avg * 1.5)).round().max(1.0) as u32;
        n.min(room)
    }

    fn roll_dice(&mut self, n: u32) -> u32 {
        let die = Uniform::new_inclusive(1, self.sides).unwrap();
        (0..n).map(|_| self.rng.sample(die)).sum()
    }

    fn print_record(&self) {
        clear_screen();

        println!("{}M{}s", self.max, self.sides);
        println!("{}:{}", self.names[0], self.names[1]);
        println!();

        for turn in &self.turns {
            match turn {
                Turn::Stand => println!("s"),
                Turn::Roll { dice, sum, bust } => {
                    if *bust {
                        println!("{dice}d+{sum}!");
                    } else {
                        println!("{dice}d+{sum}");
                    }
                }
            }
        }

        println!();

        let busted = matches!(self.turns.last(), Some(Turn::Roll { bust: true, .. }));
        if busted {
            if self.cur == 0 {
                println!("!:{}", self.scores[1].min(self.max));
            } else {
                println!("{}:!", self.scores[0].min(self.max));
            }
        } else {
            println!(
                "{}:{}",
                self.scores[0].min(self.max),
                self.scores[1].min(self.max)
            );
        }
    }
}

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    stdout().flush().unwrap();
}

fn set_game_line(msg: &str) {
    print!("\x1b[;1H\x1b[J{msg}");
    stdout().flush().unwrap();
}

fn pause(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
