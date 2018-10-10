extern crate boa;
use boa::syntax::lexer::Lexer;
use boa::syntax::parser::Parser;
use std::fs::read_to_string;

pub fn main() {
    let buffer = read_to_string("tests/js/defineVar.js").unwrap();
    let mut lexer = Lexer::new(&buffer);
    lexer.lex().unwrap();
    let tokens = lexer.tokens;

    // Setup executor
    let expr = Parser::new(tokens).parse_all().unwrap();
    println!("{}", expr);
}
