use crate::{Error, error::ParseResult};
use boa_ast::{self as ast, Position, function::FormalParameterList};

pub(crate) enum FormalParameterListOrExpression {
    FormalParameterList {
        fpl: FormalParameterList,
        span_start: Position,
    },
    AsyncArrowHead {
        fpl: FormalParameterList,
        params_start_position: Position,
    },
    Expression(ast::Expression),
}

impl FormalParameterListOrExpression {
    pub(crate) fn try_into_expression(self) -> ParseResult<ast::Expression> {
        match self {
            FormalParameterListOrExpression::Expression(expr) => Ok(expr),
            FormalParameterListOrExpression::FormalParameterList { span_start, .. } => {
                Err(Error::General {
                    message: "invalid arrow-function arguments (parentheses around the arrow-function may help)".into(),
                    position: span_start,
                })
            }
            FormalParameterListOrExpression::AsyncArrowHead {
                params_start_position,
                ..
            } => Err(Error::General {
                message: "invalid arrow-function arguments (parentheses around the arrow-function may help)".into(),
                position: params_start_position,
            }),
        }
    }
}

impl<T> From<T> for FormalParameterListOrExpression
where
    T: Into<ast::Expression>,
{
    fn from(value: T) -> Self {
        Self::Expression(value.into())
    }
}
