use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::node::TaggedTemplate,
        ast::{Node, Position, Punctuator},
        lexer::TokenKind,
        parser::{
            cursor::Cursor, expression::Expression, AllowAwait, AllowYield, ParseError,
            ParseResult, TokenParser,
        },
    },
    Interner,
};
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
    tag: Node,
}

impl TaggedTemplateLiteral {
    /// Creates a new `TaggedTemplateLiteral` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A, start: Position, tag: Node) -> Self
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
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("TaggedTemplateLiteral", "Parsing");

        let mut raws = Vec::new();
        let mut cookeds = Vec::new();
        let mut exprs = Vec::new();

        let mut token = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;

        loop {
            match token.kind() {
                TokenKind::TemplateMiddle(template_string) => {
                    raws.push(
                        interner
                            .resolve(template_string.as_raw())
                            .expect("string disappeared")
                            .to_owned()
                            .into_boxed_str(),
                    );
                    cookeds.push(template_string.to_owned_cooked(interner).ok());
                    exprs.push(
                        Expression::new(true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    );
                    cursor.expect(
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                        "template literal",
                        interner,
                    )?;
                }
                TokenKind::TemplateNoSubstitution(template_string) => {
                    raws.push(
                        interner
                            .resolve(template_string.as_raw())
                            .expect("string disappeared")
                            .to_owned()
                            .into_boxed_str(),
                    );
                    cookeds.push(template_string.to_owned_cooked(interner).ok());
                    return Ok(Node::from(TaggedTemplate::new(
                        self.tag, raws, cookeds, exprs,
                    )));
                }
                _ => {
                    return Err(ParseError::general(
                        "cannot parse tagged template literal",
                        self.start,
                    ))
                }
            }
            token = cursor.lex_template(self.start, interner)?;
        }
    }
}
