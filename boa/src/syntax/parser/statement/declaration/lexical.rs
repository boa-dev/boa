//! Lexical declaration parsing.
//!
//! This parses `let` and `const` declarations.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{
            node::{
                declaration::{Declaration, DeclarationList},
                Node,
            },
            Keyword, Punctuator,
        },
        parser::{
            cursor::{Cursor, SemicolonResult},
            expression::Initializer,
            statement::BindingIdentifier,
            AllowAwait, AllowIn, AllowYield, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// Parses a lexical declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-LexicalDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct LexicalDeclaration {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    const_init_required: bool,
}

impl LexicalDeclaration {
    /// Creates a new `LexicalDeclaration` parser.
    pub(super) fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
        const_init_required: bool,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            const_init_required,
        }
    }
}

impl<R> TokenParser<R> for LexicalDeclaration
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("LexicalDeclaration", "Parsing");
        let tok = cursor.next()?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::Const) => BindingList::new(
                self.allow_in,
                self.allow_yield,
                self.allow_await,
                true,
                self.const_init_required,
            )
            .parse(cursor),
            TokenKind::Keyword(Keyword::Let) => BindingList::new(
                self.allow_in,
                self.allow_yield,
                self.allow_await,
                false,
                self.const_init_required,
            )
            .parse(cursor),
            _ => unreachable!("unknown token found: {:?}", tok),
        }
    }
}

/// Parses a binding list.
///
/// It will return an error if a `const` declaration is being parsed and there is no
/// initializer.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-BindingList
#[derive(Debug, Clone, Copy)]
struct BindingList {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_const: bool,
    const_init_required: bool,
}

impl BindingList {
    /// Creates a new `BindingList` parser.
    fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
        is_const: bool,
        const_init_required: bool,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            is_const,
            const_init_required,
        }
    }
}

impl<R> TokenParser<R> for BindingList
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("BindingList", "Parsing");

        // Create vectors to store the variable declarations
        // Const and Let signatures are slightly different, Const needs definitions, Lets don't
        let mut let_decls = Vec::new();
        let mut const_decls = Vec::new();

        loop {
            let (ident, init) =
                LexicalBinding::new(self.allow_in, self.allow_yield, self.allow_await)
                    .parse(cursor)?;

            if self.is_const {
                if self.const_init_required {
                    if init.is_some() {
                        const_decls.push(Declaration::new(ident, init));
                    } else {
                        return Err(ParseError::expected(
                            vec![TokenKind::Punctuator(Punctuator::Assign)],
                            cursor.next()?.ok_or(ParseError::AbruptEnd)?,
                            "const declaration",
                        ));
                    }
                } else {
                    const_decls.push(Declaration::new(ident, init))
                }
            } else {
                let_decls.push(Declaration::new(ident, init));
            }

            match cursor.peek_semicolon()? {
                SemicolonResult::Found(_) => break,
                SemicolonResult::NotFound(tk)
                    if tk.kind() == &TokenKind::Keyword(Keyword::Of)
                        || tk.kind() == &TokenKind::Keyword(Keyword::In) =>
                {
                    break
                }
                SemicolonResult::NotFound(tk)
                    if tk.kind() == &TokenKind::Punctuator(Punctuator::Comma) =>
                {
                    // We discard the comma
                    let _ = cursor.next()?;
                }
                _ => {
                    return Err(ParseError::expected(
                        vec![
                            TokenKind::Punctuator(Punctuator::Semicolon),
                            TokenKind::LineTerminator,
                        ],
                        cursor.next()?.ok_or(ParseError::AbruptEnd)?,
                        "lexical declaration binding list",
                    ))
                }
            }
        }

        if self.is_const {
            Ok(DeclarationList::Const(const_decls.into()).into())
        } else {
            Ok(DeclarationList::Let(let_decls.into()).into())
        }
    }
}

/// Lexical binding parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-LexicalBinding
struct LexicalBinding {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl LexicalBinding {
    /// Creates a new `BindingList` parser.
    fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for LexicalBinding
where
    R: Read,
{
    type Output = (Box<str>, Option<Node>);

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("LexicalBinding", "Parsing");

        let ident = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;

        let init = if let Some(t) = cursor.peek(0)? {
            if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                Some(
                    Initializer::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(cursor)?,
                )
            } else {
                None
            }
        } else {
            None
        };

        Ok((ident, init))
    }
}
