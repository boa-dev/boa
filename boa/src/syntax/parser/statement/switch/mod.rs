#[cfg(test)]
mod tests;

use crate::{
    syntax::{
        ast::{node, node::Switch, Keyword, Punctuator},
        lexer::TokenKind,
        parser::{
            expression::Expression, statement::StatementList, Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// The possible TokenKind which indicate the end of a case statement.
const CASE_BREAK_TOKENS: [TokenKind; 3] = [
    TokenKind::Punctuator(Punctuator::CloseBlock),
    TokenKind::Keyword(Keyword::Case),
    TokenKind::Keyword(Keyword::Default),
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
pub(super) struct SwitchStatement<const YIELD: bool, const AWAIT: bool, const RETURN: bool>;

impl<R, const YIELD: bool, const AWAIT: bool, const RETURN: bool> TokenParser<R>
    for SwitchStatement<YIELD, AWAIT, RETURN>
where
    R: Read,
{
    type Output = Switch;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("SwitchStatement", "Parsing");
        cursor.expect(Keyword::Switch, "switch statement")?;
        cursor.expect(Punctuator::OpenParen, "switch statement")?;

        let condition = Expression::<true, YIELD, AWAIT>.parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "switch statement")?;

        let (cases, default) = CaseBlock::<YIELD, AWAIT, RETURN>.parse(cursor)?;

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
struct CaseBlock<const YIELD: bool, const AWAIT: bool, const RETURN: bool>;

impl<R, const YIELD: bool, const AWAIT: bool, const RETURN: bool> TokenParser<R>
    for CaseBlock<YIELD, AWAIT, RETURN>
where
    R: Read,
{
    type Output = (Box<[node::Case]>, Option<node::StatementList>);

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Punctuator::OpenBlock, "switch case block")?;

        let mut cases = Vec::new();
        let mut default = None;

        loop {
            match cursor.next()? {
                Some(token) if token.kind() == &TokenKind::Keyword(Keyword::Case) => {
                    // Case statement.
                    let cond = Expression::<true, YIELD, AWAIT>.parse(cursor)?;

                    cursor.expect(Punctuator::Colon, "switch case block")?;

                    let statement_list =
                        StatementList::<YIELD, AWAIT, RETURN, false>::new(&CASE_BREAK_TOKENS)
                            .parse(cursor)?;

                    cases.push(node::Case::new(cond, statement_list));
                }
                Some(token) if token.kind() == &TokenKind::Keyword(Keyword::Default) => {
                    if default.is_some() {
                        // If default has already been defined then it cannot be defined again and to do so is an error.
                        return Err(ParseError::unexpected(
                            token,
                            Some("more than one switch default"),
                        ));
                    }

                    cursor.expect(Punctuator::Colon, "switch default block")?;

                    let statement_list =
                        StatementList::<YIELD, AWAIT, RETURN, false>::new(&CASE_BREAK_TOKENS)
                            .parse(cursor)?;

                    default = Some(statement_list);
                }
                Some(token) if token.kind() == &TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    break
                }
                Some(token) => {
                    return Err(ParseError::expected(
                        vec![
                            TokenKind::Keyword(Keyword::Case),
                            TokenKind::Keyword(Keyword::Default),
                            TokenKind::Punctuator(Punctuator::CloseBlock),
                        ],
                        token,
                        "switch case block",
                    ))
                }
                None => return Err(ParseError::AbruptEnd),
            }
        }

        Ok((cases.into_boxed_slice(), default))
    }
}
