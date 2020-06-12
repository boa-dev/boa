use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::Position;
use crate::syntax::lexer::Token;
use std::io::Read;

/// Number literal parsing.
///
///
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-literals-numeric-literals
/// [mdn]:
#[derive(Debug, Clone, Copy)]
pub(super) struct NumberLiteral {
    init: char,
}

impl NumberLiteral {
    /// Creates a new string literal lexer.
    pub(super) fn new(init: char) -> Self {
        Self { init }
    }
}

impl<R> Tokenizer<R> for NumberLiteral {
    fn lex(&mut self, _cursor: &mut Cursor<R>, _start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        unimplemented!("Number literal lexing");
    }
}

/*
impl<R> Tokenizer<R> for NumberLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        /// This is a helper structure
        ///
        /// This structure helps with identifying what numerical type it is and what base is it.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum NumericKind {
            Rational,
            Integer(u8),
            BigInt(u8),
        }

        impl NumericKind {
            /// Get the base of the number kind.
            fn base(self) -> u32 {
                match self {
                    Self::Rational => 10,
                    Self::Integer(base) => base as u32,
                    Self::BigInt(base) => base as u32,
                }
            }

            /// Converts `self` to BigInt kind.
            fn to_bigint(self) -> Self {
                match self {
                    Self::Rational => unreachable!("can not convert rational number to BigInt"),
                    Self::Integer(base) => Self::BigInt(base),
                    Self::BigInt(base) => Self::BigInt(base),
                }
            }
        }

        // TODO: Setup strict mode.
        let strict_mode = false;

        let mut buf = self.init.to_string();

        let mut kind = NumericKind::Integer(10);

        if self.init == '0' {
            match cursor.peek() {
                None => {
                    cursor.next_column();
                    return Ok(Token::new(
                        TokenKind::NumericLiteral(NumericLiteral::Integer(0)),
                        Span::new(start_pos, cursor.pos()),
                    ));
                }
                Some(r) => {
                    match r.map_err(|e| Error::IO(e))? {
                        'x' | 'X' => {
                            cursor.next();
                            cursor.next_column();
                            kind = NumericKind::Integer(16);
                        }
                        'o' | 'O' => {
                            cursor.next();
                            cursor.next_column();
                            kind = NumericKind::Integer(8);
                        }
                        'b' | 'B' => {
                            cursor.next();
                            cursor.next_column();
                            kind = NumericKind::Integer(2);
                        }
                        ch if ch.is_ascii_digit() => {
                            let mut is_implicit_octal = true;
                            while let Some(ch) = cursor.peek(){
                                let c = ch.map_err(|e| Error::IO(e))?;
                                if !c.is_ascii_digit() {
                                    break;
                                } else if !c.is_digit(8) {
                                    is_implicit_octal = false;
                                }
                                cursor.next();
                                buf.push(c);
                            }
                            if !strict_mode {
                                if is_implicit_octal {
                                    kind = NumericKind::Integer(8);
                                }
                            } else {
                                return Err(if is_implicit_octal {
                                    Error::strict(
                                        "Implicit octal literals are not allowed in strict mode.",
                                    )
                                } else {
                                    Error::strict(
                                        "Decimals with leading zeros are not allowed in strict mode.",
                                    )
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        while let Some(ch) = cursor.peek() {
            let c = ch.map_err(|e| Error::IO(e))?;
            if !c.is_digit(kind.base()) {
                break;
            }
            cursor.next();
            buf.push(c);
        }

        if cursor.next_is('n')? {
            kind = kind.to_bigint();
        }

        if let NumericKind::Integer(10) = kind {
            'digitloop: while let Some(cx) = cursor.peek() {
                match cx.map_err(|e| Error::IO(e))? {
                    '.' => loop {
                        kind = NumericKind::Rational;
                        cursor.next();
                        buf.push('.');

                        let c = match cursor.peek() {
                            Some(ch) => ch.map_err(|e| Error::IO(e))?,
                            None => break,
                        };

                        match c {
                            'e' | 'E' => {
                                cursor.next(); // Consume 'e' or 'E'

                                match cursor.peek() {
                                    None => {
                                        cursor.next();
                                    }
                                    Some(x) => {
                                        let val = x.map_err(|e| Error::IO(e))?;
                                        match val.to_digit(10) {
                                            Some(0..=9) => {
                                                cursor.next(); // Consume digit.
                                                buf.push(val);
                                            }
                                            _ => {
                                                break 'digitloop;
                                            }
                                        }
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
                        cursor.next(); // Consume 'e' or 'E'
                        kind = NumericKind::Rational;
                        match cursor.peek() {
                            None => {
                                cursor.next();
                            }
                            Some(x) => {
                                let val = x.map_err(|e| Error::IO(e))?;
                                match val.to_digit(10) {
                                    Some(0..=9) => {
                                        cursor.next(); // Consume digit.
                                        buf.push(val);
                                    }
                                    _ => {
                                        break;
                                    }
                                }
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    '+' | '-' => {
                        break;
                    }
                    x if x.is_digit(10) => {
                        buf.push(x);
                    }
                    _ => break,
                }
            }
        }

        // Check the NumericLiteral is not followed by an `IdentifierStart` or `DecimalDigit` character.
        match cursor.peek() {
            Some(r) => {
                let c = r.map_err(|e| Error::IO(e))?;
                if c.is_ascii_alphabetic() || c == '$' || c == '_' || c.is_ascii_digit() {
                    return Err(Error::syntax("NumericLiteral token must not be followed by IdentifierStart nor DecimalDigit characters"));
                }
            },
            _ => {}
        }

        let num = match kind {
            NumericKind::BigInt(base) => {
                NumericLiteral::BigInt(
                    BigInt::from_string_radix(&buf, base as u32).expect("Could not conver to BigInt")
                    )
            }
            NumericKind::Rational /* base: 10 */ => {
                NumericLiteral::Rational(
                    f64::from_str(&buf)
                        .map_err(|_| Error::syntax("Could not convert value to f64"))?,
                )
            }
            NumericKind::Integer(base) => {
                if let Ok(num) = i32::from_str_radix(&buf, base as u32) {
                    NumericLiteral::Integer(
                        num
                    )
                } else {
                    let b = f64::from(base);
                    let mut result = 0.0_f64;
                    for c in buf.chars() {
                        let digit = f64::from(c.to_digit(base as u32).unwrap());
                        result = result * b + digit;
                    }

                    NumericLiteral::Rational(result)
                }

            }
        };

        Ok(Token::new(
            TokenKind::NumericLiteral(num),
            Span::new(start_pos, cursor.pos()),
        ))
    }
}

*/
