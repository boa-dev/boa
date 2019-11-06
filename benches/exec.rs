#[macro_use]
extern crate criterion;

use boa::exec;
use boa::realm::Realm;
use criterion::{black_box, Criterion};

static SRC: &str = r#"
let a = Symbol();
let b = Symbol();
let c = Symbol();
"#;

fn symbol_creation(c: &mut Criterion) {
    c.bench_function("fibonacci (Execution)", move |b| {
        b.iter(|| exec(black_box(SRC)))
    });
}

fn create_realm(c: &mut Criterion) {
    c.bench_function("Create Realm", move |b| b.iter(|| Realm::create()));
}

criterion_group!(benches, create_realm, symbol_creation);
criterion_main!(benches);
