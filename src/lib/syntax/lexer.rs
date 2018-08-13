use std::str::Chars;
use syntax::ast::punc::Punctuator;
use syntax::ast::token::{Token, TokenData};

/// A javascript Lexer
pub struct Lexer<'a> {
    // The list fo tokens generated so far
    pub tokens: Vec<Token>,
    // The current line number in the script
    line_number: u64,
    // the current column number in the script
    column_number: u64,
    // The full string
    buffer: Chars<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(buffer: &'a str) -> Lexer<'a> {
        Lexer {
            tokens: Vec::new(),
            line_number: 1,
            column_number: 0,
            buffer: buffer.chars(),
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

    fn next(&mut self) -> Option<char> {
        self.buffer.next()
    }

    pub fn lex(&mut self) {
        loop {
            let ch = self.next();
            match ch {
                Some(c) => {
                    println!("{}", c);
                }
                None => {
                    break;
                }
            }
        }
    }
}
