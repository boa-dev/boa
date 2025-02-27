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
                return self.parse_initial_inc_dec_token(
                    cursor,
                    interner,
                    position,
                    UpdateOp::IncrementPre,
                )
            }
            TokenKind::Punctuator(Punctuator::Dec) => {
                return self.parse_initial_inc_dec_token(
                    cursor,
                    interner,
                    position,
                    UpdateOp::DecrementPre,
                )
            }
            _ => {}
        }

        let lhs = LeftHandSideExpression::new(self.allow_yield, self.allow_await)
            .parse_boxed(cursor, interner)?;

        Self::parse_tail(cursor, interner, position, lhs)
    }
}

#[allow(clippy::expect_fun_call)]
impl UpdateExpression {
    fn update_expr_ctor(
        expr: &Expression,
        pos: Position,
        err_pos: Position,
        op: UpdateOp,
        strict: bool,
    ) -> ParseResult<Box<Expression>> {
        as_simple(expr, pos, strict)?.map_or_else(
            || {
                Err(Error::lex(LexError::Syntax(
                    "Invalid left-hand side in assignment".into(),
                    err_pos,
                )))
            },
            |target| Ok(Box::new(Update::new(op, target).into())),
        )
    }

    /// This function was added to optimize the stack size.
    /// It has an stack size optimization impact only for `profile.#.opt-level = 0`.
    /// It allow to reduce stack size allocation in `parse_boxed`,
    /// and an often called function in recursion stays outside of this function.
    fn parse_initial_inc_dec_token<R: ReadChar>(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
        position: Position,
        op: UpdateOp,
    ) -> ParseResult<Box<Expression>> {
        cursor.next(interner)?.expect(disappeared_err(op.is_inc()));

        let target = UnaryExpression::new(self.allow_yield, self.allow_await)
            .parse_boxed(cursor, interner)?;

        // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
        Self::update_expr_ctor(&target, position, position, op, cursor.strict())
    }

    /// This function was added to optimize the stack size.
    /// It has an stack size optimization impact only for `profile.#.opt-level = 0`.
    /// It allow to reduce stack size allocation in `parse_boxed`,
    /// and an often called function in recursion stays outside of this function.
    fn parse_tail<R: ReadChar>(
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
        position: Position,
        lhs: Box<Expression>,
    ) -> ParseResult<Box<Expression>> {
        if cursor.peek_is_line_terminator(0, interner)?.unwrap_or(true) {
            return Ok(lhs);
        }

        if let Some(tok) = cursor.peek(0, interner)? {
            let token_start = tok.span().start();
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Inc) => {
                    cursor.next(interner)?.expect(disappeared_err(true));

                    // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                    return Self::update_expr_ctor(
                        &lhs,
                        position,
                        token_start,
                        UpdateOp::IncrementPost,
                        cursor.strict(),
                    );
                }
                TokenKind::Punctuator(Punctuator::Dec) => {
                    cursor.next(interner)?.expect(disappeared_err(false));

                    // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                    return Self::update_expr_ctor(
                        &lhs,
                        position,
                        token_start,
                        UpdateOp::DecrementPost,
                        cursor.strict(),
                    );
                }
                _ => {}
            }
        }

        Ok(lhs)
    }
}

const fn disappeared_err(is_inc: bool) -> &'static str {
    if is_inc {
        "Punctuator::Inc token disappeared"
    } else {
        "Punctuator::Dec token disappeared"
    }
}
