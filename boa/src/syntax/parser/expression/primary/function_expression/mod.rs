//! Function expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
//! [spec]: https://tc39.es/ecma262/#prod-FunctionExpression

#[cfg(test)]
mod tests;

use crate::{
    syntax::{
        ast::{node::FunctionExpr, Keyword, Punctuator},
        lexer::{Error as LexError, Position, TokenKind},
        parser::{
            function::{FormalParameters, FunctionBody},
            statement::BindingIdentifier,
            Cursor, DeclaredNames, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// Function expression parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
/// [spec]: https://tc39.es/ecma262/#prod-FunctionExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct FunctionExpression;

impl<R> TokenParser<R> for FunctionExpression
where
    R: Read,
{
    type Output = FunctionExpr;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        env: &mut DeclaredNames,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FunctionExpression", "Parsing");

        let name = if let Some(token) = cursor.peek(0)? {
            match token.kind() {
                TokenKind::Identifier(_)
                | TokenKind::Keyword(Keyword::Yield)
                | TokenKind::Keyword(Keyword::Await) => {
                    Some(BindingIdentifier::new(false, false).parse(cursor, env)?)
                }
                _ => None,
            }
        } else {
            None
        };

        cursor.expect(Punctuator::OpenParen, "function expression")?;

        let params = FormalParameters::new(false, false).parse(cursor, env)?;

        cursor.expect(Punctuator::CloseParen, "function expression")?;
        cursor.expect(Punctuator::OpenBlock, "function expression")?;

        let body = FunctionBody::new(false, false).parse(cursor, env)?;

        cursor.expect(Punctuator::CloseBlock, "function expression")?;

        // It is a Syntax Error if any element of the BoundNames of FormalParameters
        // also occurs in the LexicallyDeclaredNames of FunctionBody.
        // https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors
        {
            let lexically_declared_names = body.lexically_declared_names();
            for param in params.as_ref() {
                if lexically_declared_names.contains(param.name()) {
                    return Err(ParseError::lex(LexError::Syntax(
                        format!("Redeclaration of formal parameter `{}`", param.name()).into(),
                        match cursor.peek(0)? {
                            Some(token) => token.span().end(),
                            None => Position::new(1, 1),
                        },
                    )));
                }
            }
        }

        Ok(FunctionExpr::new(name, params, body))
    }
}
