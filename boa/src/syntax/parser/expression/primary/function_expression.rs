//! Function expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
//! [spec]: https://tc39.es/ecma262/#prod-FunctionExpression

use crate::syntax::{
    ast::{node::Node, punc::Punctuator, token::TokenKind},
    parser::{
        function::{FormalParameters, FunctionBody},
        Cursor, ParseError, ParseResult, TokenParser,
    },
};

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

impl TokenParser for FunctionExpression {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let name = if let TokenKind::Identifier(name) =
            &cursor.peek(0).ok_or(ParseError::AbruptEnd)?.kind
        {
            Some(name)
        } else {
            None
        };
        if name.is_some() {
            // We move the cursor forward.
            let _ = cursor.next().expect("nex token disappeared");
        }

        cursor.expect(Punctuator::OpenParen, "function expression")?;

        let params = FormalParameters::new(false, false).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "function expression")?;
        cursor.expect(Punctuator::OpenBlock, "function expression")?;

        let body = FunctionBody::new(false, false)
            .parse(cursor)
            .map(Node::StatementList)?;

        cursor.expect(Punctuator::CloseBlock, "function expression")?;

        Ok(Node::function_decl::<_, &String, _, _>(name, params, body))
    }
}
