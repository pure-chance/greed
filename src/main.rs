use clap::{Arg, Command};
use greed::GreedSolver;

fn main() {
    let matches = Command::new("dice-tool")
        .about("Simulates dice rolls")
        .arg(
            Arg::new("max")
                .short('m')
                .long("max")
                .value_name("MAX")
                .help("Maximum score")
                .required(true)
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("sides")
                .short('s')
                .long("sides")
                .value_name("SIDES")
                .help("Number of sides on the dice")
                .required(true)
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_parser(["stdout", "csv"])
                .default_value("stdout")
                .help("Output format: stdout or csv"),
        )
        .get_matches();

    let max = *matches.get_one::<u16>("max").unwrap();
    let sides = *matches.get_one::<u16>("sides").unwrap();

    let mut greed_solver = GreedSolver::new(max, sides);
    greed_solver.solve();

    match matches.get_one::<String>("format").unwrap().as_str() {
        "stdout" => greed_solver.display(),
        "csv" => greed_solver
            .csv(&format!("visualize/greed_{}_{}.csv", max, sides))
            .unwrap(),
        _ => unreachable!(),
    }
}
