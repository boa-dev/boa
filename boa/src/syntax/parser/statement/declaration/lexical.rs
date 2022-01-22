//! Lexical declaration parsing.
//!
//! This parses `let` and `const` declarations.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations

use crate::{
    syntax::{
        ast::{
            node::{
                declaration::{Declaration, DeclarationList},
                Node,
            },
            Keyword, Punctuator,
        },
        lexer::TokenKind,
        parser::{
            cursor::{Cursor, SemicolonResult},
            expression::Initializer,
            statement::{ArrayBindingPattern, BindingIdentifier, ObjectBindingPattern},
            AllowAwait, AllowIn, AllowYield, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler, Interner,
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

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("LexicalDeclaration", "Parsing");
        let tok = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::Const) => BindingList::new(
                self.allow_in,
                self.allow_yield,
                self.allow_await,
                true,
                self.const_init_required,
            )
            .parse(cursor, interner),
            TokenKind::Keyword(Keyword::Let) => BindingList::new(
                self.allow_in,
                self.allow_yield,
                self.allow_await,
                false,
                self.const_init_required,
            )
            .parse(cursor, interner),
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

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("BindingList", "Parsing");

        // Create vectors to store the variable declarations
        // Const and Let signatures are slightly different, Const needs definitions, Lets don't
        let mut let_decls = Vec::new();
        let mut const_decls = Vec::new();

        loop {
            let decl = LexicalBinding::new(self.allow_in, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;

            if self.is_const {
                if self.const_init_required {
                    let init_is_some = match &decl {
                        Declaration::Identifier { init, .. } if init.is_some() => true,
                        Declaration::Pattern(p) if p.init().is_some() => true,
                        _ => false,
                    };

                    if init_is_some {
                        const_decls.push(decl);
                    } else {
                        let next = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;
                        return Err(ParseError::expected(
                            ["=".to_owned()],
                            next.to_string(interner),
                            next.span(),
                            "const declaration",
                        ));
                    }
                } else {
                    const_decls.push(decl)
                }
            } else {
                let_decls.push(decl);
            }

            match cursor.peek_semicolon(interner)? {
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
                    let _ = cursor.next(interner)?;
                }
                _ => {
                    let next = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;
                    return Err(ParseError::expected(
                        [";".to_owned(), "line terminator".to_owned()],
                        next.to_string(interner),
                        next.span(),
                        "lexical declaration binding list",
                    ));
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
    type Output = Declaration;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("LexicalBinding", "Parsing");

        let peek_token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;

        match peek_token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let bindings =
                    ObjectBindingPattern::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;

                let init = if let Some(t) = cursor.peek(0, interner)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(
                            Initializer::new(self.allow_in, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?,
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Declaration::new_with_object_pattern(bindings, init))
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let bindings =
                    ArrayBindingPattern::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;

                let init = if let Some(t) = cursor.peek(0, interner)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(
                            Initializer::new(self.allow_in, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?,
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Declaration::new_with_array_pattern(bindings, init))
            }

            _ => {
                let ident = BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let init = if let Some(t) = cursor.peek(0, interner)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(
                            Initializer::new(true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?,
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Declaration::new_with_identifier(ident, init))
            }
        }
    }
}
