//! Benchmarks of the lexing process in Boa.

use boa::syntax::lexer::Lexer;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

static HELLO_WORLD: &str = "let foo = 'hello world!'; foo;";

fn hello_world_lexer(c: &mut Criterion) {
    c.bench_function("Hello World (Lexer)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(HELLO_WORLD.as_bytes()));
            // return the value into the blackbox so its not optimized away
            // https://gist.github.com/jasonwilliams/5325da61a794d8211dcab846d466c4fd
            // Goes through and lexes entire given string.
            while lexer.next().expect("Failed to lex").is_some() {}
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

fn for_loop_lexer(c: &mut Criterion) {
    c.bench_function("For loop (Lexer)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(FOR_LOOP.as_bytes()));

            // Goes through and lexes entire given string.
            while lexer.next().expect("Failed to lex").is_some() {}
        })
    });
}

criterion_group!(
    lexer,
    // expression_lexer,
    hello_world_lexer,
    for_loop_lexer
);
criterion_main!(lexer);
