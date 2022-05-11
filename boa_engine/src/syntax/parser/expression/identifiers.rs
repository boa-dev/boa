//! Identifiers parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-identifiers
//!

use crate::syntax::{
    ast::{node::Identifier, Keyword},
    lexer::{Error as LexError, TokenKind},
    parser::{cursor::Cursor, AllowAwait, AllowYield, ParseError, TokenParser},
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

pub(in crate::syntax) const RESERVED_IDENTIFIERS_STRICT: [Sym; 9] = [
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
pub(in crate::syntax::parser) struct IdentifierReference {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl IdentifierReference {
    /// Creates a new `IdentifierReference` parser.
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

impl<R> TokenParser<R> for IdentifierReference
where
    R: Read,
{
    type Output = Identifier;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("IdentifierReference", "Parsing");

        let token = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;

        match token.kind() {
            TokenKind::Identifier(ident)
                if cursor.strict_mode() && RESERVED_IDENTIFIERS_STRICT.contains(ident) =>
            {
                Err(ParseError::general(
                    "using future reserved keyword not allowed in strict mode IdentifierReference",
                    token.span().start(),
                ))
            }
            TokenKind::Identifier(ident) => Ok(Identifier::new(*ident)),
            TokenKind::Keyword((Keyword::Let, _)) if cursor.strict_mode() => {
                Err(ParseError::general(
                    "using future reserved keyword not allowed in strict mode IdentifierReference",
                    token.span().start(),
                ))
            }
            TokenKind::Keyword((Keyword::Let, _)) => Ok(Identifier::new(Sym::LET)),
            TokenKind::Keyword((Keyword::Yield, _)) if self.allow_yield.0 => {
                // Early Error: It is a Syntax Error if this production has a [Yield] parameter and StringValue of Identifier is "yield".
                Err(ParseError::general(
                    "Unexpected identifier",
                    token.span().start(),
                ))
            }
            TokenKind::Keyword((Keyword::Yield, _)) if !self.allow_yield.0 => {
                if cursor.strict_mode() {
                    return Err(ParseError::general(
                        "Unexpected strict mode reserved word",
                        token.span().start(),
                    ));
                }
                Ok(Identifier::new(Sym::YIELD))
            }
            TokenKind::Keyword((Keyword::Await, _)) if self.allow_await.0 => {
                // Early Error: It is a Syntax Error if this production has an [Await] parameter and StringValue of Identifier is "await".
                Err(ParseError::general(
                    "Unexpected identifier",
                    token.span().start(),
                ))
            }
            TokenKind::Keyword((Keyword::Await, _)) if !self.allow_await.0 => {
                Ok(Identifier::new(Sym::AWAIT))
            }
            _ => Err(ParseError::unexpected(
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
pub(in crate::syntax::parser) struct BindingIdentifier {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BindingIdentifier {
    /// Creates a new `BindingIdentifier` parser.
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

impl<R> TokenParser<R> for BindingIdentifier
where
    R: Read,
{
    type Output = Sym;

    /// Strict mode parsing as per <https://tc39.es/ecma262/#sec-identifiers-static-semantics-early-errors>.
    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("BindingIdentifier", "Parsing");

        let next_token = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;

        match next_token.kind() {
            TokenKind::Identifier(Sym::ARGUMENTS) if cursor.strict_mode() => {
                Err(ParseError::lex(LexError::Syntax(
                    "unexpected identifier 'arguments' in strict mode".into(),
                    next_token.span().start(),
                )))
            }
            TokenKind::Identifier(Sym::EVAL) if cursor.strict_mode() => {
                Err(ParseError::lex(LexError::Syntax(
                    "unexpected identifier 'eval' in strict mode".into(),
                    next_token.span().start(),
                )))
            }
            TokenKind::Identifier(ident) => {
                if cursor.strict_mode() && RESERVED_IDENTIFIERS_STRICT.contains(ident) {
                    return Err(ParseError::general(
                        "using future reserved keyword not allowed in strict mode",
                        next_token.span().start(),
                    ));
                }
                Ok(*ident)
            }
            TokenKind::Keyword((Keyword::Let, _)) if cursor.strict_mode() => {
                Err(ParseError::lex(LexError::Syntax(
                    "unexpected identifier 'let' in strict mode".into(),
                    next_token.span().start(),
                )))
            }
            TokenKind::Keyword((Keyword::Let, _)) => Ok(Sym::LET),
            TokenKind::Keyword((Keyword::Yield, _)) if self.allow_yield.0 => {
                // Early Error: It is a Syntax Error if this production has a [Yield] parameter and StringValue of Identifier is "yield".
                Err(ParseError::general(
                    "Unexpected identifier",
                    next_token.span().start(),
                ))
            }
            TokenKind::Keyword((Keyword::Yield, _)) if !self.allow_yield.0 => {
                if cursor.strict_mode() {
                    Err(ParseError::general(
                        "yield keyword in binding identifier not allowed in strict mode",
                        next_token.span().start(),
                    ))
                } else {
                    Ok(Sym::YIELD)
                }
            }
            TokenKind::Keyword((Keyword::Await, _)) if cursor.arrow() => Ok(Sym::AWAIT),
            TokenKind::Keyword((Keyword::Await, _)) if self.allow_await.0 => {
                // Early Error: It is a Syntax Error if this production has an [Await] parameter and StringValue of Identifier is "await".
                Err(ParseError::general(
                    "Unexpected identifier",
                    next_token.span().start(),
                ))
            }
            TokenKind::Keyword((Keyword::Await, _)) if !self.allow_await.0 => Ok(Sym::AWAIT),
            _ => Err(ParseError::expected(
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
pub(in crate::syntax::parser) type LabelIdentifier = BindingIdentifier;
