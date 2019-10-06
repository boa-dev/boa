#[macro_use]
extern crate criterion;

use boa::realm::Realm;
use criterion::Criterion;

fn create_realm(c: &mut Criterion) {
    c.bench_function("Create Realm", move |b| b.iter(|| Realm::create()));
}

criterion_group!(benches, create_realm);
criterion_main!(benches);
