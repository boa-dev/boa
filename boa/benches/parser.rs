use boa::syntax::{lexer::Lexer, parser::Parser};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

static EXPRESSION: &str = r#"
1 + 1 + 1 + 1 + 1 + 1 / 1 + 1 + 1 * 1 + 1 + 1 + 1;
"#;

fn expression_parser(c: &mut Criterion) {
    // Don't include lexing as part of the parser benchmark
    let mut lexer = Lexer::new(EXPRESSION);
    lexer.lex().expect("failed to lex");
    let tokens = lexer.tokens;
    c.bench_function_over_inputs(
        "Expression (Parser)",
        move |b, tok| {
            b.iter(|| {
                Parser::new(&black_box(tok.to_vec())).parse_all().unwrap();
            })
        },
        vec![tokens],
    );
}

criterion_group!(benches, expression_parser);
criterion_main!(benches);
