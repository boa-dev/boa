//! Expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators
//! [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-expressions

mod assignment;
mod identifiers;
mod left_hand_side;
mod primary;
mod unary;
mod update;

pub(in crate::parser) mod await_expr;

#[cfg(test)]
mod tests;

use crate::{
    lexer::{InputElement, TokenKind},
    parser::{
        expression::assignment::ExponentiationExpression, AllowAwait, AllowIn, AllowYield, Cursor,
        OrAbrupt, ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{
    self as ast,
    expression::{
        operator::{
            binary::{BinaryOp, LogicalOp},
            Binary,
        },
        Identifier,
    },
    Keyword, Position, Punctuator,
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

pub(super) use self::{assignment::AssignmentExpression, primary::Initializer};
pub(in crate::parser) use {
    identifiers::{BindingIdentifier, LabelIdentifier},
    left_hand_side::LeftHandSideExpression,
    primary::object_initializer::{
        AsyncGeneratorMethod, AsyncMethod, GeneratorMethod, PropertyName,
    },
};

/// Generates an expression parser for a number of expressions whose production rules are of the following pattern.
///
/// ```text
/// <TargetExpression>[allowed_identifiers]
///     => <InnerExpression>[?allowed_identifiers]
///     => <TargetExpression>[?allowed_identifiers] <op1> <InnerExpression>[?allowed_identifiers]
///     => <TargetExpression>[?allowed_identifiers] <op2> <InnerExpression>[?allowed_identifiers]
///     ...
/// ```
///
/// This macro has 2 mandatory identifiers:
///  - The `$name` identifier is the name of the `TargetExpression` struct that the parser will be implemented for.
///  - The `$lower` identifier is the name of the `InnerExpression` struct according to the pattern above.
///
/// A list of punctuators (operands between the <TargetExpression> and <InnerExpression>) are passed as the third parameter.
///
/// The fifth parameter is an Option<InputElement> which sets the goal symbol to set before parsing (or None to leave it as is).
macro_rules! expression {
    ($name:ident, $lower:ident, [$( $op:path ),*], [$( $low_param:ident ),*], $goal:expr ) => {
        impl<R> TokenParser<R> for $name
        where
            R: Read
        {
            type Output = ast::Expression;

            fn parse(mut self, cursor: &mut Cursor<R>, interner: &mut Interner)-> ParseResult<ast::Expression> {
                let _timer = Profiler::global().start_event(stringify!($name), "Parsing");

                if $goal.is_some() {
                    cursor.set_goal($goal.unwrap());
                }

                let mut lhs = $lower::new($( self.$low_param ),*).parse(cursor, interner)?;
                self.name = None;
                while let Some(tok) = cursor.peek(0, interner)? {
                    match *tok.kind() {
                        TokenKind::Punctuator(op) if $( op == $op )||* => {
                            cursor.advance(interner);
                            lhs = Binary::new(
                                op.as_binary_op().expect("Could not get binary operation."),
                                lhs,
                                $lower::new($( self.$low_param ),*).parse(cursor, interner)?
                            ).into();
                        }
                        _ => break
                    }
                }

                Ok(lhs)
            }
        }
    };
}

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
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl Expression {
    /// Creates a new `Expression` parser.
    pub(super) fn new<N, I, Y, A>(name: N, allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for Expression
where
    R: Read,
{
    type Output = ast::Expression;

    fn parse(
        mut self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("Expression", "Parsing");

        let mut lhs =
            AssignmentExpression::new(self.name, self.allow_in, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
        self.name = None;
        while let Some(tok) = cursor.peek(0, interner)? {
            match *tok.kind() {
                TokenKind::Punctuator(Punctuator::Comma) => {
                    if cursor.peek(1, interner).or_abrupt()?.kind()
                        == &TokenKind::Punctuator(Punctuator::CloseParen)
                    {
                        return Ok(lhs);
                    }

                    if cursor.peek(1, interner).or_abrupt()?.kind()
                        == &TokenKind::Punctuator(Punctuator::Spread)
                    {
                        return Ok(lhs);
                    }

                    cursor.advance(interner);

                    lhs = Binary::new(
                        Punctuator::Comma
                            .as_binary_op()
                            .expect("Could not get binary operation."),
                        lhs,
                        AssignmentExpression::new(
                            self.name,
                            self.allow_in,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)?,
                    )
                    .into();
                }
                _ => break,
            }
        }

        Ok(lhs)
    }
}

/// Parses a logical expression expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_Operators
/// [spec]: https://tc39.es/ecma262/#prod-ShortCircuitExpression
#[derive(Debug, Clone, Copy)]
struct ShortCircuitExpression {
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    previous: PreviousExpr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PreviousExpr {
    None,
    Logical,
    Coalesce,
}

impl ShortCircuitExpression {
    /// Creates a new `ShortCircuitExpression` parser.
    pub(super) fn new<N, I, Y, A>(name: N, allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            previous: PreviousExpr::None,
        }
    }

    fn with_previous<N, I, Y, A>(
        name: N,
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
        previous: PreviousExpr,
    ) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            previous,
        }
    }
}

impl<R> TokenParser<R> for ShortCircuitExpression
where
    R: Read,
{
    type Output = ast::Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("ShortCircuitExpression", "Parsing");

        let mut current_node =
            BitwiseORExpression::new(self.name, self.allow_in, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
        let mut previous = self.previous;

        while let Some(tok) = cursor.peek(0, interner)? {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::BoolAnd) => {
                    if previous == PreviousExpr::Coalesce {
                        return Err(Error::expected(
                            ["??".to_owned()],
                            tok.to_string(interner), tok.span(),
                            "logical expression (cannot use '??' without parentheses within '||' or '&&')",
                        ));
                    }
                    cursor.advance(interner);
                    previous = PreviousExpr::Logical;
                    let rhs = BitwiseORExpression::new(
                        self.name,
                        self.allow_in,
                        self.allow_yield,
                        self.allow_await,
                    )
                    .parse(cursor, interner)?;

                    current_node =
                        Binary::new(BinaryOp::Logical(LogicalOp::And), current_node, rhs).into();
                }
                TokenKind::Punctuator(Punctuator::BoolOr) => {
                    if previous == PreviousExpr::Coalesce {
                        return Err(Error::expected(
                            ["??".to_owned()],
                            tok.to_string(interner), tok.span(),
                            "logical expression (cannot use '??' without parentheses within '||' or '&&')",
                        ));
                    }
                    cursor.advance(interner);
                    previous = PreviousExpr::Logical;
                    let rhs = Self::with_previous(
                        self.name,
                        self.allow_in,
                        self.allow_yield,
                        self.allow_await,
                        PreviousExpr::Logical,
                    )
                    .parse(cursor, interner)?;
                    current_node =
                        Binary::new(BinaryOp::Logical(LogicalOp::Or), current_node, rhs).into();
                }
                TokenKind::Punctuator(Punctuator::Coalesce) => {
                    if previous == PreviousExpr::Logical {
                        return Err(Error::expected(
                            ["&&".to_owned(), "||".to_owned()],
                            tok.to_string(interner),
                            tok.span(),
                            "cannot use '??' unparenthesized within '||' or '&&'",
                        ));
                    }
                    cursor.advance(interner);
                    previous = PreviousExpr::Coalesce;
                    let rhs = BitwiseORExpression::new(
                        self.name,
                        self.allow_in,
                        self.allow_yield,
                        self.allow_await,
                    )
                    .parse(cursor, interner)?;
                    current_node =
                        Binary::new(BinaryOp::Logical(LogicalOp::Coalesce), current_node, rhs)
                            .into();
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
struct BitwiseORExpression {
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BitwiseORExpression {
    /// Creates a new `BitwiseORExpression` parser.
    pub(super) fn new<N, I, Y, A>(name: N, allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
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
    [name, allow_in, allow_yield, allow_await],
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
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BitwiseXORExpression {
    /// Creates a new `BitwiseXORExpression` parser.
    pub(super) fn new<N, I, Y, A>(name: N, allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
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
    [name, allow_in, allow_yield, allow_await],
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
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BitwiseANDExpression {
    /// Creates a new `BitwiseANDExpression` parser.
    pub(super) fn new<N, I, Y, A>(name: N, allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
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
    [name, allow_in, allow_yield, allow_await],
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
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl EqualityExpression {
    /// Creates a new `EqualityExpression` parser.
    pub(super) fn new<N, I, Y, A>(name: N, allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
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
    [name, allow_in, allow_yield, allow_await],
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
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl RelationalExpression {
    /// Creates a new `RelationalExpression` parser.
    pub(super) fn new<N, I, Y, A>(name: N, allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
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
    type Output = ast::Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("Relation Expression", "Parsing");

        let mut lhs = ShiftExpression::new(self.name, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;
        while let Some(tok) = cursor.peek(0, interner)? {
            match *tok.kind() {
                TokenKind::Punctuator(op)
                    if op == Punctuator::LessThan
                        || op == Punctuator::GreaterThan
                        || op == Punctuator::LessThanOrEq
                        || op == Punctuator::GreaterThanOrEq =>
                {
                    cursor.advance(interner);
                    lhs = Binary::new(
                        op.as_binary_op().expect("Could not get binary operation."),
                        lhs,
                        ShiftExpression::new(self.name, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    )
                    .into();
                }
                TokenKind::Keyword((Keyword::InstanceOf | Keyword::In, true)) => {
                    return Err(Error::general(
                        "Keyword must not contain escaped characters",
                        tok.span().start(),
                    ));
                }
                TokenKind::Keyword((op, false))
                    if op == Keyword::InstanceOf
                        || (op == Keyword::In && self.allow_in == AllowIn(true)) =>
                {
                    cursor.advance(interner);
                    lhs = Binary::new(
                        op.as_binary_op().expect("Could not get binary operation."),
                        lhs,
                        ShiftExpression::new(self.name, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
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
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ShiftExpression {
    /// Creates a new `ShiftExpression` parser.
    pub(super) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
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
    [name, allow_yield, allow_await],
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
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AdditiveExpression {
    /// Creates a new `AdditiveExpression` parser.
    pub(super) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

expression!(
    AdditiveExpression,
    MultiplicativeExpression,
    [Punctuator::Add, Punctuator::Sub],
    [name, allow_yield, allow_await],
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
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl MultiplicativeExpression {
    /// Creates a new `MultiplicativeExpression` parser.
    pub(super) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

expression!(
    MultiplicativeExpression,
    ExponentiationExpression,
    [Punctuator::Mul, Punctuator::Div, Punctuator::Mod],
    [name, allow_yield, allow_await],
    Some(InputElement::Div)
);

/// Returns an error if `arguments` or `eval` are used as identifier in strict mode.
const fn check_strict_arguments_or_eval(ident: Identifier, position: Position) -> ParseResult<()> {
    match ident.sym() {
        Sym::ARGUMENTS => Err(Error::general(
            "unexpected identifier 'arguments' in strict mode",
            position,
        )),
        Sym::EVAL => Err(Error::general(
            "unexpected identifier 'eval' in strict mode",
            position,
        )),
        _ => Ok(()),
    }
}
