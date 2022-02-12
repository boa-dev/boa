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
            Position, Punctuator,
        },
        lexer::TokenKind,
        parser::{expression::Expression, AllowAwait, AllowYield, Cursor, ParseError, TokenParser},
    },
};
use boa_interner::{Interner, Sym};
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
    first: Sym,
}

impl TemplateLiteral {
    /// Creates a new `TemplateLiteral` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A, start: Position, first: Sym) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            start,
            first,
        }
    }
}

impl<R> TokenParser<R> for TemplateLiteral
where
    R: Read,
{
    type Output = TemplateLit;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("TemplateLiteral", "Parsing");

        let mut elements = vec![
            TemplateElement::String(self.first),
            TemplateElement::Expr(
                Expression::new(true, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?,
            ),
        ];
        cursor.expect(
            TokenKind::Punctuator(Punctuator::CloseBlock),
            "template literal",
            interner,
        )?;

        loop {
            match cursor.lex_template(self.start, interner)?.kind() {
                TokenKind::TemplateMiddle(template_string) => {
                    let cooked = template_string
                        .to_owned_cooked(interner)
                        .map_err(ParseError::lex)?;

                    elements.push(TemplateElement::String(cooked));
                    elements.push(TemplateElement::Expr(
                        Expression::new(true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    ));
                    cursor.expect(
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                        "template literal",
                        interner,
                    )?;
                }
                TokenKind::TemplateNoSubstitution(template_string) => {
                    let cooked = template_string
                        .to_owned_cooked(interner)
                        .map_err(ParseError::lex)?;

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
