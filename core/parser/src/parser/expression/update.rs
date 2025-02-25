//! Update expression parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-update-expressions

use crate::{
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::{
            check_strict_arguments_or_eval, left_hand_side::LeftHandSideExpression,
            unary::UnaryExpression,
        },
        AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    source::ReadChar,
    Error,
};
use boa_ast::{
    expression::operator::{
        update::{UpdateOp, UpdateTarget},
        Update,
    },
    Expression, Position, Punctuator,
};
use boa_interner::Interner;
use boa_profiler::Profiler;

/// Parses an update expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-UpdateExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct UpdateExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl UpdateExpression {
    /// Creates a new `UpdateExpression` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

/// Check if the assignment target type is simple and return the target as an `UpdateTarget`.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-assignmenttargettype
fn as_simple(
    expr: &Expression,
    position: Position,
    strict: bool,
) -> ParseResult<Option<UpdateTarget>> {
    match expr {
        Expression::Identifier(ident) => {
            if strict {
                check_strict_arguments_or_eval(*ident, position)?;
            }
            Ok(Some(UpdateTarget::Identifier(*ident)))
        }
        Expression::PropertyAccess(access) => {
            Ok(Some(UpdateTarget::PropertyAccess(access.clone())))
        }
        Expression::Parenthesized(p) => as_simple(p.expression(), position, strict),
        _ => Ok(None),
    }
}

impl<R> TokenParser<R> for UpdateExpression
where
    R: ReadChar,
{
    type Output = Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        self.parse_boxed(cursor, interner).map(|ok| *ok)
    }

    fn parse_boxed(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<Box<Self::Output>> {
        let _timer = Profiler::global().start_event("UpdateExpression", "Parsing");

        let tok = cursor.peek(0, interner).or_abrupt()?;
        let position = tok.span().start();
        match tok.kind() {
            TokenKind::Punctuator(Punctuator::Inc) => {
                cursor
                    .next(interner)?
                    .expect("Punctuator::Inc token disappeared");

                let target = UnaryExpression::new(self.allow_yield, self.allow_await)
                    .parse_boxed(cursor, interner)?;

                // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                return (as_simple(&target, position, cursor.strict())?).map_or_else(
                    || {
                        Err(Error::lex(LexError::Syntax(
                            "Invalid left-hand side in assignment".into(),
                            position,
                        )))
                    },
                    |target| Ok(Box::new(Update::new(UpdateOp::IncrementPre, target).into())),
                );
            }
            TokenKind::Punctuator(Punctuator::Dec) => {
                cursor
                    .next(interner)?
                    .expect("Punctuator::Dec token disappeared");

                let target = UnaryExpression::new(self.allow_yield, self.allow_await)
                    .parse_boxed(cursor, interner)?;

                // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                return (as_simple(&target, position, cursor.strict())?).map_or_else(
                    || {
                        Err(Error::lex(LexError::Syntax(
                            "Invalid left-hand side in assignment".into(),
                            position,
                        )))
                    },
                    |target| Ok(Box::new(Update::new(UpdateOp::DecrementPre, target).into())),
                );
            }
            _ => {}
        }

        let lhs = LeftHandSideExpression::new(self.allow_yield, self.allow_await)
            .parse_boxed(cursor, interner)?;

        if cursor.peek_is_line_terminator(0, interner)?.unwrap_or(true) {
            return Ok(lhs);
        }

        if let Some(tok) = cursor.peek(0, interner)? {
            let token_start = tok.span().start();
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Inc) => {
                    cursor
                        .next(interner)?
                        .expect("Punctuator::Inc token disappeared");

                    // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                    return (as_simple(&lhs, position, cursor.strict())?).map_or_else(
                        || {
                            Err(Error::lex(LexError::Syntax(
                                "Invalid left-hand side in assignment".into(),
                                token_start,
                            )))
                        },
                        |target| {
                            Ok(Box::new(
                                Update::new(UpdateOp::IncrementPost, target).into(),
                            ))
                        },
                    );
                }
                TokenKind::Punctuator(Punctuator::Dec) => {
                    cursor
                        .next(interner)?
                        .expect("Punctuator::Dec token disappeared");

                    // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                    return (as_simple(&lhs, position, cursor.strict())?).map_or_else(
                        || {
                            Err(Error::lex(LexError::Syntax(
                                "Invalid left-hand side in assignment".into(),
                                token_start,
                            )))
                        },
                        |target| {
                            Ok(Box::new(
                                Update::new(UpdateOp::DecrementPost, target).into(),
                            ))
                        },
                    );
                }
                _ => {}
            }
        }

        Ok(lhs)
    }
}
