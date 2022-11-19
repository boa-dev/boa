#[cfg(test)]
mod tests;

use crate::{
    lexer::{Token, TokenKind},
    parser::{
        cursor::Cursor, expression::left_hand_side::arguments::Arguments, expression::Expression,
        AllowAwait, AllowYield, OrAbrupt, ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{
    self as ast,
    expression::{access::PropertyAccessField, Optional, OptionalOperation, OptionalOperationKind},
    Punctuator,
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

/// Parses an optional expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Optional_chaining
/// [spec]: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#prod-OptionalExpression
#[derive(Debug, Clone)]
pub(in crate::parser) struct OptionalExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    target: ast::Expression,
}

impl OptionalExpression {
    /// Creates a new `OptionalExpression` parser.
    pub(in crate::parser) fn new<Y, A>(
        allow_yield: Y,
        allow_await: A,
        target: ast::Expression,
    ) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            target,
        }
    }
}

impl<R> TokenParser<R> for OptionalExpression
where
    R: Read,
{
    type Output = Optional;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        fn parse_const_access<R: Read>(
            cursor: &mut Cursor<R>,
            token: &Token,
            interner: &mut Interner,
        ) -> ParseResult<OptionalOperationKind> {
            let item = match token.kind() {
                TokenKind::Identifier(name) => OptionalOperationKind::SimplePropertyAccess {
                    field: PropertyAccessField::Const(*name),
                },
                TokenKind::Keyword((kw, _)) => OptionalOperationKind::SimplePropertyAccess {
                    field: PropertyAccessField::Const(kw.to_sym(interner)),
                },
                TokenKind::BooleanLiteral(true) => OptionalOperationKind::SimplePropertyAccess {
                    field: PropertyAccessField::Const(Sym::TRUE),
                },
                TokenKind::BooleanLiteral(false) => OptionalOperationKind::SimplePropertyAccess {
                    field: PropertyAccessField::Const(Sym::FALSE),
                },
                TokenKind::NullLiteral => OptionalOperationKind::SimplePropertyAccess {
                    field: PropertyAccessField::Const(Sym::NULL),
                },
                TokenKind::PrivateIdentifier(name) => {
                    cursor.push_used_private_identifier(*name, token.span().start())?;
                    OptionalOperationKind::PrivatePropertyAccess { field: *name }
                }
                _ => {
                    return Err(Error::expected(
                        ["identifier".to_owned()],
                        token.to_string(interner),
                        token.span(),
                        "optional chain",
                    ))
                }
            };
            Ok(item)
        }
        let _timer = Profiler::global().start_event("OptionalExpression", "Parsing");

        let mut items = Vec::new();

        while let Some(token) = cursor.peek(0, interner)? {
            let shorted = match token.kind() {
                TokenKind::Punctuator(Punctuator::Optional) => {
                    cursor.advance(interner);
                    true
                }
                TokenKind::Punctuator(Punctuator::OpenParen | Punctuator::OpenBracket) => false,
                TokenKind::Punctuator(Punctuator::Dot) => {
                    cursor.advance(interner);
                    let field = cursor.next(interner).or_abrupt()?;

                    let item = parse_const_access(cursor, &field, interner)?;

                    items.push(OptionalOperation::new(item, false));
                    continue;
                }
                TokenKind::TemplateMiddle(_) | TokenKind::TemplateNoSubstitution(_) => {
                    return Err(Error::general(
                        "Invalid tagged template on optional chain",
                        token.span().start(),
                    ))
                }
                _ => break,
            };

            let token = cursor.peek(0, interner).or_abrupt()?;

            let item = match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let args = Arguments::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    OptionalOperationKind::Call { args }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    cursor
                        .next(interner)?
                        .expect("open bracket punctuator token disappeared"); // We move the parser forward.
                    let idx = Expression::new(None, true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    cursor.expect(Punctuator::CloseBracket, "optional chain", interner)?;
                    OptionalOperationKind::SimplePropertyAccess {
                        field: PropertyAccessField::Expr(Box::new(idx)),
                    }
                }
                TokenKind::TemplateMiddle(_) | TokenKind::TemplateNoSubstitution(_) => {
                    return Err(Error::general(
                        "Invalid tagged template on optional chain",
                        token.span().start(),
                    ))
                }
                _ => {
                    let token = cursor.next(interner)?.expect("token disappeared");
                    parse_const_access(cursor, &token, interner)?
                }
            };

            items.push(OptionalOperation::new(item, shorted));
        }

        Ok(Optional::new(self.target, items.into()))
    }
}
