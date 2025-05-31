use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use greed::GreedSolver;

fn terminal_states(c: &mut Criterion) {
    let mut group = c.benchmark_group("terminal_states");
    const BOARD_SIZES: &[u32] = &[5, 10, 20, 50, 100];

    // Benchmark terminal states solving
    for size in BOARD_SIZES {
        group.bench_with_input(
            BenchmarkId::new("solve_terminal_states", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut solver = GreedSolver::new(black_box(size), black_box(6));
                    solver.solve_terminal_states();
                });
            },
        );
    }

    // Benchmark terminal state solving with different dice sides
    const DICE_SIDES: &[u32] = &[4, 6, 8, 10, 12, 20];
    for sides in DICE_SIDES {
        group.bench_with_input(
            BenchmarkId::new("terminal_states_dice_sides", sides),
            sides,
            |b, &sides| {
                b.iter(|| {
                    let mut solver = GreedSolver::new(black_box(20), black_box(sides));
                    solver.solve_terminal_states();
                });
            },
        );
    }

    // Benchmark normal payoff calculation which involves terminal state lookups
    for size in BOARD_SIZES {
        group.bench_with_input(
            BenchmarkId::new("calc_normal_payoff_terminal_lookup", size),
            size,
            |b, &size| {
                let mut solver = GreedSolver::new(size, 6);
                solver.solve_terminal_states();
                let state = greed::State::new(size / 2, size / 2, false);
                b.iter(|| {
                    let _ = solver.calc_normal_payoff(black_box(state), black_box(0));
                    // 0 dice triggers terminal lookup
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, terminal_states);
criterion_main!(benches);
