use crate::syntax::{
    ast::{Keyword, Node},
    lexer::TokenKind,
    parser::{
        expression::BindingIdentifier, statement::ClassTail, AllowAwait, AllowYield, Cursor,
        ParseError, TokenParser,
    },
};
use boa_interner::{Interner, Sym};
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
    name: Option<Sym>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassExpression {
    /// Creates a new `ClassExpression` parser.
    pub(in crate::syntax::parser) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Sym>>,
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
    type Output = Node;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("ClassExpression", "Parsing");
        let strict = cursor.strict_mode();
        cursor.set_strict_mode(true);

        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        let name = match token.kind() {
            TokenKind::Identifier(_) | TokenKind::Keyword((Keyword::Yield | Keyword::Await, _)) => {
                BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?
            }
            _ => {
                if let Some(name) = self.name {
                    name
                } else {
                    return Err(ParseError::unexpected(
                        token.to_string(interner),
                        token.span(),
                        "expected class identifier",
                    ));
                }
            }
        };
        cursor.set_strict_mode(strict);

        Ok(Node::ClassExpr(
            ClassTail::new(name, self.allow_yield, self.allow_await).parse(cursor, interner)?,
        ))
    }
}
