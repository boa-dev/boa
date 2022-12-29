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
    Error,
};
use boa_ast::{
    expression::{
        operator::{unary::UnaryOp, Unary},
        Identifier,
    },
    Expression, Position, Punctuator,
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Parses an update expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-UpdateExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct UpdateExpression {
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl UpdateExpression {
    /// Creates a new `UpdateExpression` parser.
    pub(super) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

/// <https://tc39.es/ecma262/multipage/syntax-directed-operations.html#sec-static-semantics-assignmenttargettype>
/// This function checks if the target type is simple
fn is_simple(expr: &Expression, position: Position, strict: bool) -> ParseResult<bool> {
    match expr {
        Expression::Identifier(ident) => {
            if strict {
                check_strict_arguments_or_eval(*ident, position)?;
            }
            Ok(true)
        }
        Expression::PropertyAccess(_) => Ok(true),
        _ => Ok(false),
    }
}

impl<R> TokenParser<R> for UpdateExpression
where
    R: Read,
{
    type Output = Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("UpdateExpression", "Parsing");

        let tok = cursor.peek(0, interner).or_abrupt()?;
        let position = tok.span().start();
        match tok.kind() {
            TokenKind::Punctuator(Punctuator::Inc) => {
                cursor
                    .next(interner)?
                    .expect("Punctuator::Inc token disappeared");

                let target = UnaryExpression::new(self.name, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
                // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                if !is_simple(&target, position, cursor.strict_mode())? {
                    return Err(Error::lex(LexError::Syntax(
                        "Invalid left-hand side in assignment".into(),
                        position,
                    )));
                }

                return Ok(Unary::new(UnaryOp::IncrementPre, target).into());
            }
            TokenKind::Punctuator(Punctuator::Dec) => {
                cursor
                    .next(interner)?
                    .expect("Punctuator::Dec token disappeared");

                let target = UnaryExpression::new(self.name, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
                // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                if !is_simple(&target, position, cursor.strict_mode())? {
                    return Err(Error::lex(LexError::Syntax(
                        "Invalid left-hand side in assignment".into(),
                        position,
                    )));
                }

                return Ok(Unary::new(UnaryOp::DecrementPre, target).into());
            }
            _ => {}
        }

        let lhs = LeftHandSideExpression::new(self.name, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        if let Some(tok) = cursor.peek(0, interner)? {
            let token_start = tok.span().start();
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Inc) => {
                    cursor
                        .next(interner)?
                        .expect("Punctuator::Inc token disappeared");
                    // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                    if !is_simple(&lhs, token_start, cursor.strict_mode())? {
                        return Err(Error::lex(LexError::Syntax(
                            "Invalid left-hand side in assignment".into(),
                            token_start,
                        )));
                    }

                    return Ok(Unary::new(UnaryOp::IncrementPost, lhs).into());
                }
                TokenKind::Punctuator(Punctuator::Dec) => {
                    cursor
                        .next(interner)?
                        .expect("Punctuator::Dec token disappeared");
                    // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                    if !is_simple(&lhs, token_start, cursor.strict_mode())? {
                        return Err(Error::lex(LexError::Syntax(
                            "Invalid left-hand side in assignment".into(),
                            token_start,
                        )));
                    }

                    return Ok(Unary::new(UnaryOp::DecrementPost, lhs).into());
                }
                _ => {}
            }
        }

        Ok(lhs)
    }
}
