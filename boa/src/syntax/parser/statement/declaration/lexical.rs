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
            ParseError, ParseResult, TokenParser,
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
pub(super) struct LexicalDeclaration<
    const IN: bool,
    const YIELD: bool,
    const AWAIT: bool,
    const CONST_INIT: bool,
>;

impl<R, const IN: bool, const YIELD: bool, const AWAIT: bool, const CONST_INIT: bool> TokenParser<R>
    for LexicalDeclaration<IN, YIELD, AWAIT, CONST_INIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("LexicalDeclaration", "Parsing");
        let tok = cursor.next()?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::Const) => {
                BindingList::<IN, YIELD, AWAIT, true, CONST_INIT>.parse(cursor)
            }
            TokenKind::Keyword(Keyword::Let) => {
                BindingList::<IN, YIELD, AWAIT, false, CONST_INIT>.parse(cursor)
            }
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
struct BindingList<
    const IN: bool,
    const YIELD: bool,
    const AWAIT: bool,
    const CONST: bool,
    const CONST_INIT: bool,
>;

impl<
        R,
        const IN: bool,
        const YIELD: bool,
        const AWAIT: bool,
        const CONST: bool,
        const CONST_INIT: bool,
    > TokenParser<R> for BindingList<IN, YIELD, AWAIT, CONST, CONST_INIT>
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
            let decl = LexicalBinding::<IN, YIELD, AWAIT>.parse(cursor)?;

            if CONST {
                if CONST_INIT {
                    let init_is_some = match &decl {
                        Declaration::Identifier { init, .. } if init.is_some() => true,
                        Declaration::Pattern(p) if p.init().is_some() => true,
                        _ => false,
                    };

                    if init_is_some {
                        const_decls.push(decl);
                    } else {
                        return Err(ParseError::expected(
                            vec![TokenKind::Punctuator(Punctuator::Assign)],
                            cursor.next()?.ok_or(ParseError::AbruptEnd)?,
                            "const declaration",
                        ));
                    }
                } else {
                    const_decls.push(decl)
                }
            } else {
                let_decls.push(decl);
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

        if CONST {
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
struct LexicalBinding<const IN: bool, const YIELD: bool, const AWAIT: bool>;

impl<R, const IN: bool, const YIELD: bool, const AWAIT: bool> TokenParser<R>
    for LexicalBinding<IN, YIELD, AWAIT>
where
    R: Read,
{
    type Output = Declaration;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("LexicalBinding", "Parsing");

        let peek_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        match peek_token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let bindings = ObjectBindingPattern::<IN, YIELD, AWAIT>.parse(cursor)?;

                let init = if let Some(t) = cursor.peek(0)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(Initializer::<IN, YIELD, AWAIT>.parse(cursor)?)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Declaration::new_with_object_pattern(bindings, init))
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let bindings = ArrayBindingPattern::<IN, YIELD, AWAIT>.parse(cursor)?;

                let init = if let Some(t) = cursor.peek(0)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(Initializer::<IN, YIELD, AWAIT>.parse(cursor)?)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Declaration::new_with_array_pattern(bindings, init))
            }

            _ => {
                let ident = BindingIdentifier::<YIELD, AWAIT>.parse(cursor)?;

                let init = if let Some(t) = cursor.peek(0)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(Initializer::<true, YIELD, AWAIT>.parse(cursor)?)
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
