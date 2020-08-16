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

static CLEAN_JS: &str = r#"
!function () {
	var M = new Array();
	for (i = 0; i < 100; i++) {
		M.push(Math.floor(Math.random() * 100));
	}
	var test = [];
	for (i = 0; i < 100; i++) {
		if (M[i] > 50) {
			test.push(M[i]);
		}
	}
	test.forEach(elem => {
        0
    });
}();
"#;

fn clean_js(c: &mut Criterion) {
    c.bench_function("Clean js (Parser)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(CLEAN_JS));
            lexer.lex().expect("failed to lex");

            Parser::new(&black_box(lexer.tokens)).parse_all()
        })
    });
}

static MINI_JS: &str = r#"
!function(){var r=new Array();for(i=0;i<100;i++)r.push(Math.floor(100*Math.random()));var a=[];for(i=0;i<100;i++)r[i]>50&&a.push(r[i]);a.forEach(i=>{0})}();
"#;

fn mini_js(c: &mut Criterion) {
    c.bench_function("Mini js (Parser)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(MINI_JS));
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
    clean_js,
    mini_js,
);
criterion_main!(parser);
