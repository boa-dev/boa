//! Template literal parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals
//! [spec]: https://tc39.es/ecma262/#sec-template-literals

use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::node::template::{TemplateElement, TemplateLit},
        ast::Position,
        ast::Punctuator,
        lexer::TokenKind,
        parser::cursor::Cursor,
        parser::expression::Expression,
        parser::{ParseError, TokenParser},
    },
};
use std::io::Read;

/// Parses a template literal.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals
/// [spec]: https://tc39.es/ecma262/#prod-TemplateLiteral
#[derive(Debug, Clone)]
pub(super) struct TemplateLiteral<const YIELD: bool, const AWAIT: bool> {
    start: Position,
    first: String,
}

impl<const YIELD: bool, const AWAIT: bool> TemplateLiteral<YIELD, AWAIT> {
    /// Creates a new `TemplateLiteral` parser.
    pub(super) fn new(start: Position, first: &str) -> Self {
        Self {
            start,
            first: first.to_owned(),
        }
    }
}

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for TemplateLiteral<YIELD, AWAIT>
where
    R: Read,
{
    type Output = TemplateLit;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("TemplateLiteral", "Parsing");

        let mut elements = vec![
            TemplateElement::String(self.first.into_boxed_str()),
            TemplateElement::Expr(Expression::<true, YIELD, AWAIT>.parse(cursor)?),
        ];
        cursor.expect(
            TokenKind::Punctuator(Punctuator::CloseBlock),
            "template literal",
        )?;

        loop {
            match cursor.lex_template(self.start)?.kind() {
                TokenKind::TemplateMiddle(template_string) => {
                    let cooked = template_string.to_owned_cooked().map_err(ParseError::lex)?;

                    elements.push(TemplateElement::String(cooked));
                    elements.push(TemplateElement::Expr(
                        Expression::<true, YIELD, AWAIT>.parse(cursor)?,
                    ));
                    cursor.expect(
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                        "template literal",
                    )?;
                }
                TokenKind::TemplateNoSubstitution(template_string) => {
                    let cooked = template_string.to_owned_cooked().map_err(ParseError::lex)?;

                    elements.push(TemplateElement::String(cooked));
                    return Ok(TemplateLit::new(elements));
                }
                _ => {
                    return Err(ParseError::general(
                        "cannot parse template literal",
                        self.start,
                    ))
                }
            }
        }
    }
}
