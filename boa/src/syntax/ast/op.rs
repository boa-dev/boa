use gc_derive::{Finalize, Trace};
use std::fmt::{Display, Formatter, Result};

#[cfg(feature = "serde-ast")]
use serde::{Deserialize, Serialize};

/// Represents an operator
pub trait Operator {
    /// Get the associativity as a boolean that is true if it goes rightwards
    fn get_assoc(&self) -> bool;
    /// Get the precedence as an unsigned integer, where the lower it is, the more precedence/priority it has
    fn get_precedence(&self) -> u64;
    /// Get the precedence and associativity of this operator
    fn get_precedence_and_assoc(&self) -> (u64, bool) {
        (self.get_precedence(), self.get_assoc())
    }
}

/// Arithmetic operators take numerical values (either literals or variables)
/// as their operands and return a single numerical value.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Arithmetic
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum NumOp {
    /// The addition operator produces the sum of numeric operands or string concatenation.
    ///
    /// Syntax: `x + y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-addition-operator-plus).
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Addition
    Add,

    /// The subtraction operator subtracts the two operands, producing their difference.
    ///
    /// Syntax: `x - y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-subtraction-operator-minus).
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Subtraction
    Sub,

    /// The division operator produces the quotient of its operands where the left operand
    /// is the dividend and the right operand is the divisor.
    ///
    /// Syntax: `x / y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-MultiplicativeOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Division
    Div,

    /// The multiplication operator produces the product of the operands.
    ///
    /// Syntax: `x * y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-MultiplicativeExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Multiplication
    Mul,

    /// The exponentiation operator returns the result of raising the first operand to
    /// the power of the second operand.
    ///
    /// Syntax: `x ** y`
    ///
    /// The exponentiation operator is right-associative. a ** b ** c is equal to a ** (b ** c).
    ///
    /// More information:
    ///  - [ECMAScript reference]: <https://tc39.es/ecma262/#sec-exp-operator>.
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Exponentiation
    Exp,

    /// The remainder operator returns the remainder left over when one operand is divided by a second operand.
    ///
    /// Syntax: `x % y`
    ///
    /// The remainder operator always takes the sign of the dividend.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-MultiplicativeOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Remainder
    Mod,
}

impl Display for NumOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match *self {
                NumOp::Add => "+",
                NumOp::Sub => "-",
                NumOp::Div => "/",
                NumOp::Mul => "*",
                NumOp::Exp => "**",
                NumOp::Mod => "%",
            }
        )
    }
}

/// A unary operator is one that takes a single operand/argument and performs an operation.
///
/// A unary operation is an operation with only one operand. This operand comes either
/// before or after the operator. Unary operators are more efficient than standard JavaScript
/// function calls.
///
/// More information:
///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-UnaryExpression)
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum UnaryOp {
    /// The increment operator increments (adds one to) its operand and returns a value.
    ///
    /// Syntax: `++x`
    ///
    /// This operator increments and returns the value after incrementing.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-postfix-increment-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Increment
    IncrementPost,

    /// The increment operator increments (adds one to) its operand and returns a value.
    ///
    /// Syntax: `x++`
    ///
    /// This operator increments and returns the value before incrementing.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-prefix-increment-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Increment
    IncrementPre,

    /// The decrement operator decrements (subtracts one from) its operand and returns a value.
    ///
    /// Syntax: `--x`
    ///
    /// This operator decrements and returns the value before decrementing.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-postfix-decrement-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Decrement
    DecrementPost,

    /// The decrement operator decrements (subtracts one from) its operand and returns a value.
    ///
    /// Syntax: `x--`
    ///
    /// This operator decrements the operand and returns the value after decrementing.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-prefix-decrement-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Decrement
    DecrementPre,

    /// The unary negation operator precedes its operand and negates it.
    ///
    /// Syntax: `-x`
    ///
    /// Converts non-numbers data types to numbers like unary plus,
    /// however, it performs an additional operation, negation.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-unary-minus-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Unary_negation
    Minus,

    /// The unary plus operator attempts to convert the operand into a number, if it isn't already.
    ///
    /// Syntax: `+x`
    ///
    /// Although unary negation (`-`) also can convert non-numbers, unary plus is the fastest and preferred
    /// way of converting something into a number, because it does not perform any other operations on the number.
    /// It can convert `string` representations of integers and floats, as well as the non-string values `true`, `false`, and `null`.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-unary-plus-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Unary_plus
    Plus,

    /// Returns `false` if its single operand can be converted to `true`; otherwise, returns `true`.
    ///
    /// Syntax: `!x`
    ///
    /// Boolean values simply get inverted: `!true === false` and `!false === true`.
    /// Non-boolean values get converted to boolean values first, then are negated.
    /// This means that it is possible to use a couple of NOT operators in series to explicitly
    /// force the conversion of any value to the corresponding boolean primitive.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-logical-not-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_Operators#Logical_NOT
    Not,

    /// Performs the NOT operator on each bit.
    ///
    /// Syntax: `~x`
    ///
    /// NOT `a` yields the inverted value (or one's complement) of `a`.
    /// Bitwise NOTing any number x yields -(x + 1). For example, ~-5 yields 4.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-bitwise-not-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_NOT
    Tilde,

    /// The `typeof` operator returns a string indicating the type of the unevaluated operand.
    ///
    /// Syntax: `typeof x` or `typeof(x)`
    ///
    /// The `typeof` is a JavaScript keyword that will return the type of a variable when you call it.
    /// You can use this to validate function parameters or check if variables are defined.
    /// There are other uses as well.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-typeof-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/typeof
    TypeOf,

    /// The JavaScript `delete` operator removes a property from an object.
    ///
    /// Syntax: `delete expression`
    ///
    /// Unlike what common belief suggests, the delete operator has nothing to do with
    /// directly freeing memory. Memory management is done indirectly via breaking references.
    /// If no more references to the same property are held, it is eventually released automatically.
    ///
    /// The `delete` operator returns `true` for all cases except when the property is an
    /// [own](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/hasOwnProperty)
    /// [non-configurable](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Errors/Cant_delete)
    /// property, in which case, `false` is returned in non-strict mode.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-delete-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/delete
    Delete,

    /// The `void` operator evaluates the given `expression` and then returns `undefined`.
    ///
    /// Syntax: `void expression`
    ///
    /// This operator allows evaluating expressions that produce a value into places where an
    /// expression that evaluates to `undefined` is desired.
    /// The `void` operator is often used merely to obtain the `undefined` primitive value, usually using `void(0)`
    /// (which is equivalent to `void 0`). In these cases, the global variable undefined can be used.
    ///
    /// When using an [immediately-invoked function expression](https://developer.mozilla.org/en-US/docs/Glossary/IIFE),
    /// `void` can be used to force the function keyword to be treated as an expression instead of a declaration.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-void-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/void
    Void,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match *self {
                UnaryOp::IncrementPost | UnaryOp::IncrementPre => "++",
                UnaryOp::DecrementPost | UnaryOp::DecrementPre => "--",
                UnaryOp::Plus => "+",
                UnaryOp::Minus => "-",
                UnaryOp::Not => "!",
                UnaryOp::Tilde => "~",
                UnaryOp::Delete => "delete",
                UnaryOp::TypeOf => "typeof",
                UnaryOp::Void => "void",
            }
        )
    }
}

/// A bitwise operator is an operator used to perform bitwise operations
/// on bit patterns or binary numerals that involve the manipulation of individual bits.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Bitwise
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum BitOp {
    /// Performs the AND operation on each pair of bits. a AND b yields 1 only if both a and b are 1.
    ///
    /// Syntax: `x & y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-BitwiseANDExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_AND
    And,

    /// Performs the OR operation on each pair of bits. a OR b yields 1 if either a or b is 1.
    ///
    /// Syntax: `x | y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-BitwiseORExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_OR
    Or,

    /// Performs the XOR operation on each pair of bits. a XOR b yields 1 if a and b are different.
    ///
    /// Syntax: `x ^ y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-BitwiseXORExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_XOR
    Xor,

    /// This operator shifts the first operand the specified number of bits to the left.
    ///
    /// Syntax: `x << y`
    ///
    /// Excess bits shifted off to the left are discarded. Zero bits are shifted in from the right.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-left-shift-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Left_shift
    Shl,

    /// This operator shifts the first operand the specified number of bits to the right.
    ///
    /// Syntax: `x >> y`
    ///
    /// Excess bits shifted off to the right are discarded. Copies of the leftmost bit
    /// are shifted in from the left. Since the new leftmost bit has the same value as
    /// the previous leftmost bit, the sign bit (the leftmost bit) does not change.
    /// Hence the name "sign-propagating".
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-signed-right-shift-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Right_shift
    Shr,

    /// This operator shifts the first operand the specified number of bits to the right.
    ///
    /// Syntax: `x >>> y`
    ///
    /// Excess bits shifted off to the right are discarded. Zero bits are shifted in
    /// from the left. The sign bit becomes 0, so the result is always non-negative.
    /// Unlike the other bitwise operators, zero-fill right shift returns an unsigned 32-bit integer.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-unsigned-right-shift-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Unsigned_right_shift
    UShr,
}

impl Display for BitOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match *self {
                BitOp::And => "&",
                BitOp::Or => "|",
                BitOp::Xor => "^",
                BitOp::Shl => "<<",
                BitOp::Shr => ">>",
                BitOp::UShr => ">>>",
            }
        )
    }
}

/// A comparison operator compares its operands and returns a logical value based on whether the comparison is true.
///
/// The operands can be numerical, string, logical, or object values. Strings are compared based on standard
/// lexicographical ordering, using Unicode values. In most cases, if the two operands are not of the same type,
/// JavaScript attempts to convert them to an appropriate type for the comparison. This behavior generally results in
/// comparing the operands numerically. The sole exceptions to type conversion within comparisons involve the `===` and `!==`
/// operators, which perform strict equality and inequality comparisons. These operators do not attempt to convert the operands
/// to compatible types before checking equality.
///
/// More information:
///  - [ECMAScript reference](tc39.es/ecma262/#sec-testing-and-comparison-operations)
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Comparison
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum CompOp {
    /// The equality operator converts the operands if they are not of the same type, then applies strict comparison.
    ///
    /// Syntax: `y == y`
    ///
    /// If both operands are objects, then JavaScript compares internal references which are equal when operands
    /// refer to the same object in memory.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-abstract-equality-comparison)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Equality
    Equal,

    /// The inequality operator returns true if the operands are not equal.
    ///
    /// Syntax: `x != y`
    ///
    ///  If the two operands are not of the same type, JavaScript attempts to convert the operands to
    /// an appropriate type for the comparison. If both operands are objects, then JavaScript compares
    /// internal references which are not equal when operands refer to different objects in memory.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-EqualityExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Inequality
    NotEqual,

    /// The identity operator returns true if the operands are strictly equal **with no type conversion**.
    ///
    /// Syntax: `x === y`
    ///
    /// Returns `true` if the operands are equal and of the same type.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-strict-equality-comparison)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Identity
    StrictEqual,

    /// The non-identity operator returns true if the operands **are not equal and/or not of the same type**.
    ///
    /// Syntax: `x !== y`
    ///
    /// Returns `true` if the operands are of the same type but not equal, or are of different type.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-EqualityExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Nonidentity>
    StrictNotEqual,

    /// The greater than operator returns true if the left operand is greater than the right operand.
    ///
    /// Syntax: `x > y`
    ///
    /// Returns `true` if the left operand is greater than the right operand.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-RelationalExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Greater_than_operator
    GreaterThan,

    /// The greater than or equal operator returns true if the left operand is greater than or equal to the right operand.
    ///
    /// Syntax: `x >= y`
    ///
    /// Returns `true` if the left operand is greater than the right operand.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-RelationalExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Greater_than_operator
    GreaterThanOrEqual,

    /// The less than operator returns true if the left operand is less than the right operand.
    ///
    /// Syntax: `x < y`
    ///
    /// Returns `true` if the left operand is less than the right operand.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-RelationalExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Less_than_operator
    LessThan,

    /// The less than or equal operator returns true if the left operand is less than or equal to the right operand.
    ///
    /// Syntax: `x <= y`
    ///
    /// Returns `true` if the left operand is less than or equal to the right operand.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-RelationalExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Less_than_or_equal_operator
    LessThanOrEqual,
}

impl Display for CompOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match *self {
                CompOp::Equal => "==",
                CompOp::NotEqual => "!=",
                CompOp::StrictEqual => "===",
                CompOp::StrictNotEqual => "!==",
                CompOp::GreaterThan => ">",
                CompOp::GreaterThanOrEqual => ">=",
                CompOp::LessThan => "<",
                CompOp::LessThanOrEqual => "<=",
            }
        )
    }
}

/// Logical operators are typically used with Boolean (logical) values; when they are, they return a Boolean value.
///
/// However, the `&&` and `||` operators actually return the value of one of the specified operands,
/// so if these operators are used with non-Boolean values, they may return a non-Boolean value.
///
/// More information:
///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-binary-logical-operators)
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Logical
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum LogOp {
    /// The logical AND operator returns the value of the first operand if it can be coerced into `false`;
    /// otherwise, it returns the second operand.
    ///
    /// Syntax: `x && y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-LogicalANDExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_Operators#Logical_AND
    And,

    /// The logical OR operator returns the value the first operand if it can be coerced into `true`;
    /// otherwise, it returns the second operand.
    ///
    /// Syntax: `x || y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-LogicalORExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_Operators#Logical_OR
    Or,
}

impl Display for LogOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match *self {
                LogOp::And => "&&",
                LogOp::Or => "||",
            }
        )
    }
}

/// A binary operation between 2 values
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum BinOp {
    /// Numeric operation
    Num(NumOp),
    /// Bitwise operation
    Bit(BitOp),
    /// Comparitive operation
    Comp(CompOp),
    /// Logical operation
    Log(LogOp),
    /// Assign operation
    Assign(AssignOp),
}

impl Operator for BinOp {
    fn get_assoc(&self) -> bool {
        true
    }
    fn get_precedence(&self) -> u64 {
        match *self {
            BinOp::Num(NumOp::Exp) => 4,
            BinOp::Num(NumOp::Mul) | BinOp::Num(NumOp::Div) | BinOp::Num(NumOp::Mod) => 5,
            BinOp::Num(NumOp::Add) | BinOp::Num(NumOp::Sub) => 6,
            BinOp::Bit(BitOp::Shl) | BinOp::Bit(BitOp::Shr) | BinOp::Bit(BitOp::UShr) => 7,
            BinOp::Comp(CompOp::LessThan)
            | BinOp::Comp(CompOp::LessThanOrEqual)
            | BinOp::Comp(CompOp::GreaterThan)
            | BinOp::Comp(CompOp::GreaterThanOrEqual) => 8,
            BinOp::Comp(CompOp::Equal)
            | BinOp::Comp(CompOp::NotEqual)
            | BinOp::Comp(CompOp::StrictEqual)
            | BinOp::Comp(CompOp::StrictNotEqual) => 9,
            BinOp::Bit(BitOp::And) => 10,
            BinOp::Bit(BitOp::Xor) => 11,
            BinOp::Bit(BitOp::Or) => 12,
            BinOp::Log(LogOp::And) => 13,
            BinOp::Log(LogOp::Or) => 14,
            BinOp::Assign(_) => 15,
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match *self {
                BinOp::Num(ref op) => op.to_string(),
                BinOp::Bit(ref op) => op.to_string(),
                BinOp::Comp(ref op) => op.to_string(),
                BinOp::Log(ref op) => op.to_string(),
                BinOp::Assign(ref op) => op.to_string(),
            }
        )
    }
}

/// An assignment operator assigns a value to its left operand based on the value of its right operand.
///
/// The simple assignment operator is equal (`=`), which assigns the value of its right operand to its
/// left operand. That is, `x = y` assigns the value of `y to x`.
///
/// There are also compound assignment operators that are shorthand for the operations
///
/// More information:
///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Assignment
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum AssignOp {
    /// The addition assignment operator adds the value of the right operand to a variable and assigns the result to the variable.
    ///
    /// Syntax: `x += y`
    ///
    /// The types of the two operands determine the behavior of the addition assignment operator. Addition or concatenation is possible.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Addition_assignment
    Add,

    /// The subtraction assignment operator subtracts the value of the right operand from a variable and assigns the result to the variable.
    ///
    /// Syntax: `x -= y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation](mdn)
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Subtraction_assignment
    Sub,

    /// The multiplication assignment operator multiplies a variable by the value of the right operand and assigns the result to the variable.
    ///
    /// Syntax: `x *= y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Multiplication_assignment
    Mul,

    /// The division assignment operator divides a variable by the value of the right operand and assigns the result to the variable.
    ///
    /// Syntax: `x /= y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Division_assignment
    Div,

    /// The remainder assignment operator divides a variable by the value of the right operand and assigns the remainder to the variable.
    ///
    /// Syntax: `x %= y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Remainder_assignment
    Mod,

    /// The exponentiation assignment operator raises the value of a variable to the power of the right operand.
    ///
    /// Syntax: `x ** y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Exponentiation_assignment
    Exp,

    /// The bitwise AND assignment operator uses the binary representation of both operands, does a bitwise AND operation on
    /// them and assigns the result to the variable.
    ///
    /// Syntax: `x &= y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Bitwise_AND_assignment
    And,

    /// The bitwise OR assignment operator uses the binary representation of both operands, does a bitwise OR operation on
    /// them and assigns the result to the variable.
    ///
    /// Syntax: `x |= y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Bitwise_OR_assignment
    Or,

    /// The bitwise XOR assignment operator uses the binary representation of both operands, does a bitwise XOR operation on
    /// them and assigns the result to the variable.
    ///
    /// Syntax: `x ^= y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Bitwise_XOR_assignment
    Xor,

    /// The left shift assignment operator moves the specified amount of bits to the left and assigns the result to the variable.
    ///
    /// Syntax: `x <<= y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Left_shift_assignment
    Shl,

    /// The right shift assignment operator moves the specified amount of bits to the right and assigns the result to the variable.
    ///
    /// Syntax: `x >>= y`
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AssignmentOperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Right_shift_assignment
    Shr,
    // TODO: Add UShl (unsigned shift left).
}

impl Display for AssignOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match *self {
                AssignOp::Add => "+=",
                AssignOp::Sub => "-=",
                AssignOp::Mul => "*=",
                AssignOp::Exp => "**=",
                AssignOp::Div => "/=",
                AssignOp::Mod => "%=",
                AssignOp::And => "&=",
                AssignOp::Or => "|=",
                AssignOp::Xor => "^=",
                AssignOp::Shl => "<<=",
                AssignOp::Shr => ">>=",
            }
        )
    }
}
