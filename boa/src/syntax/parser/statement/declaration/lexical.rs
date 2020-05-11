//! Lexical declaration parsing.
//!
//! This parses `let` and `const` declarations.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations

use crate::syntax::{
    ast::{keyword::Keyword, node::Node, punc::Punctuator, token::TokenKind},
    parser::{
        expression::Initializer, statement::BindingIdentifier, AllowAwait, AllowIn, AllowYield,
        Cursor, ParseError, ParseResult, TokenParser,
    },
};

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
}

impl LexicalDeclaration {
    /// Creates a new `LexicalDeclaration` parser.
    pub(super) fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
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

impl TokenParser for LexicalDeclaration {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let tok = cursor.next().ok_or(ParseError::AbruptEnd)?;

        match tok.kind {
            TokenKind::Keyword(Keyword::Const) => {
                BindingList::new(self.allow_in, self.allow_yield, self.allow_await, true)
                    .parse(cursor)
            }
            TokenKind::Keyword(Keyword::Let) => {
                BindingList::new(self.allow_in, self.allow_yield, self.allow_await, false)
                    .parse(cursor)
            }
            _ => unreachable!("unknown token found"),
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
}

impl BindingList {
    /// Creates a new `BindingList` parser.
    fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A, is_const: bool) -> Self
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
        }
    }
}

impl TokenParser for BindingList {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        // Create vectors to store the variable declarations
        // Const and Let signatures are slightly different, Const needs definitions, Lets don't
        let mut let_decls = Vec::new();
        let mut const_decls = Vec::new();

        loop {
            let lexical_binding =
                LexicalBinding::new(self.allow_in, self.allow_yield, self.allow_await)
                    .parse(cursor)?;

            if self.is_const {
                if let (ident, Some(init)) = lexical_binding {
                    const_decls.push((ident, init));
                } else {
                    return Err(ParseError::Expected(
                        vec![TokenKind::Punctuator(Punctuator::Assign)],
                        cursor.next().ok_or(ParseError::AbruptEnd)?.clone(),
                        "const declaration",
                    ));
                }
            } else {
                let_decls.push(lexical_binding);
            }

            match cursor.peek_semicolon(false) {
                (true, _) => break,
                (false, Some(tk)) if tk.kind == TokenKind::Punctuator(Punctuator::Comma) => {
                    let _ = cursor.next();
                }
                _ => {
                    return Err(ParseError::Expected(
                        vec![
                            TokenKind::Punctuator(Punctuator::Semicolon),
                            TokenKind::LineTerminator,
                        ],
                        cursor.next().ok_or(ParseError::AbruptEnd)?.clone(),
                        "lexical declaration",
                    ))
                }
            }
        }

        if self.is_const {
            Ok(Node::const_decl(const_decls))
        } else {
            Ok(Node::let_decl(let_decls))
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

impl TokenParser for LexicalBinding {
    type Output = (String, Option<Node>);

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<(String, Option<Node>), ParseError> {
        let ident = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
        let initializer =
            Initializer::new(self.allow_in, self.allow_yield, self.allow_await).try_parse(cursor);

        Ok((ident, initializer))
    }
}
