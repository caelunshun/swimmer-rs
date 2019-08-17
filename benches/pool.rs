#[macro_use]
extern crate criterion;

use criterion::Criterion;
use lifeguard::{StartingSize, Supplier};
use swimmer::Pool;

fn pool(c: &mut Criterion) {
    let pool: Pool<Vec<u8>> = swimmer::builder()
        .with_starting_size(1024)
        .with_supplier(|| Vec::with_capacity(1024))
        .build();
    c.bench_function("pool", move |b| b.iter(|| pool.get()));
}

fn alloc(c: &mut Criterion) {
    c.bench_function("alloc", move |b| b.iter(|| Vec::<u8>::with_capacity(1024)));
}

fn lifeguard(c: &mut Criterion) {
    let pool: lifeguard::Pool<Vec<u8>> = lifeguard::pool()
        .with(Supplier(|| Vec::with_capacity(1024)))
        .with(StartingSize(1024))
        .build();
    c.bench_function("lifeguard", move |b| b.iter(|| pool.new()));
}
criterion_group!(benches, pool, alloc, lifeguard);
criterion_main!(benches);
