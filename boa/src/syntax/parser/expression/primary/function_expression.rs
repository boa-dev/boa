//! Function expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
//! [spec]: https://tc39.es/ecma262/#prod-FunctionExpression

use crate::{
    syntax::{
        ast::{node::FunctionExpr, Punctuator},
        parser::{
            function::{FormalParameters, FunctionBody},
            statement::BindingIdentifier,
            Parser, ParseError, TokenParser,
        },
    },
    BoaProfiler,
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

impl<R> TokenParser<R> for FunctionExpression {
    type Output = FunctionExpr;

    fn parse(self, parser: &mut Parser<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FunctionExpression", "Parsing");
        let name = BindingIdentifier::new(false, false).try_parse(parser);

        parser.expect(Punctuator::OpenParen, "function expression")?;

        let params = FormalParameters::new(false, false).parse(parser)?;

        parser.expect(Punctuator::CloseParen, "function expression")?;
        parser.expect(Punctuator::OpenBlock, "function expression")?;

        let body = FunctionBody::new(false, false).parse(parser)?;

        parser.expect(Punctuator::CloseBlock, "function expression")?;

        Ok(FunctionExpr::new(name, params, body))
    }
}
