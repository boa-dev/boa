//! Identifiers parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-identifiers

use crate::{
    lexer::{Error as LexError, TokenKind},
    parser::{cursor::Cursor, AllowAwait, AllowYield, OrAbrupt, ParseResult, TokenParser},
    Error,
};
use boa_ast::{expression::Identifier, Keyword};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

pub(crate) const RESERVED_IDENTIFIERS_STRICT: [Sym; 9] = [
    Sym::IMPLEMENTS,
    Sym::INTERFACE,
    Sym::LET,
    Sym::PACKAGE,
    Sym::PRIVATE,
    Sym::PROTECTED,
    Sym::PUBLIC,
    Sym::STATIC,
    Sym::YIELD,
];

/// Identifier reference parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-IdentifierReference
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct IdentifierReference {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl IdentifierReference {
    /// Creates a new `IdentifierReference` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for IdentifierReference
where
    R: Read,
{
    type Output = Identifier;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("IdentifierReference", "Parsing");

        let token = cursor.next(interner).or_abrupt()?;

        match token.kind() {
            TokenKind::Identifier((ident, _))
                if cursor.strict_mode() && RESERVED_IDENTIFIERS_STRICT.contains(ident) =>
            {
                Err(Error::general(
                    "using future reserved keyword not allowed in strict mode IdentifierReference",
                    token.span().start(),
                ))
            }
            TokenKind::Identifier((ident, _)) => Ok(Identifier::new(*ident)),
            TokenKind::Keyword((Keyword::Let, _)) if cursor.strict_mode() => Err(Error::general(
                "using future reserved keyword not allowed in strict mode IdentifierReference",
                token.span().start(),
            )),
            TokenKind::Keyword((Keyword::Let, _)) => Ok(Identifier::new(Sym::LET)),
            TokenKind::Keyword((Keyword::Yield, _)) if self.allow_yield.0 => {
                // Early Error: It is a Syntax Error if this production has a [Yield] parameter and StringValue of Identifier is "yield".
                Err(Error::general(
                    "Unexpected identifier",
                    token.span().start(),
                ))
            }
            TokenKind::Keyword((Keyword::Yield, _)) if !self.allow_yield.0 => {
                if cursor.strict_mode() {
                    return Err(Error::general(
                        "Unexpected strict mode reserved word",
                        token.span().start(),
                    ));
                }
                Ok(Identifier::new(Sym::YIELD))
            }
            TokenKind::Keyword((Keyword::Await, _)) if self.allow_await.0 => {
                // Early Error: It is a Syntax Error if this production has an [Await] parameter and StringValue of Identifier is "await".
                Err(Error::general(
                    "Unexpected identifier",
                    token.span().start(),
                ))
            }
            TokenKind::Keyword((Keyword::Await, _)) if !self.allow_await.0 => {
                Ok(Identifier::new(Sym::AWAIT))
            }
            TokenKind::Keyword((Keyword::Async, _)) => Ok(Identifier::new(Sym::ASYNC)),
            TokenKind::Keyword((Keyword::Of, _)) => Ok(Identifier::new(Sym::OF)),
            _ => Err(Error::unexpected(
                token.to_string(interner),
                token.span(),
                "IdentifierReference",
            )),
        }
    }
}

/// Binding identifier parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-BindingIdentifier
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct BindingIdentifier {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BindingIdentifier {
    /// Creates a new `BindingIdentifier` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for BindingIdentifier
where
    R: Read,
{
    type Output = Identifier;

    /// Strict mode parsing as per <https://tc39.es/ecma262/#sec-identifiers-static-semantics-early-errors>.
    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("BindingIdentifier", "Parsing");

        let next_token = cursor.next(interner).or_abrupt()?;

        match next_token.kind() {
            TokenKind::Identifier((Sym::ARGUMENTS, _)) if cursor.strict_mode() => {
                Err(Error::lex(LexError::Syntax(
                    "unexpected identifier 'arguments' in strict mode".into(),
                    next_token.span().start(),
                )))
            }
            TokenKind::Identifier((Sym::EVAL, _)) if cursor.strict_mode() => {
                Err(Error::lex(LexError::Syntax(
                    "unexpected identifier 'eval' in strict mode".into(),
                    next_token.span().start(),
                )))
            }
            TokenKind::Identifier((ident, _)) => {
                if cursor.strict_mode() && RESERVED_IDENTIFIERS_STRICT.contains(ident) {
                    return Err(Error::general(
                        "using future reserved keyword not allowed in strict mode",
                        next_token.span().start(),
                    ));
                }
                Ok((*ident).into())
            }
            TokenKind::Keyword((Keyword::Let, _)) if cursor.strict_mode() => {
                Err(Error::lex(LexError::Syntax(
                    "unexpected identifier 'let' in strict mode".into(),
                    next_token.span().start(),
                )))
            }
            TokenKind::Keyword((Keyword::Let, _)) => Ok(Sym::LET.into()),
            TokenKind::Keyword((Keyword::Yield, _)) if self.allow_yield.0 => {
                // Early Error: It is a Syntax Error if this production has a [Yield] parameter and StringValue of Identifier is "yield".
                Err(Error::general(
                    "Unexpected identifier",
                    next_token.span().start(),
                ))
            }
            TokenKind::Keyword((Keyword::Yield, _)) if !self.allow_yield.0 => {
                if cursor.strict_mode() {
                    Err(Error::general(
                        "yield keyword in binding identifier not allowed in strict mode",
                        next_token.span().start(),
                    ))
                } else {
                    Ok(Sym::YIELD.into())
                }
            }
            TokenKind::Keyword((Keyword::Await, _)) if cursor.arrow() => Ok(Sym::AWAIT.into()),
            TokenKind::Keyword((Keyword::Await, _)) if self.allow_await.0 => {
                // Early Error: It is a Syntax Error if this production has an [Await] parameter and StringValue of Identifier is "await".
                Err(Error::general(
                    "Unexpected identifier",
                    next_token.span().start(),
                ))
            }
            TokenKind::Keyword((Keyword::Await, _)) if !self.allow_await.0 => Ok(Sym::AWAIT.into()),
            TokenKind::Keyword((Keyword::Async, _)) => Ok(Sym::ASYNC.into()),
            TokenKind::Keyword((Keyword::Of, _)) => Ok(Sym::OF.into()),
            _ => Err(Error::expected(
                ["identifier".to_owned()],
                next_token.to_string(interner),
                next_token.span(),
                "binding identifier",
            )),
        }
    }
}

/// Label identifier parsing.
///
/// This seems to be the same as a `BindingIdentifier`.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-LabelIdentifier
pub(in crate::parser) type LabelIdentifier = BindingIdentifier;
