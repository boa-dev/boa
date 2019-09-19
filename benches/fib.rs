#[macro_use]
extern crate criterion;

use boa::exec;
use criterion::black_box;
use criterion::Criterion;

static SRC: &str = r#"
let num = 12;

function fib(n) {
  if (n <= 1) return 1;
  return fib(n - 1) + fib(n - 2);
}

let res = fib(num);

res;
"#;

fn fibonacci(c: &mut Criterion) {
    c.bench_function("fibonacci (Execution)", move |b| {
        b.iter(|| exec(black_box(SRC)))
    });
}

criterion_group!(benches, fibonacci);
criterion_main!(benches);
