//! Left hand side expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Left-hand-side_expressions
//! [spec]: https://tc39.es/ecma262/#sec-left-hand-side-expressions

#[cfg(test)]
mod tests;

mod arguments;
mod call;
mod member;
mod optional;
mod template;

use crate::{
    lexer::{InputElement, TokenKind},
    parser::{
        expression::{
            left_hand_side::{
                arguments::Arguments,
                call::{CallExpression, CallExpressionTail},
                member::MemberExpression,
                optional::OptionalExpression,
            },
            AssignmentExpression,
        },
        AllowAwait, AllowYield, Cursor, ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{
    expression::{Identifier, ImportCall, SuperCall},
    Expression, Keyword, Punctuator,
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Parses a left hand side expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Left-hand-side_expressions
/// [spec]: https://tc39.es/ecma262/#prod-LeftHandSideExpression
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct LeftHandSideExpression {
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl LeftHandSideExpression {
    /// Creates a new `LeftHandSideExpression` parser.
    pub(in crate::parser) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for LeftHandSideExpression
where
    R: Read,
{
    type Output = Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        /// Checks if we need to parse a keyword call expression `keyword()`.
        ///
        /// It first checks if the next token is `keyword`, and if it is, it checks if the second next
        /// token is the open parenthesis (`(`) punctuator.
        ///
        /// This is needed because the `if let` chain is very complex, and putting it inline in the
        /// initialization of `lhs` would make it very hard to return an expression over all
        /// possible branches of the `if let`s. Instead, we extract the check into its own function,
        /// then use it inside the condition of a simple `if ... else` expression.
        fn is_keyword_call<R: Read>(
            keyword: Keyword,
            cursor: &mut Cursor<R>,
            interner: &mut Interner,
        ) -> ParseResult<bool> {
            if let Some(next) = cursor.peek(0, interner)? {
                if let TokenKind::Keyword((kw, escaped)) = next.kind() {
                    if kw == &keyword {
                        if *escaped {
                            return Err(Error::general(
                                format!(
                                    "keyword `{}` cannot contain escaped characters",
                                    kw.as_str().0
                                ),
                                next.span().start(),
                            ));
                        }
                        if let Some(next) = cursor.peek(1, interner)? {
                            if next.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
            Ok(false)
        }

        let _timer = Profiler::global().start_event("LeftHandSideExpression", "Parsing");

        cursor.set_goal(InputElement::TemplateTail);

        let mut lhs = if is_keyword_call(Keyword::Super, cursor, interner)? {
            cursor.advance(interner);
            let args =
                Arguments::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;
            SuperCall::new(args).into()
        } else if is_keyword_call(Keyword::Import, cursor, interner)? {
            // `import`
            cursor.advance(interner);
            // `(`
            cursor.advance(interner);

            let arg = AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;

            cursor.expect(
                TokenKind::Punctuator(Punctuator::CloseParen),
                "import call",
                interner,
            )?;

            CallExpressionTail::new(
                self.allow_yield,
                self.allow_await,
                ImportCall::new(arg).into(),
            )
            .parse(cursor, interner)?
        } else {
            let mut member = MemberExpression::new(self.name, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
            if let Some(tok) = cursor.peek(0, interner)? {
                if tok.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) {
                    member = CallExpression::new(self.allow_yield, self.allow_await, member)
                        .parse(cursor, interner)?;
                }
            }
            member
        };

        if let Some(tok) = cursor.peek(0, interner)? {
            if tok.kind() == &TokenKind::Punctuator(Punctuator::Optional) {
                lhs = OptionalExpression::new(self.allow_yield, self.allow_await, lhs)
                    .parse(cursor, interner)?
                    .into();
            }
        }

        Ok(lhs)
    }
}
