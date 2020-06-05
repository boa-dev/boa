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

use self::assignment::ExponentiationExpression;
pub(super) use self::{assignment::AssignmentExpression, primary::Initializer};
use super::{AllowAwait, AllowIn, AllowYield, Cursor, ParseResult, TokenParser};
use crate::{
    profiler::BoaProfiler,
    syntax::ast::{
        node::{BinOp, Node},
        Keyword, Punctuator, TokenKind,
    },
};

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

/// Generates an expression parser.
///
/// This macro has 2 mandatory identifiers:
///  - The `$name` identifier will contain the name of the parsing structure.
///  - The `$lower` identifier will contain the parser for lower level expressions.
///
/// Those exressions are divided by the punctuators passed as the third parameter.
macro_rules! expression { ($name:ident, $lower:ident, [$( $op:path ),*], [$( $low_param:ident ),*] ) => {
    impl TokenParser for $name {
        type Output = Node;

        fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
            let _timer = BoaProfiler::global().start_event("Expression", "Parsing");
            let mut lhs = $lower::new($( self.$low_param ),*).parse(cursor)?;
            while let Some(tok) = cursor.peek(0) {
                match tok.kind {
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
    [allow_in, allow_yield, allow_await]
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
    [allow_in, allow_yield, allow_await]
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
    [allow_in, allow_yield, allow_await]
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
    [allow_in, allow_yield, allow_await]
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
    [allow_in, allow_yield, allow_await]
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
    [allow_in, allow_yield, allow_await]
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
    [allow_in, allow_yield, allow_await]
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

expression!(
    RelationalExpression,
    ShiftExpression,
    [
        Punctuator::LessThan,
        Punctuator::GreaterThan,
        Punctuator::LessThanOrEq,
        Punctuator::GreaterThanOrEq,
        Keyword::In
    ],
    [allow_yield, allow_await]
);

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
    [allow_yield, allow_await]
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
    [allow_yield, allow_await]
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
    [allow_yield, allow_await]
);
