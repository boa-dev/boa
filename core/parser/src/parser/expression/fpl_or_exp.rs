use crate::{Error, error::ParseResult};
use boa_ast::{self as ast, Position, function::FormalParameterList};

pub(crate) enum FormalParameterListOrExpression<'arena> {
    FormalParameterList {
        fpl: FormalParameterList<'arena>,
        span_start: Position,
    },
    Expression(ast::Expression<'arena>),
}

impl<'arena> FormalParameterListOrExpression<'arena> {
    pub(crate) fn expect_expression(self) -> ast::Expression<'arena> {
        match self {
            FormalParameterListOrExpression::Expression(expr) => expr,
            FormalParameterListOrExpression::FormalParameterList { .. } => {
                panic!("Unexpected arrow-function arguments");
            }
        }
    }

    pub(crate) fn try_into_expression(self) -> ParseResult<ast::Expression<'arena>> {
        match self {
            FormalParameterListOrExpression::Expression(expr) => Ok(expr),
            FormalParameterListOrExpression::FormalParameterList { span_start, .. } => {
                Err(Error::General {
                    message: "invalid arrow-function arguments (parentheses around the arrow-function may help)".into(),
                    position: span_start,
                })
            }
        }
    }
}

impl<'arena, T> From<T> for FormalParameterListOrExpression<'arena>
where
    T: Into<ast::Expression<'arena>>,
{
    fn from(value: T) -> Self {
        Self::Expression(value.into())
    }
}
