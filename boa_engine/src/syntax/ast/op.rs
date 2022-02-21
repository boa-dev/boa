//! This module implements various structure for logic handling.

use boa_gc::{unsafe_empty_trace, Finalize, Trace};
use std::fmt::{Display, Formatter, Result};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// Arithmetic operators take numerical values (either literals or variables)
/// as their operands and return a single numerical value.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Arithmetic
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Finalize, PartialEq)]
pub enum NumOp {
    /// The addition operator produces the sum of numeric operands or string concatenation.
    ///
    /// Syntax: `x + y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec].
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-addition-operator-plus
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Addition
    Add,

    /// The subtraction operator subtracts the two operands, producing their difference.
    ///
    /// Syntax: `x - y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec].
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-subtraction-operator-minus
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Subtraction
    Sub,

    /// The division operator produces the quotient of its operands where the left operand
    /// is the dividend and the right operand is the divisor.
    ///
    /// Syntax: `x / y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MultiplicativeOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Division
    Div,

    /// The multiplication operator produces the product of the operands.
    ///
    /// Syntax: `x * y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MultiplicativeExpression
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-exp-operator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Exponentiation
    Exp,

    /// The remainder operator returns the remainder left over when one operand is divided by a second operand.
    ///
    /// Syntax: `x % y`
    ///
    /// The remainder operator always takes the sign of the dividend.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MultiplicativeOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Remainder
    Mod,
}

impl NumOp {
    /// Retrieves the operation as a static string.
    fn as_str(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Div => "/",
            Self::Mul => "*",
            Self::Exp => "**",
            Self::Mod => "%",
        }
    }
}

impl Display for NumOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.as_str())
    }
}

unsafe impl Trace for NumOp {
    unsafe_empty_trace!();
}

/// A unary operator is one that takes a single operand/argument and performs an operation.
///
/// A unary operation is an operation with only one operand. This operand comes either
/// before or after the operator. Unary operators are more efficient than standard JavaScript
/// function calls.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Finalize, PartialEq)]
pub enum UnaryOp {
    /// The increment operator increments (adds one to) its operand and returns a value.
    ///
    /// Syntax: `++x`
    ///
    /// This operator increments and returns the value after incrementing.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-postfix-increment-operator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Increment
    IncrementPost,

    /// The increment operator increments (adds one to) its operand and returns a value.
    ///
    /// Syntax: `x++`
    ///
    /// This operator increments and returns the value before incrementing.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-prefix-increment-operator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Increment
    IncrementPre,

    /// The decrement operator decrements (subtracts one from) its operand and returns a value.
    ///
    /// Syntax: `--x`
    ///
    /// This operator decrements and returns the value before decrementing.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-postfix-decrement-operator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Decrement
    DecrementPost,

    /// The decrement operator decrements (subtracts one from) its operand and returns a value.
    ///
    /// Syntax: `x--`
    ///
    /// This operator decrements the operand and returns the value after decrementing.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-prefix-decrement-operator
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-unary-minus-operator
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-unary-plus-operator
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-logical-not-operator
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bitwise-not-operator
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typeof-operator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/typeof
    TypeOf,

    /// The JavaScript `delete` operator removes a property from an object.
    ///
    /// Syntax: `delete x`
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-delete-operator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/delete
    Delete,

    /// The `void` operator evaluates the given `expression` and then returns `undefined`.
    ///
    /// Syntax: `void x`
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-void-operator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/void
    Void,
}

impl UnaryOp {
    /// Retrieves the operation as a static string.
    fn as_str(self) -> &'static str {
        match self {
            Self::IncrementPost | Self::IncrementPre => "++",
            Self::DecrementPost | Self::DecrementPre => "--",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Not => "!",
            Self::Tilde => "~",
            Self::Delete => "delete",
            Self::TypeOf => "typeof",
            Self::Void => "void",
        }
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.as_str())
    }
}

unsafe impl Trace for UnaryOp {
    unsafe_empty_trace!();
}

/// A bitwise operator is an operator used to perform bitwise operations
/// on bit patterns or binary numerals that involve the manipulation of individual bits.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Bitwise
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Finalize, PartialEq)]
pub enum BitOp {
    /// Performs the AND operation on each pair of bits. a AND b yields 1 only if both a and b are 1.
    ///
    /// Syntax: `x & y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-BitwiseANDExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_AND
    And,

    /// Performs the OR operation on each pair of bits. a OR b yields 1 if either a or b is 1.
    ///
    /// Syntax: `x | y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-BitwiseORExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_OR
    Or,

    /// Performs the XOR operation on each pair of bits. a XOR b yields 1 if a and b are different.
    ///
    /// Syntax: `x ^ y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-BitwiseXORExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Bitwise_XOR
    Xor,

    /// This operator shifts the first operand the specified number of bits to the left.
    ///
    /// Syntax: `x << y`
    ///
    /// Excess bits shifted off to the left are discarded. Zero bits are shifted in from the right.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-left-shift-operator
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-signed-right-shift-operator
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-unsigned-right-shift-operator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_Operators#Unsigned_right_shift
    UShr,
}

impl BitOp {
    /// Retrieves the operation as a static string.
    fn as_str(self) -> &'static str {
        match self {
            Self::And => "&",
            Self::Or => "|",
            Self::Xor => "^",
            Self::Shl => "<<",
            Self::Shr => ">>",
            Self::UShr => ">>>",
        }
    }
}

impl Display for BitOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.as_str())
    }
}

unsafe impl Trace for BitOp {
    unsafe_empty_trace!();
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
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: tc39.es/ecma262/#sec-testing-and-comparison-operations
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Comparison
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Finalize, PartialEq)]
pub enum CompOp {
    /// The equality operator converts the operands if they are not of the same type, then applies
    /// strict comparison.
    ///
    /// Syntax: `y == y`
    ///
    /// If both operands are objects, then JavaScript compares internal references which are equal
    /// when operands refer to the same object in memory.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-abstract-equality-comparison
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Equality
    Equal,

    /// The inequality operator returns `true` if the operands are not equal.
    ///
    /// Syntax: `x != y`
    ///
    /// If the two operands are not of the same type, JavaScript attempts to convert the operands
    /// to an appropriate type for the comparison. If both operands are objects, then JavaScript
    /// compares internal references which are not equal when operands refer to different objects
    /// in memory.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-EqualityExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Inequality
    NotEqual,

    /// The identity operator returns `true` if the operands are strictly equal **with no type
    /// conversion**.
    ///
    /// Syntax: `x === y`
    ///
    /// Returns `true` if the operands are equal and of the same type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-strict-equality-comparison
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Identity
    StrictEqual,

    /// The non-identity operator returns `true` if the operands **are not equal and/or not of the
    /// same type**.
    ///
    /// Syntax: `x !== y`
    ///
    /// Returns `true` if the operands are of the same type but not equal, or are of different type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-EqualityExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Nonidentity>
    StrictNotEqual,

    /// The greater than operator returns `true` if the left operand is greater than the right
    /// operand.
    ///
    /// Syntax: `x > y`
    ///
    /// Returns `true` if the left operand is greater than the right operand.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-RelationalExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Greater_than_operator
    GreaterThan,

    /// The greater than or equal operator returns `true` if the left operand is greater than or
    /// equal to the right operand.
    ///
    /// Syntax: `x >= y`
    ///
    /// Returns `true` if the left operand is greater than the right operand.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-RelationalExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Greater_than_operator
    GreaterThanOrEqual,

    /// The less than operator returns `true` if the left operand is less than the right operand.
    ///
    /// Syntax: `x < y`
    ///
    /// Returns `true` if the left operand is less than the right operand.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-RelationalExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Less_than_operator
    LessThan,

    /// The less than or equal operator returns `true` if the left operand is less than or equal to
    /// the right operand.
    ///
    /// Syntax: `x <= y`
    ///
    /// Returns `true` if the left operand is less than or equal to the right operand.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-RelationalExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comparison_Operators#Less_than_or_equal_operator
    LessThanOrEqual,

    /// The `in` operator returns `true` if the specified property is in the specified object or
    /// its prototype chain.
    ///
    /// Syntax: `prop in object`
    ///
    /// Returns `true` the specified property is in the specified object or its prototype chain.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-RelationalExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/in
    In,

    /// The `instanceof` operator returns `true` if the specified object is an instance of the
    /// right hand side object.
    ///
    /// Syntax: `obj instanceof Object`
    ///
    /// Returns `true` the `prototype` property of the right hand side constructor appears anywhere
    /// in the prototype chain of the object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-RelationalExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/instanceof
    InstanceOf,
}

impl CompOp {
    /// Retrieves the operation as a static string.
    fn as_str(self) -> &'static str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::StrictEqual => "===",
            Self::StrictNotEqual => "!==",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::In => "in",
            Self::InstanceOf => "instanceof",
        }
    }
}

impl Display for CompOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.as_str())
    }
}

unsafe impl Trace for CompOp {
    unsafe_empty_trace!();
}

/// Logical operators are typically used with Boolean (logical) values; when they are, they return a Boolean value.
///
/// However, the `&&` and `||` operators actually return the value of one of the specified operands,
/// so if these operators are used with non-Boolean values, they may return a non-Boolean value.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-binary-logical-operators
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Logical
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Finalize, PartialEq)]
pub enum LogOp {
    /// The logical AND operator returns the value of the first operand if it can be coerced into `false`;
    /// otherwise, it returns the second operand.
    ///
    /// Syntax: `x && y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-LogicalANDExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_Operators#Logical_AND
    And,

    /// The logical OR operator returns the value the first operand if it can be coerced into `true`;
    /// otherwise, it returns the second operand.
    ///
    /// Syntax: `x || y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-LogicalORExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_Operators#Logical_OR
    Or,

    /// The nullish coalescing operator is a logical operator that returns the second operand
    /// when its first operand is null or undefined, and otherwise returns its first operand.
    ///
    /// Syntax: `x ?? y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-CoalesceExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Nullish_coalescing_operator
    Coalesce,
}

impl LogOp {
    /// Retrieves the operation as a static string.
    fn as_str(self) -> &'static str {
        match self {
            Self::And => "&&",
            Self::Or => "||",
            Self::Coalesce => "??",
        }
    }
}

impl Display for LogOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.as_str())
    }
}

unsafe impl Trace for LogOp {
    unsafe_empty_trace!();
}

/// This represents a binary operation between two values.
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Finalize, PartialEq)]
pub enum BinOp {
    /// Numeric operation.
    ///
    /// see: [`NumOp`](enum.NumOp.html)
    Num(NumOp),

    /// Bitwise operation.
    ///
    /// see: [`BitOp`](enum.BitOp.html).
    Bit(BitOp),

    /// Comparitive operation.
    ///
    /// see: [`CompOp`](enum.CompOp.html).
    Comp(CompOp),

    /// Logical operation.
    ///
    /// see: [`LogOp`](enum.LogOp.html).
    Log(LogOp),

    /// Assign operation.
    ///
    /// see: [`AssignOp`](enum.AssignOp.html).
    Assign(AssignOp),

    /// Comma operation.
    Comma,
}

impl From<NumOp> for BinOp {
    fn from(op: NumOp) -> Self {
        Self::Num(op)
    }
}

impl From<BitOp> for BinOp {
    fn from(op: BitOp) -> Self {
        Self::Bit(op)
    }
}

impl From<CompOp> for BinOp {
    fn from(op: CompOp) -> Self {
        Self::Comp(op)
    }
}

impl From<LogOp> for BinOp {
    fn from(op: LogOp) -> Self {
        Self::Log(op)
    }
}

impl From<AssignOp> for BinOp {
    fn from(op: AssignOp) -> Self {
        Self::Assign(op)
    }
}

impl BinOp {
    /// Retrieves the operation as a static string.
    fn as_str(self) -> &'static str {
        match self {
            Self::Num(ref op) => op.as_str(),
            Self::Bit(ref op) => op.as_str(),
            Self::Comp(ref op) => op.as_str(),
            Self::Log(ref op) => op.as_str(),
            Self::Assign(ref op) => op.as_str(),
            Self::Comma => ",",
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.as_str())
    }
}

unsafe impl Trace for BinOp {
    unsafe_empty_trace!();
}

/// An assignment operator assigns a value to its left operand based on the value of its right operand.
///
/// The simple assignment operator is equal (`=`), which assigns the value of its right operand to its
/// left operand. That is, `x = y` assigns the value of `y to x`.
///
/// There are also compound assignment operators that are shorthand for the operations
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Assignment
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Finalize, PartialEq)]
pub enum AssignOp {
    /// The addition assignment operator adds the value of the right operand to a variable and assigns the result to the variable.
    ///
    /// Syntax: `x += y`
    ///
    /// The types of the two operands determine the behavior of the addition assignment operator. Addition or concatenation is possible.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Addition_assignment
    Add,

    /// The subtraction assignment operator subtracts the value of the right operand from a variable and assigns the result to the variable.
    ///
    /// Syntax: `x -= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Subtraction_assignment
    Sub,

    /// The multiplication assignment operator multiplies a variable by the value of the right operand and assigns the result to the variable.
    ///
    /// Syntax: `x *= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Multiplication_assignment
    Mul,

    /// The division assignment operator divides a variable by the value of the right operand and assigns the result to the variable.
    ///
    /// Syntax: `x /= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Division_assignment
    Div,

    /// The remainder assignment operator divides a variable by the value of the right operand and assigns the remainder to the variable.
    ///
    /// Syntax: `x %= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Remainder_assignment
    Mod,

    /// The exponentiation assignment operator raises the value of a variable to the power of the right operand.
    ///
    /// Syntax: `x ** y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Exponentiation_assignment
    Exp,

    /// The bitwise AND assignment operator uses the binary representation of both operands, does a bitwise AND operation on
    /// them and assigns the result to the variable.
    ///
    /// Syntax: `x &= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Bitwise_AND_assignment
    And,

    /// The bitwise OR assignment operator uses the binary representation of both operands, does a bitwise OR operation on
    /// them and assigns the result to the variable.
    ///
    /// Syntax: `x |= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Bitwise_OR_assignment
    Or,

    /// The bitwise XOR assignment operator uses the binary representation of both operands, does a bitwise XOR operation on
    /// them and assigns the result to the variable.
    ///
    /// Syntax: `x ^= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Bitwise_XOR_assignment
    Xor,

    /// The left shift assignment operator moves the specified amount of bits to the left and assigns the result to the variable.
    ///
    /// Syntax: `x <<= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Left_shift_assignment
    Shl,

    /// The right shift assignment operator moves the specified amount of bits to the right and assigns the result to the variable.
    ///
    /// Syntax: `x >>= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Right_shift_assignment
    Shr,

    /// The unsigned right shift assignment operator moves the specified amount of bits to the right and assigns the result to the variable.
    ///
    /// Syntax: `x >>>= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Unsigned_right_shift_assignment
    Ushr,

    /// The logical and assignment operator only assigns if the target variable is truthy.
    ///
    /// Syntax: `x &&= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_AND_assignment
    BoolAnd,

    /// The logical or assignment operator only assigns if the target variable is falsy.
    ///
    /// Syntax: `x ||= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_OR_assignment
    BoolOr,

    /// The logical nullish assignment operator only assigns if the target variable is nullish (null or undefined).
    ///
    /// Syntax: `x ??= y`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_nullish_assignment
    Coalesce,
}

unsafe impl Trace for AssignOp {
    unsafe_empty_trace!();
}

impl AssignOp {
    /// Retrieves the operation as a static string.
    fn as_str(self) -> &'static str {
        match self {
            Self::Add => "+=",
            Self::Sub => "-=",
            Self::Mul => "*=",
            Self::Exp => "**=",
            Self::Div => "/=",
            Self::Mod => "%=",
            Self::And => "&=",
            Self::Or => "|=",
            Self::Xor => "^=",
            Self::Shl => "<<=",
            Self::Shr => ">>=",
            Self::Ushr => ">>>=",
            Self::BoolAnd => "&&=",
            Self::BoolOr => "||=",
            Self::Coalesce => "??=",
        }
    }
}

impl Display for AssignOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.as_str())
    }
}
