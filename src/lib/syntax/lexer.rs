use syntax::ast::token::{Token, TokenData};

/// A javascript Lexer
pub struct Lexer {
    // The list fo tokens generated so far
    pub tokens: Vec<Token>,
    // The current line number in the script
    line_number: u64,
    // the current column number in the script
    column_number: u64,
    // the reader
    buffer: String,
}

impl Lexer {
    pub fn new(buffer: String) -> Lexer {
        Lexer {
            tokens: Vec::new(),
            line_number: 1,
            column_number: 0,
            buffer: buffer,
        }
    }

    fn push_token(&mut self, tk: TokenData) {
        self.tokens
            .push(Token::new(tk, self.line_number, self.column_number))
    }
}
