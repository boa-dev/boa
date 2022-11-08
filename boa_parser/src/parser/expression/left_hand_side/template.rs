use crate::{
    lexer::TokenKind,
    parser::{
        cursor::Cursor, expression::Expression, AllowAwait, AllowYield, OrAbrupt, ParseResult,
        TokenParser,
    },
    Error,
};
use boa_ast::{self as ast, expression::TaggedTemplate, Position, Punctuator};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Parses a tagged template.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-TemplateLiteral
#[derive(Debug, Clone)]
pub(super) struct TaggedTemplateLiteral {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    start: Position,
    tag: ast::Expression,
}

impl TaggedTemplateLiteral {
    /// Creates a new `TaggedTemplateLiteral` parser.
    pub(super) fn new<Y, A>(
        allow_yield: Y,
        allow_await: A,
        start: Position,
        tag: ast::Expression,
    ) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            start,
            tag,
        }
    }
}

impl<R> TokenParser<R> for TaggedTemplateLiteral
where
    R: Read,
{
    type Output = TaggedTemplate;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("TaggedTemplateLiteral", "Parsing");

        let mut raws = Vec::new();
        let mut cookeds = Vec::new();
        let mut exprs = Vec::new();

        let mut token = cursor.next(interner).or_abrupt()?;

        loop {
            match token.kind() {
                TokenKind::TemplateMiddle(template_string) => {
                    raws.push(template_string.as_raw());
                    cookeds.push(template_string.to_owned_cooked(interner).ok());
                    exprs.push(
                        Expression::new(None, true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    );
                    cursor.expect(
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                        "template literal",
                        interner,
                    )?;
                }
                TokenKind::TemplateNoSubstitution(template_string) => {
                    raws.push(template_string.as_raw());
                    cookeds.push(template_string.to_owned_cooked(interner).ok());
                    return Ok(TaggedTemplate::new(
                        self.tag,
                        raws.into_boxed_slice(),
                        cookeds.into_boxed_slice(),
                        exprs.into_boxed_slice(),
                    ));
                }
                _ => {
                    return Err(Error::general(
                        "cannot parse tagged template literal",
                        self.start,
                    ))
                }
            }
            token = cursor.lex_template(self.start, interner)?;
        }
    }
}
