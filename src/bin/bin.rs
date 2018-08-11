extern crate js;
use js::syntax::lexer::Lexer;
use std::fs::read_to_string;
use std::fs::File;
use std::io::BufReader;

pub fn main() {
    let mut f = File::open("test.js").expect("file not found");
    let mut reader = BufReader::new(f);
    let mut lexer = Lexer::new(reader);
    lexer.lex()
}
