use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use greed::{PolicyOptimizer, Ruleset, State};

fn terminal_states(c: &mut Criterion) {
    let mut group = c.benchmark_group("terminal_states");

    const RULESETS: [Ruleset; 3] = [
        Ruleset::new(25, 4),
        Ruleset::new(100, 6),
        Ruleset::new(250, 20),
    ];

    for ruleset in RULESETS {
        // satisfy invariants
        let mut optimizer = PolicyOptimizer::optimize(ruleset);

        // Benchmark: solving normal states
        group.bench_with_input(
            BenchmarkId::new(
                "optimize",
                format!("M={},s={}", ruleset.max(), ruleset.sides()),
            ),
            &ruleset,
            |b, _| {
                b.iter(|| optimizer.optimize_terminal_states());
            },
        );

        // Benchmark: find optimal action
        group.bench_with_input(
            BenchmarkId::new(
                "calc_optimal_payoff",
                format!("M={},s={}", ruleset.max(), ruleset.sides()),
            ),
            &ruleset,
            |b, _| {
                b.iter(|| optimizer.find_optimal_terminal_action(State::new(10, 10, false)));
            },
        );

        // Benchmark: computing an optimal payoff
        group.bench_with_input(
            BenchmarkId::new(
                "calc_payoff",
                format!("M={},s={}", ruleset.max(), ruleset.sides()),
            ),
            &ruleset,
            |b, _| {
                b.iter(|| {
                    optimizer.calc_terminal_payoff(
                        State::new(ruleset.max() / 2, ruleset.sides() / 2, false),
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
