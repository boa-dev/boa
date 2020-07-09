use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::{Position, Punctuator, Span};
use crate::syntax::lexer::Token;
use std::io::Read;

/// `vop` tests the next token to see if we're on an assign operation of just a plain binary operation.
///
/// If the next value is not an assignment operation it will pattern match  the provided values and return the corresponding token.
macro_rules! vop {
    ($cursor:ident, $assign_op:expr, $op:expr) => ({
        match $cursor.peek()? {
            None => Err(Error::syntax("Abrupt end: could not preview next value as part of operator")),
            Some('=') => {
                $cursor.next()?.expect("= token vanished");
                $cursor.next_column();
                $assign_op
            }
            Some(_) => $op,
        }
    });
    ($cursor:ident, $assign_op:expr, $op:expr, {$($case:pat => $block:expr), +}) => ({
        // let punc = $cursor.peek().ok_or_else(|| Error::syntax("could not preview next value"))?;
        match $cursor.peek()? {
            None => Err(Error::syntax("Abrupt end: could not preview next value as part of operator")),
            Some('=') => {
                $cursor.next()?.expect("= token vanished");
                $cursor.next_column();
                $assign_op
            },
            $($case => {
                $cursor.next()?.expect("Token vanished");
                $cursor.next_column();
                $block
            })+,
            _ => $op,
        }
    });
    ($cursor:ident, $op:expr, {$($case:pat => $block:expr),+}) => {
        match $cursor.peek().ok_or_else(|| LexerError::syntax("could not preview next value"))? {
            $($case => {
                $cursor.next()?;
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
    init: char,
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
    pub(super) fn new(init: char) -> Self {
        Self { init }
    }
}

impl<R> Tokenizer<R> for Operator {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        match self.init {
            '*' => op!(cursor, start_pos, Ok(Punctuator::AssignMul), Ok(Punctuator::Mul), {
                Some('*') => vop!(cursor, Ok(Punctuator::AssignPow), Ok(Punctuator::Exp))
            }),
            '+' => op!(cursor, start_pos, Ok(Punctuator::AssignAdd), Ok(Punctuator::Add), {
                Some('+') => Ok(Punctuator::Inc)
            }),
            '-' => op!(cursor, start_pos, Ok(Punctuator::AssignSub), Ok(Punctuator::Sub), {
                Some('-') => {
                    Ok(Punctuator::Dec)
                }
            }),
            '%' => op!(
                cursor,
                start_pos,
                Ok(Punctuator::AssignMod),
                Ok(Punctuator::Mod)
            ),
            '|' => op!(cursor, start_pos, Ok(Punctuator::AssignOr), Ok(Punctuator::Or), {
                Some('|') => Ok(Punctuator::BoolOr)
            }),
            '&' => op!(cursor, start_pos, Ok(Punctuator::AssignAnd), Ok(Punctuator::And), {
                Some('&') => Ok(Punctuator::BoolAnd)
            }),
            '^' => op!(
                cursor,
                start_pos,
                Ok(Punctuator::AssignXor),
                Ok(Punctuator::Xor)
            ),
            '=' => op!(cursor, start_pos, if cursor.next_is('=')? {
                Ok(Punctuator::StrictEq)
            } else {
                Ok(Punctuator::Eq)
            }, Ok(Punctuator::Assign), {
                Some('>') => {
                    Ok(Punctuator::Arrow)
                }
            }),
            '<' => op!(cursor, start_pos, Ok(Punctuator::LessThanOrEq), Ok(Punctuator::LessThan), {
                Some('<') => vop!(cursor, Ok(Punctuator::AssignLeftSh), Ok(Punctuator::LeftSh))
            }),
            '>' => {
                op!(cursor, start_pos, Ok(Punctuator::GreaterThanOrEq), Ok(Punctuator::GreaterThan), {
                    Some('>') => vop!(cursor, Ok(Punctuator::AssignRightSh), Ok(Punctuator::RightSh), {
                        Some('>') => vop!(cursor, Ok(Punctuator::AssignURightSh), Ok(Punctuator::URightSh))
                    })
                })
            }
            '!' => op!(
                cursor,
                start_pos,
                vop!(cursor, Ok(Punctuator::StrictNotEq), Ok(Punctuator::NotEq)),
                Ok(Punctuator::Not)
            ),
            '~' => Ok(Token::new(
                Punctuator::Neg.into(),
                Span::new(start_pos, cursor.pos()),
            )),
            _ => unimplemented!(),
        }
    }
}
