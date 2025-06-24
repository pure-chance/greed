use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use greed::DpSolver;

fn terminal_states(c: &mut Criterion) {
    let mut group = c.benchmark_group("terminal_states");

    const RULESETS: [(u32, u32); 3] = [(25, 4), (100, 6), (250, 20)];

    for ruleset in RULESETS {
        // satisfy invariants
        let mut solver = DpSolver::new(ruleset.0, ruleset.1);
        solver.precompute_pmfs();

        // Benchmark: solving normal states
        group.bench_with_input(
            BenchmarkId::new("solve", format!("M={},s={}", ruleset.0, ruleset.1)),
            &ruleset,
            |b, _| {
                b.iter(|| solver.solve_terminal_states());
            },
        );

        // Benchmark: find optimal action
        group.bench_with_input(
            BenchmarkId::new(
                "calc_optimal_payoff",
                format!("M={},s={}", ruleset.0, ruleset.1),
            ),
            &ruleset,
            |b, _| {
                b.iter(|| {
                    solver.find_optimal_terminal_action(black_box(greed::State::new(10, 10, false)))
                });
            },
        );

        // Benchmark: computing an optimal payoff
        group.bench_with_input(
            BenchmarkId::new("calc_payoff", format!("M={},s={}", ruleset.0, ruleset.1)),
            &ruleset,
            |b, _| {
                b.iter(|| {
                    solver.calc_terminal_payoff(
                        black_box(greed::State::new(ruleset.0 / 2, ruleset.1 / 2, false)),
                        3,
                    )
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, terminal_states);
criterion_main!(benches);
