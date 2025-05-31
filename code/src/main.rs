use clap::{Arg, Command};
use greed::{Greed, GreedSolver};

fn main() {
    let play = Command::new("play")
        .about("starts a game of Greed")
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
                .help("Number of sides on the dice")
                .value_parser(clap::value_parser!(u32))
                .default_value("6"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_parser(["stdout", "csv", "png"])
                .default_value("png")
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
            let format = args.get_one::<String>("format").unwrap().as_str();

            let mut greed_solver = GreedSolver::new(max, sides);
            greed_solver.solve();

            match format {
                "stdout" => greed_solver.stdout(),
                "csv" => {
                    let csv_filename = format!("visualize/greed_{}_{}.csv", max, sides);
                    if let Err(e) = greed_solver.csv(&csv_filename) {
                        eprintln!("Failed to write CSV file: {}", e);
                    }
                }
                "png" => {
                    if let Err(e) = greed_solver.png() {
                        eprintln!("Failed to generate PNG: {}", e);
                        eprintln!("Make sure R is installed and 'Rscript' is in your PATH");
                    }
                }
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
