// Tests are in parser/class/tests.rs

use crate::syntax::{
    ast::{node::ClassDecl, Keyword, Punctuator},
    parser::{
        class::ClassElementList, statement::BindingIdentifier, AllowAwait, AllowDefault,
        AllowYield, Cursor, ParseError, TokenParser,
    },
};
use std::io::Read;

/// Class declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
/// [spec]: https://tc39.es/ecma262/#sec-class-definitions

#[derive(Debug, Clone, Copy)]
pub(super) struct ClassDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl ClassDeclaration {
    /// Creates a new `FunctionDeclaration` parser.
    pub(super) fn new<Y, A, D>(allow_yield: Y, allow_await: A, is_default: D) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        D: Into<AllowDefault>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            is_default: is_default.into(),
        }
    }
}

impl<R> TokenParser<R> for ClassDeclaration
where
    R: Read,
{
    type Output = ClassDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Class, "class declaration")?;

        // TODO: If self.is_default, then this can be empty.
        let pos = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();
        let name = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
        if *name == *"yield" {
            return Err(ParseError::general("Invalid class name `yield`", pos));
        } else if *name == *"static" {
            return Err(ParseError::general("Invalid class name `static`", pos));
        }

        cursor.expect(Punctuator::OpenBlock, "class declaration")?;

        let (constructor, fields, static_fields) =
            ClassElementList::new(false, false).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "class declaration")?;

        Ok(ClassDecl::new(name, constructor, fields, static_fields))
    }
}
