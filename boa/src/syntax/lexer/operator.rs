//! This module implements lexing for operators (+, - etc.) used in the JavaScript programing language.

use super::{Cursor, Error, Tokenizer};
use crate::syntax::lexer::TokenKind;
use crate::Interner;
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{Position, Punctuator, Span},
        lexer::Token,
    },
};
use std::io::Read;

/// `vop` tests the next token to see if we're on an assign operation of just a plain binary operation.
///
/// If the next value is not an assignment operation it will pattern match  the provided values and return the corresponding token.
macro_rules! vop {
    ($cursor:ident, $assign_op:expr, $op:expr) => ({
        match $cursor.peek()? {
            None => Err(Error::syntax("abrupt end - could not preview next value as part of the operator", $cursor.pos())),
            Some(b'=') => {
                $cursor.next_byte()?.expect("= token vanished");
                $cursor.next_column();
                $assign_op
            }
            Some(_) => $op,
        }
    });
    ($cursor:ident, $assign_op:expr, $op:expr, {$($case:pat => $block:expr), +}) => ({
        match $cursor.peek()? {
            None => Err(Error::syntax("abrupt end - could not preview next value as part of the operator", $cursor.pos())),
            Some(b'=') => {
                $cursor.next_byte()?.expect("= token vanished");
                $cursor.next_column();
                $assign_op
            },
            $($case => {
                $cursor.next_byte()?.expect("Token vanished");
                $cursor.next_column();
                $block
            })+,
            _ => $op,
        }
    });
    ($cursor:ident, $op:expr, {$($case:pat => $block:expr),+}) => {
        match $cursor.peek().ok_or_else(|| Error::syntax("could not preview next value", $cursor.pos()))? {
            $($case => {
                $cursor.next_byte()?;
                $cursor.next_column();
                $block
            })+,
            _ => $op
        }
    }
}

/// The `op` macro handles binary operations or assignment operations and converts them into tokens.
macro_rules! op {
    ($cursor:ident, $start_pos:expr, $assign_op:expr, $op:expr) => ({
        Ok(Token::new(
            vop!($cursor, $assign_op, $op)?.into(),
            Span::new($start_pos, $cursor.pos()),
        ))
    });
    ($cursor:ident, $start_pos:expr, $assign_op:expr, $op:expr, {$($case:pat => $block:expr),+}) => ({
        let punc: Punctuator = vop!($cursor, $assign_op, $op, {$($case => $block),+})?;
        Ok(Token::new(
            punc.into(),
            Span::new($start_pos, $cursor.pos()),
        ))
    });
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Operator {
    init: u8,
}

/// Operator lexing.
///
/// Assumes that the cursor has already consumed the operator starting symbol (stored in init).
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-expressions
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators
impl Operator {
    /// Creates a new operator lexer.
    pub(super) fn new(init: u8) -> Self {
        Self { init }
    }
}

impl<R> Tokenizer<R> for Operator {
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        _interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("Operator", "Lexing");

        match self.init {
            b'*' => op!(cursor, start_pos, Ok(Punctuator::AssignMul), Ok(Punctuator::Mul), {
                Some(b'*') => vop!(cursor, Ok(Punctuator::AssignPow), Ok(Punctuator::Exp))
            }),
            b'+' => op!(cursor, start_pos, Ok(Punctuator::AssignAdd), Ok(Punctuator::Add), {
                Some(b'+') => Ok(Punctuator::Inc)
            }),
            b'-' => op!(cursor, start_pos, Ok(Punctuator::AssignSub), Ok(Punctuator::Sub), {
                Some(b'-') => {
                    Ok(Punctuator::Dec)
                }
            }),
            b'%' => op!(
                cursor,
                start_pos,
                Ok(Punctuator::AssignMod),
                Ok(Punctuator::Mod)
            ),
            b'|' => op!(cursor, start_pos, Ok(Punctuator::AssignOr), Ok(Punctuator::Or), {
                Some(b'|') => vop!(cursor, Ok(Punctuator::AssignBoolOr), Ok(Punctuator::BoolOr))
            }),
            b'&' => op!(cursor, start_pos, Ok(Punctuator::AssignAnd), Ok(Punctuator::And), {
                Some(b'&') => vop!(cursor, Ok(Punctuator::AssignBoolAnd), Ok(Punctuator::BoolAnd))
            }),
            b'?' => match cursor.peek()? {
                Some(b'?') => {
                    let _ = cursor.next_byte()?.expect("? vanished");
                    op!(
                        cursor,
                        start_pos,
                        Ok(Punctuator::AssignCoalesce),
                        Ok(Punctuator::Coalesce)
                    )
                }
                _ => Ok(Token::new(
                    TokenKind::Punctuator(Punctuator::Question),
                    Span::new(start_pos, cursor.pos()),
                )),
            },
            b'^' => op!(
                cursor,
                start_pos,
                Ok(Punctuator::AssignXor),
                Ok(Punctuator::Xor)
            ),
            b'=' => op!(cursor, start_pos, if cursor.next_is(b'=')? {
                Ok(Punctuator::StrictEq)
            } else {
                Ok(Punctuator::Eq)
            }, Ok(Punctuator::Assign), {
                Some(b'>') => {
                    Ok(Punctuator::Arrow)
                }
            }),
            b'<' => {
                op!(cursor, start_pos, Ok(Punctuator::LessThanOrEq), Ok(Punctuator::LessThan), {
                    Some(b'<') => vop!(cursor, Ok(Punctuator::AssignLeftSh), Ok(Punctuator::LeftSh))
                })
            }
            b'>' => {
                op!(cursor, start_pos, Ok(Punctuator::GreaterThanOrEq), Ok(Punctuator::GreaterThan), {
                    Some(b'>') => vop!(cursor, Ok(Punctuator::AssignRightSh), Ok(Punctuator::RightSh), {
                        Some(b'>') => vop!(cursor, Ok(Punctuator::AssignURightSh), Ok(Punctuator::URightSh))
                    })
                })
            }
            b'!' => op!(
                cursor,
                start_pos,
                vop!(cursor, Ok(Punctuator::StrictNotEq), Ok(Punctuator::NotEq)),
                Ok(Punctuator::Not)
            ),
            b'~' => Ok(Token::new(
                Punctuator::Neg.into(),
                Span::new(start_pos, cursor.pos()),
            )),
            op => unimplemented!("operator {}", op),
        }
    }
}
