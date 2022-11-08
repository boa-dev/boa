//! Lexical declaration parsing.
//!
//! This parses `let` and `const` declarations.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations

use crate::{
    lexer::{Error as LexError, TokenKind},
    parser::{
        cursor::{Cursor, SemicolonResult},
        expression::Initializer,
        statement::{ArrayBindingPattern, BindingIdentifier, ObjectBindingPattern},
        AllowAwait, AllowIn, AllowYield, OrAbrupt, ParseResult, TokenParser,
    },
    Error,
};
use ast::operations::bound_names;
use boa_ast::{self as ast, declaration::Variable, pattern::Pattern, Keyword, Punctuator};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::{convert::TryInto, io::Read};

/// Parses a lexical declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-LexicalDeclaration
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct LexicalDeclaration {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    loop_init: bool,
}

impl LexicalDeclaration {
    /// Creates a new `LexicalDeclaration` parser.
    pub(in crate::parser) fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
        loop_init: bool,
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
            loop_init,
        }
    }
}

impl<R> TokenParser<R> for LexicalDeclaration
where
    R: Read,
{
    type Output = ast::declaration::LexicalDeclaration;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("LexicalDeclaration", "Parsing");
        let tok = cursor.next(interner).or_abrupt()?;

        match tok.kind() {
            TokenKind::Keyword((Keyword::Const | Keyword::Let, true)) => Err(Error::general(
                "Keyword must not contain escaped characters",
                tok.span().start(),
            )),
            TokenKind::Keyword((Keyword::Const, false)) => BindingList::new(
                self.allow_in,
                self.allow_yield,
                self.allow_await,
                true,
                self.loop_init,
            )
            .parse(cursor, interner),
            TokenKind::Keyword((Keyword::Let, false)) => BindingList::new(
                self.allow_in,
                self.allow_yield,
                self.allow_await,
                false,
                self.loop_init,
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
    loop_init: bool,
}

impl BindingList {
    /// Creates a new `BindingList` parser.
    fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
        is_const: bool,
        loop_init: bool,
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
            loop_init,
        }
    }
}

impl<R> TokenParser<R> for BindingList
where
    R: Read,
{
    type Output = ast::declaration::LexicalDeclaration;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("BindingList", "Parsing");

        // Create vectors to store the variable declarations
        // Const and Let signatures are slightly different, Const needs definitions, Lets don't
        let mut decls = Vec::new();

        loop {
            let decl = LexicalBinding::new(self.allow_in, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;

            if self.is_const {
                let init_is_some = decl.init().is_some();

                if init_is_some || self.loop_init {
                    decls.push(decl);
                } else {
                    let next = cursor.next(interner).or_abrupt()?;
                    return Err(Error::general(
                        "Expected initializer for const declaration",
                        next.span().start(),
                    ));
                }
            } else {
                decls.push(decl);
            }

            match cursor.peek_semicolon(interner)? {
                SemicolonResult::Found(_) => break,
                SemicolonResult::NotFound(tk)
                    if tk.kind() == &TokenKind::Keyword((Keyword::Of, true))
                        || tk.kind() == &TokenKind::Keyword((Keyword::In, true)) =>
                {
                    return Err(Error::general(
                        "Keyword must not contain escaped characters",
                        tk.span().start(),
                    ));
                }
                SemicolonResult::NotFound(tk)
                    if tk.kind() == &TokenKind::Keyword((Keyword::Of, false))
                        || tk.kind() == &TokenKind::Keyword((Keyword::In, false)) =>
                {
                    break
                }
                SemicolonResult::NotFound(tk)
                    if tk.kind() == &TokenKind::Punctuator(Punctuator::Comma) =>
                {
                    // We discard the comma
                    cursor.advance(interner);
                }
                SemicolonResult::NotFound(_) if self.loop_init => break,
                SemicolonResult::NotFound(_) => {
                    let next = cursor.next(interner).or_abrupt()?;
                    return Err(Error::expected(
                        [";".to_owned(), "line terminator".to_owned()],
                        next.to_string(interner),
                        next.span(),
                        "lexical declaration binding list",
                    ));
                }
            }
        }

        let decls = decls
            .try_into()
            .expect("`LexicalBinding` must return at least one variable");

        if self.is_const {
            Ok(ast::declaration::LexicalDeclaration::Const(decls))
        } else {
            Ok(ast::declaration::LexicalDeclaration::Let(decls))
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
    type Output = Variable;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("LexicalBinding", "Parsing");

        let peek_token = cursor.peek(0, interner).or_abrupt()?;
        let position = peek_token.span().start();

        match peek_token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let bindings = ObjectBindingPattern::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let init = if cursor
                    .peek(0, interner)?
                    .filter(|t| *t.kind() == TokenKind::Punctuator(Punctuator::Assign))
                    .is_some()
                {
                    Some(
                        Initializer::new(None, self.allow_in, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    )
                } else {
                    None
                };

                let declaration = Pattern::Object(bindings.into());

                if bound_names(&declaration).contains(&Sym::LET.into()) {
                    return Err(Error::lex(LexError::Syntax(
                        "'let' is disallowed as a lexically bound name".into(),
                        position,
                    )));
                }

                Ok(Variable::from_pattern(declaration, init))
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let bindings = ArrayBindingPattern::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let init = if cursor
                    .peek(0, interner)?
                    .filter(|t| *t.kind() == TokenKind::Punctuator(Punctuator::Assign))
                    .is_some()
                {
                    Some(
                        Initializer::new(None, self.allow_in, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    )
                } else {
                    None
                };

                let declaration = Pattern::Array(bindings.into());

                if bound_names(&declaration).contains(&Sym::LET.into()) {
                    return Err(Error::lex(LexError::Syntax(
                        "'let' is disallowed as a lexically bound name".into(),
                        position,
                    )));
                }

                Ok(Variable::from_pattern(declaration, init))
            }
            _ => {
                let ident = BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                if ident == Sym::LET {
                    return Err(Error::lex(LexError::Syntax(
                        "'let' is disallowed as a lexically bound name".into(),
                        position,
                    )));
                }

                let init = if cursor
                    .peek(0, interner)?
                    .filter(|t| *t.kind() == TokenKind::Punctuator(Punctuator::Assign))
                    .is_some()
                {
                    Some(
                        Initializer::new(
                            Some(ident),
                            self.allow_in,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)?,
                    )
                } else {
                    None
                };
                Ok(Variable::from_identifier(ident, init))
            }
        }
    }
}
