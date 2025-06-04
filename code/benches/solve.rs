use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use greed::GreedSolver;

fn solve_complete(c: &mut Criterion) {
    let mut group = c.benchmark_group("solve_complete");
    const BOARD_SIZES: [u32; 5] = [5, 10, 20, 50, 100];

    // Benchmark complete solving (both terminal and normal states)
    for size in BOARD_SIZES {
        group.bench_with_input(BenchmarkId::new("solve", size), &size, |b, &size| {
            b.iter(|| {
                let mut solver = GreedSolver::new(black_box(size), black_box(6));
                solver.solve();
            });
        });
    }

    // Benchmark complete solving with different dice sides
    const DICE_SIDES: [u32; 6] = [4, 6, 8, 10, 12, 20];
    for sides in DICE_SIDES {
        group.bench_with_input(
            BenchmarkId::new("solve_with_dice_sides", sides),
            &sides,
            |b, &sides| {
                b.iter(|| {
                    let mut solver = GreedSolver::new(black_box(20), black_box(sides));
                    solver.solve();
                });
            },
        );
    }

    // Benchmark memory usage pattern: create, solve, then query
    for size in BOARD_SIZES {
        group.bench_with_input(
            BenchmarkId::new("solve_and_query", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut solver = GreedSolver::new(black_box(size), black_box(6));
                    solver.solve();
                    // Query a few states to simulate real usage
                    let state1 = greed::State::new(size / 4, size / 4, false);
                    let state2 = greed::State::new(size / 2, size / 2, true);
                    let _ = solver.calc_normal_payoff(state1, 2);
                    let _ = solver.calc_normal_payoff(state2, 0);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, solve_complete);
criterion_main!(benches);
