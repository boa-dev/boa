use crate::{
    lexer::TokenKind,
    parser::{
        expression::BindingIdentifier, statement::ClassTail, AllowAwait, AllowYield, Cursor,
        OrAbrupt, ParseResult, TokenParser,
    },
};
use boa_ast::{expression::Identifier, function::Class, Keyword};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Class expression parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct ClassExpression {
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassExpression {
    /// Creates a new `ClassExpression` parser.
    pub(in crate::parser) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ClassExpression
where
    R: Read,
{
    type Output = Class;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("ClassExpression", "Parsing");
        let strict = cursor.strict();
        cursor.set_strict(true);

        let mut has_binding_identifier = false;
        let token = cursor.peek(0, interner).or_abrupt()?;
        let name = match token.kind() {
            TokenKind::IdentifierName(_)
            | TokenKind::Keyword((Keyword::Yield | Keyword::Await, _)) => {
                has_binding_identifier = true;
                BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?
                    .into()
            }
            _ => self.name,
        };
        cursor.set_strict(strict);

        ClassTail::new(
            name,
            has_binding_identifier,
            self.allow_yield,
            self.allow_await,
        )
        .parse(cursor, interner)
    }
}
