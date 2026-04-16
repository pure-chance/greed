//! # A Policy Optimization of Greed
//!
//! The Command Line Interface (CLI) for the Greed game optimizer.
//!
//! ## Usage
//!
//! ```sh
//! # generates a (mostly) human readable report
// cargo run --release -- --max=100 --sides=6 --format=stdout
//! # generates csv file `visualize/greed_[max]_[sides].csv`
//! cargo run --release -- --max=100 --sides=6 --format=csv
//! ```

use clap::{Arg, Command};
use greed::{PolicyOptimizer, Ruleset};

fn main() {
    let cli = Command::new("optimize")
        .about("A policy optimizer for the game of Greed")
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
            Arg::new("format")
                .short('f')
                .long("format")
                .value_parser(["stdout", "csv"])
                .default_value("csv")
                .help("Output format"),
        );

    let args = cli.get_matches();

    let max = *args.get_one::<u32>("max").unwrap();
    let sides = *args.get_one::<u32>("sides").unwrap();
    let format = args.get_one::<String>("format").unwrap().as_str();

    let optimizer = PolicyOptimizer::optimize(Ruleset::new(max, sides));
    let policy = optimizer.policy();

    match format {
        "stdout" => policy.stdout(),
        "csv" => {
            let csv_filename = format!("results/greed_{max}_{sides}.csv");
            match policy.csv(&csv_filename) {
                Ok(()) => println!("Policy exported to {csv_filename}"),
                Err(e) => eprintln!("Failed to write CSV file: {e}"),
            }
        }
        _ => unreachable!("clap will panic if --format is not stdout or csv"),
    }
}
