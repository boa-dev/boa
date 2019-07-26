#[macro_use]
extern crate criterion;

use boa::exec;
use boa::syntax::lexer::Lexer;
use boa::syntax::parser::Parser;
use criterion::black_box;
use criterion::Criterion;

static SRC: &str = "let foo = 'hello world!'; foo;";

fn hello_world_lexer(c: &mut Criterion) {
    c.bench_function("Hello World (Lexer)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(SRC));
            let _ = lexer.lex();
        })
    });
}

fn hello_world_parser(c: &mut Criterion) {
    c.bench_function("Hello World (Parser)", move |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(SRC);
            lexer.lex().expect("failed to lex");
            let tokens = lexer.tokens;
            Parser::new(black_box(tokens)).parse_all().unwrap();
        })
    });
}

fn hello_world(c: &mut Criterion) {
    c.bench_function("Hello World (Execution)", move |b| {
        b.iter(|| exec(black_box(SRC)))
    });
}

criterion_group!(benches, hello_world, hello_world_lexer, hello_world_parser);
criterion_main!(benches);
