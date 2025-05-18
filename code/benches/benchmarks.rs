use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use greed::GreedSolver;

fn solver(c: &mut Criterion) {
    let mut group = c.benchmark_group("solver");
    const BOARD_SIZES: &[u16] = &[5, 20, 100];

    // Benchmark terminal states solving
    for size in BOARD_SIZES {
        group.bench_with_input(
            BenchmarkId::new("terminal_states", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut solver = GreedSolver::new(black_box(size), black_box(6));
                    solver.solve_terminal_states();
                });
            },
        );
    }

    // Benchmark normal states solving
    for size in BOARD_SIZES {
        group.bench_with_input(BenchmarkId::new("normal_states", size), size, |b, &size| {
            let mut solver = GreedSolver::new(size, 6);
            solver.solve_terminal_states(); // Pre-solve terminal states
            b.iter(|| {
                let mut solver = solver.clone();
                solver.solve_normal_states();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, solver);
criterion_main!(benches);
