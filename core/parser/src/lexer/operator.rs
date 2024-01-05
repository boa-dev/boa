//! Boa's lexing for ECMAScript operators (+, - etc.).

use crate::lexer::{Cursor, Error, Token, TokenKind, Tokenizer};
use crate::source::ReadChar;
use boa_ast::{Position, Punctuator, Span};
use boa_interner::Interner;
use boa_profiler::Profiler;

/// `vop` tests the next token to see if we're on an assign operation of just a plain binary operation.
///
/// If the next value is not an assignment operation it will pattern match  the provided values and return the corresponding token.
macro_rules! vop {
    ($cursor:ident, $assign_op:expr, $op:expr) => ({
        match $cursor.peek_char()? {
            None => Err(Error::syntax("abrupt end - could not preview next value as part of the operator", $cursor.pos())),
            Some(0x3D /* = */) => {
                $cursor.next_char()?.expect("= token vanished");
                $assign_op
            }
            Some(_) => $op,
        }
    });
    ($cursor:ident, $assign_op:expr, $op:expr, {$($case:pat => $block:expr), +}) => ({
        match $cursor.peek_char()? {
            None => Err(Error::syntax("abrupt end - could not preview next value as part of the operator", $cursor.pos())),
            Some(0x3D /* = */) => {
                $cursor.next_char()?.expect("= token vanished");
                $assign_op
            },
            $($case => {
                $cursor.next_char()?.expect("Token vanished");
                $block
            })+,
            _ => $op,
        }
    });
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
    pub(super) const fn new(init: u8) -> Self {
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
        R: ReadChar,
    {
        let _timer = Profiler::global().start_event("Operator", "Lexing");

        match self.init {
            b'*' => op!(cursor, start_pos, Ok(Punctuator::AssignMul), Ok(Punctuator::Mul), {
                Some(0x2A /* * */) => vop!(cursor, Ok(Punctuator::AssignPow), Ok(Punctuator::Exp))
            }),
            b'+' => op!(cursor, start_pos, Ok(Punctuator::AssignAdd), Ok(Punctuator::Add), {
                Some(0x2B /* + */) => Ok(Punctuator::Inc)
            }),
            b'-' => op!(cursor, start_pos, Ok(Punctuator::AssignSub), Ok(Punctuator::Sub), {
                Some(0x2D /* - */) => {
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
                Some(0x7C /* | */) => vop!(cursor, Ok(Punctuator::AssignBoolOr), Ok(Punctuator::BoolOr))
            }),
            b'&' => op!(cursor, start_pos, Ok(Punctuator::AssignAnd), Ok(Punctuator::And), {
                Some(0x26 /* & */) => vop!(cursor, Ok(Punctuator::AssignBoolAnd), Ok(Punctuator::BoolAnd))
            }),
            b'?' => {
                let (first, second) = (cursor.peek_char()?, cursor.peek_n(2)?[1]);
                match first {
                    Some(0x3F /* ? */) => {
                        cursor.next_char()?.expect("? vanished");
                        op!(
                            cursor,
                            start_pos,
                            Ok(Punctuator::AssignCoalesce),
                            Ok(Punctuator::Coalesce)
                        )
                    }
                    Some(0x2E /* . */) if !matches!(second, Some(second) if (0x30..=0x39 /* 0..=9 */).contains(&second)) =>
                    {
                        cursor.next_char()?.expect(". vanished");
                        Ok(Token::new(
                            TokenKind::Punctuator(Punctuator::Optional),
                            Span::new(start_pos, cursor.pos()),
                        ))
                    }
                    _ => Ok(Token::new(
                        TokenKind::Punctuator(Punctuator::Question),
                        Span::new(start_pos, cursor.pos()),
                    )),
                }
            }
            b'^' => op!(
                cursor,
                start_pos,
                Ok(Punctuator::AssignXor),
                Ok(Punctuator::Xor)
            ),
            b'=' => op!(cursor, start_pos, if cursor.next_if(0x3D /* = */)? {
                Ok(Punctuator::StrictEq)
            } else {
                Ok(Punctuator::Eq)
            }, Ok(Punctuator::Assign), {
                Some(0x3E /* > */) => {
                    Ok(Punctuator::Arrow)
                }
            }),
            b'<' => {
                op!(cursor, start_pos, Ok(Punctuator::LessThanOrEq), Ok(Punctuator::LessThan), {
                    Some(0x3C /* < */) => vop!(cursor, Ok(Punctuator::AssignLeftSh), Ok(Punctuator::LeftSh))
                })
            }
            b'>' => {
                op!(cursor, start_pos, Ok(Punctuator::GreaterThanOrEq), Ok(Punctuator::GreaterThan), {
                    Some(0x3E /* > */) => vop!(cursor, Ok(Punctuator::AssignRightSh), Ok(Punctuator::RightSh), {
                        Some(0x3E /* > */) => vop!(cursor, Ok(Punctuator::AssignURightSh), Ok(Punctuator::URightSh))
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
