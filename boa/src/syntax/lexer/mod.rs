//! A lexical analyzer for JavaScript source code.
//!
//! The Lexer splits its input source code into a sequence of input elements called tokens, represented by the [Token](../ast/token/struct.Token.html) structure.
//! It also removes whitespace and comments and attaches them to the next token.

#[cfg(test)]
mod tests;

use crate::syntax::ast::bigint::BigInt;
use crate::syntax::ast::{
    punc::Punctuator,
    token::{NumericLiteral, Token, TokenKind},
};
use std::{
    char::{decode_utf16, from_u32},
    error, fmt,
    iter::Peekable,
    str::{Chars, FromStr},
};

/// `vop` tests the next token to see if we're on an assign operation of just a plain binary operation.
///
/// If the next value is not an assignment operation it will pattern match  the provided values and return the corresponding token.
macro_rules! vop {
    ($this:ident, $assign_op:expr, $op:expr) => ({
        let preview = $this.preview_next().ok_or_else(|| LexerError::new("Could not preview next value"))?;
        match preview {
            '=' => {
                $this.next();
                $this.column_number += 1;
                $assign_op
            }
            _ => $op,
        }
    });
    ($this:ident, $assign_op:expr, $op:expr, {$($case:pat => $block:expr), +}) => ({
        let preview = $this.preview_next().ok_or_else(|| LexerError::new("Could not preview next value"))?;
        match preview {
            '=' => {
                $this.next();
                $this.column_number += 1;
                $assign_op
            },
            $($case => {
                $this.next();
                $this.column_number += 1;
                $block
            })+,
            _ => $op
        }
    });
    ($this:ident, $op:expr, {$($case:pat => $block:expr),+}) => {
        let preview = $this.preview_next().ok_or_else(|| LexerError::new("Could not preview next value"))?;
        match preview {
            $($case => {
                $this.next()?;
                $this.column_number += 1;
                $block
            })+,
            _ => $op
        }
    }
}

/// The `op` macro handles binary operations or assignment operations and converts them into tokens.
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
///
/// [LexerError] implements [fmt::Display] so you just display this value as an error
#[derive(Debug, Clone)]
pub struct LexerError {
    /// details will be displayed when a LexerError occurs.
    details: String,
}

impl LexerError {
    /// Create a new LexerError struct
    ///
    /// * `msg` - The message to show when LexerError is displayed
    fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// A lexical analyzer for JavaScript source code.
#[derive(Debug)]
pub struct Lexer<'a> {
    /// The list of tokens generated so far.
    ///
    /// This field is public so you can use them once lexing has finished.
    pub tokens: Vec<Token>,
    /// The current line number in the script
    line_number: u64,
    /// the current column number in the script
    column_number: u64,
    /// The full Peekable buffer, an array of [Char]s
    buffer: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    /// Returns a Lexer with a buffer inside
    ///
    /// The buffer needs to have a lifetime as long as the Lexer instance itself
    pub fn new(buffer: &'a str) -> Lexer<'a> {
        Lexer {
            tokens: Vec::new(),
            line_number: 1,
            column_number: 0,
            buffer: buffer.chars().peekable(),
        }
    }

    /// Push a token onto the token queue.
    fn push_token(&mut self, tk: TokenKind) {
        self.tokens
            .push(Token::new(tk, self.line_number, self.column_number))
    }

    /// Push a punctuation token
    fn push_punc(&mut self, punc: Punctuator) {
        self.push_token(TokenKind::Punctuator(punc));
    }

    /// next fetches the next token and return it, or a LexerError if there are no more.
    fn next(&mut self) -> char {
        self.buffer.next().expect(
            "No more more characters to consume from input stream, \
             use preview_next() first to check before calling next()",
        )
    }

    /// Preview the next character but don't actually increment
    fn preview_next(&mut self) -> Option<char> {
        self.buffer.peek().copied()
    }

    /// Preview a char x indexes further in buf, without incrementing
    fn preview_multiple_next(&mut self, nb_next: usize) -> Option<char> {
        let mut next_peek = None;

        for (i, x) in self.buffer.clone().enumerate() {
            if i >= nb_next {
                break;
            }

            next_peek = Some(x);
        }

        next_peek
    }

    /// Utility Function, while ``f(char)`` is true, read chars and move curser.
    /// All chars are returned as a string
    fn take_char_while<F>(&mut self, mut f: F) -> Result<String, LexerError>
    where
        F: FnMut(char) -> bool,
    {
        let mut s = String::new();
        while self.buffer.peek().is_some()
            && f(self.preview_next().expect("Could not preview next value"))
        {
            s.push(self.next());
        }

        Ok(s)
    }

    /// Compares the character passed in to the next character, if they match true is returned and the buffer is incremented
    fn next_is(&mut self, peek: char) -> bool {
        let result = self.preview_next() == Some(peek);
        if result {
            self.buffer.next();
        }
        result
    }

    /// Utility function for checkint the NumericLiteral is not followed by an `IdentifierStart` or `DecimalDigit` character.
    ///
    /// More information:
    ///  - [ECMAScript Specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-literals-numeric-literals
    fn check_after_numeric_literal(&mut self) -> Result<(), LexerError> {
        match self.preview_next() {
            Some(ch)
                if ch.is_ascii_alphabetic() || ch == '$' || ch == '_' || ch.is_ascii_digit() =>
            {
                Err(LexerError::new("NumericLiteral token must not be followed by IdentifierStart nor DecimalDigit characters"))
            }
            Some(_) => Ok(()),
            None => Ok(())
        }
    }

    /// Lexes a numerical literal.
    ///
    /// More information:
    ///  - [ECMAScript Specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-literals-numeric-literals
    fn reed_numerical_literal(&mut self, ch: char) -> Result<(), LexerError> {
        /// This is a helper structure
        ///
        /// This structure helps with identifying what numerical type it is and what base is it.
        enum NumericKind {
            Rational,
            Integer(u32),
            BigInt(u32),
        }

        impl NumericKind {
            /// Get the base of the number kind.
            fn base(&self) -> u32 {
                match self {
                    Self::Rational => 10,
                    Self::Integer(ref base) => *base,
                    Self::BigInt(ref base) => *base,
                }
            }

            /// Converts `self` to BigInt kind.
            fn convert_to_bigint(&mut self) {
                *self = match *self {
                    Self::Rational => unreachable!("can not convert rational number to BigInt"),
                    Self::Integer(base) => Self::BigInt(base),
                    Self::BigInt(base) => Self::BigInt(base),
                };
            }
        }

        // TODO: Setup strict mode.
        let strict_mode = false;

        let mut buf = ch.to_string();
        let mut position_offset = 0;
        let mut kind = NumericKind::Integer(10);
        if ch == '0' {
            match self.preview_next() {
                None => {
                    self.push_token(TokenKind::NumericLiteral(NumericLiteral::Integer(0)));
                    self.column_number += 1;
                    return Ok(());
                }
                Some('x') | Some('X') => {
                    self.next();
                    position_offset += 1;
                    kind = NumericKind::Integer(16);
                }
                Some('o') | Some('O') => {
                    self.next();
                    position_offset += 1;
                    kind = NumericKind::Integer(8);
                }
                Some('b') | Some('B') => {
                    self.next();
                    position_offset += 1;
                    kind = NumericKind::Integer(2);
                }
                Some(ch) if ch.is_ascii_digit() => {
                    let mut is_implicit_octal = true;
                    while let Some(ch) = self.preview_next() {
                        if !ch.is_ascii_digit() {
                            break;
                        } else if !ch.is_digit(8) {
                            is_implicit_octal = false;
                        }
                        buf.push(self.next());
                    }
                    if !strict_mode {
                        if is_implicit_octal {
                            kind = NumericKind::Integer(8);
                        }
                    } else {
                        return Err(if is_implicit_octal {
                            LexerError::new(
                                "Implicit octal literals are not allowed in strict mode.",
                            )
                        } else {
                            LexerError::new(
                                "Decimals with leading zeros are not allowed in strict mode.",
                            )
                        });
                    }
                }
                Some(_) => {}
            }
        }

        while let Some(ch) = self.preview_next() {
            if !ch.is_digit(kind.base()) {
                break;
            }
            buf.push(self.next());
        }

        if self.next_is('n') {
            kind.convert_to_bigint()
        }

        if let NumericKind::Integer(10) = kind {
            'digitloop: while let Some(ch) = self.preview_next() {
                match ch {
                    '.' => loop {
                        kind = NumericKind::Rational;
                        buf.push(self.next());

                        let c = match self.preview_next() {
                            Some(ch) => ch,
                            None => break,
                        };

                        match c {
                            'e' | 'E' => {
                                match self
                                    .preview_multiple_next(2)
                                    .unwrap_or_default()
                                    .to_digit(10)
                                {
                                    Some(0..=9) | None => {
                                        buf.push(self.next());
                                    }
                                    _ => {
                                        break 'digitloop;
                                    }
                                }
                            }
                            _ => {
                                if !c.is_digit(10) {
                                    break 'digitloop;
                                }
                            }
                        }
                    },
                    'e' | 'E' => {
                        kind = NumericKind::Rational;
                        match self
                            .preview_multiple_next(2)
                            .unwrap_or_default()
                            .to_digit(10)
                        {
                            Some(0..=9) | None => {
                                buf.push(self.next());
                            }
                            _ => {
                                break;
                            }
                        }
                        buf.push(self.next());
                    }
                    '+' | '-' => {
                        break;
                    }
                    _ if ch.is_digit(10) => {
                        buf.push(self.next());
                    }
                    _ => break,
                }
            }
        }

        if let Err(e) = self.check_after_numeric_literal() {
            return Err(e);
        };

        let num = match kind {
                NumericKind::BigInt(base) => {
                    NumericLiteral::BigInt(
                        BigInt::from_str_radix(&buf, base).expect("Could not conver to BigInt")
                        )
                }
                NumericKind::Rational /* base: 10 */ => {
                    NumericLiteral::Rational(
                        f64::from_str(&buf)
                            .map_err(|_| LexerError::new("Could not convert value to f64"))?,
                    )
                }
                NumericKind::Integer(base) => {
                    if let Ok(num) = i32::from_str_radix(&buf, base) {
                        NumericLiteral::Integer(
                            num
                        )
                    } else {
                        let b = f64::from(base);
                        let mut result = 0.0_f64;
                        for c in buf.chars() {
                            let digit = f64::from(c.to_digit(base).unwrap());
                            result = result * b + digit;
                        }

                        NumericLiteral::Rational(result)
                    }

                }
            };

        self.push_token(TokenKind::NumericLiteral(num));
        self.column_number += (buf.len() as u64) + position_offset - 1;

        Ok(())
    }

    /// Runs the lexer until completion, returning a [LexerError] if there's a syntax issue, or an empty unit result
    ///
    /// # Example
    ///
    /// ```
    /// # use boa::syntax::lexer::{LexerError, Lexer};
    /// fn main() -> Result<(), LexerError> {
    ///     let buffer = String::from("Hello World");
    ///     let mut lexer = Lexer::new(&buffer);
    ///     lexer.lex()
    /// }
    /// ```
    pub fn lex(&mut self) -> Result<(), LexerError> {
        loop {
            // Check if we've reached the end
            if self.preview_next().is_none() {
                return Ok(());
            }
            self.column_number += 1;
            let ch = self.next();
            match ch {
                '"' | '\'' => {
                    let mut buf = String::new();
                    loop {
                        if self.preview_next().is_none() {
                            return Err(LexerError::new("Unterminated String"));
                        }
                        match self.next() {
                            '\'' if ch == '\'' => {
                                break;
                            }
                            '"' if ch == '"' => {
                                break;
                            }
                            '\\' => {
                                if self.preview_next().is_none() {
                                    return Err(LexerError::new("Unterminated String"));
                                }
                                let escape = self.next();
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
                                                if self.preview_next().is_none() {
                                                    return Err(LexerError::new("Unterminated String"));
                                                }
                                                nums.push(self.next());
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
                                                    .expect("Could not read chars");

                                                // We know this is a single unicode codepoint, convert to u32
                                                let as_num = match u32::from_str_radix(&s, 16) {
                                                    Ok(v) => v,
                                                    Err(_) => 0,
                                                };
                                                let c = from_u32(as_num).ok_or_else(|| LexerError::new("Invalid Unicode escape sequence"))?;

                                                if self.preview_next().is_none() {
                                                    return Err(LexerError::new("Unterminated String"));
                                                }
                                                self.next(); // '}'
                                                self.column_number +=
                                                    (s.len() as u64).wrapping_add(3);
                                                c
                                            } else {
                                                let mut codepoints: Vec<u16> = vec![];
                                                loop {
                                                    // Collect each character after \u e.g \uD83D will give "D83D"
                                                    let s = self
                                                        .take_char_while(char::is_alphanumeric)
                                                        .expect("Could not read chars");

                                                    // Convert to u16
                                                    let as_num = match u16::from_str_radix(&s, 16) {
                                                        Ok(v) => v,
                                                        Err(_) => 0,
                                                    };

                                                    codepoints.push(as_num);
                                                    self.column_number +=
                                                        (s.len() as u64).wrapping_add(2);

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
                                                    .expect("Could not get next codepoint")
                                                    .expect("Could not get next codepoint")
                                            }
                                        }
                                        '\'' | '"' | '\\' => escape,
                                        ch => {
                                            let details = format!("{}:{}: Invalid escape `{}`", self.line_number, self.column_number, ch);
                                            return Err(LexerError { details });
                                        }
                                    };
                                    buf.push(escaped_ch);
                                }
                            }
                            next_ch => buf.push(next_ch),
                        }
                    }
                    let str_length = buf.len() as u64;
                    self.push_token(TokenKind::StringLiteral(buf));
                    // Why +1? Quotation marks are not included,
                    // So technically it would be +2, (for both " ") but we want to be 1 less
                    // to compensate for the incrementing at the top
                    self.column_number += str_length.wrapping_add(1);
                }
                _ if ch.is_digit(10) => self.reed_numerical_literal(ch)?,
                _ if ch.is_alphabetic() || ch == '$' || ch == '_' => {
                    let mut buf = ch.to_string();
                    while let Some(ch) = self.preview_next() {
                        if ch.is_alphabetic() || ch.is_digit(10) || ch == '_' {
                            buf.push(self.next());
                        } else {
                            break;
                        }
                    }

                    self.push_token(match buf.as_str() {
                        "true" => TokenKind::BooleanLiteral(true),
                        "false" => TokenKind::BooleanLiteral(false),
                        "null" => TokenKind::NullLiteral,
                        "NaN" => TokenKind::NumericLiteral(NumericLiteral::Rational(f64::NAN)),
                        slice => {
                            if let Ok(keyword) = FromStr::from_str(slice) {
                                TokenKind::Keyword(keyword)
                            } else {
                                TokenKind::identifier(slice)
                            }
                        }
                    });
                    // Move position forward the length of keyword
                    self.column_number += (buf.len().wrapping_sub(1)) as u64;
                }
                ';' => self.push_punc(Punctuator::Semicolon),
                ':' => self.push_punc(Punctuator::Colon),
                '.' => {
                    // . or ...
                    if self.next_is('.') {
                        if self.next_is('.') {
                            self.push_punc(Punctuator::Spread);
                            self.column_number += 2;
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
                        match ch {
                            // line comment
                            '/' => {
                                while self.preview_next().is_some() {
                                    if self.next() == '\n' {
                                        break;
                                    }
                                }
                                self.line_number += 1;
                                self.column_number = 0;
                            }
                            // block comment
                            '*' => {
                                let mut lines = 0;
                                loop {
                                    if self.preview_next().is_none() {
                                        return Err(LexerError::new("Unterminated Multiline Comment"));
                                    }
                                    match self.next() {
                                        '*' => {
                                            if self.next_is('/') {
                                                break;
                                            }
                                        }
                                        next_ch => {
                                            if next_ch == '\n' {
                                                lines += 1;
                                            }
                                        },
                                    }
                                }
                                self.line_number += lines;
                                self.column_number = 0;
                            }
                            // division, assigndiv or regex literal
                            _ => {
                                // if we fail to parse a regex literal, store a copy of the current
                                // buffer to restore later on
                                let original_buffer = self.buffer.clone();
                                // first, try to parse a regex literal
                                let mut body = String::new();
                                let mut regex = false;
                                loop {
                                    self.column_number +=1;
                                    match self.buffer.next() {
                                        // end of body
                                        Some('/') => {
                                            regex = true;
                                            break;
                                        }
                                        // newline/eof not allowed in regex literal
                                        n @ Some('\n') | n @ Some('\r') | n @ Some('\u{2028}')
                                        | n @ Some('\u{2029}') => {
                                            self.column_number = 0;
                                            if n != Some('\r') {
                                                self.line_number += 1;
                                            }
                                            break
                                        },
                                        None => {
                                            self.column_number -= 1;
                                            break
                                        }
                                        // escape sequence
                                        Some('\\') => {
                                            body.push('\\');
                                            if self.preview_next().is_none() {
                                                break;
                                            }
                                            match self.next() {
                                                // newline not allowed in regex literal
                                                '\n' | '\r' | '\u{2028}' | '\u{2029}' => break,
                                                ch => body.push(ch),
                                            }
                                        }
                                        Some(ch) => body.push(ch),
                                    }
                                }
                                if regex {
                                    // body was parsed, now look for flags
                                    let flags = self.take_char_while(char::is_alphabetic)?;
                                    self.push_token(TokenKind::RegularExpressionLiteral(
                                        body, flags,
                                    ));
                                } else {
                                    // failed to parse regex, restore original buffer position and
                                    // parse either div or assigndiv
                                    self.buffer = original_buffer;
                                    if self.next_is('=') {
                                        self.push_token(TokenKind::Punctuator(
                                            Punctuator::AssignDiv,
                                        ));
                                    } else {
                                        self.push_token(TokenKind::Punctuator(Punctuator::Div));
                                    }
                                }
                            }
                        }
                    } else {
                        return Err(LexerError::new("Expecting Token /,*,= or regex"));
                    }
                }
                '*' => op!(self, Punctuator::AssignMul, Punctuator::Mul, {
                    '*' => vop!(self, Punctuator::AssignPow, Punctuator::Exp)
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
                    self.push_token(TokenKind::LineTerminator);
                    self.line_number += 1;
                    self.column_number = 0;
                }
                '\r' => {
                    self.column_number = 0;
                }
                // The rust char::is_whitespace function and the ecma standard use different sets
                // of characters as whitespaces:
                //  * Rust uses \p{White_Space},
                //  * ecma standard uses \{Space_Separator} + \u{0009}, \u{000B}, \u{000C}, \u{FEFF}
                //
                // Explicit whitespace: see https://tc39.es/ecma262/#table-32
                '\u{0020}' | '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{FEFF}' |
                // Unicode Space_Seperator category (minus \u{0020} and \u{00A0} which are allready stated above)
                '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => (),
                _ => {
                    let details = format!("{}:{}: Unexpected '{}'", self.line_number, self.column_number, ch);
                    return Err(LexerError { details });
                },
            }
        }
    }
}
