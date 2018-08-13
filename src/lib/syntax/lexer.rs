use std::char::from_u32;
use std::error;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;
use syntax::ast::punc::Punctuator;
use syntax::ast::token::{Token, TokenData};

// Defining an error type
#[derive(Debug, Clone)]
pub struct LexerError {
    details: String,
}
impl LexerError {
    fn new(msg: &str) -> LexerError {
        LexerError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

// This is important for other errors to wrap this one.
impl error::Error for LexerError {
    fn description(&self) -> &str {
        &self.details
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

/// A javascript Lexer
pub struct Lexer<'a> {
    // The list fo tokens generated so far
    pub tokens: Vec<Token>,
    // The current line number in the script
    line_number: u64,
    // the current column number in the script
    column_number: u64,
    // The full string
    buffer: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(buffer: &'a str) -> Lexer<'a> {
        Lexer {
            tokens: Vec::new(),
            line_number: 1,
            column_number: 0,
            buffer: buffer.chars().peekable(),
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

    fn next(&mut self) -> Result<char, LexerError> {
        self.buffer.next().ok_or(LexerError::new("next failed"))
    }

    fn preview_next(&mut self) -> Result<&char, ()> {
        // ok_or converts Option to a Result
        self.buffer.peek().ok_or(())
    }

    fn next_is(&mut self, peek: char) -> Result<bool, ()> {
        let result = try!(self.preview_next()) == &peek;
        if result {
            self.buffer.next();
        }
        Ok(result)
    }

    pub fn lex(&mut self) -> Result<(), LexerError> {
        loop {
            let ch = match self.next() {
                Ok(ch) => ch,
                Err(LexerError) => return Err(LexerError),
            };
            self.column_number += 1;
            match ch {
                '"' | '\'' => {
                    let mut buf = String::new();
                    loop {
                        match self.next()? {
                            '\'' if ch == '\'' => {
                                break;
                            }
                            '"' if ch == '"' => {
                                break;
                            }
                            '\\' => {
                                let escape = self.next()?;
                                if escape != '\n' {
                                    let escaped_ch = match escape {
                                        'n' => '\n',
                                        'r' => '\r',
                                        't' => '\t',
                                        'b' => '\x08',
                                        'f' => '\x0c',
                                        '0' => '\0',
                                        'x' => {
                                            let mut nums = String::with_capacity(2);
                                            for _ in 0u8..2 {
                                                nums.push(self.next()?);
                                            }
                                            self.column_number += 2;
                                            let as_num = match u64::from_str_radix(&nums, 16) {
                                                Ok(v) => v,
                                                Err(e) => 0,
                                            };
                                            match from_u32(as_num as u32) {
                                                Some(v) => v,
                                                None => panic!(
                                                    "{}:{}: {} is not a valid unicode scalar value",
                                                    self.line_number, self.column_number, as_num
                                                ),
                                            }
                                        }
                                        'u' => {
                                            let mut nums = String::new();
                                            for _ in 0u8..4 {
                                                nums.push(self.next()?);
                                            }
                                            self.column_number += 4;
                                            let as_num = match u64::from_str_radix(&nums, 16) {
                                                Ok(v) => v,
                                                Err(e) => 0,
                                            };
                                            match from_u32(as_num as u32) {
                                                Some(v) => v,
                                                None => panic!(
                                                    "{}:{}: {} is not a valid unicode scalar value",
                                                    self.line_number, self.column_number, as_num
                                                ),
                                            }
                                        }
                                        '\'' | '"' => escape,
                                        _ => panic!(
                                            "{}:{}: Invalid escape `{}`",
                                            self.line_number, self.column_number, ch
                                        ),
                                    };
                                    buf.push(escaped_ch);
                                }
                            }
                            ch => buf.push(ch),
                        }
                    }
                }
            }
        }
    }
}
