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
        ast::{
            node::template::{TemplateElement, TemplateLit},
            Position, Punctuator, Span,
        },
        lexer::TokenKind,
        parser::{expression::Expression, AllowAwait, AllowYield, Cursor, ParseError, TokenParser},
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
pub(super) struct TemplateLiteral {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    start: Position,
    first: String,
}

impl TemplateLiteral {
    /// Creates a new `TemplateLiteral` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A, start: Position, first: &str) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            start,
            first: first.to_owned(),
        }
    }
}

impl<R> TokenParser<R> for TemplateLiteral
where
    R: Read,
{
    type Output = (TemplateLit, Span);

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("TemplateLiteral", "Parsing");

        let mut elements = vec![
            TemplateElement::String(self.first.into_boxed_str()),
            TemplateElement::Expr(
                Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?,
            ),
        ];
        cursor.expect(
            TokenKind::Punctuator(Punctuator::CloseBlock),
            "template literal",
        )?;

        let span_end = loop {
            let tok = cursor.lex_template(self.start)?;
            match tok.kind() {
                TokenKind::TemplateMiddle(template_string) => {
                    let cooked = template_string.to_owned_cooked().map_err(ParseError::lex)?;

                    elements.push(TemplateElement::String(cooked));
                    elements.push(TemplateElement::Expr(
                        Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?,
                    ));
                    cursor.expect(
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                        "template literal",
                    )?;
                }
                TokenKind::TemplateNoSubstitution(template_string) => {
                    let cooked = template_string.to_owned_cooked().map_err(ParseError::lex)?;

                    elements.push(TemplateElement::String(cooked));
                    break tok.span().end();
                }
                _ => {
                    return Err(ParseError::general(
                        "cannot parse template literal",
                        self.start,
                    ))
                }
            }
        };

        Ok((TemplateLit::new(elements), Span::new(self.start, span_end)))
    }
}
