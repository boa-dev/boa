#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node, node::Switch, Keyword, Punctuator},
    lexer::TokenKind,
    parser::{
        expression::Expression, statement::StatementList, AllowAwait, AllowReturn, AllowYield,
        Cursor, ParseError, TokenParser,
    },
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use rustc_hash::{FxHashMap, FxHashSet};
use std::io::Read;

/// The possible `TokenKind` which indicate the end of a case statement.
const CASE_BREAK_TOKENS: [TokenKind; 3] = [
    TokenKind::Punctuator(Punctuator::CloseBlock),
    TokenKind::Keyword((Keyword::Case, false)),
    TokenKind::Keyword((Keyword::Default, false)),
];

/// Switch statement parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
/// [spec]: https://tc39.es/ecma262/#prod-SwitchStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct SwitchStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl SwitchStatement {
    /// Creates a new `SwitchStatement` parser.
    pub(super) fn new<Y, A, R>(allow_yield: Y, allow_await: A, allow_return: R) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
        }
    }
}

impl<R> TokenParser<R> for SwitchStatement
where
    R: Read,
{
    type Output = Switch;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("SwitchStatement", "Parsing");
        cursor.expect((Keyword::Switch, false), "switch statement", interner)?;
        cursor.expect(Punctuator::OpenParen, "switch statement", interner)?;

        let condition = Expression::new(None, true, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        cursor.expect(Punctuator::CloseParen, "switch statement", interner)?;

        let (cases, default) =
            CaseBlock::new(self.allow_yield, self.allow_await, self.allow_return)
                .parse(cursor, interner)?;

        Ok(Switch::new(condition, cases, default))
    }
}

/// Switch case block parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-CaseBlock
#[derive(Debug, Clone, Copy)]
struct CaseBlock {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl CaseBlock {
    /// Creates a new `CaseBlock` parser.
    fn new<Y, A, R>(allow_yield: Y, allow_await: A, allow_return: R) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
        }
    }
}

impl<R> TokenParser<R> for CaseBlock
where
    R: Read,
{
    type Output = (Box<[node::Case]>, Option<node::StatementList>);

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        cursor.expect(Punctuator::OpenBlock, "switch case block", interner)?;

        let mut cases = Vec::new();
        let mut default = None;

        let position = cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .span()
            .start();
        loop {
            let token = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;
            match token.kind() {
                TokenKind::Keyword((Keyword::Case | Keyword::Default, true)) => {
                    return Err(ParseError::general(
                        "Keyword must not contain escaped characters",
                        token.span().start(),
                    ));
                }
                TokenKind::Keyword((Keyword::Case, false)) => {
                    // Case statement.
                    let cond = Expression::new(None, true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;

                    cursor.expect(Punctuator::Colon, "switch case block", interner)?;

                    let statement_list = StatementList::new(
                        self.allow_yield,
                        self.allow_await,
                        self.allow_return,
                        &CASE_BREAK_TOKENS,
                    )
                    .parse(cursor, interner)?;

                    cases.push(node::Case::new(cond, statement_list));
                }
                TokenKind::Keyword((Keyword::Default, false)) => {
                    if default.is_some() {
                        // If default has already been defined then it cannot be defined again and to do so is an error.
                        return Err(ParseError::unexpected(
                            token.to_string(interner),
                            token.span(),
                            Some("more than one switch default"),
                        ));
                    }

                    cursor.expect(Punctuator::Colon, "switch default block", interner)?;

                    let statement_list = StatementList::new(
                        self.allow_yield,
                        self.allow_await,
                        self.allow_return,
                        &CASE_BREAK_TOKENS,
                    )
                    .parse(cursor, interner)?;

                    default = Some(statement_list);
                }
                TokenKind::Punctuator(Punctuator::CloseBlock) => break,
                _ => {
                    return Err(ParseError::expected(
                        ["case".to_owned(), "default".to_owned(), "}".to_owned()],
                        token.to_string(interner),
                        token.span(),
                        "switch case block",
                    ))
                }
            }
        }

        // It is a Syntax Error if the LexicallyDeclaredNames of CaseBlock contains any duplicate entries.
        // It is a Syntax Error if any element of the LexicallyDeclaredNames of CaseBlock also occurs in the VarDeclaredNames of CaseBlock.
        let mut lexically_declared_names = Vec::new();
        let mut var_declared_names = FxHashSet::default();
        for case in &cases {
            lexically_declared_names.extend(case.body().lexically_declared_names());

            case.body().var_declared_names_new(&mut var_declared_names);
        }
        if let Some(default_clause) = &default {
            lexically_declared_names.extend(default_clause.lexically_declared_names());

            default_clause.var_declared_names_new(&mut var_declared_names);
        }

        let mut lexically_declared_names_map: FxHashMap<Sym, bool> = FxHashMap::default();
        for (name, is_function_declaration) in &lexically_declared_names {
            if let Some(existing_is_function_declaration) = lexically_declared_names_map.get(name) {
                if !(!cursor.strict_mode()
                    && *is_function_declaration
                    && *existing_is_function_declaration)
                {
                    return Err(ParseError::general(
                        "lexical name declared multiple times",
                        position,
                    ));
                }
            }
            lexically_declared_names_map.insert(*name, *is_function_declaration);
        }

        for (name, _) in &lexically_declared_names {
            if var_declared_names.contains(name) {
                return Err(ParseError::general(
                    "lexical name declared in var declared names",
                    position,
                ));
            }
        }

        Ok((cases.into_boxed_slice(), default))
    }
}
