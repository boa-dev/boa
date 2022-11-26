//! Import declaration parsing
//!
//! This parses `import` declarations.
//!
//! More information:
//! - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-imports
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import

use crate::{
    lexer::TokenKind,
    parser::{
        cursor::Cursor,
        statement::{declaration::FromClause, BindingIdentifier},
        Error, OrAbrupt, ParseResult, TokenParser,
    },
};
use boa_ast::{expression::Identifier, Keyword, Punctuator};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

/// Parses an import declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ImportDeclaration
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct ImportDeclaration;

impl<R> TokenParser<R> for ImportDeclaration
where
    R: Read,
{
    type Output = boa_ast::declaration::ImportDeclaration;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("ImportDeclaration", "Parsing");

        cursor.expect((Keyword::Import, false), "import declaration", interner)?;

        let tok = cursor.peek(0, interner).or_abrupt()?;

        let import_clause = match tok.kind() {
            TokenKind::StringLiteral((module_identifier, _)) => {
                let module_identifier = *module_identifier;

                cursor.advance(interner);
                cursor.expect_semicolon("import declaration", interner)?;

                return Ok(boa_ast::declaration::ImportDeclaration::Module(
                    module_identifier,
                ));
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let list = NamedImports.parse(cursor, interner)?;
                ImportClause::ImportList(None, list)
            }
            TokenKind::Punctuator(Punctuator::Mul) => {
                let alias = NameSpaceImport.parse(cursor, interner)?;
                ImportClause::Namespace(None, alias)
            }
            TokenKind::IdentifierName(_)
            | TokenKind::Keyword((Keyword::Await | Keyword::Yield, _)) => {
                let imported_binding = ImportedBinding.parse(cursor, interner)?;

                let tok = cursor.peek(0, interner).or_abrupt()?;

                match tok.kind() {
                    TokenKind::Punctuator(Punctuator::Comma) => {
                        cursor.advance(interner);
                        let tok = cursor.peek(0, interner).or_abrupt()?;

                        match tok.kind() {
                            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                                let list = NamedImports.parse(cursor, interner)?;
                                ImportClause::ImportList(Some(imported_binding), list)
                            }
                            TokenKind::Punctuator(Punctuator::Mul) => {
                                let alias = NameSpaceImport.parse(cursor, interner)?;
                                ImportClause::Namespace(Some(imported_binding), alias)
                            }
                            _ => {
                                return Err(Error::expected(
                                    [
                                        Punctuator::OpenBlock.to_string(),
                                        Punctuator::Mul.to_string(),
                                    ],
                                    tok.to_string(interner),
                                    tok.span(),
                                    "import declaration",
                                ))
                            }
                        }
                    }
                    _ => ImportClause::ImportList(Some(imported_binding), Box::new([])),
                }
            }
            _ => {
                return Err(Error::expected(
                    [
                        Punctuator::OpenBlock.to_string(),
                        Punctuator::Mul.to_string(),
                        "identifier".to_owned(),
                        "string literal".to_owned(),
                    ],
                    tok.to_string(interner),
                    tok.span(),
                    "import declaration",
                ))
            }
        };

        let module_identifier = FromClause::new("import declaration").parse(cursor, interner)?;

        Ok(import_clause.add_from(module_identifier))
    }
}

/// Parses an imported binding
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ImportedBinding
#[derive(Debug, Clone, Copy)]
struct ImportedBinding;

impl<R> TokenParser<R> for ImportedBinding
where
    R: Read,
{
    type Output = Identifier;

    #[inline]
    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        BindingIdentifier::new(false, true).parse(cursor, interner)
    }
}

/// Parses a named import list.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-NamedImports
#[derive(Debug, Clone, Copy)]
struct NamedImports;

impl<R> TokenParser<R> for NamedImports
where
    R: Read,
{
    type Output = Box<[boa_ast::declaration::ImportSpecifier]>;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        cursor.expect(Punctuator::OpenBlock, "import declaration", interner)?;

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
                            "import declaration",
                        ));
                    }
                }
                TokenKind::StringLiteral(_) | TokenKind::IdentifierName(_) => {
                    list.push(ImportSpecifier.parse(cursor, interner)?);
                }
                _ => {
                    return Err(Error::expected(
                        [
                            Punctuator::CloseBlock.to_string(),
                            Punctuator::Comma.to_string(),
                        ],
                        tok.to_string(interner),
                        tok.span(),
                        "import declaration",
                    ));
                }
            }
        }

        Ok(list.into_boxed_slice())
    }
}

/// Parses an import clause.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ImportClause
#[derive(Debug, Clone)]
enum ImportClause {
    Namespace(Option<Identifier>, Identifier),
    ImportList(
        Option<Identifier>,
        Box<[boa_ast::declaration::ImportSpecifier]>,
    ),
}

impl ImportClause {
    #[inline]
    fn add_from(
        self,
        from_clause: boa_ast::declaration::FromClause,
    ) -> boa_ast::declaration::ImportDeclaration {
        match self {
            Self::Namespace(default, alias) => {
                boa_ast::declaration::ImportDeclaration::namespace(default, alias, from_clause)
            }
            Self::ImportList(default, list) => {
                boa_ast::declaration::ImportDeclaration::list(default, list, from_clause)
            }
        }
    }
}

/// Parses an import specifier.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ImportSpecifier
#[derive(Debug, Clone, Copy)]
struct ImportSpecifier;

impl<R> TokenParser<R> for ImportSpecifier
where
    R: Read,
{
    type Output = boa_ast::declaration::ImportSpecifier;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let tok = cursor.next(interner).or_abrupt()?;

        match tok.kind() {
            TokenKind::StringLiteral((name, _)) => {
                cursor.expect(
                    TokenKind::identifier(Sym::AS),
                    "import declaration",
                    interner,
                )?;

                let alias = ImportedBinding.parse(cursor, interner)?;

                Ok(boa_ast::declaration::ImportSpecifier::new(
                    *name,
                    Some(alias),
                ))
            }
            TokenKind::IdentifierName((name, _)) => {
                if cursor
                    .next_if(TokenKind::identifier(Sym::AS), interner)?
                    .is_some()
                {
                    let alias = ImportedBinding.parse(cursor, interner)?;
                    Ok(boa_ast::declaration::ImportSpecifier::new(
                        *name,
                        Some(alias),
                    ))
                } else {
                    Ok(boa_ast::declaration::ImportSpecifier::new(*name, None))
                }
            }
            _ => Err(Error::expected(
                ["string literal".to_owned(), "identifier".to_owned()],
                tok.to_string(interner),
                tok.span(),
                "import declaration",
            )),
        }
    }
}

/// Parses a namespace import
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-NameSpaceImport
#[derive(Debug, Clone, Copy)]
struct NameSpaceImport;

impl<R> TokenParser<R> for NameSpaceImport
where
    R: Read,
{
    type Output = Identifier;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        cursor.expect(Punctuator::Mul, "import declaration", interner)?;
        cursor.expect(
            TokenKind::identifier(Sym::AS),
            "import declaration",
            interner,
        )?;

        ImportedBinding.parse(cursor, interner)
    }
}
