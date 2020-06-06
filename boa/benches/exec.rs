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

static ARRAY_ACCESS: &str = r#"
let testArr = [1,2,3,4,5];

let res = testArr[2];

res;
"#;

fn array_access(c: &mut Criterion) {
    c.bench_function("Array access (Execution)", move |b| {
        b.iter(|| exec(black_box(ARRAY_ACCESS)))
    });
}

static ARRAY_CREATE: &str = r#"
var testArr = [];
for (var a = 0; a <= 10000; a++) {
    testArr[a] = ('p' + a);
}

testArr;
"#;

fn array_creation(c: &mut Criterion) {
    c.bench_function("Array creation (Execution)", move |b| {
        b.iter(|| exec(black_box(ARRAY_CREATE)))
    });
}

static ARRAY_POP: &str = r#"
var testArray = [83, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28];
while (testArray.length > 0) {
  testArray.pop();
}

testArray;
"#;

fn array_pop(c: &mut Criterion) {
    c.bench_function("Array pop (Execution)", move |b| {
        b.iter(|| exec(black_box(ARRAY_POP)))
    });
}

criterion_group!(
    execution,
    create_realm,
    symbol_creation,
    for_loop_execution,
    fibonacci,
    array_access,
    array_creation,
    array_pop
);
criterion_main!(execution);
