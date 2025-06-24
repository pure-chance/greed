//! Command-line interface for the Greed game and optimal policy solver.
//!
//! Provides two main commands:
//! - `play`: Interactive game between two players
//! - `solve`: Compute and export optimal strategies
//!
//! # Examples
//!
//! ```bash
//! # Play a standard game
//! cargo run -- play Alice Bob
//!
//! # Solve and visualize optimal policy
//! cargo run -- solve --max 100 --sides 6 --format svg
//! ```

use clap::{Arg, Command};
use greed::{DpSolver, Greed, Policy, Solver};

fn main() {
    let play = Command::new("play")
        .about("Start an interactive two-player game of Greed")
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
                .help("Number of sides on the die")
                .value_parser(clap::value_parser!(u32))
                .default_value("6"),
        )
        .arg(
            Arg::new("p1")
                .value_name("P1")
                .help("Player 1")
                .default_value("Alice"),
        )
        .arg(
            Arg::new("p2")
                .value_name("P2")
                .help("Player 2")
                .default_value("Blair"),
        );

    let solve = Command::new("solve")
        .about("Optimizes (solves) a game of Greed")
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
        )
        .arg(
            Arg::new("method")
                .short('M')
                .long("method")
                .value_name("METHOD")
                .help("Solver method")
                .value_parser(["dp", "rl"])
                .default_value("dp"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_parser(["stdout", "csv", "svg"])
                .default_value("svg")
                .help("Output format"),
        );

    let cli = Command::new("greed").subcommand(play).subcommand(solve);

    let args = cli.get_matches();

    match args.subcommand() {
        Some(("play", args)) => {
            let max = *args.get_one::<u32>("max").unwrap();
            let sides = *args.get_one::<u32>("sides").unwrap();
            let p1 = args.get_one::<String>("p1").unwrap().as_str();
            let p2 = args.get_one::<String>("p2").unwrap().as_str();

            Greed::play(max, sides, (p1, p2));
        }
        Some(("solve", args)) => {
            let max = *args.get_one::<u32>("max").unwrap();
            let sides = *args.get_one::<u32>("sides").unwrap();
            let method = args.get_one::<String>("method").unwrap().as_str();
            let format = args.get_one::<String>("format").unwrap().as_str();

            let policy = match method {
                "dp" => DpSolver::new(max, sides).policy(),
                "rl" => todo!(),
                _ => unreachable!("clap will panic if --method is not dp or rl"),
            };

            match format {
                "stdout" => policy.stdout(),
                "csv" => {
                    let csv_filename = format!("visualize/greed_{}_{}.csv", max, sides);
                    match policy.csv(&csv_filename) {
                        Ok(()) => println!("Policy exported to {}", csv_filename),
                        Err(e) => eprintln!("Failed to write CSV file: {}", e),
                    }
                }
                "svg" => match policy.svg() {
                    Ok(()) => println!("SVG visualizations generated in visualize/ directory"),
                    Err(e) => {
                        eprintln!("Failed to generate SVG file: {}", e);
                        eprintln!("Make sure R is installed and 'Rscript' is in your PATH");
                    }
                },
                _ => unreachable!(),
            }
        }
        None => {}
        Some(_) => {
            unreachable!(
                "Clap will short-circuit with `error: unrecognized subcommand: [subcommand]` if a subcommand is not recognized"
            )
        }
    }
}
