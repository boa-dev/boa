use crate::{
    syntax::{
        ast::{
            node::{self, Identifier},
            Keyword, Punctuator,
        },
        parser::{
            statement::{block::Block, BindingIdentifier},
            AllowAwait, AllowReturn, AllowYield, Cursor, DeclaredNames, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// Catch parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#prod-Catch
#[derive(Debug, Clone, Copy)]
pub(super) struct Catch {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl Catch {
    /// Creates a new `Catch` block parser.
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

impl<R> TokenParser<R> for Catch
where
    R: Read,
{
    type Output = node::Catch;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        env: &mut DeclaredNames,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Catch", "Parsing");
        cursor.expect(Keyword::Catch, "try statement")?;
        let catch_param = if cursor.next_if(Punctuator::OpenParen)?.is_some() {
            let catch_param =
                CatchParameter::new(self.allow_yield, self.allow_await).parse(cursor, env)?;
            cursor.expect(Punctuator::CloseParen, "catch in try statement")?;
            Some(catch_param)
        } else {
            None
        };

        // Catch block
        Ok(node::Catch::new::<_, Identifier, _>(
            catch_param,
            Block::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor, env)?,
        ))
    }
}

/// CatchParameter parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#prod-CatchParameter
#[derive(Debug, Clone, Copy)]
pub(super) struct CatchParameter {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl CatchParameter {
    /// Creates a new `CatchParameter` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for CatchParameter
where
    R: Read,
{
    type Output = Identifier;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        env: &mut DeclaredNames,
    ) -> Result<Identifier, ParseError> {
        // TODO: should accept BindingPattern
        BindingIdentifier::new(self.allow_yield, self.allow_await)
            .parse(cursor, env)
            .map(Identifier::from)
    }
}
