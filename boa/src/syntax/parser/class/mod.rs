//! Function definition parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
//! [spec]: https://tc39.es/ecma262/#sec-function-definitions

#[cfg(test)]
mod tests;

use crate::{
    syntax::{
        ast::{
            node::{self},
            Punctuator,
        },
        lexer::{InputElement, TokenKind},
        parser::{
            expression::Initializer,
            statement::{BindingIdentifier, StatementList},
            AllowAwait, AllowYield, Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};
use std::io::Read;

/// Formal class element list parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
/// [spec]: https://tc39.es/ecma262/#prod-ClassElementList
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct ClassElementList {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassElementList {
    /// Creates a new `FormalElements` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for ClassElementList
where
    R: Read,
{
    type Output = Box<[node::FunctionDecl]>;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FormalParameters", "Parsing");
        cursor.set_goal(InputElement::RegExp);

        let mut params = Vec::new();

        if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
            == &TokenKind::Punctuator(Punctuator::CloseParen)
        {
            return Ok(params.into_boxed_slice());
        }

        loop {
            let mut rest_param = false;

            let next_param = match cursor.peek(0)? {
                Some(tok) if tok.kind() == &TokenKind::Punctuator(Punctuator::Spread) => {
                    rest_param = true;
                    FunctionRestParameter::new(self.allow_yield, self.allow_await).parse(cursor)?
                }
                _ => FormalParameter::new(self.allow_yield, self.allow_await).parse(cursor)?,
            };

            params.push(next_param);

            if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
                == &TokenKind::Punctuator(Punctuator::CloseParen)
            {
                break;
            }

            if rest_param {
                return Err(ParseError::unexpected(
                    cursor.next()?.expect("peeked token disappeared"),
                    "rest parameter must be the last formal parameter",
                ));
            }

            cursor.expect(Punctuator::Comma, "parameter list")?;
        }

        Ok(params.into_boxed_slice())
    }
}
