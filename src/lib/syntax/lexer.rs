//! A lexical analyzer for JavaScript source code.
//!
//! The Lexer splits its input source code into a sequence of input elements called tokens, represented by the [Token](../ast/token/struct.Token.html) structure.
//! It also removes whitespace and comments and attaches them to the next token.
use crate::syntax::ast::punc::Punctuator;
use crate::syntax::ast::token::{Token, TokenData};
use std::char::{decode_utf16, from_u32};
use std::error;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;
use std::str::FromStr;

#[allow(unused)]
macro_rules! vop {
    ($this:ident, $assign_op:expr, $op:expr) => ({
        let preview = $this.preview_next()?;
        match preview {
            '=' => {
                $this.next()?;
                $assign_op
            }
            _ => $op,
        }
    });
    ($this:ident, $assign_op:expr, $op:expr, {$($case:pat => $block:expr), +}) => ({
        let preview = $this.preview_next()?;
        match preview {
            '=' => {
                $this.next()?;
                $assign_op
            },
            $($case => $block)+,
            _ => $op
        }
    });
    ($this:ident, $op:expr, {$($case:pat => $block:expr),+}) => {
        let preview = $this.preview_next()?;
        match preview {
            $($case => $block) +,
            _ => $op
        }
    }
}

macro_rules! op {
    ($this:ident, $assign_op:expr, $op:expr) => ({
        let punc = vop!($this, $assign_op, $op);
        $this.push_punc(punc);
    });
    ($this:ident, $assign_op:expr, $op:expr, {$($case:pat => $block:expr),+}) => ({
        let punc = vop!($this, $assign_op, $op, {$($case => $block),+});
        $this.push_punc(punc);
    });
    ($this:ident, $op:expr, {$($case:pat => $block:expr),+}) => ({
        let punc = vop!($this, $op, {$($case => $block),+});
        $this.push_punc();
    });
}

/// An error that occurred during lexing or compiling of the source input.
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

impl error::Error for LexerError {
    fn description(&self) -> &str {
        &self.details
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

/// A lexical analyzer for JavaScript source code
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
    /// Returns a Lexer with a buffer inside
    ///
    /// # Arguments
    ///
    /// * `buffer` - A string slice that holds the source code.   
    /// The buffer needs to have a lifetime as long as the Lexer instance itself
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let buffer = std::fs::read_to_string("yourSourceCode.js").unwrap();
    /// let lexer = boa::syntax::lexer::Lexer::new(&buffer);
    /// ```
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
        self.push_token(TokenData::Punctuator(punc));
    }

    /// next fetches the next token and return it, or a LexerError if there are no more.
    fn next(&mut self) -> Result<char, LexerError> {
        match self.buffer.next() {
            Some(char) => Ok(char),
            None => Err(LexerError::new("finished")),
        }
    }

    /// read_line attempts to read until the end of the line and returns the String object or a LexerError
    fn read_line(&mut self) -> Result<String, LexerError> {
        let mut buf = String::new();
        loop {
            let ch = self.next()?;
            match ch {
                _ if ch.is_ascii_control() => {
                    break;
                }
                _ => {
                    buf.push(ch);
                }
            }
        }

        Ok(buf)
    }

    /// Preview the next character but don't actually increment
    fn preview_next(&mut self) -> Result<char, LexerError> {
        // No need to return a reference, we can return a copy
        match self.buffer.peek() {
            Some(v) => Ok(*v),
            None => Err(LexerError::new("finished")),
        }
    }

    /// Utility Function, while ``f(char)`` is true, read chars and move curser.
    /// All chars are returned as a string
    fn take_char_while<F>(&mut self, mut f: F) -> Result<String, LexerError>
    where
        F: FnMut(char) -> bool,
    {
        let mut s = String::new();
        while self.buffer.peek().is_some() && f(self.preview_next()?) {
            s.push(self.next()?);
        }

        Ok(s)
    }

    /// next_is compares the character passed in to the next character, if they match true is returned and the buffer is incremented
    fn next_is(&mut self, peek: char) -> Result<bool, LexerError> {
        let result = self.preview_next()? == peek;
        if result {
            self.buffer.next();
        }
        Ok(result)
    }

    pub fn lex(&mut self) -> Result<(), LexerError> {
        loop {
            // Check if we've reached the end
            match self.preview_next() {
                Ok(_) => (), // If there are still characters, carry on
                Err(e) => {
                    if e.details == "finished" {
                        // If there are no more characters left in the Chars iterator, we should just return
                        return Ok(());
                    } else {
                        return Err(e);
                    }
                }
            }
            self.column_number += 1;
            let ch = self.next()?;
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
                                                Err(_) => 0,
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
                                            // UTF-16 could be surragote pairs, "\uXXXX\uXXXX" which make up a codepoint.
                                            // We will need to loop to make sure we catch all UTF-16 codepoints
                                            // Example Test: https://github.com/tc39/test262/blob/ee3715ee56744ccc8aeb22a921f442e98090b3c1/implementation-contributed/v8/mjsunit/es6/unicode-escapes.js#L39-L44

                                            // Support \u{X..X}
                                            if self.next_is('{')? {
                                                let s = self
                                                    .take_char_while(|c| c.is_alphanumeric())
                                                    .unwrap();

                                                // Convert to u16
                                                let as_num = match u32::from_str_radix(&s, 16) {
                                                    Ok(v) => v,
                                                    Err(_) => 0,
                                                };
                                                let c = from_u32(as_num)
                                                    .expect("Invalid Unicode escape sequence");

                                                self.next()?; // '}'
                                                c
                                            } else {
                                                let mut codepoints: Vec<u16> = vec![];
                                                loop {
                                                    // Collect each character after \u e.g \uD83D will give "D83D"6o0546uu5u
                                                    let s = self
                                                        .take_char_while(|c| c.is_alphanumeric())
                                                        .unwrap();

                                                    // Convert to u16
                                                    let as_num = match u16::from_str_radix(&s, 16) {
                                                        Ok(v) => v,
                                                        Err(_) => 0,
                                                    };

                                                    codepoints.push(as_num);

                                                    // Check for another UTF-16 codepoint
                                                    if self.next_is('\\')? && self.next_is('u')? {
                                                        continue;
                                                    }
                                                    break;
                                                }

                                                // codepoints length should either be 1 (unicode codepoint) or 2 (surrogate codepoint).
                                                // Rust's decode_utf16 will deal with it regardless
                                                let c = decode_utf16(codepoints.iter().cloned())
                                                    .next()
                                                    .unwrap()
                                                    .unwrap();

                                                c
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
                    let str_length = buf.len() as u64;
                    self.push_token(TokenData::StringLiteral(buf));
                    // Why +1? Quotation marks are not included,
                    // So technically it would be +2, (for both " ") but we want to be 1 less
                    // to compensate for the incrementing at the top
                    self.column_number += str_length + 1;
                }
                '0' => {
                    let mut buf = String::new();
                    let num = if self.next_is('x')? {
                        loop {
                            let ch = self.preview_next()?;
                            match ch {
                                ch if ch.is_digit(16) => {
                                    buf.push(self.next()?);
                                }
                                _ => break,
                            }
                        }
                        u64::from_str_radix(&buf, 16).unwrap()
                    } else {
                        let mut gone_decimal = false;
                        loop {
                            let ch = self.preview_next()?;
                            match ch {
                                ch if ch.is_digit(8) => {
                                    buf.push(ch);
                                    self.next()?;
                                }
                                '8' | '9' | '.' => {
                                    gone_decimal = true;
                                    buf.push(ch);
                                    self.next()?;
                                }
                                _ => break,
                            }
                        }
                        if gone_decimal {
                            u64::from_str(&buf).unwrap()
                        } else {
                            if buf.is_empty() {
                                0
                            } else {
                                u64::from_str_radix(&buf, 8).unwrap()
                            }
                        }
                    };
                    self.push_token(TokenData::NumericLiteral(num as f64))
                }
                _ if ch.is_digit(10) => {
                    let mut buf = ch.to_string();
                    loop {
                        let ch = self.preview_next()?;
                        match ch {
                            '.' => {
                                buf.push(self.next()?);
                            }
                            _ if ch.is_digit(10) => {
                                buf.push(self.next()?);
                            }
                            _ => break,
                        }
                    }
                    // TODO make this a bit more safe -------------------------------VVVV
                    self.push_token(TokenData::NumericLiteral(f64::from_str(&buf).unwrap()))
                }
                _ if ch.is_alphabetic() || ch == '$' || ch == '_' => {
                    let mut buf = ch.to_string();
                    loop {
                        let ch = self.preview_next()?;
                        match ch {
                            _ if ch.is_alphabetic() || ch.is_digit(10) || ch == '_' => {
                                buf.push(self.next()?);
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    // Match won't compare &String to &str so i need to convert it :(
                    let buf_compare: &str = &buf;
                    self.push_token(match buf_compare {
                        "true" => TokenData::BooleanLiteral(true),
                        "false" => TokenData::BooleanLiteral(false),
                        "null" => TokenData::NullLiteral,
                        slice => match FromStr::from_str(slice) {
                            Ok(keyword) => TokenData::Keyword(keyword),
                            Err(_) => TokenData::Identifier(buf.clone()),
                        },
                    });
                    // Move position forward the length of keyword
                    self.column_number += (buf_compare.len() - 1) as u64;
                }
                ';' => self.push_punc(Punctuator::Semicolon),
                ':' => self.push_punc(Punctuator::Colon),
                '.' => self.push_punc(Punctuator::Dot),
                '(' => self.push_punc(Punctuator::OpenParen),
                ')' => self.push_punc(Punctuator::CloseParen),
                ',' => self.push_punc(Punctuator::Comma),
                '{' => self.push_punc(Punctuator::OpenBlock),
                '}' => self.push_punc(Punctuator::CloseBlock),
                '[' => self.push_punc(Punctuator::OpenBracket),
                ']' => self.push_punc(Punctuator::CloseBracket),
                '?' => self.push_punc(Punctuator::Question),
                // Comments
                '/' => {
                    let token = match self.preview_next()? {
                        // Matched comment
                        '/' => {
                            let comment = self.read_line()?;
                            TokenData::Comment(comment)
                        }
                        '*' => {
                            let mut buf = String::new();
                            loop {
                                match self.next()? {
                                    '*' => {
                                        if self.next_is('/')? {
                                            break;
                                        } else {
                                            buf.push('*')
                                        }
                                    }
                                    ch => buf.push(ch),
                                }
                            }
                            TokenData::Comment(buf)
                        }
                        '=' => TokenData::Punctuator(Punctuator::AssignDiv),
                        _ => TokenData::Punctuator(Punctuator::Div),
                    };
                    self.push_token(token)
                }
                '*' => op!(self, Punctuator::AssignMul, Punctuator::Mul),
                '+' => op!(self, Punctuator::AssignAdd, Punctuator::Add, {
                    '+' => Punctuator::Inc
                }),
                '-' => op!(self, Punctuator::AssignSub, Punctuator::Sub, {
                    '-' => {
                        self.next()?;
                        Punctuator::Dec
                    }
                }),
                '%' => op!(self, Punctuator::AssignMod, Punctuator::Mod),
                '|' => op!(self, Punctuator::AssignOr, Punctuator::Or, {
                    '|' => Punctuator::BoolOr
                }),
                '&' => op!(self, Punctuator::AssignAnd, Punctuator::And, {
                    '&' => Punctuator::BoolAnd
                }),
                '^' => op!(self, Punctuator::AssignXor, Punctuator::Xor),
                '=' => op!(self, if self.next_is('=')? {
                    Punctuator::StrictEq
                } else {
                    Punctuator::Eq
                }, Punctuator::Assign, {
                    '>' => {
                        self.next()?;
                        Punctuator::Arrow
                    }
                }),
                '<' => op!(self, Punctuator::LessThanOrEq, Punctuator::LessThan, {
                    '<' => vop!(self, Punctuator::AssignLeftSh, Punctuator::LeftSh)
                }),
                '>' => op!(self, Punctuator::GreaterThanOrEq, Punctuator::GreaterThan, {
                    '>' => vop!(self, Punctuator::AssignRightSh, Punctuator::RightSh, {
                        '>' => vop!(self, Punctuator::AssignURightSh, Punctuator::URightSh)
                    })
                }),
                '!' => op!(
                    self,
                    vop!(self, Punctuator::StrictNotEq, Punctuator::NotEq),
                    Punctuator::Not
                ),
                '~' => self.push_punc(Punctuator::Neg),
                '\n' | '\u{2028}' | '\u{2029}' => {
                    self.line_number += 1;
                    self.column_number = 0;
                }
                '\r' => {
                    self.column_number = 0;
                }
                ' ' => (),
                ch => panic!(
                    "{}:{}: Unexpected '{}'",
                    self.line_number, self.column_number, ch
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::ast::keyword::Keyword;

    #[test]
    fn check_variable_definition_tokens() {
        let s = &String::from("let a = 'hello';");
        let mut lexer = Lexer::new(s);
        lexer.lex().expect("finished");
        assert_eq!(lexer.tokens[0].data, TokenData::Keyword(Keyword::Let));
        assert_eq!(lexer.tokens[1].data, TokenData::Identifier("a".to_string()));
        assert_eq!(
            lexer.tokens[2].data,
            TokenData::Punctuator(Punctuator::Assign)
        );
        assert_eq!(
            lexer.tokens[3].data,
            TokenData::StringLiteral("hello".to_string())
        );
    }

    #[test]
    fn check_positions() {
        let s = &String::from("console.log(\"hello world\");");
        // -------------------123456789
        let mut lexer = Lexer::new(s);
        lexer.lex().expect("finished");
        // The first column is 1 (not zero indexed)
        assert_eq!(lexer.tokens[0].pos.column_number, 1);
        assert_eq!(lexer.tokens[0].pos.line_number, 1);
        // Dot Token starts on line 7
        assert_eq!(lexer.tokens[1].pos.column_number, 8);
        assert_eq!(lexer.tokens[1].pos.line_number, 1);
        // Log Token starts on line 7
        assert_eq!(lexer.tokens[2].pos.column_number, 9);
        assert_eq!(lexer.tokens[2].pos.line_number, 1);
        // Open parenthesis token starts on line 12
        assert_eq!(lexer.tokens[3].pos.column_number, 12);
        assert_eq!(lexer.tokens[3].pos.line_number, 1);
        // String token starts on line 13
        assert_eq!(lexer.tokens[4].pos.column_number, 13);
        assert_eq!(lexer.tokens[4].pos.line_number, 1);
        // Close parenthesis token starts on line 26
        assert_eq!(lexer.tokens[5].pos.column_number, 26);
        assert_eq!(lexer.tokens[5].pos.line_number, 1);
        // Semi Colon token starts on line 27
        assert_eq!(lexer.tokens[6].pos.column_number, 27);
        assert_eq!(lexer.tokens[6].pos.line_number, 1);
    }

    // Increment/Decrement
    #[test]
    fn check_decrement_advances_lexer_2_places() {
        // Here we want an example of decrementing an integer
        let s = &String::from("let a = b--;");
        let mut lexer = Lexer::new(s);
        lexer.lex().expect("finished");
        assert_eq!(lexer.tokens[4].data, TokenData::Punctuator(Punctuator::Dec));
        // Decrementing means adding 2 characters '--', the lexer should consume it as a single token
        // and move the curser forward by 2, meaning the next token should be a semicolon
        assert_eq!(
            lexer.tokens[5].data,
            TokenData::Punctuator(Punctuator::Semicolon)
        );
    }

}
