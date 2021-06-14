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
            node::{self, FunctionDecl},
            Punctuator,
        },
        lexer::{InputElement, TokenKind},
        parser::{
            function::{FormalParameters, FunctionBody},
            statement::BindingIdentifier,
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
    type Output = Box<[FunctionDecl]>;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ClassElementList", "Parsing");
        cursor.set_goal(InputElement::RegExp);

        let mut elems = Vec::new();

        if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
            == &TokenKind::Punctuator(Punctuator::CloseBlock)
        {
            return Ok(elems.into_boxed_slice());
        }

        loop {
            let name = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;

            cursor.expect(Punctuator::OpenParen, "class function declaration")?;

            let params = FormalParameters::new(false, false).parse(cursor)?;

            cursor.expect(Punctuator::CloseParen, "class function declaration")?;
            cursor.expect(Punctuator::OpenBlock, "class function declaration")?;

            let body = FunctionBody::new(self.allow_yield, self.allow_await).parse(cursor)?;

            cursor.expect(Punctuator::CloseBlock, "class function declaration")?;

            elems.push(FunctionDecl::new(name, params, body));

            if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
                == &TokenKind::Punctuator(Punctuator::CloseBlock)
            {
                break;
            }
        }

        Ok(elems.into_boxed_slice())
    }
}
