use std::io::{BufRead, BufReader, ErrorKind};
use std::str::Chars;
use syntax::ast::punc::Punctuator;
use syntax::ast::token::{Token, TokenData};

/// A javascript Lexer
pub struct Lexer<B> {
    // The list fo tokens generated so far
    pub tokens: Vec<Token>,
    // The current line number in the script
    line_number: u64,
    // the current column number in the script
    column_number: u64,
    // The full string
    buffer: B,
}

impl<B: BufRead> Lexer<B> {
    pub fn new(buffer: B) -> Lexer<B> {
        Lexer {
            tokens: Vec::new(),
            line_number: 1,
            column_number: 0,
            buffer: buffer,
        }
    }
    /// Push tokens onto the token queue
    fn push_token(&mut self, tk: TokenData) {
        self.tokens
            .push(Token::new(tk, self.line_number, self.column_number))
    }

    /// Push a punctuation token
    fn push_punc(&mut self, punc: Punctuator) {
        self.push_token(TokenData::TPunctuator(punc));
    }

    fn next(&mut self) -> char {
        let mut buffer = [0; 1];
        self.buffer.read_exact(&mut buffer).unwrap();
        let result = buffer as char;
        result
    }

    pub fn lex(&mut self) {
        loop {
            let ch = self.next();
            println!("{}", ch.unwrap());
        }
    }
}
