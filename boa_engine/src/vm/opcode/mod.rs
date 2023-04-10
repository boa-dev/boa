/// The opcodes of the vm.
use crate::{vm::CompletionType, Context, JsResult};

use num_enum::TryFromPrimitive;

// Operation modules
mod await_stm;
mod binary_ops;
mod call;
mod concat;
mod control_flow;
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
mod push;
mod require;
mod rest_parameter;
mod set;
mod swap;
mod switch;
mod to;
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
pub(crate) use control_flow::*;
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
pub(crate) use push::*;
#[doc(inline)]
pub(crate) use require::*;
#[doc(inline)]
pub(crate) use rest_parameter::*;
#[doc(inline)]
pub(crate) use set::*;
#[doc(inline)]
pub(crate) use swap::*;
#[doc(inline)]
pub(crate) use switch::*;
#[doc(inline)]
pub(crate) use to::*;
#[doc(inline)]
pub(crate) use unary_ops::*;
#[doc(inline)]
pub(crate) use value::*;

macro_rules! generate_impl {
    (
        $(#[$outer:meta])*
        pub enum $Type:ident {
            $(
                $(#[$inner:ident $($args:tt)*])*
                $Variant:ident $(= $index:expr)*
            ),*
            $(,)?
        }
    ) => {
        /// The opcodes of the vm.
        $(#[$outer])*
        pub enum $Type {
            $(
                $(#[$inner $($args)*])*
                $Variant $(= $index)*
            ),*
        }

        impl $Type {

            /// Create opcode from `u8` byte.
            ///
            /// # Safety
            ///
            /// Does not check if `u8` type is a valid `Opcode`.
            #[must_use]
            pub unsafe fn from_raw(value: u8) -> Self {
                // Safety:
                // The caller is responsible for ensuring that the value is a valid opcode.
                unsafe { std::mem::transmute(value) }
            }

            const NAMES: &[&'static str] = &[
                $($Variant::NAME),*
            ];

            /// Name of this opcode.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                Self::NAMES[self as usize]
            }

            const INSTRUCTIONS: &[&'static str] = &[
                $($Variant::INSTRUCTION),*
            ];

            /// Name of the profiler event for this opcode.
            #[must_use]
            pub const fn as_instruction_str(self) -> &'static str {
                Self::INSTRUCTIONS[self as usize]
            }

            const EXECUTE_FNS: &[fn(&mut Context<'_>) -> JsResult<CompletionType>] = &[
                $($Variant::execute),*
            ];

            pub(super) fn execute(self, context: &mut Context<'_>) -> JsResult<CompletionType> {
                Self::EXECUTE_FNS[self as usize](context)
            }
        }
    };
}

/// The `Operation` trait implements the execution code along with the
/// identifying Name and Instruction value for an Boa Opcode.
///
/// This trait should be implemented for a struct that corresponds with
/// any arm of the `OpCode` enum.
///
pub(crate) trait Operation {
    const NAME: &'static str;
    const INSTRUCTION: &'static str;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType>;
}

generate_impl! {
    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[repr(u8)]
    pub enum Opcode {
        /// Pop the top value from the stack.
        ///
        /// Operands:
        ///
        /// Stack: value **=>**
        Pop = 0,

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

        /// Rotates the top `n` values of the stack to the left by `1`.
        ///
        /// Equivalent to calling [`slice::rotate_left`] with argument `1` on the top `n` values of the
        /// stack.
        ///
        /// Operands: n: `u8`
        ///
        /// Stack: v\[n\], v\[n-1\], ... , v\[1\], v\[0\] **=>** v\[n-1\], ... , v\[1\], v\[0\], v\[n\]
        RotateLeft,

        /// Rotates the top `n` values of the stack to the right by `1`.
        ///
        /// Equivalent to calling [`slice::rotate_right`] with argument `1` on the top `n` values of the
        /// stack.
        ///
        /// Operands: n: `u8`
        ///
        /// Stack: v\[n\], v\[n-1\], ... , v\[1\], v\[0\] **=>** v\[0\], v\[n\], v\[n-1\], ... , v\[1\]
        RotateRight,

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

        /// Set the prototype of an object if the value is an object or null.
        ///
        /// Operands:
        ///
        /// Stack: object, value **=>**
        SetPrototype,

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
        /// Stack: array, iterator, next_method **=>** array
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
        /// Stack: lhs, rhs **=>** `(lhs << rhs)`
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

        /// Binary `in` operator for private names.
        ///
        /// Operands: private_name_index: `u32`
        ///
        /// Stack: rhs **=>** (private_name `in` rhs)
        InPrivate,

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
        /// Stack: lhs, rhs **=>** `(lhs < rhs)`
        LessThan,

        /// Binary `<=` operator.
        ///
        /// Operands:
        ///
        /// Stack: lhs, rhs **=>** `(lhs <= rhs)`
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
        /// Stack: value **=>**
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

        /// Deletes a property of the global object.
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: **=>** deleted
        DeleteName,

        /// Get a property by name from an object an push it on the stack.
        ///
        /// Like `object.name`
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: object **=>** value
        GetPropertyByName,

        /// Get a property method or undefined if the property is null or undefined.
        ///
        /// Throws if the method is not a callable object.
        ///
        /// Operands: name_index: `u32`
        /// Stack: object **=>** object, method
        GetMethod,

        /// Get a property by value from an object an push it on the stack.
        ///
        /// Like `object[key]`
        ///
        /// Operands:
        ///
        /// Stack: object, key **=>** value
        GetPropertyByValue,

        /// Get a property by value from an object an push the key and value on the stack.
        ///
        /// Like `object[key]`
        ///
        /// Operands:
        ///
        /// Stack: object, key **=>** key, value
        GetPropertyByValuePush,

        /// Sets a property by name of an object.
        ///
        /// Like `object.name = value`
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: object, receiver, value **=>** value
        SetPropertyByName,

        /// Sets the name of a function object.
        ///
        /// This operation is corresponds to the `SetFunctionName` abstract operation in the [spec].
        ///
        ///  The prefix operand is mapped as follows:
        /// * 0 -> no prefix
        /// * 1 -> "get "
        /// * 2 -> "set "
        ///
        ///  Operands: prefix: `u8`
        ///
        /// Stack: name, function **=>** function
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-setfunctionname
        SetFunctionName,

        /// Defines a own property of an object by name.
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: object, value **=>**
        DefineOwnPropertyByName,

        /// Defines a static class method by name.
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: class, function **=>**
        DefineClassStaticMethodByName,

        /// Defines a class method by name.
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: class_proto, function **=>**
        DefineClassMethodByName,

        /// Sets a property by value of an object.
        ///
        /// Like `object[key] = value`
        ///
        /// Operands:
        ///
        /// Stack: object, key, value **=>** value
        SetPropertyByValue,

        /// Defines a own property of an object by value.
        ///
        /// Operands:
        ///
        /// Stack: object, key, value **=>**
        DefineOwnPropertyByValue,

        /// Defines a static class method by value.
        ///
        /// Operands:
        ///
        /// Stack: class, key, function **=>**
        DefineClassStaticMethodByValue,

        /// Defines a class method by value.
        ///
        /// Operands:
        ///
        /// Stack: class_proto, key, function **=>**
        DefineClassMethodByValue,

        /// Sets a getter property by name of an object.
        ///
        /// Like `get name() value`
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: object, value **=>**
        SetPropertyGetterByName,

        /// Defines a static getter class method by name.
        ///
        /// Like `static get name() value`
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: class, function **=>**
        DefineClassStaticGetterByName,

        /// Defines a getter class method by name.
        ///
        /// Like `get name() value`
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: class_proto, function **=>** class
        DefineClassGetterByName,

        /// Sets a getter property by value of an object.
        ///
        /// Like `get [key]() value`
        ///
        /// Operands:
        ///
        /// Stack: object, key, value **=>**
        SetPropertyGetterByValue,

        /// Defines a static getter class method by value.
        ///
        /// Like `static get [key]() value`
        ///
        /// Operands:
        ///
        /// Stack: class, key, function **=>**
        DefineClassStaticGetterByValue,

        /// Defines a getter class method by value.
        ///
        /// Like `get [key]() value`
        ///
        /// Operands:
        ///
        /// Stack: class_proto, key, function **=>**
        DefineClassGetterByValue,

        /// Sets a setter property by name of an object.
        ///
        /// Like `set name() value`
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: object, value **=>**
        SetPropertySetterByName,

        /// Defines a static setter class method by name.
        ///
        /// Like `static set name() value`
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: class, function **=>**
        DefineClassStaticSetterByName,

        /// Defines a setter class method by name.
        ///
        /// Like `set name() value`
        ///
        /// Operands: name_index: `u32`
        ///
        /// Stack: class_proto, function **=>**
        DefineClassSetterByName,

        /// Sets a setter property by value of an object.
        ///
        /// Like `set [key]() value`
        ///
        /// Operands:
        ///
        /// Stack: object, key, value **=>**
        SetPropertySetterByValue,

        /// Defines a static setter class method by value.
        ///
        /// Like `static set [key]() value`
        ///
        /// Operands:
        ///
        /// Stack: class, key, function **=>**
        DefineClassStaticSetterByValue,

        /// Defines a setter class method by value.
        ///
        /// Like `set [key]() value`
        ///
        /// Operands:
        ///
        /// Stack: class_proto, key, function **=>**
        DefineClassSetterByValue,

        /// Set the value of a private property of an object by it's name.
        ///
        /// Like `obj.#name = value`
        ///
        /// Operands: private_name_index: `u32`
        ///
        /// Stack: object, value **=>** value
        SetPrivateField,

        /// Define a private property of a class constructor by it's name.
        ///
        /// Like `#name = value`
        ///
        /// Operands: private_name_index: `u32`
        ///
        /// Stack: object, value **=>**
        DefinePrivateField,

        /// Set a private method of a class constructor by it's name.
        ///
        /// Like `#name() {}`
        ///
        /// Operands: private_name_index: `u32`
        ///
        /// Stack: object, value **=>**
        SetPrivateMethod,

        /// Set a private setter property of a class constructor by it's name.
        ///
        /// Like `set #name() {}`
        ///
        /// Operands: private_name_index: `u32`
        ///
        /// Stack: object, value **=>**
        SetPrivateSetter,

        /// Set a private getter property of a class constructor by it's name.
        ///
        /// Like `get #name() {}`
        ///
        /// Operands: private_name_index: `u32`
        ///
        /// Stack: object, value **=>**
        SetPrivateGetter,

        /// Get a private property by name from an object an push it on the stack.
        ///
        /// Like `object.#name`
        ///
        /// Operands: private_name_index: `u32`
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
        /// Operands: private_name_index: `u32`
        ///
        /// Stack: class, field_function **=>**
        PushClassFieldPrivate,

        /// Push a private getter to the class.
        ///
        /// Operands: private_name_index: `u32`
        ///
        /// Stack: class, getter **=>**
        PushClassPrivateGetter,

        /// Push a private setter to the class.
        ///
        /// Operands: private_name_index: `u32`
        ///
        /// Stack: class, setter **=>**
        PushClassPrivateSetter,

        /// Push a private method to the class.
        ///
        /// Operands: private_name_index: `u32`
        ///
        /// Stack: class, method **=>**
        PushClassPrivateMethod,

        /// Deletes a property by name of an object.
        ///
        /// Like `delete object.key`
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
        /// Stack: object, key **=>**
        DeletePropertyByValue,

        /// Throws an error when trying to delete a property of `super`
        ///
        /// Operands:
        ///
        /// Stack: **=>**
        DeleteSuperThrow,

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
        /// If the value popped is [`truthy`][truthy] then jump to `address`.
        ///
        /// Operands: address: `u32`
        ///
        /// Stack: cond **=>**
        ///
        /// [truthy]: https://developer.mozilla.org/en-US/docs/Glossary/Truthy
        JumpIfTrue,

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

        /// Conditional jump to address.
        ///
        /// If the value popped is undefined jump to `address`.
        ///
        /// Operands: address: `u32`
        ///
        /// Stack: value **=>** value
        JumpIfNullOrUndefined,

        /// Throw exception
        ///
        /// Operands:
        ///
        /// Stack: value **=>**
        Throw,

        /// Throw a new `TypeError` exception
        ///
        /// Operands: message: u32
        ///
        /// Stack: **=>**
        ThrowNewTypeError,

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
        /// Operands: finally_address: `u32`
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

        /// Jumps to a target location and pops the environments involved.
        ///
        /// Operands: Jump Address: u32, Target address: u32
        ///
        /// Stack: **=>**
        Break,

        /// Sets the `AbruptCompletionRecord` for a delayed continue
        ///
        /// Operands: Jump Address: u32, Target address: u32,
        ///
        /// Stack: **=>**
        Continue,

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

        /// Get the super constructor and the new target of the current environment.
        ///
        /// Operands:
        ///
        /// Stack: **=>** super_constructor, new_target
        SuperCallPrepare,

        /// Execute the `super()` method.
        ///
        /// Operands: argument_count: `u32`
        ///
        /// Stack: super_constructor, new_target, argument_1, ... argument_n **=>**
        SuperCall,

        /// Execute the `super()` method where the arguments contain spreads.
        ///
        /// Operands:
        ///
        /// Stack: super_constructor, new_target, arguments_array **=>**
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

        /// Get arrow function from the pre-compiled inner functions.
        ///
        /// Operands: address: `u32`, method: `u8`
        ///
        /// Stack: **=>** func
        GetArrowFunction,

        /// Get async arrow function from the pre-compiled inner functions.
        ///
        /// Operands: address: `u32`, method: `u8`
        ///
        /// Stack: **=>** func
        GetAsyncArrowFunction,

        /// Get function from the pre-compiled inner functions.
        ///
        /// Operands: address: `u32`, method: `u8`
        ///
        /// Stack: **=>** func
        GetFunction,

        /// Get async function from the pre-compiled inner functions.
        ///
        /// Operands: address: `u32`, method: `u8`
        ///
        /// Stack: **=>** func
        GetFunctionAsync,

        /// Get generator function from the pre-compiled inner functions.
        ///
        /// Operands: address: `u32`, method: `u8`
        ///
        /// Stack: **=>** func
        GetGenerator,

        /// Get async generator function from the pre-compiled inner functions.
        ///
        /// Operands: address: `u32`, method: `u8`
        ///
        /// Stack: **=>** func
        GetGeneratorAsync,

        /// Call a function named "eval".
        ///
        /// Operands: argument_count: `u32`
        ///
        /// Stack: this, func, argument_1, ... argument_n **=>** result
        CallEval,

        /// Call a function named "eval" where the arguments contain spreads.
        ///
        /// Operands:
        ///
        /// Stack: this, func, arguments_array **=>** result
        CallEvalSpread,

        /// Call a function.
        ///
        /// Operands: argument_count: `u32`
        ///
        /// Stack: this, func, argument_1, ... argument_n **=>** result
        Call,

        /// Call a function where the arguments contain spreads.
        ///
        /// Operands:
        ///
        /// Stack: this, func, arguments_array **=>** result
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

        /// Push an object environment.
        ///
        /// Operands:
        ///
        /// Stack: object **=>**
        PushObjectEnvironment,

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
        /// Operands: Exit Address: `u32`
        ///
        /// Stack: **=>**
        LoopStart,

        /// Clean up environments when a loop continues.
        ///
        /// Operands: Start Address: `u32`, Exit Address: `u32`
        ///
        /// Stack: **=>**
        LoopContinue,

        /// Clean up environments at the end of a loop.
        ///
        /// Operands:
        ///
        /// Stack: **=>**
        LoopEnd,

        /// Push labelled start marker.
        ///
        /// Operands: Exit Address: u32,
        ///
        /// Stack: **=>**
        LabelledStart,

        /// Clean up environments at the end of a labelled block.
        ///
        /// Operands:
        ///
        /// Stack: **=>**
        LabelledEnd,

        /// Creates the ForInIterator of an object.
        ///
        /// Stack: object **=>** iterator, next_method
        CreateForInIterator,

        /// Gets the iterator of an object.
        ///
        /// Operands:
        ///
        /// Stack: object **=>** iterator, next_method
        GetIterator,

        /// Gets the async iterator of an object.
        ///
        /// Operands:
        ///
        /// Stack: object **=>** iterator, next_method
        GetAsyncIterator,

        /// Calls the `next` method of `iterator` and puts its return value on the stack.
        ///
        /// Operands:
        ///
        /// Stack: iterator, next_method **=>** iterator, next_method, next_value
        IteratorNext,

        /// Calls the `next` method of `iterator`, puts its return value on the stack
        /// and sets the `[[Done]]` value of the iterator on the call frame.
        ///
        /// Operands:
        ///
        /// Stack: iterator, next_method **=>** iterator, next_method, next_value
        IteratorNextSetDone,

        /// Gets the `value` and `done` properties of an iterator result.
        ///
        /// Stack: next_result **=>** done, next_value
        IteratorUnwrapNext,

        /// Gets the `value` property of an iterator result.
        ///
        /// Stack: next_result **=>** next_value
        IteratorUnwrapValue,

        /// Gets the `value` and `done` properties of an iterator result, or jump to `address` if
        /// `done` is true.
        ///
        /// Operands: address: `u32`
        ///
        /// Stack: next_result **=>** done, next_value ( if done != true  )
        IteratorUnwrapNextOrJump,

        /// Consume the iterator and construct and array with all the values.
        ///
        /// Operands:
        ///
        /// Stack: iterator, next_method **=>** iterator, next_method, array
        IteratorToArray,

        /// Push an iterator to the call frame close iterator stack.
        ///
        /// Operands:
        ///
        /// Stack: iterator, next_method => iterator, next_method
        IteratorClosePush,

        /// Pop an iterator from the call frame close iterator stack.
        ///
        /// Operands:
        ///
        /// Stack:
        IteratorClosePop,

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
        /// Operands: skip_yield: u32, skip_yield_await: u32
        ///
        /// Stack: received **=>** `Option<value>`
        AsyncGeneratorNext,

        /// Delegates the current generator function to another generator.
        ///
        /// Operands: done_address: `u32`
        ///
        /// Stack: iterator, next_method, received **=>** iterator, next_method
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

        /// Pushes `true` to the stack if the top stack value is an object, or `false` otherwise.
        ///
        /// Operands:
        ///
        /// Stack: value **=>** is_object
        IsObject,

        /// No-operation instruction, does nothing.
        ///
        /// Operands:
        ///
        /// Stack: **=>**
        // Safety: Must be last in the list since, we use this for range checking
        // in `TryFrom<u8>` impl.
        Nop,
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
