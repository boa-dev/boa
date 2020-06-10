//! Benchmarks of the parsing process in Boa.

use boa::syntax::{lexer::Lexer, parser::Parser};
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
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(EXPRESSION));
            lexer.lex().expect("failed to lex");

            Parser::new(&black_box(lexer.tokens)).parse_all()
        })
    });
}

static HELLO_WORLD: &str = "let foo = 'hello world!'; foo;";

fn hello_world_parser(c: &mut Criterion) {
    // We include the lexing in the benchmarks, since they will get together soon, anyways.

    c.bench_function("Hello World (Parser)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(HELLO_WORLD));
            lexer.lex().expect("failed to lex");

            Parser::new(&black_box(lexer.tokens)).parse_all()
        })
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
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(FOR_LOOP));
            lexer.lex().expect("failed to lex");

            Parser::new(&black_box(lexer.tokens)).parse_all()
        })
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
    c.bench_function("Long file (Parser)", move |b| {
        b.iter(|| {
            let file_str = fs::read_to_string(FILE_NAME)
                .unwrap_or_else(|_| panic!("could not read {}", FILE_NAME));

            let mut lexer = Lexer::new(black_box(&file_str));
            lexer.lex().expect("failed to lex");

            Parser::new(&black_box(lexer.tokens)).parse_all()
        })
    });

    fs::remove_file(FILE_NAME).unwrap_or_else(|_| panic!("could not remove {}", FILE_NAME));
}

criterion_group!(
    parser,
    expression_parser,
    hello_world_parser,
    for_loop_parser,
    long_file_parser,
);
criterion_main!(parser);
