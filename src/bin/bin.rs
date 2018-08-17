extern crate js;
use js::syntax::lexer::Lexer;
use std::fs::read_to_string;

pub fn main() {
    let buffer = read_to_string("test.js").unwrap();
    let mut lexer = Lexer::new(&buffer);
    lexer.lex().expect("finished");
    println!("{:?}", lexer.tokens);
}
