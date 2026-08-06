use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use greed::{PolicyOptimizer, Ruleset};

fn normal_states(c: &mut Criterion) {
    let mut group = c.benchmark_group("normal_states");

    const RULESETS: [Ruleset; 3] = [
        Ruleset::new(25, 4),
        Ruleset::new(100, 6),
        Ruleset::new(250, 20),
    ];

    for ruleset in RULESETS {
        // satisfy invariants
        let optimizer = PolicyOptimizer::new(ruleset);
        let mut policy = PolicyOptimizer::optimize(ruleset);

        // Benchmark: optimizing normal states
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("M={},s={}", ruleset.max(), ruleset.sides())),
            &ruleset,
            |b, _| {
                b.iter(|| optimizer.optimize_normal_states(&mut policy));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, normal_states);
criterion_main!(benches);
