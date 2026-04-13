//! # A Policy Optimization of Greed
//!
//! Command-line interface for the Greed game optimizer.

use std::process;

use clap::{Arg, Command};

use greed::{PolicyOptimizer, Ruleset, verify_payoffs, verify_policy};

fn build_cli() -> Command {
    Command::new("greed")
        .about("A Policy optimizer for the game of Greed.")
        .arg_required_else_help(true)
        .subcommand(build_optimize_command())
        .subcommand(build_verify_command())
}

fn build_optimize_command() -> Command {
    Command::new("optimize")
        .about("Compute the optimal policy.")
        .arg(
            Arg::new("max")
                .short('m')
                .long("max")
                .value_name("MAX")
                .help("Maximum score before busting")
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
                .help("Output format: human-readable table or CSV file"),
        )
}

fn build_verify_command() -> Command {
    Command::new("verify")
        .about("Verifies the computed policy and/or payoffs.")
        .long_about("Verifies that the computed policy and/or payoffs match empirical results via statistical monte-carlo.")
        .subcommand_required(true)
        .subcommand(build_verify_payoffs_subcommand())
        .subcommand(build_verify_policy_subcommand())
}

fn build_verify_payoffs_subcommand() -> Command {
    Command::new("payoffs")
        .about("Verifies that computed payoffs match empirical results.")
        .arg(
            Arg::new("max")
                .short('m')
                .long("max")
                .value_name("MAX")
                .help("Maximum score before busting")
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
            Arg::new("seed")
                .long("seed")
                .value_name("SEED")
                .help("RNG seed")
                .value_parser(clap::value_parser!(u64))
                .default_value("0"),
        )
        .arg(
            Arg::new("obs_target")
                .short('o')
                .long("obs-target")
                .value_name("OBS_TARGET")
                .help("Minimum Monte Carlo observations per state")
                .value_parser(clap::value_parser!(u64))
                .default_value("10000"),
        )
        .arg(
            Arg::new("fdr_q")
                .short('q')
                .long("fdr-q")
                .value_name("FDR_Q")
                .help("Target false discovery rate for BH-BY correction")
                .value_parser(clap::value_parser!(f64))
                .default_value("0.01"),
        )
        .arg(
            Arg::new("batch_size")
                .short('b')
                .long("batch-size")
                .value_name("BATCH_SIZE")
                .help("Games per adaptive-scheduler batch")
                .value_parser(clap::value_parser!(u64))
                .default_value("10000"),
        )
        .arg(
            Arg::new("max_games")
                .short('g')
                .long("max-games")
                .value_name("MAX_GAMES")
                .help("Hard ceiling on total games simulated")
                .value_parser(clap::value_parser!(u64))
                .default_value("1000000000"),
        )
        .arg(
            Arg::new("trials")
                .short('t')
                .long("trials")
                .value_name("TRIALS")
                .help("Dice rolls per (state, action) pair")
                .value_parser(clap::value_parser!(u64))
                .default_value("10000"),
        )
}

pub fn build_verify_policy_subcommand() -> Command {
    Command::new("policy")
        .about("Verifies that the policy matches optimal computed policy for each state.")
        .arg(
            Arg::new("max")
                .short('m')
                .long("max")
                .value_name("MAX")
                .help("Maximum score before busting")
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
            Arg::new("seed")
                .long("seed")
                .value_name("SEED")
                .help("RNG seed")
                .value_parser(clap::value_parser!(u64))
                .default_value("0"),
        )
        .arg(
            Arg::new("fdr_q")
                .short('q')
                .long("fdr-q")
                .value_name("FDR_Q")
                .help("Target false discovery rate for BH-BY correction")
                .value_parser(clap::value_parser!(f64))
                .default_value("0.01"),
        )
        .arg(
            Arg::new("trials")
                .short('t')
                .long("trials")
                .value_name("TRIALS")
                .help("Dice rolls per (state, action) pair")
                .value_parser(clap::value_parser!(u64))
                .default_value("10000"),
        )
}

fn run_optimize(args: &clap::ArgMatches) {
    let max = *args.get_one::<u32>("max").unwrap();
    let sides = *args.get_one::<u32>("sides").unwrap();
    let format = args.get_one::<String>("format").unwrap().as_str();

    let optimizer = PolicyOptimizer::optimize(Ruleset::new(max, sides));
    let policy = optimizer.policy();

    match format {
        "stdout" => policy.stdout(),
        "csv" => {
            let path = format!("results/greed_{max}_{sides}.csv");
            match policy.csv(&path) {
                Ok(()) => println!("Policy exported to {path}"),
                Err(e) => {
                    eprintln!("Failed to write CSV: {e}");
                    process::exit(1);
                }
            }
        }
        _ => unreachable!("clap enforces --format value"),
    }
}

fn run_verify_payoffs(args: &clap::ArgMatches) {
    let max = *args.get_one::<u32>("max").unwrap();
    let sides = *args.get_one::<u32>("sides").unwrap();
    let seed = *args.get_one::<u64>("seed").unwrap();
    let obs_target = *args.get_one::<u64>("obs_target").unwrap();
    let fdr_q = *args.get_one::<f64>("fdr_q").unwrap();
    let batch_size = *args.get_one::<u64>("batch_size").unwrap();
    let max_games = *args.get_one::<u64>("max_games").unwrap();

    let ruleset = Ruleset::new(max, sides);
    let optimizer = PolicyOptimizer::optimize(ruleset);
    let policy = optimizer.policy();
    let report = verify_payoffs(
        seed, ruleset, policy, obs_target, fdr_q, batch_size, max_games,
    );
    report
        .print_summary(std::io::stdout())
        .expect("write failed");
}

fn run_verify_policy(args: &clap::ArgMatches) {
    let max = *args.get_one::<u32>("max").unwrap();
    let sides = *args.get_one::<u32>("sides").unwrap();
    let seed = *args.get_one::<u64>("seed").unwrap();
    let fdr_q = *args.get_one::<f64>("fdr_q").unwrap();
    let trials = *args.get_one::<u64>("trials").unwrap();

    let ruleset = Ruleset::new(max, sides);
    let optimizer = PolicyOptimizer::optimize(ruleset);
    let policy = optimizer.policy();
    let report = verify_policy(seed, ruleset, policy, trials, fdr_q);
    report
        .print_summary(std::io::stdout())
        .expect("write failed");
}

fn main() {
    let cli = build_cli();
    let args = cli.get_matches();

    match args.subcommand() {
        Some(("optimize", args)) => run_optimize(args),
        Some(("verify", sub)) => match sub.subcommand() {
            Some(("payoffs", args)) => run_verify_payoffs(args),
            Some(("policy", args)) => run_verify_policy(args),
            _ => unreachable!("subcommand required"),
        },
        _ => unreachable!(),
    }
}
