use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use greed::{PolicyOptimizer, Ruleset, State};

fn normal_states(c: &mut Criterion) {
    let mut group = c.benchmark_group("normal_states");

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
                b.iter(|| optimizer.optimize_normal_states());
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
                b.iter(|| optimizer.find_optimal_normal_action(State::new(10, 10, false)));
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
                    optimizer.calc_normal_payoff(
                        State::new(ruleset.max() / 2, ruleset.sides() / 2, false),
                        3,
                    )
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, normal_states);
criterion_main!(benches);
