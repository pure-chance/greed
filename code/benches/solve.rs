use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use greed::DpSolver;

fn all_states(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_states");

    const RULESETS: [(u32, u32); 3] = [(25, 4), (100, 6), (250, 20)];

    // Benchmark: complete solve
    for ruleset in RULESETS {
        group.bench_with_input(
            BenchmarkId::new("solve", format!("M={},s={}", ruleset.0, ruleset.1)),
            &ruleset,
            |b, &ruleset| {
                b.iter(|| {
                    let mut solver = DpSolver::new(black_box(ruleset.0), black_box(ruleset.1));
                    solver.solve();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, all_states);
criterion_main!(benches);
