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
    gc::{Finalize, Trace},
    syntax::{
        ast::{
            node::{Assign, FunctionDecl},
            Keyword, Punctuator,
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

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum ClassField {
    /// A method on a class.
    Method(FunctionDecl),
    /// A field on a class (includes an initializer)
    Field(Assign),
    /// A getter function. This will never take any arguments.
    Getter(FunctionDecl),
    /// A setter function. This will always take an argument.
    Setter(FunctionDecl),
}

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
    type Output = (
        Option<FunctionDecl>, // Constructor
        Box<[ClassField]>,    // Methods/fields
        Box<[ClassField]>,    // Static methods/fields
    );

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ClassElementList", "Parsing");
        cursor.set_goal(InputElement::RegExp);

        let mut constructor = None;
        let mut fields = Vec::new();
        let mut static_fields = Vec::new();

        if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
            == &TokenKind::Punctuator(Punctuator::CloseBlock)
        {
            return Ok((
                None,
                fields.into_boxed_slice(),
                static_fields.into_boxed_slice(),
            ));
        }

        loop {
            let next = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
            let static_method = match next.kind() {
                TokenKind::Keyword(Keyword::Static) => {
                    // Consume the static token.
                    cursor.next()?;
                    true
                }
                _ => false,
            };

            // No matter if there was a static token, a `get` or `set` token is valid.
            let next = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
            match next.kind() {
                TokenKind::Keyword(Keyword::Get) => {
                    // Consume the get token.
                    cursor.next()?;
                    // TODO: Do something here to say this is a getter.
                }
                TokenKind::Keyword(Keyword::Set) => {
                    // Consume the set token.
                    cursor.next()?;
                    // TODO: Do something here to say this is a setter.
                }
                _ => (),
            };

            // TODO: Parse async/yeild here

            let position = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();
            let name = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
            if *name == *"constructor" {
                if constructor.is_some() {
                    return Err(ParseError::general(
                        "Cannot have multiple constructors on an object",
                        position,
                    ));
                }
            }

            let next = cursor.next()?.ok_or(ParseError::AbruptEnd)?;
            let field = match next.kind() {
                // A method definition
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let position = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();
                    let params = FormalParameters::new(false, false).parse(cursor)?;

                    // This is only partially correct. A method can enable strict mode with "using strict"; which is not handled here.
                    if let Some(last) = params.last() {
                        if cursor.strict_mode() && last.is_rest_param() {
                            return Err(ParseError::general(
                                "Cannot have spread parameters on a class method in strict mode",
                                position,
                            ));
                        }
                    }

                    cursor.expect(Punctuator::CloseParen, "class function declaration")?;
                    cursor.expect(Punctuator::OpenBlock, "class function declaration")?;

                    let body =
                        FunctionBody::new(self.allow_yield, self.allow_await).parse(cursor)?;

                    cursor.expect(Punctuator::CloseBlock, "class function declaration")?;

                    ClassField::Method(FunctionDecl::new(name, params, body))
                }
                // A field definition
                TokenKind::Punctuator(Punctuator::Assign) => {
                    // TODO: Parse an expr here
                    unimplemented!()
                }
                _ => {
                    return Err(ParseError::expected(
                        vec![
                            TokenKind::Punctuator(Punctuator::OpenParen),
                            TokenKind::Punctuator(Punctuator::Assign),
                        ],
                        next,
                        "class method or field declatation",
                    ))
                }
            }

            if *name == *"constructor" {
                // constructor = Some(FunctionDecl::new(name, params, body));
            } else {
                if static_method {
                    static_fields.push(field);
                } else {
                    fields.push(field);
                }
            }

            if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
                == &TokenKind::Punctuator(Punctuator::CloseBlock)
            {
                break;
            }
        }

        Ok((
            constructor,
            fields.into_boxed_slice(),
            static_fields.into_boxed_slice(),
        ))
    }
}
