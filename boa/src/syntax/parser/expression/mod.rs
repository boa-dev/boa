//! Expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators
//! [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-expressions

mod assignment;
mod left_hand_side;
mod primary;
#[cfg(test)]
mod tests;
mod unary;
mod update;

pub(in crate::syntax::parser) mod await_expr;

use self::assignment::ExponentiationExpression;
pub(super) use self::{assignment::AssignmentExpression, primary::Initializer};
use super::{AllowAwait, AllowIn, AllowYield, Cursor, ParseResult, TokenParser};
use crate::syntax::lexer::{InputElement, TokenKind};
use crate::{
    profiler::BoaProfiler,
    syntax::ast::{
        node::{BinOp, Node},
        Keyword, Punctuator,
    },
};

use std::io::Read;

// For use in the expression! macro to allow for both Punctuator and Keyword parameters.
// Always returns false.
impl PartialEq<Keyword> for Punctuator {
    fn eq(&self, _other: &Keyword) -> bool {
        false
    }
}

// For use in the expression! macro to allow for both Punctuator and Keyword parameters.
// Always returns false.
impl PartialEq<Punctuator> for Keyword {
    fn eq(&self, _other: &Punctuator) -> bool {
        false
    }
}

/// Generates an expression parser for a number of expressions whose production rules are of the following pattern.
/// <TargetExpression>[allowed_identifiers]
///     => <InnerExpression>[?allowed_identifiers]
///     => <TargetExpression>[?allowed_identifiers] <op1> <InnerExpression>[?allowed_identifiers]
///     => <TargetExpression>[?allowed_identifiers] <op2> <InnerExpression>[?allowed_identifiers]
///     ...
///
/// This macro has 2 mandatory identifiers:
///  - The `$name` identifier is the name of the TargetExpression struct that the parser will be implemented for.
///  - The `$lower` identifier is the name of the InnerExpression struct according to the pattern above.
///
/// A list of punctuators (operands between the <TargetExpression> and <InnerExpression>) are passed as the third parameter.
///
/// The fifth parameter is an Option<InputElement> which sets the goal symbol to set before parsing (or None to leave it as is).
macro_rules! expression { ($name:ident, $lower:ident, [$( $op:path ),*], [$( $low_param:ident ),*], $goal:expr ) => {
    impl<R> TokenParser<R> for $name
    where
        R: Read
    {
        type Output = Node;

        fn parse(self, cursor: &mut Cursor<R>)-> ParseResult {
            let _timer = BoaProfiler::global().start_event(stringify!($name), "Parsing");

            if $goal.is_some() {
                cursor.set_goal($goal.unwrap());
            }

            let mut lhs = $lower::new($( self.$low_param ),*).parse(cursor)?;
            while let Some(tok) = cursor.peek(0)? {
                match *tok.kind() {
                    TokenKind::Punctuator(op) if $( op == $op )||* => {
                        let _ = cursor.next().expect("token disappeared");
                        lhs = BinOp::new(
                            op.as_binop().expect("Could not get binary operation."),
                            lhs,
                            $lower::new($( self.$low_param ),*).parse(cursor)?
                        ).into();
                    }
                    TokenKind::Keyword(op) if $( op == $op )||* => {
                        let _ = cursor.next().expect("token disappeared");
                        lhs = BinOp::new(
                            op.as_binop().expect("Could not get binary operation."),
                            lhs,
                            $lower::new($( self.$low_param ),*).parse(cursor)?
                        ).into();
                    }
                    _ => break
                }
            }

            Ok(lhs)
        }
    }
} }

/// Expression parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators
/// [spec]: https://tc39.es/ecma262/#prod-Expression
#[derive(Debug, Clone, Copy)]
pub(super) struct Expression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl Expression {
    /// Creates a new `Expression` parser.
    pub(super) fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

expression!(
    Expression,
    AssignmentExpression,
    [Punctuator::Comma],
    [allow_in, allow_yield, allow_await],
    None::<InputElement>
);

/// Parses a logical `OR` expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_Operators#Logical_OR_2
/// [spec]: https://tc39.es/ecma262/#prod-LogicalORExpression
#[derive(Debug, Clone, Copy)]
struct LogicalORExpression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl LogicalORExpression {
    /// Creates a new `LogicalORExpression` parser.
    pub(super) fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

expression!(
    LogicalORExpression,
    LogicalANDExpression,
    [Punctuator::BoolOr],
    [allow_in, allow_yield, allow_await],
    None::<InputElement>
);

/// Parses a logical `AND` expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_Operators#Logical_AND_2
/// [spec]: https://tc39.es/ecma262/#prod-LogicalANDExpression
#[derive(Debug, Clone, Copy)]
struct LogicalANDExpression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl LogicalANDExpression {
    /// Creates a new `LogicalANDExpression` parser.
    pub(super) fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

expression!(
    LogicalANDExpression,
    BitwiseORExpression,
    [Punctuator::BoolAnd],
    [allow_in, allow_yield, allow_await],
    None::<InputElement>
);

/// Parses a bitwise `OR` expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_OR
/// [spec]: https://tc39.es/ecma262/#prod-BitwiseORExpression
#[derive(Debug, Clone, Copy)]
struct BitwiseORExpression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BitwiseORExpression {
    /// Creates a new `BitwiseORExpression` parser.
    pub(super) fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

expression!(
    BitwiseORExpression,
    BitwiseXORExpression,
    [Punctuator::Or],
    [allow_in, allow_yield, allow_await],
    None::<InputElement>
);

/// Parses a bitwise `XOR` expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_XOR
/// [spec]: https://tc39.es/ecma262/#prod-BitwiseXORExpression
#[derive(Debug, Clone, Copy)]
struct BitwiseXORExpression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BitwiseXORExpression {
    /// Creates a new `BitwiseXORExpression` parser.
    pub(super) fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

expression!(
    BitwiseXORExpression,
    BitwiseANDExpression,
    [Punctuator::Xor],
    [allow_in, allow_yield, allow_await],
    None::<InputElement>
);

/// Parses a bitwise `AND` expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_AND
/// [spec]: https://tc39.es/ecma262/#prod-BitwiseANDExpression
#[derive(Debug, Clone, Copy)]
struct BitwiseANDExpression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BitwiseANDExpression {
    /// Creates a new `BitwiseANDExpression` parser.
    pub(super) fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

expression!(
    BitwiseANDExpression,
    EqualityExpression,
    [Punctuator::And],
    [allow_in, allow_yield, allow_await],
    None::<InputElement>
);

/// Parses an equality expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Equality_operators
/// [spec]: https://tc39.es/ecma262/#sec-equality-operators
#[derive(Debug, Clone, Copy)]
struct EqualityExpression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl EqualityExpression {
    /// Creates a new `EqualityExpression` parser.
    pub(super) fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

expression!(
    EqualityExpression,
    RelationalExpression,
    [
        Punctuator::Eq,
        Punctuator::NotEq,
        Punctuator::StrictEq,
        Punctuator::StrictNotEq
    ],
    [allow_in, allow_yield, allow_await],
    None::<InputElement>
);

/// Parses a relational expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Relational_operators
/// [spec]: https://tc39.es/ecma262/#sec-relational-operators
#[derive(Debug, Clone, Copy)]
struct RelationalExpression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl RelationalExpression {
    /// Creates a new `RelationalExpression` parser.
    pub(super) fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for RelationalExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("Relation Expression", "Parsing");

        if None::<InputElement>.is_some() {
            cursor.set_goal(None::<InputElement>.unwrap());
        }

        let mut lhs = ShiftExpression::new(self.allow_yield, self.allow_await).parse(cursor)?;
        while let Some(tok) = cursor.peek(0)? {
            match *tok.kind() {
                TokenKind::Punctuator(op)
                    if op == Punctuator::LessThan
                        || op == Punctuator::GreaterThan
                        || op == Punctuator::LessThanOrEq
                        || op == Punctuator::GreaterThanOrEq =>
                {
                    let _ = cursor.next().expect("token disappeared");
                    lhs = BinOp::new(
                        op.as_binop().expect("Could not get binary operation."),
                        lhs,
                        ShiftExpression::new(self.allow_yield, self.allow_await).parse(cursor)?,
                    )
                    .into();
                }
                TokenKind::Keyword(op)
                    if op == Keyword::InstanceOf
                        || (op == Keyword::In && self.allow_in == AllowIn(true)) =>
                {
                    let _ = cursor.next().expect("token disappeared");
                    lhs = BinOp::new(
                        op.as_binop().expect("Could not get binary operation."),
                        lhs,
                        ShiftExpression::new(self.allow_yield, self.allow_await).parse(cursor)?,
                    )
                    .into();
                }
                _ => break,
            }
        }

        Ok(lhs)
    }
}

/// Parses a bitwise shift expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_shift_operators
/// [spec]: https://tc39.es/ecma262/#sec-bitwise-shift-operators
#[derive(Debug, Clone, Copy)]
struct ShiftExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ShiftExpression {
    /// Creates a new `ShiftExpression` parser.
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

expression!(
    ShiftExpression,
    AdditiveExpression,
    [
        Punctuator::LeftSh,
        Punctuator::RightSh,
        Punctuator::URightSh
    ],
    [allow_yield, allow_await],
    None::<InputElement>
);

/// Parses an additive expression.
///
/// This can be either an addition or a subtraction.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators
/// [spec]: https://tc39.es/ecma262/#sec-additive-operators
#[derive(Debug, Clone, Copy)]
struct AdditiveExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AdditiveExpression {
    /// Creates a new `AdditiveExpression` parser.
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

expression!(
    AdditiveExpression,
    MultiplicativeExpression,
    [Punctuator::Add, Punctuator::Sub],
    [allow_yield, allow_await],
    None::<InputElement>
);

/// Parses a multiplicative expression.
///
/// This can be either a multiplication, division or a modulo (remainder) expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Division
/// [spec]: https://tc39.es/ecma262/#sec-multiplicative-operators
#[derive(Debug, Clone, Copy)]
struct MultiplicativeExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl MultiplicativeExpression {
    /// Creates a new `MultiplicativeExpression` parser.
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

expression!(
    MultiplicativeExpression,
    ExponentiationExpression,
    [Punctuator::Mul, Punctuator::Div, Punctuator::Mod],
    [allow_yield, allow_await],
    Some(InputElement::Div)
);
