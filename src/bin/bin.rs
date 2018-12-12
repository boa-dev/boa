extern crate boa;
use boa::exec::{Executor, Interpreter};
use boa::syntax::lexer::Lexer;
use boa::syntax::parser::Parser;
use std::fs::read_to_string;

pub fn main() {
    let buffer = read_to_string("tests/js/test.js").unwrap();
    let mut lexer = Lexer::new(&buffer);
    lexer.lex().unwrap();
    let tokens = lexer.tokens;

    // Setup executor
    let expr = Parser::new(tokens).parse_all().unwrap();
    // print!("{:#?}", expr);

    let mut engine: Interpreter = Executor::new();
    let result = engine.run(&expr);
    match result {
        Ok(v) => print!("{}", v),
        Err(v) => print!("Error: {}", v),
    }
}
