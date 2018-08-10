use syntax::ast::punc::Punctuator;
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
    /// Push tokens onto the token queue
    fn push_token(&mut self, tk: TokenData) {
        self.tokens
            .push(Token::new(tk, self.line_number, self.column_number))
    }

    /// Push a punctuation token
    fn push_punc(&mut self, punc: Punctuator) {
        self.push_token(TokenData::TPunctuator(punc));
    }

    /// Processes an input stream from a string into an array of tokens
    pub fn lex_str(script: String) -> Vec<Token> {
        let mut lexer = Lexer::new(script);
        lexer.tokens
    }

    fn next(&mut self) -> Option<char> {
        self.buffer.chars().next()
    }

    pub fn lex(&mut self) -> Result<(), &str> {
        loop {
            let ch = match self.next() {
                Some(ch) => ch,
                None => return Err("oh my days"),
            };
        }
    }
}
