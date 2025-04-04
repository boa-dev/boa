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
    source::ReadChar,
    Error,
};
use boa_ast::{
    self as ast,
    expression::{
        operator::{
            binary::{BinaryOp, LogicalOp},
            Binary, BinaryInPrivate,
        },
        Identifier,
    },
    function::PrivateName,
    Keyword, Position, Punctuator,
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;

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
/// A list of punctuators (operands between the `TargetExpression` and `InnerExpression`) are passed as the third parameter.
///
/// The fifth parameter is an `Option<InputElement>` which sets the goal symbol to set before parsing (or None to leave it as is).
macro_rules! expression {
    ($name:ident, $lower:ident, [$( $op:path ),*], [$( $low_param:ident ),*], $($goal:expr)? ) => {
        impl<R> TokenParser<R> for $name
        where
            R: ReadChar
        {
            type Output = ast::Expression;

            fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner)-> ParseResult<ast::Expression> {
                self.parse_boxed(cursor, interner).map(|ok|*ok)
            }

            fn parse_boxed(self, cursor: &mut Cursor<R>, interner: &mut Interner)-> ParseResult<Box<ast::Expression>> {
                let mut lhs;
                expression!([PREFIX][cursor] $name $(, $goal)?);
                expression!([SUBCALL][self; cursor; interner; lhs] $lower, [$($low_param),*]);
                expression!([POSTFIX][self; cursor; interner; lhs] $lower, [$($op),*], [$($low_param),*])
            }
        }
    };
    ([PREFIX][$cursor:ident] $name:ident $(, $goal:expr)? ) => {{
        let _timer = Profiler::global().start_event(stringify!($name), "Parsing");
        $($cursor.set_goal($goal);)?
    }};
    ([LOWER_CTOR][$self:ident] $lower:ident, [$( $low_param:ident ),*] ) => {
        $lower::new($( $self.$low_param ),*)
    };
    ([SUBCALL][$self:ident; $cursor:ident; $interner:ident; $lhs:ident] $lower:ident, [$( $low_param:ident ),*] ) => {{
        $lhs = expression!([LOWER_CTOR][$self] $lower, [$($low_param),*]).parse_boxed($cursor, $interner)?;
    }};
    ([POSTFIX][$self:ident; $cursor:ident; $interner:ident; $lhs:ident] $lower:ident, [$( $op:path ),*], [$( $low_param:ident ),*]) => {{
        while let Some(tok) = $cursor.peek(0, $interner)? {
            match *tok.kind() {
                TokenKind::Punctuator(op) if $( op == $op )||* => {
                    $cursor.advance($interner);
                    $lhs = Binary::new_boxed_expr(
                        op.as_binary_op().expect("Could not get binary operation."),
                        $lhs,
                        $lower::new($( $self.$low_param ),*).parse_boxed($cursor, $interner)?
                    );
                }
                _ => break
            }
        }

        Ok($lhs)
    }};
    ([POSTFIX][$self:ident; $cursor:ident; $interner:ident; $lhs:ident] $lower:ident, [$( $op:path ),*]) => {{
        while let Some(tok) = $cursor.peek(0, $interner)? {
            match *tok.kind() {
                TokenKind::Punctuator(op) if $( op == $op )||* => {
                    $cursor.advance($interner);
                    $lhs = Binary::new_boxed_expr(
                        op.as_binary_op().expect("Could not get binary operation."),
                        $lhs,
                        $lower.parse_boxed($cursor, $interner)?
                    );
                }
                _ => break
            }
        }

        $lhs
    }};
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

impl<R> TokenParser<R> for Expression
where
    R: ReadChar,
{
    type Output = ast::Expression;
    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        self.parse_boxed(cursor, interner).map(|ok| *ok)
    }

    fn parse_boxed(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<Box<Self::Output>> {
        let _timer = Profiler::global().start_event("Expression", "Parsing");

        let lhs = AssignmentExpression::new(self.allow_in, self.allow_yield, self.allow_await)
            .parse_boxed(cursor, interner)?;

        self.parse_boxed_tail(cursor, interner, lhs)
    }
}
impl Expression {
    /// This function was added to optimize the stack size.
    /// It has an stack size optimization impact only for `profile.#.opt-level = 0`.
    /// It allow to reduce stack size allocation in `parse_boxed`,
    /// and an often called function in recursion stays outside of this function.
    fn parse_boxed_tail<R: ReadChar>(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
        mut lhs: Box<ast::Expression>,
    ) -> ParseResult<Box<ast::Expression>> {
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

                    lhs = Binary::new_boxed_expr(
                        Punctuator::Comma
                            .as_binary_op()
                            .expect("Could not get binary operation."),
                        lhs,
                        AssignmentExpression::new(
                            self.allow_in,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse_boxed(cursor, interner)?,
                    );
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
            previous: PreviousExpr::None,
        }
    }

    fn with_previous<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
        previous: PreviousExpr,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            previous,
        }
    }
}

impl<R> TokenParser<R> for ShortCircuitExpression
where
    R: ReadChar,
{
    type Output = ast::Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        self.parse_boxed(cursor, interner).map(|ok| *ok)
    }

    fn parse_boxed(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<Box<Self::Output>> {
        let _timer = Profiler::global().start_event("ShortCircuitExpression", "Parsing");

        let current_node =
            BitwiseORExpression::new(self.allow_in, self.allow_yield, self.allow_await)
                .parse_boxed(cursor, interner)?;

        self.parse_boxed_tail(cursor, interner, current_node)
    }
}
impl ShortCircuitExpression {
    /// This function was added to optimize the stack size.
    /// It has an stack size optimization impact only for `profile.#.opt-level = 0`.
    /// It allow to reduce stack size allocation in `parse_boxed`,
    /// and an often called function in recursion stays outside of this function.
    fn parse_boxed_tail<R: ReadChar>(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
        mut current_node: Box<ast::Expression>,
    ) -> ParseResult<Box<ast::Expression>> {
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
                    let rhs =
                        BitwiseORExpression::new(self.allow_in, self.allow_yield, self.allow_await)
                            .parse_boxed(cursor, interner)?;

                    current_node = Binary::new_boxed_expr(
                        BinaryOp::Logical(LogicalOp::And),
                        current_node,
                        rhs,
                    );
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
                        self.allow_in,
                        self.allow_yield,
                        self.allow_await,
                        PreviousExpr::Logical,
                    )
                    .parse_boxed(cursor, interner)?;
                    current_node =
                        Binary::new_boxed_expr(BinaryOp::Logical(LogicalOp::Or), current_node, rhs);
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
                    let rhs =
                        BitwiseORExpression::new(self.allow_in, self.allow_yield, self.allow_await)
                            .parse_boxed(cursor, interner)?;
                    current_node = Binary::new_boxed_expr(
                        BinaryOp::Logical(LogicalOp::Coalesce),
                        current_node,
                        rhs,
                    );
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

impl<R> TokenParser<R> for BitwiseORExpression
where
    R: ReadChar,
{
    type Output = ast::Expression;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<ast::Expression> {
        self.parse_boxed(cursor, interner).map(|ok| *ok)
    }

    fn parse_boxed(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<Box<ast::Expression>> {
        // TODO: recursive `expression!`
        //
        // unwrapping of
        // ```
        // expression!(
        //     BitwiseORExpression,
        //     BitwiseXORExpression,
        //     [Punctuator::Or],
        //     [allow_in, allow_yield, allow_await],
        // );
        // ```
        // with subcall inlining to reduce stack allocation (from X * 4 to X)
        let mut lhs: Box<boa_ast::Expression>;
        expression!([PREFIX][cursor] BitwiseORExpression);
        let lower = expression!([LOWER_CTOR][self] BitwiseXORExpression, [allow_in, allow_yield, allow_await]);
        lhs = {
            expression!([PREFIX][cursor] BitwiseXORExpression);
            let lower = expression!([LOWER_CTOR][self] BitwiseANDExpression, [allow_in, allow_yield, allow_await]);
            lhs = {
                expression!([PREFIX][cursor] BitwiseANDExpression);
                let lower = expression!([LOWER_CTOR][self] EqualityExpression, [allow_in, allow_yield, allow_await]);
                lhs = {
                    expression!([PREFIX][cursor] EqualityExpression);
                    let lower = expression!([LOWER_CTOR][self] RelationalExpression, [allow_in, allow_yield, allow_await]);
                    lhs = lower.parse_boxed(cursor, interner)?;
                    expression!(
                        [POSTFIX]
                        [self; cursor; interner; lhs]
                        lower,
                        [Punctuator::Eq, Punctuator::NotEq, Punctuator::StrictEq, Punctuator::StrictNotEq]
                    )
                };
                expression!([POSTFIX][self; cursor; interner; lhs] lower, [Punctuator::And])
            };
            expression!([POSTFIX][self; cursor; interner; lhs] lower, [Punctuator::Xor])
        };
        lhs = expression!([POSTFIX][self; cursor; interner; lhs] lower, [Punctuator::Or]);
        Ok(lhs)
    }
}

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
    R: ReadChar,
{
    type Output = ast::Expression;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<ast::Expression> {
        self.parse_boxed(cursor, interner).map(|ok| *ok)
    }

    fn parse_boxed(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<Box<Self::Output>> {
        let _timer = Profiler::global().start_event("Relation Expression", "Parsing");

        if let Some(ok) = self.parse_boxed_private_name(cursor, interner)? {
            return Ok(ok);
        }

        let lhs = ShiftExpression::new(self.allow_yield, self.allow_await)
            .parse_boxed(cursor, interner)?;

        self.parse_boxed_tail(cursor, interner, lhs)
    }
}

impl RelationalExpression {
    /// This function was added to optimize the stack size.
    /// It has an stack size optimization impact only for `profile.#.opt-level = 0`.
    /// It allow to reduce stack size allocation in `parse_boxed`,
    /// and an often called function in recursion stays outside of this function.
    ///
    /// # Return
    /// * `Err(_)` if error occurs;
    /// * `Ok(Some(Box<Expr>))` if the next expression is `BinaryInPrivate`;
    /// * `Ok(None)` otherwise;
    fn parse_boxed_private_name<R: ReadChar>(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<Option<Box<ast::Expression>>> {
        if self.allow_in.0 {
            let token = cursor.peek(0, interner).or_abrupt()?;
            if let TokenKind::PrivateIdentifier(identifier) = token.kind() {
                let identifier = *identifier;
                let token = cursor.peek(1, interner).or_abrupt()?;
                match token.kind() {
                    TokenKind::Keyword((Keyword::In, true)) => {
                        return Err(Error::general(
                            "Keyword must not contain escaped characters",
                            token.span().start(),
                        ));
                    }
                    TokenKind::Keyword((Keyword::In, false)) => {
                        cursor.advance(interner);
                        cursor.advance(interner);

                        let rhs = ShiftExpression::new(self.allow_yield, self.allow_await)
                            .parse_boxed(cursor, interner)?;

                        return Ok(Some(
                            BinaryInPrivate::new_boxed(PrivateName::new(identifier), rhs).into(),
                        ));
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }

    /// This function was added to optimize the stack size.
    /// It has an stack size optimization impact only for `profile.#.opt-level = 0`.
    /// It allow to reduce stack size allocation in `parse_boxed`,
    /// and an often called function in recursion stays outside of this function.
    fn parse_boxed_tail<R: ReadChar>(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
        mut lhs: Box<ast::Expression>,
    ) -> ParseResult<Box<ast::Expression>> {
        while let Some(tok) = cursor.peek(0, interner)? {
            match *tok.kind() {
                TokenKind::Punctuator(op)
                    if op == Punctuator::LessThan
                        || op == Punctuator::GreaterThan
                        || op == Punctuator::LessThanOrEq
                        || op == Punctuator::GreaterThanOrEq =>
                {
                    cursor.advance(interner);
                    lhs = Binary::new_boxed(
                        op.as_binary_op().expect("Could not get binary operation."),
                        lhs,
                        ShiftExpression::new(self.allow_yield, self.allow_await)
                            .parse_boxed(cursor, interner)?,
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
                    lhs = Binary::new_boxed(
                        op.as_binary_op().expect("Could not get binary operation."),
                        lhs,
                        ShiftExpression::new(self.allow_yield, self.allow_await)
                            .parse_boxed(cursor, interner)?,
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

impl<R> TokenParser<R> for ShiftExpression
where
    R: ReadChar,
{
    type Output = ast::Expression;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<ast::Expression> {
        self.parse_boxed(cursor, interner).map(|ok| *ok)
    }

    fn parse_boxed(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<Box<ast::Expression>> {
        // TODO: recursive `expression!`
        //
        // unwrapping of
        // ```
        // expression!(
        //     ShiftExpression,
        //     AdditiveExpression,
        //     [
        //         Punctuator::LeftSh,
        //         Punctuator::RightSh,
        //         Punctuator::URightSh
        //     ],
        //     [allow_yield, allow_await],
        // );
        // ```
        // with subcall inlining to reduce stack allocation (from X * 3 to X)
        let mut lhs: Box<boa_ast::Expression>;
        expression!([PREFIX][cursor] ShiftExpression);
        let lower = expression!([LOWER_CTOR][self] AdditiveExpression, [allow_yield, allow_await]);
        lhs = {
            expression!([PREFIX][cursor] AdditiveExpression);
            let lower = expression!([LOWER_CTOR][self] MultiplicativeExpression, [allow_yield, allow_await]);
            lhs = {
                expression!([PREFIX][cursor] MultiplicativeExpression, InputElement::Div);
                let lower = expression!([LOWER_CTOR][self] ExponentiationExpression, [allow_yield, allow_await]);
                lhs = lower.parse_boxed(cursor, interner)?;
                expression!([POSTFIX][self; cursor; interner; lhs] lower, [Punctuator::Mul, Punctuator::Div, Punctuator::Mod])
            };
            expression!([POSTFIX][self; cursor; interner; lhs] lower, [Punctuator::Add, Punctuator::Sub])
        };
        lhs = expression!([POSTFIX][self; cursor; interner; lhs] lower, [Punctuator::LeftSh, Punctuator::RightSh, Punctuator::URightSh]);
        Ok(lhs)
    }
}

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
    InputElement::Div
);

/// Returns an error if `arguments` or `eval` are used as identifier in strict mode.
fn check_strict_arguments_or_eval(ident: Identifier, position: Position) -> ParseResult<()> {
    match ident.sym() {
        Sym::ARGUMENTS => Err(Error::general(
            "unexpected identifier `arguments` in strict mode",
            position,
        )),
        Sym::EVAL => Err(Error::general(
            "unexpected identifier `eval` in strict mode",
            position,
        )),
        _ => Ok(()),
    }
}
