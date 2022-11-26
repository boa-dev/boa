//! Export declaration parsing
//!
//! This parses `export` declarations.
//!
//! More information:
//! - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-exports
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/export

use crate::{
    lexer::{token::ContainsEscapeSequence, TokenKind},
    parser::{
        cursor::Cursor,
        statement::{declaration::ClassDeclaration, variable::VariableStatement},
        Error, OrAbrupt, ParseResult, TokenParser,
    },
};
use boa_ast::{Keyword, Punctuator};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

use super::FromClause;

/// Parses an export declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ExportDeclaration
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct ExportDeclaration;

impl<R> TokenParser<R> for ExportDeclaration
where
    R: Read,
{
    type Output = boa_ast::declaration::ExportDeclaration;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("ExportDeclaration", "Parsing");

        cursor.expect((Keyword::Export, false), "export declaration", interner)?;

        let tok = cursor.peek(0, interner).or_abrupt()?;

        let export_clause: Self::Output = match tok.kind() {
            TokenKind::Punctuator(Punctuator::Mul) => {
                cursor.advance(interner);

                let next = cursor.peek(0, interner).or_abrupt()?;

                match next.kind() {
                    TokenKind::IdentifierName((Sym::AS, _)) => {
                        cursor.advance(interner);
                        let tok = cursor.next(interner).or_abrupt()?;

                        let alias = match tok.kind() {
                            TokenKind::StringLiteral((export_name, _))
                            | TokenKind::IdentifierName((export_name, _)) => *export_name,
                            _ => {
                                return Err(Error::expected(
                                    ["identifier".to_owned(), "string literal".to_owned()],
                                    tok.to_string(interner),
                                    tok.span(),
                                    "export declaration",
                                ))
                            }
                        };

                        let from = FromClause::new("export declaration").parse(cursor, interner)?;

                        boa_ast::declaration::ExportDeclaration::ReExportAll {
                            alias: Some(alias),
                            from,
                        }
                    }
                    TokenKind::IdentifierName((Sym::FROM, _)) => {
                        let from = FromClause::new("export declaration").parse(cursor, interner)?;

                        boa_ast::declaration::ExportDeclaration::ReExportAll { alias: None, from }
                    }
                    _ => {
                        return Err(Error::expected(
                            ["as".to_owned(), "from".to_owned()],
                            next.to_string(interner),
                            next.span(),
                            "export declaration",
                        ))
                    }
                }
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let list = NamedExports.parse(cursor, interner)?;

                let next = cursor.peek(0, interner).or_abrupt()?;

                if let TokenKind::IdentifierName((Sym::FROM, ContainsEscapeSequence(false))) =
                    next.kind()
                {
                    let from = FromClause::new("export declaration").parse(cursor, interner)?;
                    boa_ast::declaration::ExportDeclaration::List {
                        list,
                        from: Some(from),
                    }
                } else {
                    boa_ast::declaration::ExportDeclaration::List { list, from: None }
                }
            }
            TokenKind::Keyword((Keyword::Var, false)) => VariableStatement::new(false, true)
                .parse(cursor, interner)
                .map(boa_ast::declaration::ExportDeclaration::VarStatement)?,
            TokenKind::Keyword((Keyword::Default, _)) => {
                cursor.advance(interner);

                let tok = cursor.peek(0, interner).or_abrupt()?;

                match tok.kind() {
                    TokenKind::Keyword((Keyword::Class, _)) => {
                        ClassDeclaration::new(false, true, true)
                            .parse(cursor, interner)
                            .map(boa_ast::declaration::ExportDeclaration::DefaultClassDeclaration)?
                    }
                    _ => todo!("default export parsing"),
                }
            }
            _ => {
                todo!()
            }
        };

        Ok(export_clause)
    }
}

/// Parses a named export list.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-NamedExports
#[derive(Debug, Clone, Copy)]
struct NamedExports;

impl<R> TokenParser<R> for NamedExports
where
    R: Read,
{
    type Output = Box<[boa_ast::declaration::ExportSpecifier]>;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        cursor.expect(Punctuator::OpenBlock, "export declaration", interner)?;

        let mut list = Vec::new();

        loop {
            let tok = cursor.next(interner).or_abrupt()?;
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    break;
                }
                TokenKind::Punctuator(Punctuator::Comma) => {
                    if list.is_empty() {
                        return Err(Error::expected(
                            [
                                Punctuator::CloseBlock.to_string(),
                                "string literal".to_owned(),
                                "identifier".to_owned(),
                            ],
                            tok.to_string(interner),
                            tok.span(),
                            "export declaration",
                        ));
                    }
                }
                TokenKind::StringLiteral(_) | TokenKind::IdentifierName(_) => {
                    list.push(ExportSpecifier.parse(cursor, interner)?);
                }
                _ => {
                    return Err(Error::expected(
                        [
                            Punctuator::CloseBlock.to_string(),
                            Punctuator::Comma.to_string(),
                        ],
                        tok.to_string(interner),
                        tok.span(),
                        "export declaration",
                    ));
                }
            }
        }

        Ok(list.into_boxed_slice())
    }
}

/// Parses a module export name.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ModuleExportName
#[derive(Debug, Clone, Copy)]
pub(super) struct ModuleExportName;

impl<R> TokenParser<R> for ModuleExportName
where
    R: Read,
{
    type Output = Sym;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let tok = cursor.next(interner).or_abrupt()?;

        match tok.kind() {
            TokenKind::IdentifierName((ident, _)) | TokenKind::StringLiteral((ident, _)) => {
                Ok(*ident)
            }
            _ => Err(Error::expected(
                ["identifier".to_owned(), "string literal".to_owned()],
                tok.to_string(interner),
                tok.span(),
                "export specifier parsing",
            )),
        }
    }
}

/// Parses an export specifier.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ExportSpecifier
#[derive(Debug, Clone, Copy)]
struct ExportSpecifier;

impl<R> TokenParser<R> for ExportSpecifier
where
    R: Read,
{
    type Output = boa_ast::declaration::ExportSpecifier;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let export = ModuleExportName.parse(cursor, interner)?;

        let alias = if cursor
            .next_if(TokenKind::identifier(Sym::AS), interner)?
            .is_some()
        {
            Some(ModuleExportName.parse(cursor, interner)?)
        } else {
            None
        };

        Ok(boa_ast::declaration::ExportSpecifier::new(export, alias))
    }
}
