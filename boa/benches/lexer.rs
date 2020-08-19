//! Benchmarks of the lexing process in Boa.

use boa::syntax::lexer::Lexer;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

static EXPRESSION: &str = include_str!("bench_scripts/expression.js");

fn expression_lexer(c: &mut Criterion) {
    c.bench_function("Expression (Lexer)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(EXPRESSION));

            lexer.lex()
        })
    });
}

static HELLO_WORLD: &str = include_str!("bench_scripts/hello_world.js");

fn hello_world_lexer(c: &mut Criterion) {
    c.bench_function("Hello World (Lexer)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(HELLO_WORLD));
            // return the value into the blackbox so its not optimized away
            // https://gist.github.com/jasonwilliams/5325da61a794d8211dcab846d466c4fd
            lexer.lex()
        })
    });
}

static FOR_LOOP: &str = include_str!("bench_scripts/for_loop.js");

fn for_loop_lexer(c: &mut Criterion) {
    c.bench_function("For loop (Lexer)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(FOR_LOOP));

            lexer.lex()
        })
    });
}

criterion_group!(lexer, expression_lexer, hello_world_lexer, for_loop_lexer);
criterion_main!(lexer);
