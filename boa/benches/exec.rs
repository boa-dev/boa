//! Benchmarks of the whole execution engine in Boa.

use boa::{exec, realm::Realm};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

static SYMBOL_CREATION: &str = r#"
let a = Symbol();
let b = Symbol();
let c = Symbol();
"#;

fn create_realm(c: &mut Criterion) {
    c.bench_function("Create Realm", move |b| b.iter(Realm::create));
}

fn symbol_creation(c: &mut Criterion) {
    c.bench_function("Symbols (Execution)", move |b| {
        b.iter(|| exec(black_box(SYMBOL_CREATION)))
    });
}

static FOR_LOOP: &str = r#"
let a = 10;
let b = "hello";
for (;a > 100;) {
    a += 5;

    if (a < 50) {
        b += "world";
    }
}

b
"#;

fn for_loop_execution(c: &mut Criterion) {
    c.bench_function("For loop (Execution)", move |b| {
        b.iter(|| exec(black_box(FOR_LOOP)))
    });
}

static FIBONACCI: &str = r#"
let num = 12;

function fib(n) {
  if (n <= 1) return 1;
  return fib(n - 1) + fib(n - 2);
}

let res = fib(num);

res;
"#;

fn fibonacci(c: &mut Criterion) {
    c.bench_function("Fibonacci (Execution)", move |b| {
        b.iter(|| exec(black_box(FIBONACCI)))
    });
}

criterion_group!(
    execution,
    create_realm,
    symbol_creation,
    for_loop_execution,
    fibonacci
);
criterion_main!(execution);
