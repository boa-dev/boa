use crate::{
    lexer::TokenKind,
    parser::{
        expression::BindingIdentifier, statement::ClassTail, AllowAwait, AllowYield, Cursor,
        OrAbrupt, ParseResult, TokenParser,
    },
    source::ReadChar,
};
use boa_ast::{function::ClassExpression as ClassExpressionNode, Keyword};
use boa_interner::Interner;
use boa_profiler::Profiler;

/// Class expression parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct ClassExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassExpression {
    /// Creates a new `ClassExpression` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ClassExpression
where
    R: ReadChar,
{
    type Output = ClassExpressionNode;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("ClassExpression", "Parsing");
        let strict = cursor.strict();
        cursor.set_strict(true);

        let token = cursor.peek(0, interner).or_abrupt()?;
        let name = match token.kind() {
            TokenKind::IdentifierName(_)
            | TokenKind::Keyword((Keyword::Yield | Keyword::Await, _)) => {
                BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?
                    .into()
            }
            _ => None,
        };
        cursor.set_strict(strict);

        let (super_ref, constructor, elements) =
            ClassTail::new(name, self.allow_yield, self.allow_await).parse(cursor, interner)?;

        Ok(ClassExpressionNode::new(
            name,
            super_ref,
            constructor,
            elements.into_boxed_slice(),
            name.is_some(),
        ))
    }
}
