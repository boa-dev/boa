use super::{Cursor, Error, TokenKind, Tokenizer};
use crate::{
    builtins::BigInt,
    profiler::BoaProfiler,
    syntax::{
        ast::{Position, Span},
        lexer::{token::Numeric, Token},
    },
};
use std::{io::Read, str::FromStr};

/// Number literal lexing.
///
/// Assumes the digit is consumed by the cursor (stored in init).
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-literals-numeric-literals
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures#Number_type
#[derive(Debug, Clone, Copy)]
pub(super) struct NumberLiteral {
    init: char,
    strict_mode: bool,
}

impl NumberLiteral {
    /// Creates a new string literal lexer.
    pub(super) fn new(init: char, strict_mode: bool) -> Self {
        Self { init, strict_mode }
    }
}

/// This is a helper structure
///
/// This structure helps with identifying what numerical type it is and what base is it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericKind {
    Rational,
    Integer(u32),
    BigInt(u32),
}

impl NumericKind {
    /// Get the base of the number kind.
    fn base(self) -> u32 {
        match self {
            Self::Rational => 10,
            Self::Integer(base) => base,
            Self::BigInt(base) => base,
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

fn take_signed_integer<R>(
    buf: &mut String,
    cursor: &mut Cursor<R>,
    kind: &NumericKind,
) -> Result<(), Error>
where
    R: Read,
{
    // The next part must be SignedInteger.
    // This is optionally a '+' or '-' followed by 1 or more DecimalDigits.
    match cursor.next_char()? {
        Some('+') => {
            buf.push('+');
            if !cursor.next_is_pred(&|c: char| c.is_digit(kind.base()))? {
                // A digit must follow the + or - symbol.
                return Err(Error::syntax("No digit found after + symbol"));
            }
        }
        Some('-') => {
            buf.push('-');
            if !cursor.next_is_pred(&|c: char| c.is_digit(kind.base()))? {
                // A digit must follow the + or - symbol.
                return Err(Error::syntax("No digit found after - symbol"));
            }
        }
        Some(c) if c.is_digit(kind.base()) => buf.push(c),
        Some(c) => {
            return Err(Error::syntax(format!(
                "When lexing exponential value found unexpected char: '{}'",
                c
            )));
        }
        None => {
            return Err(Error::syntax("Abrupt end: No exponential value found"));
        }
    }

    // Consume the decimal digits.
    cursor.take_until_pred(buf, &|c: char| c.is_digit(kind.base()))?;

    Ok(())
}

/// Utility function for checking the NumericLiteral is not followed by an `IdentifierStart` or `DecimalDigit` character.
///
/// More information:
///  - [ECMAScript Specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-literals-numeric-literals
fn check_after_numeric_literal<R>(cursor: &mut Cursor<R>) -> Result<(), Error>
where
    R: Read,
{
    let pred = |ch: char| ch.is_ascii_alphabetic() || ch == '$' || ch == '_' || ch.is_ascii_digit();
    if cursor.next_is_pred(&pred)? {
        Err(Error::syntax("NumericLiteral token must not be followed by IdentifierStart nor DecimalDigit characters"))
    } else {
        Ok(())
    }
}

impl<R> Tokenizer<R> for NumberLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("NumberLiteral", "Lexing");

        let mut buf = self.init.to_string();

        // Default assume the number is a base 10 integer.
        let mut kind = NumericKind::Integer(10);

        let c = cursor.peek();

        if self.init == '0' {
            if let Some(ch) = c? {
                match ch {
                    'x' | 'X' => {
                        // Remove the initial '0' from buffer.
                        cursor.next_char()?.expect("x or X character vanished");
                        buf.pop();

                        // HexIntegerLiteral
                        kind = NumericKind::Integer(16);
                    }
                    'o' | 'O' => {
                        // Remove the initial '0' from buffer.
                        cursor.next_char()?.expect("o or O character vanished");
                        buf.pop();

                        // OctalIntegerLiteral
                        kind = NumericKind::Integer(8);
                    }
                    'b' | 'B' => {
                        // Remove the initial '0' from buffer.
                        cursor.next_char()?.expect("b or B character vanished");
                        buf.pop();

                        // BinaryIntegerLiteral
                        kind = NumericKind::Integer(2);
                    }
                    'n' => {
                        cursor.next_char()?.expect("n character vanished");

                        // DecimalBigIntegerLiteral '0n'
                        return Ok(Token::new(
                            TokenKind::NumericLiteral(Numeric::BigInt(0.into())),
                            Span::new(start_pos, cursor.pos()),
                        ));
                    }
                    ch => {
                        if ch.is_digit(8) {
                            // LegacyOctalIntegerLiteral
                            if self.strict_mode {
                                // LegacyOctalIntegerLiteral is forbidden with strict mode true.
                                return Err(Error::syntax(
                                    "Implicit octal literals are not allowed in strict mode.",
                                ));
                            } else {
                                // Remove the initial '0' from buffer.
                                buf.pop();

                                buf.push(cursor.next_char()?.expect("'0' character vanished"));

                                kind = NumericKind::Integer(8);
                            }
                        } else if ch.is_digit(10) {
                            // Indicates a numerical digit comes after then 0 but it isn't an octal digit
                            // so therefore this must be a number with an unneeded leading 0. This is
                            // forbidden in strict mode.
                            if self.strict_mode {
                                return Err(Error::syntax(
                                    "Leading 0's are not allowed in strict mode.",
                                ));
                            } else {
                                buf.push(cursor.next_char()?.expect("Number digit vanished"));
                            }
                        } // Else indicates that the symbol is a non-number.
                    }
                }
            } else {
                // DecimalLiteral lexing.
                // Indicates that the number is just a single 0.
                return Ok(Token::new(
                    TokenKind::NumericLiteral(Numeric::Integer(0)),
                    Span::new(start_pos, cursor.pos()),
                ));
            }
        }

        // Consume digits until a non-digit character is encountered or all the characters are consumed.
        cursor.take_until_pred(&mut buf, &|c: char| c.is_digit(kind.base()))?;

        // The non-digit character could be:
        // 'n' To indicate a BigIntLiteralSuffix.
        // '.' To indicate a decimal seperator.
        // 'e' | 'E' To indicate an ExponentPart.
        match cursor.peek()? {
            Some('n') => {
                // DecimalBigIntegerLiteral
                // Lexing finished.

                // Consume the n
                cursor.next_char()?.expect("n character vanished");

                kind = kind.to_bigint();
            }
            Some('.') => {
                if kind.base() == 10 {
                    // Only base 10 numbers can have a decimal seperator.
                    // Number literal lexing finished if a . is found for a number in a different base.

                    cursor.next_char()?.expect(". token vanished");
                    buf.push('.'); // Consume the .
                    kind = NumericKind::Rational;

                    // Consume digits until a non-digit character is encountered or all the characters are consumed.
                    cursor.take_until_pred(&mut buf, &|c: char| c.is_digit(kind.base()))?;

                    // The non-digit character at this point must be an 'e' or 'E' to indicate an Exponent Part.
                    // Another '.' or 'n' is not allowed.
                    match cursor.peek()? {
                        Some('e') | Some('E') => {
                            // Consume the ExponentIndicator.
                            cursor.next_char()?.expect("e or E token vanished");

                            buf.push('E');

                            take_signed_integer(&mut buf, cursor, &kind)?;
                        }
                        Some(_) | None => {
                            // Finished lexing.
                        }
                    }
                }
            }
            Some('e') | Some('E') => {
                kind = NumericKind::Rational;
                cursor.next_char()?.expect("e or E character vanished"); // Consume the ExponentIndicator.
                buf.push('E');
                take_signed_integer(&mut buf, cursor, &kind)?;
            }
            Some(_) | None => {
                // Indicates lexing finished.
            }
        }

        check_after_numeric_literal(cursor)?;

        let num = match kind {
            NumericKind::BigInt(base) => {
                Numeric::BigInt(
                    BigInt::from_string_radix(&buf, base).expect("Could not convert to BigInt")
                    )
            }
            NumericKind::Rational /* base: 10 */ => {
                let val = f64::from_str(&buf).expect("Failed to parse float after checks");
                let int_val = val as i32;

                // The truncated float should be identically to the non-truncated float for the conversion to be loss-less, 
                // any other different and the number must be stored as a rational.
                #[allow(clippy::float_cmp)]
                if (int_val as f64) == val {
                    // For performance reasons we attempt to store values as integers if possible.
                    Numeric::Integer(int_val)
                } else {
                    Numeric::Rational(val)
                }
            },
            NumericKind::Integer(base) => {
                if let Ok(num) = i32::from_str_radix(&buf, base) {
                    Numeric::Integer(num)
                } else {
                    let b = f64::from(base);
                    let mut result = 0.0_f64;
                    for c in buf.chars() {
                        let digit = f64::from(c.to_digit(base).expect("Couldn't parse digit after already checking validity"));
                        result = result * b + digit;
                    }
                    Numeric::Rational(result)
                }
            }
        };

        Ok(Token::new(
            TokenKind::NumericLiteral(num),
            Span::new(start_pos, cursor.pos()),
        ))
    }
}
