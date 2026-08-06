use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use greed::{PolicyOptimizer, Ruleset};

fn all_states(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_states");

    const RULESETS: [Ruleset; 3] = [
        Ruleset::new(25, 4),
        Ruleset::new(100, 6),
        Ruleset::new(250, 20),
    ];

    // Benchmark: complete optimization
    for ruleset in RULESETS {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("M={},s={}", ruleset.max(), ruleset.sides())),
            &ruleset,
            |b, &ruleset| {
                b.iter(|| {
                    let _optimizer = PolicyOptimizer::optimize(ruleset);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, all_states);
criterion_main!(benches);
