use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use greed::GreedSolver;

fn normal_states(c: &mut Criterion) {
    let mut group = c.benchmark_group("normal_states");
    const BOARD_SIZES: &[u32] = &[5, 10, 20, 50, 100];

    // Benchmark normal states solving (requires terminal states to be solved first)
    for size in BOARD_SIZES {
        group.bench_with_input(
            BenchmarkId::new("solve_normal_states", size),
            size,
            |b, &size| {
                let mut solver = GreedSolver::new(size, 6);
                solver.solve_terminal_states(); // Pre-solve terminal states
                b.iter(|| {
                    let mut solver = solver.clone();
                    solver.solve_normal_states();
                });
            },
        );
    }

    // Benchmark normal payoff calculation
    for size in BOARD_SIZES {
        group.bench_with_input(
            BenchmarkId::new("calc_normal_payoff", size),
            size,
            |b, &size| {
                let mut solver = GreedSolver::new(size, 6);
                solver.solve(); // Pre-solve all states
                let state = greed::State::new(size / 2, size / 2, false);
                let dice_rolled = 3;
                b.iter(|| {
                    let _ = solver.calc_normal_payoff(black_box(state), black_box(dice_rolled));
                });
            },
        );
    }

    // Benchmark normal state solving with different dice sides
    const DICE_SIDES: &[u32] = &[4, 6, 8, 10, 12, 20];
    for sides in DICE_SIDES {
        group.bench_with_input(
            BenchmarkId::new("normal_states_dice_sides", sides),
            sides,
            |b, &sides| {
                let mut solver = GreedSolver::new(20, sides);
                solver.solve_terminal_states(); // Pre-solve terminal states
                b.iter(|| {
                    let mut solver = solver.clone();
                    solver.solve_normal_states();
                });
            },
        );
    }

    // Benchmark normal payoff calculation with varying dice counts
    const DICE_COUNTS: &[u32] = &[0, 1, 2, 3, 4, 5, 8, 10];
    for dice_count in DICE_COUNTS {
        group.bench_with_input(
            BenchmarkId::new("calc_normal_payoff_dice_count", dice_count),
            dice_count,
            |b, &dice_count| {
                let mut solver = GreedSolver::new(20, 6);
                solver.solve(); // Pre-solve all states
                let state = greed::State::new(10, 10, false);
                b.iter(|| {
                    let _ = solver.calc_normal_payoff(black_box(state), black_box(dice_count));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, normal_states);
criterion_main!(benches);
