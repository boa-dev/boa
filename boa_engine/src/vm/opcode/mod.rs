/// The opcodes of the vm.
use crate::{vm::ShouldExit, Context, JsResult};

// Operation modules
pub(crate) mod await_stm;
pub(crate) mod call;
pub(crate) mod concat;
pub(crate) mod conversion;
pub(crate) mod copy;
pub(crate) mod define;
pub(crate) mod delete;
pub(crate) mod dup;
pub(crate) mod environment;
pub(crate) mod general;
pub(crate) mod generator;
pub(crate) mod get;
pub(crate) mod iteration;
pub(crate) mod new;
pub(crate) mod pop;
pub(crate) mod promise;
pub(crate) mod push;
pub(crate) mod require;
pub(crate) mod rest_parameter;
pub(crate) mod return_stm;
pub(crate) mod set;
pub(crate) mod swap;
pub(crate) mod switch;
pub(crate) mod throw;
pub(crate) mod try_catch;
pub(crate) mod value;

// Operation structs
pub(crate) use await_stm::*;
pub(crate) use call::*;
pub(crate) use concat::*;
pub(crate) use conversion::*;
pub(crate) use copy::*;
pub(crate) use define::*;
pub(crate) use delete::*;
pub(crate) use dup::*;
pub(crate) use environment::*;
pub(crate) use general::*;
pub(crate) use generator::*;
pub(crate) use get::*;
pub(crate) use iteration::*;
pub(crate) use new::*;
pub(crate) use pop::*;
pub(crate) use promise::*;
pub(crate) use push::*;
pub(crate) use require::*;
pub(crate) use rest_parameter::*;
pub(crate) use return_stm::*;
pub(crate) use set::*;
pub(crate) use swap::*;
pub(crate) use switch::*;
pub(crate) use throw::*;
pub(crate) use try_catch::*;
pub(crate) use value::*;

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
    Pop(Pop),

    /// Pop the top value from the stack if the last try block has thrown a value.
    ///
    /// Operands:
    ///
    /// Stack: value **=>**
    PopIfThrown(PopIfThrown),

    /// Push a copy of the top value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** value, value
    Dup(Dup),

    /// Swap the top two values on the stack.
    ///
    /// Operands:
    ///
    /// Stack: second, first **=>** first, second
    Swap(Swap),

    /// Push integer `0` on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `0`
    PushZero(PushZero),

    /// Push integer `1` on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `1`
    PushOne(PushZero),

    /// Push `i8` value on the stack.
    ///
    /// Operands: value: `i8`
    ///
    /// Stack: **=>** value
    PushInt8(PushInt8),

    /// Push i16 value on the stack.
    ///
    /// Operands: value: `i16`
    ///
    /// Stack: **=>** value
    PushInt16(PushInt16),

    /// Push i32 value on the stack.
    ///
    /// Operands: value: `i32`
    ///
    /// Stack: **=>** value
    PushInt32(PushInt32),

    /// Push `f64` value on the stack.
    ///
    /// Operands: value: `f64`
    ///
    /// Stack: **=>** value
    PushRational(PushRational),

    /// Push `NaN` integer on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `NaN`
    PushNaN(PushNaN),

    /// Push `Infinity` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `Infinity`
    PushPositiveInfinity(PushPositiveInfinity),

    /// Push `-Infinity` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `-Infinity`
    PushNegativeInfinity(PushNegativeInfinity),

    /// Push `null` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `null`
    PushNull(PushNull),

    /// Push `true` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `true`
    PushTrue(PushTrue),

    /// Push `false` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `false`
    PushFalse(PushFalse),

    /// Push `undefined` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `undefined`
    PushUndefined(PushUndefined),

    /// Push literal value on the stack.
    ///
    /// Like strings and bigints. The index operand is used to index into the `literals`
    /// array to get the value.
    ///
    /// Operands: index: `u32`
    ///
    /// Stack: **=>** (`literals[index]`)
    PushLiteral(PushLiteral),

    /// Push empty object `{}` value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `{}`
    PushEmptyObject(PushEmptyObject),

    /// Get the prototype of a superclass and push it on the stack.
    ///
    /// Operands:
    ///
    /// Stack: class, superclass **=>** class, superclass.prototype
    PushClassPrototype(PushClassPrototype),

    /// Set the prototype of a class object.
    ///
    /// Operands:
    ///
    /// Stack: class, prototype **=>** class.prototype
    SetClassPrototype(SetClassPrototype),

    /// Set home object internal slot of a function object.
    ///
    /// Operands:
    ///
    /// Stack: home, function **=>** home, function
    SetHomeObject(SetHomeObject),

    /// Push an empty array value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** `[]`
    PushNewArray(PushNewArray),

    /// Push a value to an array.
    ///
    /// Operands:
    ///
    /// Stack: array, value **=>** array
    PushValueToArray(PushValueToArray),

    /// Push an empty element/hole to an array.
    ///
    /// Operands:
    ///
    /// Stack: array **=>** array
    PushElisionToArray(PushElisionToArray),

    /// Push all iterator values to an array.
    ///
    /// Operands:
    ///
    /// Stack: array, iterator, next_method, done **=>** array
    PushIteratorToArray(PushIteratorToArray),

    /// Binary `+` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs + rhs)
    Add(Add),

    /// Binary `-` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs - rhs)
    Sub(Sub),

    /// Binary `/` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs / rhs)
    Div(Div),

    /// Binary `*` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs * rhs)
    Mul(Mul),

    /// Binary `%` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs % rhs)
    Mod(Mod),

    /// Binary `**` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs ** rhs)
    Pow(Pow),

    /// Binary `>>` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs >> rhs)
    ShiftRight(ShiftRight),

    /// Binary `<<` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs << rhs)
    ShiftLeft(ShiftLeft),

    /// Binary `>>>` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs >>> rhs)
    UnsignedShiftRight(UnsignedShiftRight),

    /// Binary bitwise `|` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs | rhs)
    BitOr(BitOr),

    /// Binary bitwise `&` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs & rhs)
    BitAnd(BitAnd),

    /// Binary bitwise `^` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs ^ rhs)
    BitXor(BitXor),

    /// Unary bitwise `~` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** ~value
    BitNot(BitNot),

    /// Binary `in` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs `in` rhs)
    In(In),

    /// Binary `==` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs `==` rhs)
    Eq(Eq),

    /// Binary `===` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs `===` rhs)
    StrictEq(StrictEq),

    /// Binary `!=` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs `!=` rhs)
    NotEq(NotEq),

    /// Binary `!==` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs `!==` rhs)
    StrictNotEq(StrictNotEq),

    /// Binary `>` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs > rhs)
    GreaterThan(GreaterThan),

    /// Binary `>=` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs >= rhs)
    GreaterThanOrEq(GreaterThanOrEq),

    /// Binary `<` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs < rhs)
    LessThan(LessThan),

    /// Binary `<=` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs <= rhs)
    LessThanOrEq(LessThanOrEq),

    /// Binary `instanceof` operator.
    ///
    /// Operands:
    ///
    /// Stack: lhs, rhs **=>** (lhs instanceof rhs)
    InstanceOf(InstanceOf),

    /// Binary logical `&&` operator.
    ///
    /// This is a short-circuit operator, if the `lhs` value is `false`, then it jumps to `exit` address.
    ///
    /// Operands: exit: `u32`
    ///
    /// Stack: lhs, rhs **=>** (lhs && rhs)
    LogicalAnd(LogicalAnd),

    /// Binary logical `||` operator.
    ///
    /// This is a short-circuit operator, if the `lhs` value is `true`, then it jumps to `exit` address.
    ///
    /// Operands: exit: `u32`
    ///
    /// Stack: lhs, rhs **=>** (lhs || rhs)
    LogicalOr(LogicalOr),

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
    TypeOf(TypeOf),

    /// Unary `void` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** `undefined`
    Void(Void),

    /// Unary logical `!` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (!value)
    LogicalNot(LogicalNot),

    /// Unary `+` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (+value)
    Pos(Pos),

    /// Unary `-` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (-value)
    Neg(Neg),

    /// Unary `++` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (value + 1)
    Inc(Inc),

    /// Unary postfix `++` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (ToNumeric(value)), (value + 1)
    IncPost(IncPost),

    /// Unary `--` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (value - 1)
    Dec(Dec),

    /// Unary postfix `--` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (ToNumeric(value)), (value - 1)
    DecPost(DecPost),

    /// Declare and initialize a function argument.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value **=>**
    DefInitArg(DefInitArg),

    /// Declare `var` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>**
    DefVar(DefVar),

    /// Declare and initialize `var` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, has_declarative_binding **=>**
    DefInitVar(DefInitVar),

    /// Declare `let` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>**
    DefLet(DefLet),

    /// Declare and initialize `let` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value **=>**
    DefInitLet(DefInitLet),

    /// Declare and initialize `const` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value **=>**
    DefInitConst(DefInitConst),

    /// Find a binding on the environment chain and push its value.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>** value
    GetName(GetName),

    /// Find a binding on the environment chain and push its value. If the binding does not exist push undefined.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>** value
    GetNameOrUndefined(GetNameOrUndefined),

    /// Find a binding on the environment chain and assign its value.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value **=>**
    SetName(SetName),

    /// Get a property by name from an object an push it on the stack.
    ///
    /// Like `object.name`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object **=>** value
    GetPropertyByName(GetPropertyByName),

    /// Get a property by value from an object an push it on the stack.
    ///
    /// Like `object[key]`
    ///
    /// Operands:
    ///
    /// Stack: key, object **=>** value
    GetPropertyByValue(GetPropertyByValue),

    /// Get a property by value from an object an push the key and value on the stack.
    ///
    /// Like `object[key]`
    ///
    /// Operands:
    ///
    /// Stack: key, object **=>** key, value
    GetPropertyByValuePush(GetPropertyByValuePush),

    /// Sets a property by name of an object.
    ///
    /// Like `object.name = value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    SetPropertyByName(SetPropertyByName),

    /// Defines a own property of an object by name.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    DefineOwnPropertyByName(DefineOwnPropertyByName),

    /// Defines a class method by name.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    DefineClassMethodByName(DefineClassMethodByName),

    /// Sets a property by value of an object.
    ///
    /// Like `object[key] = value`
    ///
    /// Operands:
    ///
    /// Stack: value, key, object **=>**
    SetPropertyByValue(SetPropertyByValue),

    /// Defines a own property of an object by value.
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    DefineOwnPropertyByValue(DefineOwnPropertyByValue),

    /// Defines a class method by value.
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    DefineClassMethodByValue(DefineClassMethodByValue),

    /// Sets a getter property by name of an object.
    ///
    /// Like `get name() value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    SetPropertyGetterByName(SetPropertyGetterByName),

    /// Defines a getter class method by name.
    ///
    /// Like `get name() value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    DefineClassGetterByName(DefineClassGetterByName),

    /// Sets a getter property by value of an object.
    ///
    /// Like `get [key]() value`
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    SetPropertyGetterByValue(SetPropertyGetterByValue),

    /// Defines a getter class method by value.
    ///
    /// Like `get [key]() value`
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    DefineClassGetterByValue(DefineClassGetterByValue),

    /// Sets a setter property by name of an object.
    ///
    /// Like `set name() value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    SetPropertySetterByName(SetPropertySetterByName),

    /// Defines a setter class method by name.
    ///
    /// Like `set name() value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    DefineClassSetterByName(DefineClassSetterByName),

    /// Sets a setter property by value of an object.
    ///
    /// Like `set [key]() value`
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    SetPropertySetterByValue(SetPropertySetterByValue),

    /// Defines a setter class method by value.
    ///
    /// Like `set [key]() value`
    ///
    /// Operands:
    ///
    /// Stack: object, key, value **=>**
    DefineClassSetterByValue(DefineClassSetterByValue),

    /// Assign the value of a private property of an object by it's name.
    ///
    /// Like `obj.#name = value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    AssignPrivateField(AssignPrivateField),

    /// Set a private property of a class constructor by it's name.
    ///
    /// Like `#name = value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    SetPrivateField(SetPrivateField),

    /// Set a private method of a class constructor by it's name.
    ///
    /// Like `#name() {}`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    SetPrivateMethod(SetPrivateMethod),

    /// Set a private setter property of a class constructor by it's name.
    ///
    /// Like `set #name() {}`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    SetPrivateSetter(SetPrivateSetter),

    /// Set a private getter property of a class constructor by it's name.
    ///
    /// Like `get #name() {}`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    SetPrivateGetter(SetPrivateGetter),

    /// Get a private property by name from an object an push it on the stack.
    ///
    /// Like `object.#name`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object **=>** value
    GetPrivateField(GetPrivateField),

    /// Push a field to a class.
    ///
    /// Operands:
    ///
    /// Stack: class, field_name, field_function **=>**
    PushClassField(PushClassField),

    /// Push a private field to the class.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: class, field_function **=>**
    PushClassFieldPrivate(PushClassFieldPrivate),

    /// Push a private getter to the class.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: class, getter **=>**
    PushClassPrivateGetter(PushClassPrivateGetter),

    /// Push a private setter to the class.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: class, setter **=>**
    PushClassPrivateSetter(PushClassPrivateSetter),

    /// Push a private method to the class.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: class, method **=>**
    PushClassPrivateMethod(PushClassPrivateMethod),

    /// Deletes a property by name of an object.
    ///
    /// Like `delete object.key.`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object **=>**
    DeletePropertyByName(DeletePropertyByName),

    /// Deletes a property by value of an object.
    ///
    /// Like `delete object[key]`
    ///
    /// Operands:
    ///
    /// Stack: key, object **=>**
    DeletePropertyByValue(DeletePropertyByValue),

    /// Copy all properties of one object to another object.
    ///
    /// Operands: excluded_key_count: `u32`, excluded_key_count_computed: `u32`
    ///
    /// Stack: excluded_key_computed_0 ... excluded_key_computed_n, source, value, excluded_key_0 ... excluded_key_n **=>** value
    CopyDataProperties(CopyDataProperties),

    /// Call ToPropertyKey on the value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** key
    ToPropertyKey(ToPropertyKey),

    /// Unconditional jump to address.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>**
    Jump(Jump),

    /// Conditional jump to address.
    ///
    /// If the value popped is [`falsy`][falsy] then jump to `address`.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: cond **=>**
    ///
    /// [falsy]: https://developer.mozilla.org/en-US/docs/Glossary/Falsy
    JumpIfFalse(Jump),

    /// Conditional jump to address.
    ///
    /// If the value popped is not undefined jump to `address`.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: value **=>** value
    JumpIfNotUndefined(Jump),

    /// Throw exception
    ///
    /// Operands:
    ///
    /// Stack: value **=>**
    Throw(Throw),

    /// Start of a try block.
    ///
    /// Operands: next_address: `u32`, finally_address: `u32`
    ///
    /// Stack: **=>**
    TryStart(TryStart),

    /// End of a try block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    TryEnd(TryEnd),

    /// Start of a catch block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    CatchStart(CatchStart),

    /// End of a catch block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    CatchEnd(CatchEnd),

    /// End of a catch block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    CatchEnd2(CatchEnd2),

    /// Start of a finally block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    FinallyStart(FinallyStart),

    /// End of a finally block.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    FinallyEnd(FinallyEnd),

    /// Set the address for a finally jump.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    FinallySetJump(FinallySetJump),

    /// Pops value converts it to boolean and pushes it back.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (`ToBoolean(value)`)
    ToBoolean(ToBoolean),

    /// Pushes `this` value
    ///
    /// Operands:
    ///
    /// Stack: **=>** this
    This(This),

    /// Pushes the current `super` value to the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** super
    Super(Super),

    /// Execute the `super()` method.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: argument_1, ... argument_n **=>**
    SuperCall(SuperCall),

    /// Execute the `super()` method where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: arguments_array **=>**
    SuperCallSpread(SuperCallSpread),

    /// Execute the `super()` method when no constructor of the class is defined.
    ///
    /// Operands:
    ///
    /// Stack: argument_1, ... argument_n **=>**
    SuperCallDerived(SuperCallDerived),

    /// Pop the two values of the stack, strict equal compares the two values,
    /// if true jumps to address, otherwise push the second pop'ed value.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: value, cond **=>** cond (if `cond !== value`).
    Case(Case),

    /// Pops the top of stack and jump to address.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: `value` **=>**
    Default(Default),

    /// Get function from the pre-compiled inner functions.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>** func
    GetFunction(GetFunction),

    /// Get async function from the pre-compiled inner functions.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>** func
    GetFunctionAsync(GetFunctionAsync),

    /// Get generator function from the pre-compiled inner functions.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>** func
    GetGenerator(GetGenerator),

    /// Get async generator function from the pre-compiled inner functions.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>** func
    GetGeneratorAsync(GetGeneratorAsync),

    /// Call a function named "eval".
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, this, argument_1, ... argument_n **=>** result
    CallEval(CallEval),

    /// Call a function named "eval" where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: arguments_array, func, this **=>** result
    CallEvalSpread(CallEvalSpread),

    /// Call a function.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, this, argument_1, ... argument_n **=>** result
    Call(Call),

    /// Call a function where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: arguments_array, func, this **=>** result
    CallSpread(CallSpread),

    /// Call construct on a function.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, argument_1, ... argument_n **=>** result
    New(New),

    /// Call construct on a function where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: arguments_array, func **=>** result
    NewSpread(NewSpread),

    /// Return from a function.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    Return(Return),

    /// Push a declarative environment.
    ///
    /// Operands: num_bindings: `u32`, compile_environments_index: `u32`
    ///
    /// Stack: **=>**
    PushDeclarativeEnvironment(PushDeclarativeEnvironment),

    /// Push a function environment.
    ///
    /// Operands: num_bindings: `u32`, compile_environments_index: `u32`
    ///
    /// Stack: **=>**
    PushFunctionEnvironment(PushFunctionEnvironment),

    /// Pop the current environment.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    PopEnvironment(PopEnvironment),

    /// Push loop start marker.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    LoopStart(LoopStart),

    /// Clean up environments when a loop continues.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    LoopContinue(LoopContinue),

    /// Clean up environments at the end of a loop.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    LoopEnd(LoopEnd),

    /// Initialize the iterator for a for..in loop or jump to after the loop if object is null or undefined.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: object **=>** iterator, next_method, done
    ForInLoopInitIterator(ForInLoopInitIterator),

    /// Initialize an iterator.
    ///
    /// Operands:
    ///
    /// Stack: object **=>** iterator, next_method, done
    InitIterator(InitIterator),

    /// Initialize an async iterator.
    ///
    /// Operands:
    ///
    /// Stack: object **=>** iterator, next_method, done
    InitIteratorAsync(InitIteratorAsync),

    /// Advance the iterator by one and put the value on the stack.
    ///
    /// Operands:
    ///
    /// Stack: iterator, next_method, done **=>** iterator, next_method, done, next_value
    IteratorNext(IteratorNext),

    /// Close an iterator.
    ///
    /// Operands:
    ///
    /// Stack: iterator, next_method, done **=>**
    IteratorClose(IteratorClose),

    /// Consume the iterator and construct and array with all the values.
    ///
    /// Operands:
    ///
    /// Stack: iterator, next_method, done **=>** iterator, next_method, done, array
    IteratorToArray(IteratorToArray),

    /// Move to the next value in a for..in loop or jump to exit of the loop if done.
    ///
    /// Note: next_result is only pushed if the iterator is not done.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: iterator, next_method, done **=>** iterator, next_method, done, next_result
    ForInLoopNext(ForInLoopNext),

    /// Move to the next value in a for await..of loop.
    ///
    /// Operands:
    ///
    /// Stack: iterator, next_method, done **=>** iterator, next_method, next_result
    ForAwaitOfLoopIterate(ForAwaitOfLoopIterate),

    /// Get the value from a for await..of loop next result.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: next_result **=>** done, value
    ForAwaitOfLoopNext(ForAwaitOfLoopNext),

    /// Concat multiple stack objects into a string.
    ///
    /// Operands: value_count: `u32`
    ///
    /// Stack: value_1,...value_n **=>** string
    ConcatToString(ConcatToString),

    /// Call RequireObjectCoercible on the stack value.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** value
    RequireObjectCoercible(RequireObjectCoercible),

    /// Require the stack value to be neither null nor undefined.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** value
    ValueNotNullOrUndefined(ValueNotNullOrUndefined),

    /// Initialize the rest parameter value of a function from the remaining arguments.
    ///
    /// Operands:
    ///
    /// Stack: `argument_1` .. `argument_n` **=>** `array`
    RestParameterInit(RestParameterInit),

    /// Pop the remaining arguments of a function.
    ///
    /// Operands:
    ///
    /// Stack: `argument_1` .. `argument_n` **=>**
    RestParameterPop(RestParameterPop),

    /// Add one to the pop on return count.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    PopOnReturnAdd(PopOnReturnAdd),

    /// Subtract one from the pop on return count.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    PopOnReturnSub(PopOnReturnSub),

    /// Yield from the current execution.
    ///
    /// Operands:
    ///
    /// Stack: value **=>**
    Yield(Yield),

    /// Resumes the current generator function.
    ///
    /// Operands:
    ///
    /// Stack: received **=>**
    GeneratorNext(GeneratorNext),

    /// Resumes the current generator function.
    ///
    /// Operands:
    ///
    /// Stack: received **=>** Option<value>, skip_0, skip_1
    AsyncGeneratorNext(AsyncGeneratorNext),

    /// Delegates the current generator function another generator.
    ///
    /// Operands: done_address: `u32`
    ///
    /// Stack: iterator, next_method, done, received **=>** iterator, next_method, done
    GeneratorNextDelegate(GeneratorNextDelegate),

    /// Stops the current async function and schedules it to resume later.
    ///
    /// Operands:
    ///
    /// Stack: promise **=>**
    Await(Await),

    /// Push the current new target to the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** new_target
    PushNewTarget(PushNewTarget),

    /// No-operation instruction, does nothing.
    ///
    /// Operands:
    ///
    /// Stack: **=>**
    // Safety: Must be last in the list since, we use this for range checking
    // in TryFrom<u8> impl.
    Nop(Nop),
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
            Self::Pop(op) => Pop::NAME,
            Self::PopIfThrown(op) => PopIfThrown::NAME,
            Self::Dup(op) => Dup::NAME,
            Self::Swap(op) => Swap::NAME,
            Self::PushZero(op) => PushZero::NAME,
            Self::PushOne(op) => PushOne::NAME,
            Self::PushInt8(op) => PushInt8::NAME,
            Self::PushInt16(op) => PushInt16::NAME,
            Self::PushInt32(op) => PushInt32::NAME,
            Self::PushRational(op) => PushRational::NAME,
            Self::PushNaN(op) => PushNaN::NAME,
            Self::PushPositiveInfinity(op) => PushPositiveInfinity::NAME,
            Self::PushNegativeInfinity(op) => PushNegativeInfinity::NAME,
            Self::PushNull(op) => PushNull::NAME,
            Self::PushTrue(op) => PushTrue::NAME,
            Self::PushFalse(op) => PushFalse::NAME,
            Self::PushUndefined(op) => PushUndefined::NAME,
            Self::PushLiteral(op) => PushLiteral::NAME,
            Self::PushEmptyObject(op) => PushEmptyObject::NAME,
            Self::PushClassPrototype(op) => PushClassPrototype::NAME,
            Self::SetClassPrototype(op) => SetClassPrototype::NAME,
            Self::SetHomeObject(op) => SetHomeObject::NAME,
            Self::PushNewArray(op) => PushNewArray::NAME,
            Self::PushValueToArray(op) => PushValueToArray::NAME,
            Self::PushElisionToArray(op) => PushElisionToArray::NAME,
            Self::PushIteratorToArray(op) => PushIteratorToArray::NAME,
            Self::Add(op) => Add::NAME,
            Self::Sub(op) => Sub::NAME,
            Self::Div(op) => Div::NAME,
            Self::Mul(op) => Mul::NAME,
            Self::Mod(op) => Mod::NAME,
            Self::Pow(op) => Pow::NAME,
            Self::ShiftRight(op) => ShiftRight::NAME,
            Self::ShiftLeft(op) => ShiftLeft::NAME,
            Self::UnsignedShiftRight(op) => UnsignedShiftRight::NAME,
            Self::BitOr(op) => BitOr::NAME,
            Self::BitAnd(op) => BitAnd::NAME,
            Self::BitXor(op) => BitXor::NAME,
            Self::BitNot(op) => "BitNot",
            Self::In(op) => "In",
            Self::Eq(op) => Eq::NAME,
            Self::StrictEq(op) => "StrictEq",
            Self::NotEq(op) => "NotEq",
            Self::StrictNotEq(op) => "StrictNotEq",
            Self::GreaterThan(op) => GreaterThan::NAME,
            Self::GreaterThanOrEq(op) => GreaterThanOrEq::NAME,
            Self::LessThan(op) => LessThan::NAME,
            Self::LessThanOrEq(op) => LessThanOrEq::NAME,
            Self::InstanceOf(op) => InstanceOf::NAME,
            Self::TypeOf(op) => TypeOf::NAME,
            Self::Void(op) => Void::NAME,
            Self::LogicalNot(op) => LogicalNot::NAME,
            Self::LogicalAnd(op) => "LogicalAnd",
            Self::LogicalOr(op) => "LogicalOr",
            Self::Coalesce(op) => "Coalesce",
            Self::Pos(op) => Pos::NAME,
            Self::Neg(op) => Neg::NAME,
            Self::Inc(op) => Inc::NAME,
            Self::IncPost(op) => IncPost::NAME,
            Self::Dec(op) => Dec::NAME,
            Self::DecPost(op) => DecPost::NAME,
            Self::DefInitArg(op) => DefInitArg::NAME,
            Self::DefVar(op) => DefVar::NAME,
            Self::DefInitVar(op) => DefInitVar::NAME,
            Self::DefLet(op) => DefLet::NAME,
            Self::DefInitLet(op) => DefInitLet::NAME,
            Self::DefInitConst(op) => DefInitConst::NAME,
            Self::GetName(op) => GetName::NAME,
            Self::GetNameOrUndefined(op) => GetNameOrUndefined::NAME,
            Self::SetName(op) => SetName::NAME,
            Self::GetPropertyByName(op) => GetPropertyByName::NAME,
            Self::GetPropertyByValue(op) => GetPropertyByValue::NAME,
            Self::GetPropertyByValuePush(op) => GetPropertyByValuePush::NAME,
            Self::SetPropertyByName(op) => "SetPropertyByName",
            Self::DefineOwnPropertyByName(op) => "DefineOwnPropertyByName",
            Self::DefineClassMethodByName(op) => "DefineClassMethodByName",
            Self::SetPropertyByValue(op) => "SetPropertyByValue",
            Self::DefineOwnPropertyByValue(op) => "DefineOwnPropertyByValue",
            Self::DefineClassMethodByValue(op) => "DefineClassMethodByValue",
            Self::SetPropertyGetterByName(op) => "SetPropertyGetterByName",
            Self::DefineClassGetterByName(op) => "DefineClassGetterByName",
            Self::SetPropertyGetterByValue(op) => "SetPropertyGetterByValue",
            Self::DefineClassGetterByValue(op) => "DefineClassGetterByValue",
            Self::SetPropertySetterByName(op) => "SetPropertySetterByName",
            Self::DefineClassSetterByName(op) => "DefineClassSetterByName",
            Self::SetPropertySetterByValue(op) => "SetPropertySetterByValue",
            Self::DefineClassSetterByValue(op) => "DefineClassSetterByValue",
            Self::AssignPrivateField(op) => "AssignPrivateField",
            Self::SetPrivateField(op) => "SetPrivateValue",
            Self::SetPrivateMethod(op) => "SetPrivateMethod",
            Self::SetPrivateSetter(op) => "SetPrivateSetter",
            Self::SetPrivateGetter(op) => "SetPrivateGetter",
            Self::GetPrivateField(op) => "GetPrivateField",
            Self::PushClassField(op) => "PushClassField",
            Self::PushClassFieldPrivate(op) => "PushClassFieldPrivate",
            Self::PushClassPrivateGetter(op) => "PushClassPrivateGetter",
            Self::PushClassPrivateSetter(op) => "PushClassPrivateSetter",
            Self::PushClassPrivateMethod(op) => "PushClassPrivateMethod",
            Self::DeletePropertyByName(op) => "DeletePropertyByName",
            Self::DeletePropertyByValue(op) => "DeletePropertyByValue",
            Self::CopyDataProperties(op) => "CopyDataProperties",
            Self::ToPropertyKey(op) => "ToPropertyKey",
            Self::Jump(op) => Jump::NAME,
            Self::JumpIfFalse(op) => JumpIfFalse::NAME,
            Self::JumpIfNotUndefined(op) => JumpIfNotUndefined::NAME,
            Self::Throw(op) => "Throw",
            Self::TryStart(op) => "TryStart",
            Self::TryEnd(op) => "TryEnd",
            Self::CatchStart(op) => "CatchStart",
            Self::CatchEnd(op) => "CatchEnd",
            Self::CatchEnd2(op) => "CatchEnd2",
            Self::FinallyStart(op) => "FinallyStart",
            Self::FinallyEnd(op) => "FinallyEnd",
            Self::FinallySetJump(op) => "FinallySetJump",
            Self::ToBoolean(op) => "ToBoolean",
            Self::This(op) => "This",
            Self::Super(op) => "Super",
            Self::SuperCall(op) => "SuperCall",
            Self::SuperCallSpread(op) => "SuperCallWithRest",
            Self::SuperCallDerived(op) => "SuperCallDerived",
            Self::Case(op) => "Case",
            Self::Default(op) => "Default",
            Self::GetFunction(op) => "GetFunction",
            Self::GetFunctionAsync(op) => "GetFunctionAsync",
            Self::GetGenerator(op) => "GetGenerator",
            Self::GetGeneratorAsync(op) => "GetGeneratorAsync",
            Self::CallEval(op) => "CallEval",
            Self::CallEvalSpread(op) => "CallEvalSpread",
            Self::Call(op) => "Call",
            Self::CallSpread(op) => "CallSpread",
            Self::New(op) => "New",
            Self::NewSpread(op) => "NewSpread",
            Self::Return(op) => "Return",
            Self::PushDeclarativeEnvironment(op) => "PushDeclarativeEnvironment",
            Self::PushFunctionEnvironment(op) => "PushFunctionEnvironment",
            Self::PopEnvironment(op) => "PopEnvironment",
            Self::LoopStart(op) => "LoopStart",
            Self::LoopContinue(op) => "LoopContinue",
            Self::LoopEnd(op) => "LoopEnd",
            Self::ForInLoopInitIterator(op) => "ForInLoopInitIterator",
            Self::InitIterator(op) => "InitIterator",
            Self::InitIteratorAsync(op) => "InitIteratorAsync",
            Self::IteratorNext(op) => "IteratorNext",
            Self::IteratorClose(op) => "IteratorClose",
            Self::IteratorToArray(op) => "IteratorToArray",
            Self::ForInLoopNext(op) => "ForInLoopNext",
            Self::ForAwaitOfLoopNext(op) => "ForAwaitOfLoopNext",
            Self::ForAwaitOfLoopIterate(op) => "ForAwaitOfLoopIterate",
            Self::ConcatToString(op) => "ConcatToString",
            Self::RequireObjectCoercible(op) => "RequireObjectCoercible",
            Self::ValueNotNullOrUndefined(op) => "ValueNotNullOrUndefined",
            Self::RestParameterInit(op) => "FunctionRestParameter",
            Self::RestParameterPop(op) => "RestParameterPop",
            Self::PopOnReturnAdd(op) => "PopOnReturnAdd",
            Self::PopOnReturnSub(op) => "PopOnReturnSub",
            Self::Yield(op) => "Yield",
            Self::GeneratorNext(op) => "GeneratorNext",
            Self::AsyncGeneratorNext(op) => "AsyncGeneratorNext",
            Self::Await(op) => "Await",
            Self::PushNewTarget(op) => "PushNewTarget",
            Self::GeneratorNextDelegate(op) => "GeneratorNextDelegate",
            Self::Nop(op) => "Nop",
        }
    }

    /// Name of the profiler event for this opcode
    pub fn as_instruction_str(self) -> &'static str {
        match self {
            Self::Pop(op) => Pop::INSTRUCTION,
            Self::PopIfThrown(op) => PopIfThrown::INSTRUCTION,
            Self::Dup(op) => Dup::INSTRUCTION,
            Self::Swap(op) => Swap::INSTRUCTION,
            Self::PushZero(op) => PushZero::INSTRUCTION,
            Self::PushOne(op) => PushOne::INSTRUCTION,
            Self::PushInt8(op) => PushInt8::INSTRUCTION,
            Self::PushInt16(op) => PushInt16::INSTRUCTION,
            Self::PushInt32(op) => PushInt32::INSTRUCTION,
            Self::PushRational(op) => PushRational::INSTRUCTION,
            Self::PushNaN(op) => PushNaN::INSTRUCTION,
            Self::PushPositiveInfinity(op) => PushPositiveInfinity::INSTRUCTION,
            Self::PushNegativeInfinity(op) => PushNegativeInfinity::INSTRUCTION,
            Self::PushNull(op) => PushNull::INSTRUCTION,
            Self::PushTrue(op) => PushTrue::INSTRUCTION,
            Self::PushFalse(op) => PushFalse::INSTRUCTION,
            Self::PushUndefined(op) => PushUndefined::INSTRUCTION,
            Self::PushLiteral(op) => PushLiteral::INSTRUCTION,
            Self::PushEmptyObject(op) => PushEmptyObject::INSTRUCTION,
            Self::PushNewArray(op) => PushNewArray::INSTRUCTION,
            Self::PushValueToArray(op) => PushValueToArray::INSTRUCTION,
            Self::PushElisionToArray(op) => PushElisionToArray::INSTRUCTION,
            Self::PushIteratorToArray(op) => PushIteratorToArray::INSTRUCTION,
            Self::Add(op) => Add::INSTRUCTION,
            Self::Sub(op) => Sub::INSTRUCTION,
            Self::Div(op) => Div::INSTRUCTION,
            Self::Mul(op) => Mul::INSTRUCTION,
            Self::Mod(op) => Mod::INSTRUCTION,
            Self::Pow(op) => Pow::INSTRUCTION,
            Self::ShiftRight(op) => ShiftRight::INSTRUCTION,
            Self::ShiftLeft(op) => ShiftLeft::INSTRUCTION,
            Self::UnsignedShiftRight(op) => UnsignedShiftRight::INSTRUCTION,
            Self::BitOr(op) => BitOr::INSTRUCTION,
            Self::BitAnd(op) => BitAnd::INSTRUCTION,
            Self::BitXor(op) => BitXor::INSTRUCTION,
            Self::BitNot(op) => BitNot::INSTRUCTION,
            Self::In(op) => In::INSTRUCTION,
            Self::Eq(op) => Eq::INSTRUCTION,
            Self::StrictEq(op) => StrictEq::INSTRUCTION,
            Self::NotEq(op) => NotEq::INSTRUCTION,
            Self::StrictNotEq(op) => StrictNotEq::INSTRUCTION,
            Self::GreaterThan(op) => GreaterThan::INSTRUCTION,
            Self::GreaterThanOrEq(op) => GreaterThanOrEq::INSTRUCTION,
            Self::LessThan(op) => LessThan::INSTRUCTION,
            Self::LessThanOrEq(op) => LessThanOrEq::INSTRUCTION,
            Self::InstanceOf(op) => InstanceOf::INSTRUCTION,
            Self::TypeOf(op) => TypeOf::INSTRUCTION,
            Self::Void(op) => Void::INSTRUCTION,
            Self::LogicalNot(op) => LogicalNot::INSTRUCTION,
            Self::LogicalAnd(op) => "INST - LogicalAnd",
            Self::LogicalOr(op) => "INST - LogicalOr",
            Self::Coalesce(op) => "INST - Coalesce",
            Self::Pos(op) => Pos::INSTRUCTION,
            Self::Neg(op) => Neg::INSTRUCTION,
            Self::Inc(op) => Inc::INSTRUCTION,
            Self::IncPost(op) => IncPost::INSTRUCTION,
            Self::Dec(op) => Dec::INSTRUCTION,
            Self::DecPost(op) => DecPost::INSTRUCTION,
            Self::DefInitArg(op) => DefInitArg::INSTRUCTION,
            Self::DefVar(op) => DefVar::INSTRUCTION,
            Self::DefInitVar(op) => DefInitVar::INSTRUCTION,
            Self::DefLet(op) => DefLet::INSTRUCTION,
            Self::DefInitLet(op) => DefInitLet::INSTRUCTION,
            Self::DefInitConst(op) => DefInitConst::INSTRUCTION,
            Self::GetName(op) => GetName::INSTRUCTION,
            Self::GetNameOrUndefined(op) => GetNameOrUndefined::INSTRUCTION,
            Self::SetName(op) => SetName::INSTRUCTION,
            Self::GetPropertyByName(op) => GetPropertyByName::INSTRUCTION,
            Self::GetPropertyByValue(op) => GetPropertyByValue::INSTRUCTION,
            Self::GetPropertyByValuePush(op) => GetPropertyByValuePush::INSTRUCTION,
            Self::SetPropertyByName(op) => "INST - SetPropertyByName",
            Self::DefineOwnPropertyByName(op) => "INST - DefineOwnPropertyByName",
            Self::SetPropertyByValue(op) => "INST - SetPropertyByValue",
            Self::DefineOwnPropertyByValue(op) => "INST - DefineOwnPropertyByValue",
            Self::SetPropertyGetterByName(op) => "INST - SetPropertyGetterByName",
            Self::SetPropertyGetterByValue(op) => "INST - SetPropertyGetterByValue",
            Self::SetPropertySetterByName(op) => "INST - SetPropertySetterByName",
            Self::SetPropertySetterByValue(op) => "INST - SetPropertySetterByValue",
            Self::DeletePropertyByName(op) => "INST - DeletePropertyByName",
            Self::DeletePropertyByValue(op) => "INST - DeletePropertyByValue",
            Self::CopyDataProperties(op) => "INST - CopyDataProperties",
            Self::Jump(op) => Jump::INSTRUCTION,
            Self::JumpIfFalse(op) => JumpIfFalse::INSTRUCTION,
            Self::JumpIfNotUndefined(op) => JumpIfNotUndefined::INSTRUCTION,
            Self::Throw(op) => "INST - Throw",
            Self::TryStart(op) => "INST - TryStart",
            Self::TryEnd(op) => "INST - TryEnd",
            Self::CatchStart(op) => "INST - CatchStart",
            Self::CatchEnd(op) => "INST - CatchEnd",
            Self::CatchEnd2(op) => "INST - CatchEnd2",
            Self::FinallyStart(op) => "INST - FinallyStart",
            Self::FinallyEnd(op) => "INST - FinallyEnd",
            Self::FinallySetJump(op) => "INST - FinallySetJump",
            Self::ToBoolean(op) => "INST - ToBoolean",
            Self::This(op) => "INST - This",
            Self::Super(op) => "INST - Super",
            Self::SuperCall(op) => "INST - SuperCall",
            Self::SuperCallSpread(op) => "INST - SuperCallWithRest",
            Self::SuperCallDerived(op) => "INST - SuperCallDerived",
            Self::Case(op) => "INST - Case",
            Self::Default(op) => "INST - Default",
            Self::GetFunction(op) => "INST - GetFunction",
            Self::GetFunctionAsync(op) => "INST - GetFunctionAsync",
            Self::GetGenerator(op) => "INST - GetGenerator",
            Self::GetGeneratorAsync(op) => "INST - GetGeneratorAsync",
            Self::CallEval(op) => "INST - CallEval",
            Self::CallEvalSpread(op) => "INST - CallEvalSpread",
            Self::Call(op) => "INST - Call",
            Self::CallSpread(op) => "INST - CallSpread",
            Self::New(op) => "INST - New",
            Self::NewSpread(op) => "INST - NewSpread",
            Self::Return(op) => "INST - Return",
            Self::PushDeclarativeEnvironment(op) => "INST - PushDeclarativeEnvironment",
            Self::PushFunctionEnvironment(op) => "INST - PushFunctionEnvironment",
            Self::PopEnvironment(op) => "INST - PopEnvironment",
            Self::LoopStart(op) => "INST - LoopStart",
            Self::LoopContinue(op) => "INST - LoopContinue",
            Self::LoopEnd(op) => "INST - LoopEnd",
            Self::ForInLoopInitIterator(op) => "INST - ForInLoopInitIterator",
            Self::InitIterator(op) => "INST - InitIterator",
            Self::InitIteratorAsync(op) => "INST - InitIteratorAsync",
            Self::IteratorNext(op) => "INST - IteratorNext",
            Self::IteratorClose(op) => "INST - IteratorClose",
            Self::IteratorToArray(op) => "INST - IteratorToArray",
            Self::ForInLoopNext(op) => "INST - ForInLoopNext",
            Self::ForAwaitOfLoopIterate(op) => "INST - ForAwaitOfLoopIterate",
            Self::ForAwaitOfLoopNext(op) => "INST - ForAwaitOfLoopNext",
            Self::ConcatToString(op) => "INST - ConcatToString",
            Self::RequireObjectCoercible(op) => "INST - RequireObjectCoercible",
            Self::ValueNotNullOrUndefined(op) => "INST - ValueNotNullOrUndefined",
            Self::RestParameterInit(op) => "INST - FunctionRestParameter",
            Self::RestParameterPop(op) => "INST - RestParameterPop",
            Self::PopOnReturnAdd(op) => "INST - PopOnReturnAdd",
            Self::PopOnReturnSub(op) => "INST - PopOnReturnSub",
            Self::Yield(op) => "INST - Yield",
            Self::GeneratorNext(op) => "INST - GeneratorNext",
            Self::AsyncGeneratorNext(op) => "INST - AsyncGeneratorNext",
            Self::PushNewTarget(op) => "INST - PushNewTarget",
            Self::Await(op) => "INST - Await",
            Self::GeneratorNextDelegate(op) => "INST - GeneratorNextDelegate",
            Self::Nop(op) => "INST - Nop",
            Self::PushClassPrototype(op) => PushClassPrototype::INSTRUCTION,
            Self::SetClassPrototype(op) => SetClassPrototype::INSTRUCTION,
            Self::SetHomeObject(op) => SetHomeObject::INSTRUCTION,
            Self::DefineClassMethodByName(op) => "INST - DefineClassMethodByName",
            Self::DefineClassMethodByValue(op) => "INST - DefineClassMethodByValue",
            Self::DefineClassGetterByName(op) => "INST - DefineClassGetterByName",
            Self::DefineClassGetterByValue(op) => "INST - DefineClassGetterByValue",
            Self::DefineClassSetterByName(op) => "INST - DefineClassSetterByName",
            Self::DefineClassSetterByValue(op) => "INST - DefineClassSetterByValue",
            Self::AssignPrivateField(op) => "INST - AssignPrivateField",
            Self::SetPrivateField(op) => "INST - SetPrivateValue",
            Self::SetPrivateMethod(op) => "INST - SetPrivateMethod",
            Self::SetPrivateSetter(op) => "INST - SetPrivateSetter",
            Self::SetPrivateGetter(op) => "INST - SetPrivateGetter",
            Self::GetPrivateField(op) => "INST - GetPrivateField",
            Self::PushClassField(op) => "INST - PushClassField",
            Self::PushClassFieldPrivate(op) => "INST - PushClassFieldPrivate",
            Self::PushClassPrivateGetter(op) => "INST - PushClassPrivateGetter",
            Self::PushClassPrivateSetter(op) => "INST - PushClassPrivateSetter",
            Self::PushClassPrivateMethod(op) => "INST - PushClassPrivateMethod",
            Self::ToPropertyKey(op) => "INST - ToPropertyKey",
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
