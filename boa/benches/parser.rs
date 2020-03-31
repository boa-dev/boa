#[macro_use]
extern crate criterion;

use boa::syntax::lexer::Lexer;
use boa::syntax::parser::Parser;
use criterion::black_box;
use criterion::Criterion;

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
