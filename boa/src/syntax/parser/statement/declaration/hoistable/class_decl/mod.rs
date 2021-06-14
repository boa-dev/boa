#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::FunctionDecl, Keyword, Punctuator},
    parser::{
        function::FunctionBody, statement::BindingIdentifier, AllowAwait, AllowDefault, AllowYield,
        Cursor, ParseError, TokenParser,
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
    type Output = FunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Class, "class declaration")?;

        // TODO: If self.is_default, then this can be empty.
        let name = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.expect(Punctuator::OpenBlock, "class declaration")?;

        let body = FunctionBody::new(self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "class declaration")?;

        // It is a Syntax Error if any element of the BoundNames of FormalParameters
        // also occurs in the LexicallyDeclaredNames of FunctionBody.
        // https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors
        {
            // let lexically_declared_names = body.lexically_declared_names();
            // for param in fields.as_ref() {
            //     if lexically_declared_names.contains(param.name()) {
            //         return Err(ParseError::lex(LexError::Syntax(
            //             format!("Redeclaration of formal parameter `{}`", param.name()).into(),
            //             match cursor.peek(0)? {
            //                 Some(token) => token.span().end(),
            //                 None => Position::new(1, 1),
            //             },
            //         )));
            //     }
            // }
        }

        Ok(FunctionDecl::new(name, vec![], body))
    }
}
