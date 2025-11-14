use criterion::{criterion_group, criterion_main, Criterion};
use greed_solve::PMFLookup;

fn pmf_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("pmf");

    const MAX: u16 = 1000;
    const SIDES: u16 = 20;
    let pmf_lookup = PMFLookup::precompute(MAX, SIDES);

    group.bench_function("precompute", |b| {
        b.iter(|| {
            let _ = PMFLookup::precompute(MAX, SIDES);
        });
    });

    group.bench_function("index", |b| {
        b.iter(|| {
            for n in 0..=MAX {
                for total in n..=n * SIDES {
                    let _ = pmf_lookup[(n, total)];
                }
            }
        });
    });

    group.finish();
}

criterion_group!(benches, pmf_benchmarks);
criterion_main!(benches);
