#[macro_use]
extern crate criterion;

use boa::exec;
use criterion::Criterion;

fn hello_world() {
    let s = &String::from("let a = 'hello world';");
    exec(s.to_string());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("hello_world", |b| b.iter(|| hello_world()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
