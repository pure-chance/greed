use criterion::{Criterion, criterion_group, criterion_main};

fn pmf_benchmarks(c: &mut Criterion) {
    let mut _group = c.benchmark_group("pmf");

    // TODO: Implement (proper) benchmarks for PMF

    _group.finish();
}

criterion_group!(benches, pmf_benchmarks);
criterion_main!(benches);
