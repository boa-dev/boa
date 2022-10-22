/// The opcodes of the vm.
use crate::{vm::ShouldExit, Context, JsResult};

// Operation modules
mod await_stm;
mod binary_ops;
mod call;
mod concat;
mod copy;
mod define;
mod delete;
mod dup;
mod environment;
mod generator;
mod get;
mod iteration;
mod jump;
mod new;
mod nop;
mod pop;
mod promise;
mod push;
mod require;
mod rest_parameter;
mod return_stm;
mod set;
mod swap;
mod switch;
mod throw;
mod to;
mod try_catch;
mod unary_ops;
mod value;

// Operation structs
#[doc(inline)]
pub(crate) use await_stm::*;
#[doc(inline)]
pub(crate) use binary_ops::*;
#[doc(inline)]
pub(crate) use call::*;
#[doc(inline)]
pub(crate) use concat::*;
#[doc(inline)]
pub(crate) use copy::*;
#[doc(inline)]
pub(crate) use define::*;
#[doc(inline)]
pub(crate) use delete::*;
#[doc(inline)]
pub(crate) use dup::*;
#[doc(inline)]
pub(crate) use environment::*;
#[doc(inline)]
pub(crate) use generator::*;
#[doc(inline)]
pub(crate) use get::*;
#[doc(inline)]
pub(crate) use iteration::*;
#[doc(inline)]
pub(crate) use jump::*;
#[doc(inline)]
pub(crate) use new::*;
#[doc(inline)]
pub(crate) use nop::*;
#[doc(inline)]
pub(crate) use pop::*;
#[doc(inline)]
pub(crate) use promise::*;
#[doc(inline)]
pub(crate) use push::*;
#[doc(inline)]
pub(crate) use require::*;
#[doc(inline)]
pub(crate) use rest_parameter::*;
#[doc(inline)]
pub(crate) use return_stm::*;
#[doc(inline)]
pub(crate) use set::*;
#[doc(inline)]
pub(crate) use swap::*;
#[doc(inline)]
pub(crate) use switch::*;
#[doc(inline)]
pub(crate) use throw::*;
#[doc(inline)]
pub(crate) use to::*;
#[doc(inline)]
pub(crate) use try_catch::*;
#[doc(inline)]
pub(crate) use unary_ops::*;
#[doc(inline)]
pub(crate) use value::*;

/// The `Operation` trait implements the execution code along with the
/// identifying Name and Instruction value for an Boa Opcode
///
///
/// This trait should be implemented for a struct that corresponds with
/// any arm of the `OpCode` enum.
///
pub(crate) trait Operation {
    const NAME: &'static str;
    const INSTRUCTION: &'static str;

    fn execute(context: &mut Context) -> JsResult<ShouldExit>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    /// Pop the top value from the stack.
    ///
    /// Operands:
    ///
    /// Stack: value **=>**
    Pop,

    /// Pop the top value from the stack if the last try block has thrown a value.
    ///
    /// Operands:
    ///
    /// Stack: value **=>**
    PopIfThrown,

    /// Push a copy of the top value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** value, value
    Dup,

    /// Swap the top two values on the stack.
    ///
    /// Operands:
    ///
    /// Stack: second, first **=>** first, second
    Swap,

    /// Push integer `0` on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `0`
    PushZero,

    /// Push integer `1` on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `1`
    PushOne,

    /// Push `i8` value on the stack.
    ///
    /// Operands: value: `i8`
    ///
    /// Stack: **=>** value
    PushInt8,

    /// Push i16 value on the stack.
    ///
    /// Operands: value: `i16`
    ///
    /// Stack: **=>** value
    PushInt16,

    /// Push i32 value on the stack.
    ///
    /// Operands: value: `i32`
    ///
    /// Stack: **=>** value
    PushInt32,

    /// Push `f64` value on the stack.
    ///
    /// Operands: value: `f64`
    ///
    /// Stack: **=>** value
    PushRational,

    /// Push `NaN` integer on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `NaN`
    PushNaN,

    /// Push `Infinity` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `Infinity`
    PushPositiveInfinity,

    /// Push `-Infinity` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `-Infinity`
    PushNegativeInfinity,

    /// Push `null` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `null`
    PushNull,

    /// Push `true` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `true`
    PushTrue,

    /// Push `false` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `false`
    PushFalse,

    /// Push `undefined` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `undefined`
    PushUndefined,

    /// Push literal value on the stack.
    ///
    /// Like strings and bigints. The index operand is used to index into the `literals`
    /// array to get the value.
    ///
    /// Operands: index: `u32`
    ///
    /// Stack: **=>** (`literals[index]`)
    PushLiteral,

    /// Push empty object `{}` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `{}`
    PushEmptyObject,

    /// Get the prototype of a superclass and push it on the stack.
    ///
    /// Operands:
    ///
    /// Stack: class, superclass **=>** class, superclass.prototype
    PushClassPrototype,

    /// Set the prototype of a class object.
    ///
    /// Operands:
    ///
    /// Stack: class, prototype **=>** class.prototype
    SetClassPrototype,

    /// Set home object internal slot of a function object.
    ///
    /// Operands:
    ///
    /// Stack: home, function **=>** home, function
    SetHomeObject,

    /// Push an empty array value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `[]`
    PushNewArray,

    /// Push a value to an array.
    ///
    /// Operands:
    ///
    /// Stack: array, value **=>** array
    PushValueToArray,

    /// Push an empty element/hole to an array.
    ///
    /// Operands:
    ///
    /// Stack: array **=>** array
    PushElisionToArray,

    /// Push all iterator values to an array.
    ///
    /// Operands:
    ///
    /// Stack: array, iterator, next_method, done **=>** array
    PushIteratorToArray,

    /// Binary `+` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs + rhs)
    Add,

    /// Binary `-` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs - rhs)
    Sub,

    /// Binary `/` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs / rhs)
    Div,

    /// Binary `*` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs * rhs)
    Mul,

    /// Binary `%` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs % rhs)
    Mod,

    /// Binary `**` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs ** rhs)
    Pow,

    /// Binary `>>` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs >> rhs)
    ShiftRight,

    /// Binary `<<` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs << rhs)
    ShiftLeft,

    /// Binary `>>>` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs >>> rhs)
    UnsignedShiftRight,

    /// Binary bitwise `|` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs | rhs)
    BitOr,

    /// Binary bitwise `&` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs & rhs)
    BitAnd,

    /// Binary bitwise `^` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs ^ rhs)
    BitXor,

    /// Unary bitwise `~` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** ~value
    BitNot,

    /// Binary `in` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs `in` rhs)
    In,

    /// Binary `==` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs `==` rhs)
    Eq,

    /// Binary `===` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs `===` rhs)
    StrictEq,

    /// Binary `!=` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs `!=` rhs)
    NotEq,

    /// Binary `!==` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs `!==` rhs)
    StrictNotEq,

    /// Binary `>` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs > rhs)
    GreaterThan,

    /// Binary `>=` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs >= rhs)
    GreaterThanOrEq,

    /// Binary `<` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs < rhs)
    LessThan,

    /// Binary `<=` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs <= rhs)
    LessThanOrEq,

    /// Binary `instanceof` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs instanceof rhs)
    InstanceOf,

    /// Binary logical `&&` operator.
    ///
    /// This is a short-circuit operator, if the `lhs` value is `false`, then it jumps to `exit` address.
    ///
    /// Operands: exit: `u32`
    ///
    /// Stack: lhs, rhs **=>** (lhs && rhs)
    LogicalAnd,

    /// Binary logical `||` operator.
    ///
    /// This is a short-circuit operator, if the `lhs` value is `true`, then it jumps to `exit` address.
    ///
    /// Operands: exit: `u32`
    ///
    /// Stack: lhs, rhs **=>** (lhs || rhs)
    LogicalOr,

    /// Binary `??` operator.
    ///
    /// This is a short-circuit operator, if the `lhs` value is **not** `null` or `undefined`,
    /// then it jumps to `exit` address.
    ///
    /// Operands: exit: `u32`
    ///
    /// Stack: lhs, rhs **=>** (lhs ?? rhs)
    Coalesce,

    /// Unary `typeof` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (`typeof` value)
    TypeOf,

    /// Unary `void` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** `undefined`
    Void,

    /// Unary logical `!` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (!value)
    LogicalNot,

    /// Unary `+` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (+value)
    Pos,

    /// Unary `-` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (-value)
    Neg,

    /// Unary `++` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (value + 1)
    Inc,

    /// Unary postfix `++` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (ToNumeric(value)), (value + 1)
    IncPost,

    /// Unary `--` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (value - 1)
    Dec,

    /// Unary postfix `--` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (ToNumeric(value)), (value - 1)
    DecPost,

    /// Declare and initialize a function argument.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value **=>**
    DefInitArg,

    /// Declare `var` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>**
    DefVar,

    /// Declare and initialize `var` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, has_declarative_binding **=>**
    DefInitVar,

    /// Declare `let` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>**
    DefLet,

    /// Declare and initialize `let` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value **=>**
    DefInitLet,

    /// Declare and initialize `const` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value **=>**
    DefInitConst,

    /// Find a binding on the environment chain and push its value.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>** value
    GetName,

    /// Find a binding on the environment chain and push its value. If the binding does not exist push undefined.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>** value
    GetNameOrUndefined,

    /// Find a binding on the environment chain and assign its value.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value **=>**
    SetName,

    /// Get a property by name from an object an push it on the stack.
    ///
    /// Like `object.name`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object **=>** value
    GetPropertyByName,

    /// Get a property by value from an object an push it on the stack.
    ///
    /// Like `object[key]`
    ///
    /// Operands:
    ///
    /// Stack: key, object **=>** value
    GetPropertyByValue,

    /// Get a property by value from an object an push the key and value on the stack.
    ///
    /// Like `object[key]`
    ///
    /// Operands:
    ///
    /// Stack: key, object **=>** key, value
    GetPropertyByValuePush,

    /// Sets a property by name of an object.
    ///
    /// Like `object.name = value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    SetPropertyByName,

    /// Defines a own property of an object by name.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    DefineOwnPropertyByName,

    /// Defines a class method by name.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    DefineClassMethodByName,

    /// Sets a property by value of an object.
    ///
    /// Like `object[key] = value`
    ///
    /// Operands:
    ///
    /// Stack: value, key, object **=>**
    SetPropertyByValue,

    /// Defines a own property of an object by value.
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    DefineOwnPropertyByValue,

    /// Defines a class method by value.
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    DefineClassMethodByValue,

    /// Sets a getter property by name of an object.
    ///
    /// Like `get name() value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    SetPropertyGetterByName,

    /// Defines a getter class method by name.
    ///
    /// Like `get name() value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    DefineClassGetterByName,

    /// Sets a getter property by value of an object.
    ///
    /// Like `get [key]() value`
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    SetPropertyGetterByValue,

    /// Defines a getter class method by value.
    ///
    /// Like `get [key]() value`
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    DefineClassGetterByValue,

    /// Sets a setter property by name of an object.
    ///
    /// Like `set name() value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    SetPropertySetterByName,

    /// Defines a setter class method by name.
    ///
    /// Like `set name() value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    DefineClassSetterByName,

    /// Sets a setter property by value of an object.
    ///
    /// Like `set [key]() value`
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    SetPropertySetterByValue,

    /// Defines a setter class method by value.
    ///
    /// Like `set [key]() value`
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    DefineClassSetterByValue,

    /// Assign the value of a private property of an object by it's name.
    ///
    /// Like `obj.#name = value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    AssignPrivateField,

    /// Set a private property of a class constructor by it's name.
    ///
    /// Like `#name = value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    SetPrivateField,

    /// Set a private method of a class constructor by it's name.
    ///
    /// Like `#name() {}`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    SetPrivateMethod,

    /// Set a private setter property of a class constructor by it's name.
    ///
    /// Like `set #name() {}`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    SetPrivateSetter,

    /// Set a private getter property of a class constructor by it's name.
    ///
    /// Like `get #name() {}`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    SetPrivateGetter,

    /// Get a private property by name from an object an push it on the stack.
    ///
    /// Like `object.#name`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object **=>** value
    GetPrivateField,

    /// Push a field to a class.
    ///
    /// Operands:
    ///
    /// Stack: class, field_name, field_function **=>**
    PushClassField,

    /// Push a private field to the class.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: class, field_function **=>**
    PushClassFieldPrivate,

    /// Push a private getter to the class.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: class, getter **=>**
    PushClassPrivateGetter,

    /// Push a private setter to the class.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: class, setter **=>**
    PushClassPrivateSetter,

    /// Push a private method to the class.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: class, method **=>**
    PushClassPrivateMethod,

    /// Deletes a property by name of an object.
    ///
    /// Like `delete object.key.`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object **=>**
    DeletePropertyByName,

    /// Deletes a property by value of an object.
    ///
    /// Like `delete object[key]`
    ///
    /// Operands:
    ///
    /// Stack: key, object **=>**
    DeletePropertyByValue,

    /// Copy all properties of one object to another object.
    ///
    /// Operands: excluded_key_count: `u32`, excluded_key_count_computed: `u32`
    ///
    /// Stack: excluded_key_computed_0 ... excluded_key_computed_n, source, value, excluded_key_0 ... excluded_key_n **=>** value
    CopyDataProperties,

    /// Call ToPropertyKey on the value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** key
    ToPropertyKey,

    /// Unconditional jump to address.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>**
    Jump,

    /// Conditional jump to address.
    ///
    /// If the value popped is [`falsy`][falsy] then jump to `address`.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: cond **=>**
    ///
    /// [falsy]: https://developer.mozilla.org/en-US/docs/Glossary/Falsy
    JumpIfFalse,

    /// Conditional jump to address.
    ///
    /// If the value popped is not undefined jump to `address`.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: value **=>** value
    JumpIfNotUndefined,

    /// Throw exception
    ///
    /// Operands:
    ///
    /// Stack: value **=>**
    Throw,

    /// Start of a try block.
    ///
    /// Operands: next_address: `u32`, finally_address: `u32`
    ///
    /// Stack: **=>**
    TryStart,

    /// End of a try block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    TryEnd,

    /// Start of a catch block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    CatchStart,

    /// End of a catch block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    CatchEnd,

    /// End of a catch block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    CatchEnd2,

    /// Start of a finally block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    FinallyStart,

    /// End of a finally block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    FinallyEnd,

    /// Set the address for a finally jump.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    FinallySetJump,

    /// Pops value converts it to boolean and pushes it back.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (`ToBoolean(value)`)
    ToBoolean,

    /// Pushes `this` value
    ///
    /// Operands:
    ///
    /// Stack: **=>** this
    This,

    /// Pushes the current `super` value to the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** super
    Super,

    /// Execute the `super()` method.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: argument_1, ... argument_n **=>**
    SuperCall,

    /// Execute the `super()` method where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: arguments_array **=>**
    SuperCallSpread,

    /// Execute the `super()` method when no constructor of the class is defined.
    ///
    /// Operands:
    ///
    /// Stack: argument_1, ... argument_n **=>**
    SuperCallDerived,

    /// Pop the two values of the stack, strict equal compares the two values,
    /// if true jumps to address, otherwise push the second pop'ed value.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: value, cond **=>** cond (if `cond !== value`).
    Case,

    /// Pops the top of stack and jump to address.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: `value` **=>**
    Default,

    /// Get function from the pre-compiled inner functions.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>** func
    GetFunction,

    /// Get async function from the pre-compiled inner functions.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>** func
    GetFunctionAsync,

    /// Get generator function from the pre-compiled inner functions.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>** func
    GetGenerator,

    /// Get async generator function from the pre-compiled inner functions.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>** func
    GetGeneratorAsync,

    /// Call a function named "eval".
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, this, argument_1, ... argument_n **=>** result
    CallEval,

    /// Call a function named "eval" where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: arguments_array, func, this **=>** result
    CallEvalSpread,

    /// Call a function.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, this, argument_1, ... argument_n **=>** result
    Call,

    /// Call a function where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: arguments_array, func, this **=>** result
    CallSpread,

    /// Call construct on a function.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, argument_1, ... argument_n **=>** result
    New,

    /// Call construct on a function where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: arguments_array, func **=>** result
    NewSpread,

    /// Return from a function.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    Return,

    /// Push a declarative environment.
    ///
    /// Operands: num_bindings: `u32`, compile_environments_index: `u32`
    ///
    /// Stack: **=>**
    PushDeclarativeEnvironment,

    /// Push a function environment.
    ///
    /// Operands: num_bindings: `u32`, compile_environments_index: `u32`
    ///
    /// Stack: **=>**
    PushFunctionEnvironment,

    /// Pop the current environment.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    PopEnvironment,

    /// Push loop start marker.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    LoopStart,

    /// Clean up environments when a loop continues.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    LoopContinue,

    /// Clean up environments at the end of a loop.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    LoopEnd,

    /// Initialize the iterator for a for..in loop or jump to after the loop if object is null or undefined.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: object **=>** iterator, next_method, done
    ForInLoopInitIterator,

    /// Initialize an iterator.
    ///
    /// Operands:
    ///
    /// Stack: object **=>** iterator, next_method, done
    InitIterator,

    /// Initialize an async iterator.
    ///
    /// Operands:
    ///
    /// Stack: object **=>** iterator, next_method, done
    InitIteratorAsync,

    /// Advance the iterator by one and put the value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: iterator, next_method, done **=>** iterator, next_method, done, next_value
    IteratorNext,

    /// Close an iterator.
    ///
    /// Operands:
    ///
    /// Stack: iterator, next_method, done **=>**
    IteratorClose,

    /// Consume the iterator and construct and array with all the values.
    ///
    /// Operands:
    ///
    /// Stack: iterator, next_method, done **=>** iterator, next_method, done, array
    IteratorToArray,

    /// Move to the next value in a for..in loop or jump to exit of the loop if done.
    ///
    /// Note: next_result is only pushed if the iterator is not done.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: iterator, next_method, done **=>** iterator, next_method, done, next_result
    ForInLoopNext,

    /// Move to the next value in a for await..of loop.
    ///
    /// Operands:
    ///
    /// Stack: iterator, next_method, done **=>** iterator, next_method, next_result
    ForAwaitOfLoopIterate,

    /// Get the value from a for await..of loop next result.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: next_result **=>** done, value
    ForAwaitOfLoopNext,

    /// Concat multiple stack objects into a string.
    ///
    /// Operands: value_count: `u32`
    ///
    /// Stack: value_1,...value_n **=>** string
    ConcatToString,

    /// Call RequireObjectCoercible on the stack value.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** value
    RequireObjectCoercible,

    /// Require the stack value to be neither null nor undefined.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** value
    ValueNotNullOrUndefined,

    /// Initialize the rest parameter value of a function from the remaining arguments.
    ///
    /// Operands:
    ///
    /// Stack: `argument_1` .. `argument_n` **=>** `array`
    RestParameterInit,

    /// Pop the remaining arguments of a function.
    ///
    /// Operands:
    ///
    /// Stack: `argument_1` .. `argument_n` **=>**
    RestParameterPop,

    /// Add one to the pop on return count.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    PopOnReturnAdd,

    /// Subtract one from the pop on return count.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    PopOnReturnSub,

    /// Yield from the current execution.
    ///
    /// Operands:
    ///
    /// Stack: value **=>**
    Yield,

    /// Resumes the current generator function.
    ///
    /// Operands:
    ///
    /// Stack: received **=>**
    GeneratorNext,

    /// Resumes the current generator function.
    ///
    /// Operands:
    ///
    /// Stack: received **=>** Option<value>, skip_0, skip_1
    AsyncGeneratorNext,

    /// Delegates the current generator function another generator.
    ///
    /// Operands: done_address: `u32`
    ///
    /// Stack: iterator, next_method, done, received **=>** iterator, next_method, done
    GeneratorNextDelegate,

    /// Stops the current async function and schedules it to resume later.
    ///
    /// Operands:
    ///
    /// Stack: promise **=>**
    Await,

    /// Push the current new target to the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** new_target
    PushNewTarget,

    /// No-operation instruction, does nothing.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    // Safety: Must be last in the list since, we use this for range checking
    // in TryFrom<u8> impl.
    Nop,
}

impl Opcode {
    /// Create opcode from `u8` byte.
    ///
    /// # Safety
    ///
    /// Does not check if `u8` type is a valid `Opcode`.
    pub unsafe fn from_raw(value: u8) -> Self {
        std::mem::transmute(value)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pop => Pop::NAME,
            Self::PopIfThrown => PopIfThrown::NAME,
            Self::Dup => Dup::NAME,
            Self::Swap => Swap::NAME,
            Self::PushZero => PushZero::NAME,
            Self::PushOne => PushOne::NAME,
            Self::PushInt8 => PushInt8::NAME,
            Self::PushInt16 => PushInt16::NAME,
            Self::PushInt32 => PushInt32::NAME,
            Self::PushRational => PushRational::NAME,
            Self::PushNaN => PushNaN::NAME,
            Self::PushPositiveInfinity => PushPositiveInfinity::NAME,
            Self::PushNegativeInfinity => PushNegativeInfinity::NAME,
            Self::PushNull => PushNull::NAME,
            Self::PushTrue => PushTrue::NAME,
            Self::PushFalse => PushFalse::NAME,
            Self::PushUndefined => PushUndefined::NAME,
            Self::PushLiteral => PushLiteral::NAME,
            Self::PushEmptyObject => PushEmptyObject::NAME,
            Self::PushClassPrototype => PushClassPrototype::NAME,
            Self::SetClassPrototype => SetClassPrototype::NAME,
            Self::SetHomeObject => SetHomeObject::NAME,
            Self::PushNewArray => PushNewArray::NAME,
            Self::PushValueToArray => PushValueToArray::NAME,
            Self::PushElisionToArray => PushElisionToArray::NAME,
            Self::PushIteratorToArray => PushIteratorToArray::NAME,
            Self::Add => Add::NAME,
            Self::Sub => Sub::NAME,
            Self::Div => Div::NAME,
            Self::Mul => Mul::NAME,
            Self::Mod => Mod::NAME,
            Self::Pow => Pow::NAME,
            Self::ShiftRight => ShiftRight::NAME,
            Self::ShiftLeft => ShiftLeft::NAME,
            Self::UnsignedShiftRight => UnsignedShiftRight::NAME,
            Self::BitOr => BitOr::NAME,
            Self::BitAnd => BitAnd::NAME,
            Self::BitXor => BitXor::NAME,
            Self::BitNot => BitNot::NAME,
            Self::In => In::NAME,
            Self::Eq => Eq::NAME,
            Self::StrictEq => StrictEq::NAME,
            Self::NotEq => NotEq::NAME,
            Self::StrictNotEq => StrictNotEq::NAME,
            Self::GreaterThan => GreaterThan::NAME,
            Self::GreaterThanOrEq => GreaterThanOrEq::NAME,
            Self::LessThan => LessThan::NAME,
            Self::LessThanOrEq => LessThanOrEq::NAME,
            Self::InstanceOf => InstanceOf::NAME,
            Self::TypeOf => TypeOf::NAME,
            Self::Void => Void::NAME,
            Self::LogicalNot => LogicalNot::NAME,
            Self::LogicalAnd => LogicalAnd::NAME,
            Self::LogicalOr => LogicalOr::NAME,
            Self::Coalesce => Coalesce::NAME,
            Self::Pos => Pos::NAME,
            Self::Neg => Neg::NAME,
            Self::Inc => Inc::NAME,
            Self::IncPost => IncPost::NAME,
            Self::Dec => Dec::NAME,
            Self::DecPost => DecPost::NAME,
            Self::DefInitArg => DefInitArg::NAME,
            Self::DefVar => DefVar::NAME,
            Self::DefInitVar => DefInitVar::NAME,
            Self::DefLet => DefLet::NAME,
            Self::DefInitLet => DefInitLet::NAME,
            Self::DefInitConst => DefInitConst::NAME,
            Self::GetName => GetName::NAME,
            Self::GetNameOrUndefined => GetNameOrUndefined::NAME,
            Self::SetName => SetName::NAME,
            Self::GetPropertyByName => GetPropertyByName::NAME,
            Self::GetPropertyByValue => GetPropertyByValue::NAME,
            Self::GetPropertyByValuePush => GetPropertyByValuePush::NAME,
            Self::SetPropertyByName => SetPropertyByName::NAME,
            Self::DefineOwnPropertyByName => DefineOwnPropertyByName::NAME,
            Self::DefineClassMethodByName => DefineClassMethodByName::NAME,
            Self::SetPropertyByValue => SetPropertyByValue::NAME,
            Self::DefineOwnPropertyByValue => DefineOwnPropertyByValue::NAME,
            Self::DefineClassMethodByValue => DefineClassMethodByValue::NAME,
            Self::SetPropertyGetterByName => SetPropertyGetterByName::NAME,
            Self::DefineClassGetterByName => DefineClassGetterByName::NAME,
            Self::SetPropertyGetterByValue => SetPropertyGetterByValue::NAME,
            Self::DefineClassGetterByValue => DefineClassGetterByValue::NAME,
            Self::SetPropertySetterByName => SetPropertySetterByName::NAME,
            Self::DefineClassSetterByName => DefineClassSetterByName::NAME,
            Self::SetPropertySetterByValue => SetPropertySetterByValue::NAME,
            Self::DefineClassSetterByValue => DefineClassSetterByValue::NAME,
            Self::AssignPrivateField => AssignPrivateField::NAME,
            Self::SetPrivateField => SetPrivateField::NAME,
            Self::SetPrivateMethod => SetPrivateMethod::NAME,
            Self::SetPrivateSetter => SetPrivateSetter::NAME,
            Self::SetPrivateGetter => SetPrivateGetter::NAME,
            Self::GetPrivateField => GetPrivateField::NAME,
            Self::PushClassField => PushClassField::NAME,
            Self::PushClassFieldPrivate => PushClassFieldPrivate::NAME,
            Self::PushClassPrivateGetter => PushClassPrivateGetter::NAME,
            Self::PushClassPrivateSetter => PushClassPrivateSetter::NAME,
            Self::PushClassPrivateMethod => PushClassPrivateMethod::NAME,
            Self::DeletePropertyByName => DeletePropertyByName::NAME,
            Self::DeletePropertyByValue => DeletePropertyByValue::NAME,
            Self::CopyDataProperties => CopyDataProperties::NAME,
            Self::ToPropertyKey => ToPropertyKey::NAME,
            Self::Jump => Jump::NAME,
            Self::JumpIfFalse => JumpIfFalse::NAME,
            Self::JumpIfNotUndefined => JumpIfNotUndefined::NAME,
            Self::Throw => Throw::NAME,
            Self::TryStart => TryStart::NAME,
            Self::TryEnd => TryEnd::NAME,
            Self::CatchStart => CatchStart::NAME,
            Self::CatchEnd => CatchEnd::NAME,
            Self::CatchEnd2 => CatchEnd2::NAME,
            Self::FinallyStart => FinallyStart::NAME,
            Self::FinallyEnd => FinallyEnd::NAME,
            Self::FinallySetJump => FinallySetJump::NAME,
            Self::ToBoolean => ToBoolean::NAME,
            Self::This => This::NAME,
            Self::Super => Super::NAME,
            Self::SuperCall => SuperCall::NAME,
            Self::SuperCallSpread => SuperCallSpread::NAME,
            Self::SuperCallDerived => SuperCallDerived::NAME,
            Self::Case => Case::NAME,
            Self::Default => Default::NAME,
            Self::GetFunction => GetFunction::NAME,
            Self::GetFunctionAsync => GetFunctionAsync::NAME,
            Self::GetGenerator => GetGenerator::NAME,
            Self::GetGeneratorAsync => GetGeneratorAsync::NAME,
            Self::CallEval => CallEval::NAME,
            Self::CallEvalSpread => CallEvalSpread::NAME,
            Self::Call => Call::NAME,
            Self::CallSpread => CallSpread::NAME,
            Self::New => New::NAME,
            Self::NewSpread => NewSpread::NAME,
            Self::Return => Return::NAME,
            Self::PushDeclarativeEnvironment => PushDeclarativeEnvironment::NAME,
            Self::PushFunctionEnvironment => PushFunctionEnvironment::NAME,
            Self::PopEnvironment => PopEnvironment::NAME,
            Self::LoopStart => LoopStart::NAME,
            Self::LoopContinue => LoopContinue::NAME,
            Self::LoopEnd => LoopEnd::NAME,
            Self::ForInLoopInitIterator => ForInLoopInitIterator::NAME,
            Self::InitIterator => InitIterator::NAME,
            Self::InitIteratorAsync => InitIteratorAsync::NAME,
            Self::IteratorNext => IteratorNext::NAME,
            Self::IteratorClose => IteratorClose::NAME,
            Self::IteratorToArray => IteratorToArray::NAME,
            Self::ForInLoopNext => ForInLoopNext::NAME,
            Self::ForAwaitOfLoopNext => ForAwaitOfLoopNext::NAME,
            Self::ForAwaitOfLoopIterate => ForAwaitOfLoopIterate::NAME,
            Self::ConcatToString => ConcatToString::NAME,
            Self::RequireObjectCoercible => RequireObjectCoercible::NAME,
            Self::ValueNotNullOrUndefined => ValueNotNullOrUndefined::NAME,
            Self::RestParameterInit => RestParameterInit::NAME,
            Self::RestParameterPop => RestParameterPop::NAME,
            Self::PopOnReturnAdd => PopOnReturnAdd::NAME,
            Self::PopOnReturnSub => PopOnReturnSub::NAME,
            Self::Yield => Yield::NAME,
            Self::GeneratorNext => GeneratorNext::NAME,
            Self::AsyncGeneratorNext => AsyncGeneratorNext::NAME,
            Self::Await => Await::NAME,
            Self::PushNewTarget => PushNewTarget::NAME,
            Self::GeneratorNextDelegate => GeneratorNextDelegate::NAME,
            Self::Nop => Nop::NAME,
        }
    }

    /// Name of the profiler event for this opcode
    pub fn as_instruction_str(self) -> &'static str {
        match self {
            Self::Pop => Pop::INSTRUCTION,
            Self::PopIfThrown => PopIfThrown::INSTRUCTION,
            Self::Dup => Dup::INSTRUCTION,
            Self::Swap => Swap::INSTRUCTION,
            Self::PushZero => PushZero::INSTRUCTION,
            Self::PushOne => PushOne::INSTRUCTION,
            Self::PushInt8 => PushInt8::INSTRUCTION,
            Self::PushInt16 => PushInt16::INSTRUCTION,
            Self::PushInt32 => PushInt32::INSTRUCTION,
            Self::PushRational => PushRational::INSTRUCTION,
            Self::PushNaN => PushNaN::INSTRUCTION,
            Self::PushPositiveInfinity => PushPositiveInfinity::INSTRUCTION,
            Self::PushNegativeInfinity => PushNegativeInfinity::INSTRUCTION,
            Self::PushNull => PushNull::INSTRUCTION,
            Self::PushTrue => PushTrue::INSTRUCTION,
            Self::PushFalse => PushFalse::INSTRUCTION,
            Self::PushUndefined => PushUndefined::INSTRUCTION,
            Self::PushLiteral => PushLiteral::INSTRUCTION,
            Self::PushEmptyObject => PushEmptyObject::INSTRUCTION,
            Self::PushNewArray => PushNewArray::INSTRUCTION,
            Self::PushValueToArray => PushValueToArray::INSTRUCTION,
            Self::PushElisionToArray => PushElisionToArray::INSTRUCTION,
            Self::PushIteratorToArray => PushIteratorToArray::INSTRUCTION,
            Self::Add => Add::INSTRUCTION,
            Self::Sub => Sub::INSTRUCTION,
            Self::Div => Div::INSTRUCTION,
            Self::Mul => Mul::INSTRUCTION,
            Self::Mod => Mod::INSTRUCTION,
            Self::Pow => Pow::INSTRUCTION,
            Self::ShiftRight => ShiftRight::INSTRUCTION,
            Self::ShiftLeft => ShiftLeft::INSTRUCTION,
            Self::UnsignedShiftRight => UnsignedShiftRight::INSTRUCTION,
            Self::BitOr => BitOr::INSTRUCTION,
            Self::BitAnd => BitAnd::INSTRUCTION,
            Self::BitXor => BitXor::INSTRUCTION,
            Self::BitNot => BitNot::INSTRUCTION,
            Self::In => In::INSTRUCTION,
            Self::Eq => Eq::INSTRUCTION,
            Self::StrictEq => StrictEq::INSTRUCTION,
            Self::NotEq => NotEq::INSTRUCTION,
            Self::StrictNotEq => StrictNotEq::INSTRUCTION,
            Self::GreaterThan => GreaterThan::INSTRUCTION,
            Self::GreaterThanOrEq => GreaterThanOrEq::INSTRUCTION,
            Self::LessThan => LessThan::INSTRUCTION,
            Self::LessThanOrEq => LessThanOrEq::INSTRUCTION,
            Self::InstanceOf => InstanceOf::INSTRUCTION,
            Self::TypeOf => TypeOf::INSTRUCTION,
            Self::Void => Void::INSTRUCTION,
            Self::LogicalNot => LogicalNot::INSTRUCTION,
            Self::LogicalAnd => LogicalAnd::INSTRUCTION,
            Self::LogicalOr => LogicalOr::INSTRUCTION,
            Self::Coalesce => Coalesce::INSTRUCTION,
            Self::Pos => Pos::INSTRUCTION,
            Self::Neg => Neg::INSTRUCTION,
            Self::Inc => Inc::INSTRUCTION,
            Self::IncPost => IncPost::INSTRUCTION,
            Self::Dec => Dec::INSTRUCTION,
            Self::DecPost => DecPost::INSTRUCTION,
            Self::DefInitArg => DefInitArg::INSTRUCTION,
            Self::DefVar => DefVar::INSTRUCTION,
            Self::DefInitVar => DefInitVar::INSTRUCTION,
            Self::DefLet => DefLet::INSTRUCTION,
            Self::DefInitLet => DefInitLet::INSTRUCTION,
            Self::DefInitConst => DefInitConst::INSTRUCTION,
            Self::GetName => GetName::INSTRUCTION,
            Self::GetNameOrUndefined => GetNameOrUndefined::INSTRUCTION,
            Self::SetName => SetName::INSTRUCTION,
            Self::GetPropertyByName => GetPropertyByName::INSTRUCTION,
            Self::GetPropertyByValue => GetPropertyByValue::INSTRUCTION,
            Self::GetPropertyByValuePush => GetPropertyByValuePush::INSTRUCTION,
            Self::SetPropertyByName => SetPropertyByName::INSTRUCTION,
            Self::DefineOwnPropertyByName => DefineOwnPropertyByName::INSTRUCTION,
            Self::SetPropertyByValue => SetPropertyByValue::INSTRUCTION,
            Self::DefineOwnPropertyByValue => DefineOwnPropertyByValue::INSTRUCTION,
            Self::SetPropertyGetterByName => SetPropertyGetterByName::INSTRUCTION,
            Self::SetPropertyGetterByValue => SetPropertyGetterByValue::INSTRUCTION,
            Self::SetPropertySetterByName => SetPropertySetterByName::INSTRUCTION,
            Self::SetPropertySetterByValue => SetPropertySetterByValue::INSTRUCTION,
            Self::DeletePropertyByName => DeletePropertyByName::INSTRUCTION,
            Self::DeletePropertyByValue => DeletePropertyByValue::INSTRUCTION,
            Self::CopyDataProperties => CopyDataProperties::INSTRUCTION,
            Self::Jump => Jump::INSTRUCTION,
            Self::JumpIfFalse => JumpIfFalse::INSTRUCTION,
            Self::JumpIfNotUndefined => JumpIfNotUndefined::INSTRUCTION,
            Self::Throw => Throw::INSTRUCTION,
            Self::TryStart => TryStart::INSTRUCTION,
            Self::TryEnd => TryEnd::INSTRUCTION,
            Self::CatchStart => CatchStart::INSTRUCTION,
            Self::CatchEnd => CatchEnd::INSTRUCTION,
            Self::CatchEnd2 => CatchEnd2::INSTRUCTION,
            Self::FinallyStart => FinallyStart::INSTRUCTION,
            Self::FinallyEnd => FinallyEnd::INSTRUCTION,
            Self::FinallySetJump => FinallySetJump::INSTRUCTION,
            Self::ToBoolean => ToBoolean::INSTRUCTION,
            Self::This => This::INSTRUCTION,
            Self::Super => Super::INSTRUCTION,
            Self::SuperCall => SuperCall::INSTRUCTION,
            Self::SuperCallSpread => SuperCallSpread::INSTRUCTION,
            Self::SuperCallDerived => SuperCallDerived::INSTRUCTION,
            Self::Case => Case::INSTRUCTION,
            Self::Default => Default::INSTRUCTION,
            Self::GetFunction => GetFunction::INSTRUCTION,
            Self::GetFunctionAsync => GetFunctionAsync::INSTRUCTION,
            Self::GetGenerator => GetGenerator::INSTRUCTION,
            Self::GetGeneratorAsync => GetGeneratorAsync::INSTRUCTION,
            Self::CallEval => CallEval::INSTRUCTION,
            Self::CallEvalSpread => CallEvalSpread::INSTRUCTION,
            Self::Call => Call::INSTRUCTION,
            Self::CallSpread => CallSpread::INSTRUCTION,
            Self::New => New::INSTRUCTION,
            Self::NewSpread => NewSpread::INSTRUCTION,
            Self::Return => Return::INSTRUCTION,
            Self::PushDeclarativeEnvironment => PushDeclarativeEnvironment::INSTRUCTION,
            Self::PushFunctionEnvironment => PushFunctionEnvironment::INSTRUCTION,
            Self::PopEnvironment => PopEnvironment::INSTRUCTION,
            Self::LoopStart => LoopStart::INSTRUCTION,
            Self::LoopContinue => LoopContinue::INSTRUCTION,
            Self::LoopEnd => LoopEnd::INSTRUCTION,
            Self::ForInLoopInitIterator => ForInLoopInitIterator::INSTRUCTION,
            Self::InitIterator => InitIterator::INSTRUCTION,
            Self::InitIteratorAsync => InitIteratorAsync::INSTRUCTION,
            Self::IteratorNext => IteratorNext::INSTRUCTION,
            Self::IteratorClose => IteratorClose::INSTRUCTION,
            Self::IteratorToArray => IteratorToArray::INSTRUCTION,
            Self::ForInLoopNext => ForInLoopNext::INSTRUCTION,
            Self::ForAwaitOfLoopIterate => ForAwaitOfLoopIterate::INSTRUCTION,
            Self::ForAwaitOfLoopNext => ForAwaitOfLoopNext::INSTRUCTION,
            Self::ConcatToString => ConcatToString::INSTRUCTION,
            Self::RequireObjectCoercible => RequireObjectCoercible::INSTRUCTION,
            Self::ValueNotNullOrUndefined => ValueNotNullOrUndefined::INSTRUCTION,
            Self::RestParameterInit => RestParameterInit::INSTRUCTION,
            Self::RestParameterPop => RestParameterPop::INSTRUCTION,
            Self::PopOnReturnAdd => PopOnReturnAdd::INSTRUCTION,
            Self::PopOnReturnSub => PopOnReturnSub::INSTRUCTION,
            Self::Yield => Yield::INSTRUCTION,
            Self::GeneratorNext => GeneratorNext::INSTRUCTION,
            Self::AsyncGeneratorNext => AsyncGeneratorNext::INSTRUCTION,
            Self::PushNewTarget => PushNewTarget::INSTRUCTION,
            Self::Await => Await::INSTRUCTION,
            Self::GeneratorNextDelegate => GeneratorNextDelegate::INSTRUCTION,
            Self::Nop => Nop::INSTRUCTION,
            Self::PushClassPrototype => PushClassPrototype::INSTRUCTION,
            Self::SetClassPrototype => SetClassPrototype::INSTRUCTION,
            Self::SetHomeObject => SetHomeObject::INSTRUCTION,
            Self::DefineClassMethodByName => DefineClassMethodByName::INSTRUCTION,
            Self::DefineClassMethodByValue => DefineClassMethodByValue::INSTRUCTION,
            Self::DefineClassGetterByName => DefineClassGetterByName::INSTRUCTION,
            Self::DefineClassGetterByValue => DefineClassGetterByValue::INSTRUCTION,
            Self::DefineClassSetterByName => DefineClassSetterByName::INSTRUCTION,
            Self::DefineClassSetterByValue => DefineClassSetterByValue::INSTRUCTION,
            Self::AssignPrivateField => AssignPrivateField::INSTRUCTION,
            Self::SetPrivateField => SetPrivateField::INSTRUCTION,
            Self::SetPrivateMethod => SetPrivateMethod::INSTRUCTION,
            Self::SetPrivateSetter => SetPrivateSetter::INSTRUCTION,
            Self::SetPrivateGetter => SetPrivateGetter::INSTRUCTION,
            Self::GetPrivateField => GetPrivateField::INSTRUCTION,
            Self::PushClassField => PushClassField::INSTRUCTION,
            Self::PushClassFieldPrivate => PushClassFieldPrivate::INSTRUCTION,
            Self::PushClassPrivateGetter => PushClassPrivateGetter::INSTRUCTION,
            Self::PushClassPrivateSetter => PushClassPrivateSetter::INSTRUCTION,
            Self::PushClassPrivateMethod => PushClassPrivateMethod::INSTRUCTION,
            Self::ToPropertyKey => ToPropertyKey::INSTRUCTION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidOpcodeError {
    value: u8,
}

impl std::fmt::Display for InvalidOpcodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid opcode: {:#04x}", self.value)
    }
}

impl std::error::Error for InvalidOpcodeError {}

impl TryFrom<u8> for Opcode {
    type Error = InvalidOpcodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::Nop as u8 {
            return Err(InvalidOpcodeError { value });
        }

        // Safety: we already checked if it is in the Opcode range,
        // so this is safe.
        let opcode = unsafe { Self::from_raw(value) };

        Ok(opcode)
    }
}

/// Specific opcodes for bindings.
///
/// This separate enum exists to make matching exhaustive where needed.
#[derive(Clone, Copy, Debug)]
pub(crate) enum BindingOpcode {
    Var,
    Let,
    InitVar,
    InitLet,
    InitArg,
    InitConst,
    SetName,
}
