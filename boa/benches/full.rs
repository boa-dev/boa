//! Benchmarks of whole program execution in Boa.

use boa::exec;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

include!("constants.rs");

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn symbol_creation(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("Symbols (Full)", move |b| {
        b.iter(|| exec(black_box(SYMBOL_CREATION)))
    });
}

fn for_loop(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("For loop (Full)", move |b| {
        b.iter(|| exec(black_box(FOR_LOOP)))
    });
}

fn fibonacci(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("Fibonacci (Full)", move |b| {
        b.iter(|| exec(black_box(FIBONACCI)))
    });
}

fn object_creation(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("Object Creation (Full)", move |b| {
        b.iter(|| exec(black_box(OBJECT_CREATION)))
    });
}

fn object_prop_access_const(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("Static Object Property Access (Full)", move |b| {
        b.iter(|| exec(black_box(OBJECT_PROP_ACCESS_CONST)))
    });
}

fn object_prop_access_dyn(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("Dynamic Object Property Access (Full)", move |b| {
        b.iter(|| exec(black_box(OBJECT_PROP_ACCESS_DYN)))
    });
}

fn regexp_literal_creation(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("RegExp Literal Creation (Full)", move |b| {
        b.iter(|| exec(black_box(REGEXP_LITERAL_CREATION)))
    });
}

fn regexp_creation(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("RegExp (Full)", move |b| {
        b.iter(|| exec(black_box(REGEXP_CREATION)))
    });
}

fn regexp_literal(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("RegExp Literal (Full)", move |b| {
        b.iter(|| exec(black_box(REGEXP_LITERAL)))
    });
}

fn regexp(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("RegExp (Full)", move |b| b.iter(|| exec(black_box(REGEXP))));
}

fn array_access(c: &mut Criterion) {
    c.bench_function("Array access (Full)", move |b| {
        b.iter(|| exec(black_box(ARRAY_ACCESS)))
    });
}

fn array_creation(c: &mut Criterion) {
    c.bench_function("Array creation (Full)", move |b| {
        b.iter(|| exec(black_box(ARRAY_CREATE)))
    });
}

fn array_pop(c: &mut Criterion) {
    c.bench_function("Array pop (Full)", move |b| {
        b.iter(|| exec(black_box(ARRAY_POP)))
    });
}

fn string_concat(c: &mut Criterion) {
    c.bench_function("String concatenation (Full)", move |b| {
        b.iter(|| exec(black_box(STRING_CONCAT)))
    });
}

fn string_compare(c: &mut Criterion) {
    c.bench_function("String comparison (Full)", move |b| {
        b.iter(|| exec(black_box(STRING_COMPARE)))
    });
}

fn string_copy(c: &mut Criterion) {
    c.bench_function("String copy (Full)", move |b| {
        b.iter(|| exec(black_box(STRING_COPY)))
    });
}

fn number_object_access(c: &mut Criterion) {
    c.bench_function("Number Object Access (Full)", move |b| {
        b.iter(|| exec(black_box(NUMBER_OBJECT_ACCESS)))
    });
}

fn boolean_object_access(c: &mut Criterion) {
    c.bench_function("Boolean Object Access (Full)", move |b| {
        b.iter(|| exec(black_box(BOOLEAN_OBJECT_ACCESS)))
    });
}

fn string_object_access(c: &mut Criterion) {
    c.bench_function("String Object Access (Full)", move |b| {
        b.iter(|| exec(black_box(STRING_OBJECT_ACCESS)))
    });
}

fn arithmetic_operations(c: &mut Criterion) {
    c.bench_function("Arithmetic operations (Full)", move |b| {
        b.iter(|| exec(black_box(ARITHMETIC_OPERATIONS)))
    });
}

criterion_group!(
    full,
    symbol_creation,
    for_loop,
    fibonacci,
    array_access,
    array_creation,
    array_pop,
    object_creation,
    object_prop_access_const,
    object_prop_access_dyn,
    regexp_literal_creation,
    regexp_creation,
    regexp_literal,
    regexp,
    string_concat,
    string_compare,
    string_copy,
    number_object_access,
    boolean_object_access,
    string_object_access,
    arithmetic_operations,
);
criterion_main!(full);
