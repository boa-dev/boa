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

use crate::syntax::{
    ast::{
        node::{self},
        Punctuator, TokenKind,
    },
    parser::{
        expression::Initializer,
        statement::{BindingIdentifier, StatementList},
        AllowAwait, AllowYield, Cursor, ParseError, TokenParser,
    },
};

/// Formal parameters parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Parameter
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameters
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct FormalParameters {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FormalParameters {
    /// Creates a new `FormalParameters` parser.
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

impl TokenParser for FormalParameters {
    type Output = Box<[node::FormalParameter]>;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let mut params = Vec::new();

        if cursor.peek(0).ok_or(ParseError::AbruptEnd)?.kind
            == TokenKind::Punctuator(Punctuator::CloseParen)
        {
            return Ok(params.into_boxed_slice());
        }

        loop {
            let mut rest_param = false;

            params.push(if cursor.next_if(Punctuator::Spread).is_some() {
                rest_param = true;
                FunctionRestParameter::new(self.allow_yield, self.allow_await).parse(cursor)?
            } else {
                FormalParameter::new(self.allow_yield, self.allow_await).parse(cursor)?
            });

            if cursor.peek(0).ok_or(ParseError::AbruptEnd)?.kind
                == TokenKind::Punctuator(Punctuator::CloseParen)
            {
                break;
            }

            if rest_param {
                return Err(ParseError::unexpected(
                    cursor
                        .peek_prev()
                        .expect("current token disappeared")
                        .clone(),
                    "rest parameter must be the last formal parameter",
                ));
            }

            cursor.expect(Punctuator::Comma, "parameter list")?;
        }

        Ok(params.into_boxed_slice())
    }
}

/// Rest parameter parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/rest_parameters
/// [spec]: https://tc39.es/ecma262/#prod-FunctionRestParameter
type FunctionRestParameter = BindingRestElement;

/// Rest parameter parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/rest_parameters
/// [spec]: https://tc39.es/ecma262/#prod-BindingRestElement
#[derive(Debug, Clone, Copy)]
struct BindingRestElement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BindingRestElement {
    /// Creates a new `BindingRestElement` parser.
    fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl TokenParser for BindingRestElement {
    type Output = node::FormalParameter;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        // FIXME: we are reading the spread operator before the rest element.
        // cursor.expect(Punctuator::Spread, "rest parameter")?;

        let param = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
        // TODO: BindingPattern

        Ok(Self::Output::new(param, None, true))
    }
}

/// Formal parameter parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Parameter
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameter
#[derive(Debug, Clone, Copy)]
struct FormalParameter {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FormalParameter {
    /// Creates a new `FormalParameter` parser.
    fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl TokenParser for FormalParameter {
    type Output = node::FormalParameter;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        // TODO: BindingPattern

        let param = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;

        let init = Initializer::new(true, self.allow_yield, self.allow_await).try_parse(cursor);

        Ok(Self::Output::new(param, init, false))
    }
}

/// A `FunctionBody` is equivalent to a `FunctionStatementList`.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionBody
pub(in crate::syntax::parser) type FunctionBody = FunctionStatementList;

/// A function statement list
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionStatementList
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct FunctionStatementList {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FunctionStatementList {
    /// Creates a new `FunctionStatementList` parser.
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

impl TokenParser for FunctionStatementList {
    type Output = node::StatementList;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        if let Some(tk) = cursor.peek(0) {
            if tk.kind == Punctuator::CloseBlock.into() {
                return Ok(Vec::new().into());
            }
        }

        StatementList::new(self.allow_yield, self.allow_await, true, true).parse(cursor)
    }
}
