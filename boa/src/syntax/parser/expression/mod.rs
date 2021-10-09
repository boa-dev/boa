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
use super::{Cursor, ParseResult, TokenParser};

use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::op::LogOp,
        ast::{
            node::{BinOp, Node},
            Keyword, Punctuator,
        },
        lexer::{InputElement, TokenKind},
        parser::ParseError,
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
    impl<R, $( const $low_param: bool ),*> TokenParser<R> for $name< $( $low_param ),* >
    where
        R: Read
    {
        type Output = Node;

        fn parse(self, cursor: &mut Cursor<R>)-> ParseResult {
            let _timer = BoaProfiler::global().start_event(stringify!($name), "Parsing");

            if $goal.is_some() {
                cursor.set_goal($goal.unwrap());
            }

            let mut lhs = $lower::<$( $low_param ),*>.parse(cursor)?;
            while let Some(tok) = cursor.peek(0)? {
                match *tok.kind() {
                    TokenKind::Punctuator(op) if $( op == $op )||* => {
                        let _ = cursor.next().expect("token disappeared");
                        lhs = BinOp::new(
                            op.as_binop().expect("Could not get binary operation."),
                            lhs,
                            $lower::<$( $low_param ),*>.parse(cursor)?
                        ).into();
                    }
                    TokenKind::Keyword(op) if $( op == $op )||* => {
                        let _ = cursor.next().expect("token disappeared");
                        lhs = BinOp::new(
                            op.as_binop().expect("Could not get binary operation."),
                            lhs,
                            $lower::<$( $low_param ),*>.parse(cursor)?
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
pub(super) struct Expression<const IN: bool, const YIELD: bool, const AWAIT: bool>;

expression!(
    Expression,
    AssignmentExpression,
    [Punctuator::Comma],
    [IN, YIELD, AWAIT],
    None::<InputElement>
);

/// Parses a logical expression expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_Operators
/// [spec]: https://tc39.es/ecma262/#prod-ShortCircuitExpression
#[derive(Debug, Clone, Copy)]
struct ShortCircuitExpression<const IN: bool, const YIELD: bool, const AWAIT: bool> {
    previous: PreviousExpr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PreviousExpr {
    None,
    Logical,
    Coalesce,
}

impl<const IN: bool, const YIELD: bool, const AWAIT: bool>
    ShortCircuitExpression<IN, YIELD, AWAIT>
{
    /// Creates a new `ShortCircuitExpression` parser.
    pub(super) fn new() -> Self {
        Self {
            previous: PreviousExpr::None,
        }
    }

    fn with_previous(previous: PreviousExpr) -> Self {
        Self { previous }
    }
}

impl<R, const IN: bool, const YIELD: bool, const AWAIT: bool> TokenParser<R>
    for ShortCircuitExpression<IN, YIELD, AWAIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("ShortCircuitExpression", "Parsing");

        let mut current_node = BitwiseORExpression::<IN, YIELD, AWAIT>.parse(cursor)?;
        let mut previous = self.previous;

        while let Some(tok) = cursor.peek(0)? {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::BoolAnd) => {
                    if previous == PreviousExpr::Coalesce {
                        return Err(ParseError::expected(
                            [TokenKind::Punctuator(Punctuator::Coalesce)],
                            tok.clone(),
                            "logical expression (cannot use '??' without parentheses within '||' or '&&')",
                        ));
                    }
                    let _ = cursor.next()?.expect("'&&' expected");
                    previous = PreviousExpr::Logical;
                    let rhs = BitwiseORExpression::<IN, YIELD, AWAIT>.parse(cursor)?;

                    current_node = BinOp::new(LogOp::And, current_node, rhs).into();
                }
                TokenKind::Punctuator(Punctuator::BoolOr) => {
                    if previous == PreviousExpr::Coalesce {
                        return Err(ParseError::expected(
                            [TokenKind::Punctuator(Punctuator::Coalesce)],
                            tok.clone(),
                            "logical expression (cannot use '??' without parentheses within '||' or '&&')",
                        ));
                    }
                    let _ = cursor.next()?.expect("'||' expected");
                    previous = PreviousExpr::Logical;
                    let rhs = ShortCircuitExpression::<IN, YIELD, AWAIT>::with_previous(
                        PreviousExpr::Logical,
                    )
                    .parse(cursor)?;
                    current_node = BinOp::new(LogOp::Or, current_node, rhs).into();
                }
                TokenKind::Punctuator(Punctuator::Coalesce) => {
                    if previous == PreviousExpr::Logical {
                        return Err(ParseError::expected(
                            [
                                TokenKind::Punctuator(Punctuator::BoolAnd),
                                TokenKind::Punctuator(Punctuator::BoolOr),
                            ],
                            tok.clone(),
                            "cannot use '??' unparenthesized within '||' or '&&'",
                        ));
                    }
                    let _ = cursor.next()?.expect("'??' expected");
                    previous = PreviousExpr::Coalesce;
                    let rhs = BitwiseORExpression::<IN, YIELD, AWAIT>.parse(cursor)?;
                    current_node = BinOp::new(LogOp::Coalesce, current_node, rhs).into();
                }
                _ => break,
            }
        }
        Ok(current_node)
    }
}

/// Parses a bitwise `OR` expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_OR
/// [spec]: https://tc39.es/ecma262/#prod-BitwiseORExpression
#[derive(Debug, Clone, Copy)]
struct BitwiseORExpression<const IN: bool, const YIELD: bool, const AWAIT: bool>;

expression!(
    BitwiseORExpression,
    BitwiseXORExpression,
    [Punctuator::Or],
    [IN, YIELD, AWAIT],
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
struct BitwiseXORExpression<const IN: bool, const YIELD: bool, const AWAIT: bool>;

expression!(
    BitwiseXORExpression,
    BitwiseANDExpression,
    [Punctuator::Xor],
    [IN, YIELD, AWAIT],
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
struct BitwiseANDExpression<const IN: bool, const YIELD: bool, const AWAIT: bool>;

expression!(
    BitwiseANDExpression,
    EqualityExpression,
    [Punctuator::And],
    [IN, YIELD, AWAIT],
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
struct EqualityExpression<const IN: bool, const YIELD: bool, const AWAIT: bool>;

expression!(
    EqualityExpression,
    RelationalExpression,
    [
        Punctuator::Eq,
        Punctuator::NotEq,
        Punctuator::StrictEq,
        Punctuator::StrictNotEq
    ],
    [IN, YIELD, AWAIT],
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
struct RelationalExpression<const IN: bool, const YIELD: bool, const AWAIT: bool>;

impl<R, const IN: bool, const YIELD: bool, const AWAIT: bool> TokenParser<R>
    for RelationalExpression<IN, YIELD, AWAIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("Relation Expression", "Parsing");

        if None::<InputElement>.is_some() {
            cursor.set_goal(None::<InputElement>.unwrap());
        }

        let mut lhs = ShiftExpression::<YIELD, AWAIT>.parse(cursor)?;
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
                        ShiftExpression::<YIELD, AWAIT>.parse(cursor)?,
                    )
                    .into();
                }
                TokenKind::Keyword(op)
                    if op == Keyword::InstanceOf || (op == Keyword::In && IN) =>
                {
                    let _ = cursor.next().expect("token disappeared");
                    lhs = BinOp::new(
                        op.as_binop().expect("Could not get binary operation."),
                        lhs,
                        ShiftExpression::<YIELD, AWAIT>.parse(cursor)?,
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
struct ShiftExpression<const YIELD: bool, const AWAIT: bool>;

expression!(
    ShiftExpression,
    AdditiveExpression,
    [
        Punctuator::LeftSh,
        Punctuator::RightSh,
        Punctuator::URightSh
    ],
    [YIELD, AWAIT],
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
struct AdditiveExpression<const YIELD: bool, const AWAIT: bool>;

expression!(
    AdditiveExpression,
    MultiplicativeExpression,
    [Punctuator::Add, Punctuator::Sub],
    [YIELD, AWAIT],
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
struct MultiplicativeExpression<const YIELD: bool, const AWAIT: bool>;

expression!(
    MultiplicativeExpression,
    ExponentiationExpression,
    [Punctuator::Mul, Punctuator::Div, Punctuator::Mod],
    [YIELD, AWAIT],
    Some(InputElement::Div)
);
