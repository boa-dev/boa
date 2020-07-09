//! Benchmarks of the parsing process in Boa.

use boa::syntax::parser::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

static EXPRESSION: &str = r#"
1 + 1 + 1 + 1 + 1 + 1 / 1 + 1 + 1 * 1 + 1 + 1 + 1;
"#;

fn expression_parser(c: &mut Criterion) {
    // We include the lexing in the benchmarks, since they will get together soon, anyways.

    c.bench_function("Expression (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(EXPRESSION.as_bytes())).parse_all())
    });
}

static HELLO_WORLD: &str = "let foo = 'hello world!'; foo;";

fn hello_world_parser(c: &mut Criterion) {
    // We include the lexing in the benchmarks, since they will get together soon, anyways.

    c.bench_function("Hello World (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(HELLO_WORLD.as_bytes())).parse_all())
    });
}

static FOR_LOOP: &str = r#"
for (let a = 10; a < 100; a++) {
    if (a < 10) {
        console.log("impossible D:");
    } else if (a < 50) {
        console.log("starting");
    } else {
        console.log("finishing");
    }
}
"#;

fn for_loop_parser(c: &mut Criterion) {
    // We include the lexing in the benchmarks, since they will get together soon, anyways.

    c.bench_function("For loop (Parser)", move |b| {
        b.iter(|| Parser::new(black_box(FOR_LOOP.as_bytes())).parse_all())
    });
}

static LONG_REPETITION: &str = r#"
for (let a = 10; a < 100; a++) {
    if (a < 10) {
        console.log("impossible D:");
    } else if (a < 50) {
        console.log("starting");
    } else {
        console.log("finishing");
    }
}
"#;

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
        b.iter(|| Parser::new(black_box(&file)).parse_all())
    });

    fs::remove_file(FILE_NAME).unwrap_or_else(|_| panic!("could not remove {}", FILE_NAME));
}

static GOAL_SYMBOL_SWITCH: &str = r#"
function foo(regex, num) {}

let i = 0;
while (i < 1000000) {
    foo(/ab+c/, 5.0/5);
    i++;
}
"#;

fn goal_symbol_switch(c: &mut Criterion) {
    // We include the lexing in the benchmarks, since they will get together soon, anyways.

    c.bench_function("Goal Symbols (Parser)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(GOAL_SYMBOL_SWITCH));
            lexer.lex().expect("failed to lex");

            Parser::new(&black_box(lexer.tokens)).parse_all()
        })
    });
}

criterion_group!(
    parser,
    expression_parser,
    hello_world_parser,
    for_loop_parser,
    long_file_parser,
    goal_symbol_switch,
);
criterion_main!(parser);
