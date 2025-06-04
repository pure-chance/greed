use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use greed::{fft_convolve, solver::PMFLookup};

fn pmf_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("pmf");

    // Benchmark FFT convolution with different input sizes
    const PMF_SIZES: &[usize] = &[10, 50, 100, 200, 500, 1000];
    for size in PMF_SIZES {
        group.bench_with_input(BenchmarkId::new("fft_convolve", size), size, |b, &size| {
            let pmf_a = vec![0.5; size];
            let pmf_b = vec![0.5; size];
            b.iter(|| {
                let _ = fft_convolve(black_box(&pmf_a), black_box(&pmf_b));
            });
        });
    }

    // Benchmark PMF precomputation
    const BOARD_SIZES: &[u32] = &[5, 10, 20, 50, 100];
    for size in BOARD_SIZES {
        group.bench_with_input(
            BenchmarkId::new("precompute_pmfs", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let _ = PMFLookup::precompute(black_box(size), black_box(6));
                });
            },
        );
    }

    // Benchmark PMF precomputation with different dice sides
    const DICE_SIDES: &[u32] = &[4, 6, 8, 10, 12, 20];
    for sides in DICE_SIDES {
        group.bench_with_input(
            BenchmarkId::new("precompute_pmfs_dice_sides", sides),
            sides,
            |b, &sides| {
                b.iter(|| {
                    let _ = PMFLookup::precompute(black_box(20), black_box(sides));
                });
            },
        );
    }

    // Benchmark dice PMF creation (single die)
    for sides in DICE_SIDES {
        group.bench_with_input(
            BenchmarkId::new("single_die_pmf", sides),
            sides,
            |b, &sides| {
                b.iter(|| {
                    let dice_pmf = vec![1.0 / f64::from(black_box(sides)); sides as usize];
                    black_box(dice_pmf);
                });
            },
        );
    }

    // Benchmark convolution chain (simulating multiple dice rolls)
    const DICE_COUNTS: &[usize] = &[1, 2, 3, 4, 5, 8, 10];
    for dice_count in DICE_COUNTS {
        group.bench_with_input(
            BenchmarkId::new("convolution_chain", dice_count),
            dice_count,
            |b, &dice_count| {
                let dice_pmf = vec![1.0 / 6.0; 6]; // 6-sided die
                b.iter(|| {
                    let mut result = vec![1.0];
                    for _ in 0..black_box(dice_count) {
                        result = fft_convolve(&result, &dice_pmf);
                    }
                    black_box(result);
                });
            },
        );
    }

    // Benchmark asymmetric convolution (different sized PMFs)
    let small_pmf = vec![0.5, 0.3, 0.2];
    let medium_pmf = vec![0.1; 20];
    let large_pmf = vec![0.01; 100];

    group.bench_function("fft_convolve_asymmetric_small_medium", |b| {
        b.iter(|| {
            let _ = fft_convolve(black_box(&small_pmf), black_box(&medium_pmf));
        });
    });

    group.bench_function("fft_convolve_asymmetric_medium_large", |b| {
        b.iter(|| {
            let _ = fft_convolve(black_box(&medium_pmf), black_box(&large_pmf));
        });
    });

    group.bench_function("fft_convolve_asymmetric_small_large", |b| {
        b.iter(|| {
            let _ = fft_convolve(black_box(&small_pmf), black_box(&large_pmf));
        });
    });

    group.finish();
}

criterion_group!(benches, pmf_benchmarks);
criterion_main!(benches);
