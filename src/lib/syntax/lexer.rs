//! A lexical analyzer for JavaScript source code.
//!
//! The Lexer splits its input source code into a sequence of input elements called tokens, represented by the [Token](../ast/token/struct.Token.html) structure.
//! It also removes whitespace and comments and attaches them to the next token.
use crate::syntax::ast::{
    punc::Punctuator,
    token::{Token, TokenData},
};

use std::{
    char::{decode_utf16, from_u32},
    error, fmt,
    iter::Peekable,
    str::{Chars, FromStr},
};

macro_rules! vop {
    ($this:ident, $assign_op:expr, $op:expr) => ({
        let preview = $this.preview_next().unwrap();
        match preview {
            '=' => {
                $this.next()?;
                $assign_op
            }
            _ => $op,
        }
    });
    ($this:ident, $assign_op:expr, $op:expr, {$($case:pat => $block:expr), +}) => ({
        let preview = $this.preview_next().unwrap();
        match preview {
            '=' => {
                $this.next()?;
                $assign_op
            },
            $($case => {
                $this.next()?;
                $block
            })+,
            _ => $op
        }
    });
    ($this:ident, $op:expr, {$($case:pat => $block:expr),+}) => {
        let preview = $this.preview_next().unwrap();
        match preview {
            $($case => {
                $this.next()?;
                $block
            })+,
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
    fn new(msg: &str) -> Self {
        Self {
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

    fn cause(&self) -> Option<&dyn error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

/// A lexical analyzer for JavaScript source code
#[derive(Debug)]
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
            Some(ch) => Ok(ch),
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
    fn preview_next(&mut self) -> Option<char> {
        self.buffer.peek().copied()
    }

    /// Utility Function, while ``f(char)`` is true, read chars and move curser.
    /// All chars are returned as a string
    fn take_char_while<F>(&mut self, mut f: F) -> Result<String, LexerError>
    where
        F: FnMut(char) -> bool,
    {
        let mut s = String::new();
        while self.buffer.peek().is_some() && f(self.preview_next().unwrap()) {
            s.push(self.next()?);
        }

        Ok(s)
    }

    /// next_is compares the character passed in to the next character, if they match true is returned and the buffer is incremented
    fn next_is(&mut self, peek: char) -> bool {
        let result = self.preview_next() == Some(peek);
        if result {
            self.buffer.next();
        }
        result
    }

    pub fn lex(&mut self) -> Result<(), LexerError> {
        loop {
            // Check if we've reached the end
            if self.preview_next().is_none() {
                return Ok(());
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
                                            for _ in 0_u8..2 {
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
                                            // There are 2 types of codepoints. Surragate codepoints and unicode codepoints.
                                            // UTF-16 could be surrogate codepoints, "\uXXXX\uXXXX" which make up a single unicode codepoint.
                                            // We will need to loop to make sure we catch all UTF-16 codepoints
                                            // Example Test: https://github.com/tc39/test262/blob/ee3715ee56744ccc8aeb22a921f442e98090b3c1/implementation-contributed/v8/mjsunit/es6/unicode-escapes.js#L39-L44

                                            // Support \u{X..X} (Unicode Codepoint)
                                            if self.next_is('{') {
                                                let s = self
                                                    .take_char_while(char::is_alphanumeric)
                                                    .unwrap();

                                                // We know this is a single unicode codepoint, convert to u32
                                                let as_num = match u32::from_str_radix(&s, 16) {
                                                    Ok(v) => v,
                                                    Err(_) => 0,
                                                };
                                                let c = from_u32(as_num)
                                                    .expect("Invalid Unicode escape sequence");

                                                self.next()?; // '}'
                                                self.column_number += s.len() as u64 + 3;
                                                c
                                            } else {
                                                let mut codepoints: Vec<u16> = vec![];
                                                loop {
                                                    // Collect each character after \u e.g \uD83D will give "D83D"
                                                    let s = self
                                                        .take_char_while(char::is_alphanumeric)
                                                        .unwrap();

                                                    // Convert to u16
                                                    let as_num = match u16::from_str_radix(&s, 16) {
                                                        Ok(v) => v,
                                                        Err(_) => 0,
                                                    };

                                                    codepoints.push(as_num);
                                                    self.column_number += s.len() as u64 + 2;

                                                    // Check for another UTF-16 codepoint
                                                    if self.next_is('\\') && self.next_is('u') {
                                                        continue;
                                                    }
                                                    break;
                                                }

                                                // codepoints length should either be 1 (unicode codepoint) or 2 (surrogate codepoint).
                                                // Rust's decode_utf16 will deal with it regardless
                                                decode_utf16(codepoints.iter().cloned())
                                                    .next()
                                                    .unwrap()
                                                    .unwrap()
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
                            next_ch => buf.push(next_ch),
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
                    let num = if self.next_is('x') {
                        while let Some(ch) = self.preview_next() {
                            if ch.is_digit(16) {
                                buf.push(self.next()?);
                            } else {
                                break;
                            }
                        }
                        u64::from_str_radix(&buf, 16).unwrap()
                    } else if self.next_is('b') {
                        while let Some(ch) = self.preview_next() {
                            if ch.is_digit(2) {
                                buf.push(self.next()?);
                            } else {
                                break;
                            }
                        }
                        u64::from_str_radix(&buf, 2).unwrap()
                    } else {
                        let mut gone_decimal = false;
                        loop {
                            let next_ch = self.preview_next().unwrap_or('_');
                            match next_ch {
                                next_ch if next_ch.is_digit(8) => {
                                    buf.push(next_ch);
                                    self.next()?;
                                }
                                'O' | 'o' => {
                                    self.next()?;
                                }
                                '8' | '9' | '.' => {
                                    gone_decimal = true;
                                    buf.push(next_ch);
                                    self.next()?;
                                }
                                _ => break,
                            }
                        }
                        if gone_decimal {
                            u64::from_str(&buf).unwrap()
                        } else if buf.is_empty() {
                            0
                        } else {
                            u64::from_str_radix(&buf, 8).unwrap()
                        }
                    };
                    self.push_token(TokenData::NumericLiteral(num as f64))
                }
                _ if ch.is_digit(10) => {
                    let mut buf = ch.to_string();
                    'digitloop: while let Some(ch) = self.preview_next() {
                        match ch {
                            '.' => loop {
                                buf.push(self.next()?);

                                let ch = match self.preview_next() {
                                    Some(ch) => ch,
                                    None => break,
                                };

                                if !ch.is_digit(10) {
                                    break 'digitloop;
                                }
                            },
                            'e' | '+' | '-' => {
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
                    while let Some(ch) = self.preview_next() {
                        if ch.is_alphabetic() || ch.is_digit(10) || ch == '_' {
                            buf.push(self.next()?);
                        } else {
                            break;
                        }
                    }
                    // Match won't compare &String to &str so i need to convert it :(
                    let buf_compare: &str = &buf;
                    self.push_token(match buf_compare {
                        "true" => TokenData::BooleanLiteral(true),
                        "false" => TokenData::BooleanLiteral(false),
                        "null" => TokenData::NullLiteral,
                        slice => {
                            if let Ok(keyword) = FromStr::from_str(slice) {
                                TokenData::Keyword(keyword)
                            } else {
                                TokenData::Identifier(buf.clone())
                            }
                        }
                    });
                    // Move position forward the length of keyword
                    self.column_number += (buf_compare.len() - 1) as u64;
                }
                ';' => self.push_punc(Punctuator::Semicolon),
                ':' => self.push_punc(Punctuator::Colon),
                '.' => {
                    // . or ...
                    if self.next_is('.') {
                        if self.next_is('.') {
                            self.push_punc(Punctuator::Spread);
                        } else {
                            return Err(LexerError::new("Expecting Token ."));
                        }
                    } else {
                        self.push_punc(Punctuator::Dot);
                    };
                }
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
                    if let Some(ch) = self.preview_next() {
                        let token = match ch {
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
                                            if self.next_is('/') {
                                                break;
                                            } else {
                                                buf.push('*')
                                            }
                                        }
                                        next_ch => buf.push(next_ch),
                                    }
                                }
                                TokenData::Comment(buf)
                            }
                            '=' => TokenData::Punctuator(Punctuator::AssignDiv),
                            _ => TokenData::Punctuator(Punctuator::Div),
                        };
                        self.push_token(token)
                    } else {
                        return Err(LexerError::new("Expecting Token /,*,="));
                    }
                }
                '*' => op!(self, Punctuator::AssignMul, Punctuator::Mul, {
                    '*' => vop!(self, Punctuator::AssignPow, Punctuator::Pow)
                }),
                '+' => op!(self, Punctuator::AssignAdd, Punctuator::Add, {
                    '+' => Punctuator::Inc
                }),
                '-' => op!(self, Punctuator::AssignSub, Punctuator::Sub, {
                    '-' => {
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
                '=' => op!(self, if self.next_is('=') {
                    Punctuator::StrictEq
                } else {
                    Punctuator::Eq
                }, Punctuator::Assign, {
                    '>' => {
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
    fn check_string() {
        let s = "'aaa' \"bbb\"";
        let mut lexer = Lexer::new(s);
        lexer.lex().unwrap();
        assert_eq!(
            lexer.tokens[0].data,
            TokenData::StringLiteral("aaa".to_string())
        );

        assert_eq!(
            lexer.tokens[1].data,
            TokenData::StringLiteral("bbb".to_string())
        );
    }

    #[test]
    fn check_punctuators() {
        // https://tc39.es/ecma262/#sec-punctuators
        let s = "{ ( ) [ ] . ... ; , < > <= >= == != === !== \
                 + - * % -- << >> >>> & | ^ ! ~ && || ? : \
                 = += -= *= &= **= ++ ** <<= >>= >>>= &= |= ^= =>";
        let mut lexer = Lexer::new(s);
        lexer.lex().unwrap();
        assert_eq!(
            lexer.tokens[0].data,
            TokenData::Punctuator(Punctuator::OpenBlock)
        );
        assert_eq!(
            lexer.tokens[1].data,
            TokenData::Punctuator(Punctuator::OpenParen)
        );
        assert_eq!(
            lexer.tokens[2].data,
            TokenData::Punctuator(Punctuator::CloseParen)
        );
        assert_eq!(
            lexer.tokens[3].data,
            TokenData::Punctuator(Punctuator::OpenBracket)
        );
        assert_eq!(
            lexer.tokens[4].data,
            TokenData::Punctuator(Punctuator::CloseBracket)
        );
        assert_eq!(lexer.tokens[5].data, TokenData::Punctuator(Punctuator::Dot));
        assert_eq!(
            lexer.tokens[6].data,
            TokenData::Punctuator(Punctuator::Spread)
        );
        assert_eq!(
            lexer.tokens[7].data,
            TokenData::Punctuator(Punctuator::Semicolon)
        );
        assert_eq!(
            lexer.tokens[8].data,
            TokenData::Punctuator(Punctuator::Comma)
        );
        assert_eq!(
            lexer.tokens[9].data,
            TokenData::Punctuator(Punctuator::LessThan)
        );
        assert_eq!(
            lexer.tokens[10].data,
            TokenData::Punctuator(Punctuator::GreaterThan)
        );
        assert_eq!(
            lexer.tokens[11].data,
            TokenData::Punctuator(Punctuator::LessThanOrEq)
        );
        assert_eq!(
            lexer.tokens[12].data,
            TokenData::Punctuator(Punctuator::GreaterThanOrEq)
        );
        assert_eq!(lexer.tokens[13].data, TokenData::Punctuator(Punctuator::Eq));
        assert_eq!(
            lexer.tokens[14].data,
            TokenData::Punctuator(Punctuator::NotEq)
        );
        assert_eq!(
            lexer.tokens[15].data,
            TokenData::Punctuator(Punctuator::StrictEq)
        );
        assert_eq!(
            lexer.tokens[16].data,
            TokenData::Punctuator(Punctuator::StrictNotEq)
        );
        assert_eq!(
            lexer.tokens[17].data,
            TokenData::Punctuator(Punctuator::Add)
        );
        assert_eq!(
            lexer.tokens[18].data,
            TokenData::Punctuator(Punctuator::Sub)
        );
        assert_eq!(
            lexer.tokens[19].data,
            TokenData::Punctuator(Punctuator::Mul)
        );
        assert_eq!(
            lexer.tokens[20].data,
            TokenData::Punctuator(Punctuator::Mod)
        );
        assert_eq!(
            lexer.tokens[21].data,
            TokenData::Punctuator(Punctuator::Dec)
        );
        assert_eq!(
            lexer.tokens[22].data,
            TokenData::Punctuator(Punctuator::LeftSh)
        );
        assert_eq!(
            lexer.tokens[23].data,
            TokenData::Punctuator(Punctuator::RightSh)
        );
        assert_eq!(
            lexer.tokens[24].data,
            TokenData::Punctuator(Punctuator::URightSh)
        );
        assert_eq!(
            lexer.tokens[25].data,
            TokenData::Punctuator(Punctuator::And)
        );
        assert_eq!(lexer.tokens[26].data, TokenData::Punctuator(Punctuator::Or));
        assert_eq!(
            lexer.tokens[27].data,
            TokenData::Punctuator(Punctuator::Xor)
        );
        assert_eq!(
            lexer.tokens[28].data,
            TokenData::Punctuator(Punctuator::Not)
        );
        assert_eq!(
            lexer.tokens[29].data,
            TokenData::Punctuator(Punctuator::Neg)
        );
        assert_eq!(
            lexer.tokens[30].data,
            TokenData::Punctuator(Punctuator::BoolAnd)
        );
        assert_eq!(
            lexer.tokens[31].data,
            TokenData::Punctuator(Punctuator::BoolOr)
        );
        assert_eq!(
            lexer.tokens[32].data,
            TokenData::Punctuator(Punctuator::Question)
        );
        assert_eq!(
            lexer.tokens[33].data,
            TokenData::Punctuator(Punctuator::Colon)
        );
        assert_eq!(
            lexer.tokens[34].data,
            TokenData::Punctuator(Punctuator::Assign)
        );
        assert_eq!(
            lexer.tokens[35].data,
            TokenData::Punctuator(Punctuator::AssignAdd)
        );
        assert_eq!(
            lexer.tokens[36].data,
            TokenData::Punctuator(Punctuator::AssignSub)
        );
        assert_eq!(
            lexer.tokens[37].data,
            TokenData::Punctuator(Punctuator::AssignMul)
        );
        assert_eq!(
            lexer.tokens[38].data,
            TokenData::Punctuator(Punctuator::AssignAnd)
        );
        assert_eq!(
            lexer.tokens[39].data,
            TokenData::Punctuator(Punctuator::AssignPow)
        );
        assert_eq!(
            lexer.tokens[40].data,
            TokenData::Punctuator(Punctuator::Inc)
        );
        assert_eq!(
            lexer.tokens[41].data,
            TokenData::Punctuator(Punctuator::Pow)
        );
        assert_eq!(
            lexer.tokens[42].data,
            TokenData::Punctuator(Punctuator::AssignLeftSh)
        );
        assert_eq!(
            lexer.tokens[43].data,
            TokenData::Punctuator(Punctuator::AssignRightSh)
        );
        assert_eq!(
            lexer.tokens[44].data,
            TokenData::Punctuator(Punctuator::AssignURightSh)
        );
        assert_eq!(
            lexer.tokens[45].data,
            TokenData::Punctuator(Punctuator::AssignAnd)
        );
        assert_eq!(
            lexer.tokens[46].data,
            TokenData::Punctuator(Punctuator::AssignOr)
        );
        assert_eq!(
            lexer.tokens[47].data,
            TokenData::Punctuator(Punctuator::AssignXor)
        );
        assert_eq!(
            lexer.tokens[48].data,
            TokenData::Punctuator(Punctuator::Arrow)
        );
    }

    #[test]
    fn check_keywords() {
        // https://tc39.es/ecma262/#sec-keywords
        let s = "await break case catch class const continue debugger default delete \
                 do else export extends finally for function if import in instanceof \
                 new return super switch this throw try typeof var void while with yield";

        let mut lexer = Lexer::new(s);
        lexer.lex().unwrap();
        assert_eq!(lexer.tokens[0].data, TokenData::Keyword(Keyword::Await));
        assert_eq!(lexer.tokens[1].data, TokenData::Keyword(Keyword::Break));
        assert_eq!(lexer.tokens[2].data, TokenData::Keyword(Keyword::Case));
        assert_eq!(lexer.tokens[3].data, TokenData::Keyword(Keyword::Catch));
        assert_eq!(lexer.tokens[4].data, TokenData::Keyword(Keyword::Class));
        assert_eq!(lexer.tokens[5].data, TokenData::Keyword(Keyword::Const));
        assert_eq!(lexer.tokens[6].data, TokenData::Keyword(Keyword::Continue));
        assert_eq!(lexer.tokens[7].data, TokenData::Keyword(Keyword::Debugger));
        assert_eq!(lexer.tokens[8].data, TokenData::Keyword(Keyword::Default));
        assert_eq!(lexer.tokens[9].data, TokenData::Keyword(Keyword::Delete));
        assert_eq!(lexer.tokens[10].data, TokenData::Keyword(Keyword::Do));
        assert_eq!(lexer.tokens[11].data, TokenData::Keyword(Keyword::Else));
        assert_eq!(lexer.tokens[12].data, TokenData::Keyword(Keyword::Export));
        assert_eq!(lexer.tokens[13].data, TokenData::Keyword(Keyword::Extends));
        assert_eq!(lexer.tokens[14].data, TokenData::Keyword(Keyword::Finally));
        assert_eq!(lexer.tokens[15].data, TokenData::Keyword(Keyword::For));
        assert_eq!(lexer.tokens[16].data, TokenData::Keyword(Keyword::Function));
        assert_eq!(lexer.tokens[17].data, TokenData::Keyword(Keyword::If));
        assert_eq!(lexer.tokens[18].data, TokenData::Keyword(Keyword::Import));
        assert_eq!(lexer.tokens[19].data, TokenData::Keyword(Keyword::In));
        assert_eq!(
            lexer.tokens[20].data,
            TokenData::Keyword(Keyword::InstanceOf)
        );
        assert_eq!(lexer.tokens[21].data, TokenData::Keyword(Keyword::New));
        assert_eq!(lexer.tokens[22].data, TokenData::Keyword(Keyword::Return));
        assert_eq!(lexer.tokens[23].data, TokenData::Keyword(Keyword::Super));
        assert_eq!(lexer.tokens[24].data, TokenData::Keyword(Keyword::Switch));
        assert_eq!(lexer.tokens[25].data, TokenData::Keyword(Keyword::This));
        assert_eq!(lexer.tokens[26].data, TokenData::Keyword(Keyword::Throw));
        assert_eq!(lexer.tokens[27].data, TokenData::Keyword(Keyword::Try));
        assert_eq!(lexer.tokens[28].data, TokenData::Keyword(Keyword::TypeOf));
        assert_eq!(lexer.tokens[29].data, TokenData::Keyword(Keyword::Var));
        assert_eq!(lexer.tokens[30].data, TokenData::Keyword(Keyword::Void));
        assert_eq!(lexer.tokens[31].data, TokenData::Keyword(Keyword::While));
        assert_eq!(lexer.tokens[32].data, TokenData::Keyword(Keyword::With));
        assert_eq!(lexer.tokens[33].data, TokenData::Keyword(Keyword::Yield));
    }

    #[test]
    fn check_variable_definition_tokens() {
        let s = &String::from("let a = 'hello';");
        let mut lexer = Lexer::new(s);
        lexer.lex().unwrap();
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
        lexer.lex().unwrap();
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
        lexer.lex().unwrap();
        assert_eq!(lexer.tokens[4].data, TokenData::Punctuator(Punctuator::Dec));
        // Decrementing means adding 2 characters '--', the lexer should consume it as a single token
        // and move the curser forward by 2, meaning the next token should be a semicolon
        assert_eq!(
            lexer.tokens[5].data,
            TokenData::Punctuator(Punctuator::Semicolon)
        );
    }

    #[test]
    fn numbers() {
        let mut lexer = Lexer::new("1 2 0x34 056 7.89 42. 5e3 5e+3 5e-3 0b10 0O123 0999");
        lexer.lex().unwrap();
        assert_eq!(lexer.tokens[0].data, TokenData::NumericLiteral(1.0));
        assert_eq!(lexer.tokens[1].data, TokenData::NumericLiteral(2.0));
        assert_eq!(lexer.tokens[2].data, TokenData::NumericLiteral(52.0));
        assert_eq!(lexer.tokens[3].data, TokenData::NumericLiteral(46.0));
        assert_eq!(lexer.tokens[4].data, TokenData::NumericLiteral(7.89));
        assert_eq!(lexer.tokens[5].data, TokenData::NumericLiteral(42.0));
        assert_eq!(lexer.tokens[6].data, TokenData::NumericLiteral(5000.0));
        assert_eq!(lexer.tokens[7].data, TokenData::NumericLiteral(5000.0));
        assert_eq!(lexer.tokens[8].data, TokenData::NumericLiteral(0.005));
        assert_eq!(lexer.tokens[9].data, TokenData::NumericLiteral(2.0));
        assert_eq!(lexer.tokens[10].data, TokenData::NumericLiteral(83.0));
        assert_eq!(lexer.tokens[11].data, TokenData::NumericLiteral(999.0));
    }

    #[test]
    fn test_single_number_without_semicolon() {
        let mut lexer = Lexer::new("1");
        lexer.lex().unwrap();
        dbg!(lexer.tokens);
    }

    #[test]
    fn test_number_followed_by_dot() {
        let mut lexer = Lexer::new("1..");
        lexer.lex().unwrap();
        assert_eq!(lexer.tokens[0].data, TokenData::NumericLiteral(1.0));
        assert_eq!(lexer.tokens[1].data, TokenData::Punctuator(Punctuator::Dot));
    }

}
