use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use greed::{pmf, GreedSolver};

fn bench_solver(c: &mut Criterion) {
    let mut group = c.benchmark_group("solver");
    const BOARD_SIZES: &[u16] = &[5, 20, 100];

    // // Benchmark terminal states solving
    // for size in BOARD_SIZES {
    //     group.bench_with_input(
    //         BenchmarkId::new("terminal_states", size),
    //         size,
    //         |b, &size| {
    //             b.iter(|| {
    //                 let mut solver = GreedSolver::new(black_box(size), black_box(6));
    //                 solver.solve_terminal_states();
    //             });
    //         },
    //     );
    // }

    // // Benchmark normal states solving
    // for size in BOARD_SIZES {
    //     group.bench_with_input(BenchmarkId::new("normal_states", size), size, |b, &size| {
    //         let mut solver = GreedSolver::new(size, 6);
    //         solver.solve_terminal_states(); // Pre-solve terminal states
    //         b.iter(|| {
    //             let mut solver = solver.clone();
    //             solver.solve_normal_states();
    //         });
    //     });
    // }

    // Benchmark PMF calculation
    group.bench_function("pmf_calculation", |b| {
        b.iter(|| {
            for dice in 1..=50 {
                for total in dice..=(6 * dice) {
                    black_box(pmf(black_box(total), black_box(dice), black_box(6)));
                }
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_solver);
criterion_main!(benches);
