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
            // return the value into the blackbox so its not optimized away
            // https://gist.github.com/jasonwilliams/5325da61a794d8211dcab846d466c4fd
            lexer.lex()
        })
    });
}

fn hello_world_parser(c: &mut Criterion) {
    // Don't include lexing as part of the parser benchmark
    let mut lexer = Lexer::new(SRC);
    lexer.lex().expect("failed to lex");
    let tokens = lexer.tokens;
    c.bench_function_over_inputs(
        "Hello World (Parser)",
        move |b, tok| {
            b.iter(|| {
                Parser::new(black_box(tok.to_vec())).parse_all().unwrap();
            })
        },
        vec![tokens],
    );
}

fn hello_world(c: &mut Criterion) {
    c.bench_function("Hello World (Execution)", move |b| {
        b.iter(|| exec(black_box(SRC)))
    });
}

criterion_group!(benches, hello_world, hello_world_lexer, hello_world_parser);
criterion_main!(benches);
