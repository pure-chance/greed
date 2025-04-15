use greed::GreedSolver;

fn main() {
    let mut greed_solver = GreedSolver::new(100, 6);
    greed_solver.solve();
    greed_solver.csv("visualize/greed.csv").unwrap(); // or greed_solver.display();
}
