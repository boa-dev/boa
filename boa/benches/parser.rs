//! Benchmarks of the parsing process in Boa.

use boa::syntax::parser::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

static EXPRESSION: &str = include_str!("bench_scripts/expression.js");

fn expression_parser(c: &mut Criterion) {
    c.bench_function("Expression (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(EXPRESSION.as_bytes()), false).parse_all())
    });
}

static HELLO_WORLD: &str = include_str!("bench_scripts/hello_world.js");

fn hello_world_parser(c: &mut Criterion) {
    c.bench_function("Hello World (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(HELLO_WORLD.as_bytes()), false).parse_all())
    });
}

static FOR_LOOP: &str = include_str!("bench_scripts/for_loop.js");

fn for_loop_parser(c: &mut Criterion) {
    c.bench_function("For loop (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(FOR_LOOP.as_bytes()), false).parse_all())
    });
}

static LONG_REPETITION: &str = include_str!("bench_scripts/long_repetition.js");

fn long_file_parser(c: &mut Criterion) {
    use std::{
        fs::{self, File},
        io::{BufWriter, Write},
    };
    // We include the lexing in the benchmarks, since they will get together soon, anyways.
    const FILE_NAME: &str = "long_file_test.js";

    {
        let mut file = BufWriter::new(
            File::create(FILE_NAME).unwrap_or_else(|_| panic!("could not create {}", FILE_NAME)),
        );
        for _ in 0..400 {
            file.write_all(LONG_REPETITION.as_bytes())
                .unwrap_or_else(|_| panic!("could not write {}", FILE_NAME));
        }
    }

    let file = std::fs::File::open(FILE_NAME).expect("Could not open file");
    c.bench_function("Long file (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(&file), false).parse_all())
    });

    fs::remove_file(FILE_NAME).unwrap_or_else(|_| panic!("could not remove {}", FILE_NAME));
}

static GOAL_SYMBOL_SWITCH: &str = include_str!("bench_scripts/goal_symbol_switch.js");

fn goal_symbol_switch(c: &mut Criterion) {
    c.bench_function("Goal Symbols (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(GOAL_SYMBOL_SWITCH.as_bytes()), false).parse_all())
    });
}

static CLEAN_JS: &str = include_str!("bench_scripts/clean_js.js");

fn clean_js(c: &mut Criterion) {
    c.bench_function("Clean js (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(CLEAN_JS.as_bytes()), false).parse_all())
    });
}

static MINI_JS: &str = include_str!("bench_scripts/mini_js.js");

fn mini_js(c: &mut Criterion) {
    c.bench_function("Mini js (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(MINI_JS.as_bytes()), false).parse_all())
    });
}

static ARRAY_CONCAT: &str = include_str!("bench_scripts/array_concat.js");

fn array_concat(c: &mut Criterion) {
    c.bench_function("Array concat (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(ARRAY_CONCAT.as_bytes()), false).parse_all())
    });
}

criterion_group!(
    parser,
    expression_parser,
    hello_world_parser,
    for_loop_parser,
    long_file_parser,
    goal_symbol_switch,
    clean_js,
    mini_js,
    array_concat,
);
criterion_main!(parser);
