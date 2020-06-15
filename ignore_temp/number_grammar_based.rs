use super::{Cursor, Error, TokenKind, Tokenizer};
use crate::builtins::BigInt;
use crate::syntax::ast::{Position, Span};
use crate::syntax::lexer::{token::Numeric, Token};
use std::io::Read;
use std::str::FromStr;

/// Number literal lexing.
///
/// Assumes the initial digit is consumed by the cursor (stored in init).
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

impl<R> Tokenizer<R> for NumberLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let buf = self.init.to_string();

        if let Ok(token) = DecimalLiteral::new(self.init, self.strict_mode).lex(cursor, start_pos) {
            return Ok(token); // Parsed successfully.
        }
        if let Ok(token) = DecimalBigIntegerLiteral::new(self.init).lex(cursor, start_pos) {
            return Ok(token); // Parsed successfully.
        } 
        if let Ok(token) = NonDecimalIntegerLiteral::new(self.init).lex(cursor, start_pos) {
            return Ok(token); // Parsed successfully.
        }

        Err(Error::Reverted())


        // Ok(Token::new(
        //     TokenKind::NumericLiteral(num),
        //     Span::new(start_pos, cursor.pos()),
        // ))
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DecimalLiteral {
    init: char,
    strict_mode: bool,
}

impl DecimalLiteral {
    /// Creates a new string literal lexer.
    pub(super) fn new(init: char, strict_mode: bool) -> Self {
        Self { init, strict_mode}
    }
} 

impl<R> Tokenizer<R> for DecimalLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {


        let dil = DecimalIntegerLiteral::new(self.init, self.strict_mode).lex(cursor, start_pos);
        match dil {
            Ok(dil_token) => {
                // DecimalIntegerLiteral

                if cursor.next_is('.')? {
                    // DecimalIntegerLiteral.
                    
                    // Consume the '.'
                    cursor.next();


                    // May be followed by DecimalDigits
                    let dd = DecimalDigits::new(self.strict_mode).lex(cursor, start_pos);
                    match dd {
                        Ok(dd_token) => {
                            // DecimalIntegerLiteral.DecimalDigits
                            let ep = ExponentPart::new(self.strict_mode).lex(cursor, start_pos);
                            match ep {
                                Ok(ep_token) => {
                                    // DecimalIntegerLiteral.DecimalDigits ExponentPart
                                    // Terminal pattern.
                                    dil + dd + ep
                                }
                                Err(Error::Reverted()) => {
                                    // DecimalIntegerLiteral.DecimalDigits
                                    // Terminal pattern.
                                    dil + dd                         
                                }
                                Err (e) => {
                                    // Some other error preventing lexing.
                                    Err(e)
                                }
                            }
                        }
                        Err(Error::Reverted()) => {
                            // DecimalIntegerLiteral.
                            // Terminal pattern.
                            dd                          
                        }
                        Err(e) => {
                            // Some other error preventing lexing.
                            Err(e)
                        }
                    }
                } else {
                    // DecimalIntegerLiteral

                    // May be followed by ExponentPart
                    let ep = ExponentPart::new(self.strict_mode).lex(cursor, start_pos);
                    match ep {
                        Ok(ep_token) => {
                            // DecimalIntegerLiteral ExponentPart
                            // Terminal pattern.
                            dil + ep
                        }
                        Err(Error::Reverted()) => {
                            // DecimalIntegerLiteral
                            dil
                        }
                        Err (e) => {
                            // Some other error preventing lexing.
                            Err(e)
                        }
                    }
                }
            }
            Err(Error::Reverted(buf)) => {
                // If a decimal literal doesn't start with a DecimalIntegerLiteral it must start with a '.' followed by DecimalDigits.
                if cursor.next_is('.')? {
                    // . 
                    let dd = DecimalDigits::new(self.strict_mode).lex(cursor, start_pos);
                    match dd {
                        Ok(dd_token) => {
                            // . DecimalDigits 
                            
                            // May be followed by ExponentPart
                            let ep = ExponentPart::new(self.strict_mode).lex(cursor, start_pos);
                            match ep {
                                Ok(ep_token) => {
                                    // . DecimalDigits ExponentPart
                                    dd + ep
                                }
                                Err(Error::Reverted()) => {
                                    // . DecimalDigits
                                    dd
                                }
                                Err (e) => {
                                    // Some other error preventing lexing.
                                    Err(e)
                                }
                            }
                        }
                        Err(e) => {
                            // A DecimalDigits couldn't be lexed or some other error prevents lexing.
                            Err(e)
                        }
                    }
                } else {
                    Err(Error::Reverted())
                }
            }
            Err(e) => {
                // Some other error.
                Err(e)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DecimalBigIntegerLiteral {
    init: char,
    strict_mode: bool,
}

impl DecimalBigIntegerLiteral{
    /// Creates a new string literal lexer.
    pub(super) fn new(init: char, strict_mode: bool) -> Self {
        Self { init, strict_mode }
    }
} 

impl<R> Tokenizer<R> for DecimalBigIntegerLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        Err(Error::Reverted())
    }
}


#[derive(Debug, Clone, Copy)]
pub(super) struct NonDecimalIntegerLiteral {
    init: char,
    strict_mode: bool,
}

impl NonDecimalIntegerLiteral {
    /// Creates a new string literal lexer.
    pub(super) fn new(init: char, strict_mode: bool) -> Self {
        Self { init, strict_mode }
    }
} 

impl<R> Tokenizer<R> for NonDecimalIntegerLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        Err(Error::Reverted())
    }
}