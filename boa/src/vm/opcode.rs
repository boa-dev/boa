use std::convert::TryFrom;

/// The opcodes of the vm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    /// Pop the top value from the stack.
    ///
    /// Operands:
    ///
    /// Stack: value **=>**
    Pop,

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
    /// Stack: v1, v2 **=>** v2, v1
    Swap,

    /// Swap the top value and the third value of the stack.
    ///
    /// Operands:
    ///
    /// Stack: v1, v2, v3 **=>** v3, v2, v1
    Swap2,

    /// Push integer `0` on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** 0
    PushZero,

    /// Push integer `1` on the stack.
    ///
    /// Operands:
    ///
    /// Stack: **=>** 1
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

    /// Push `NaN` teger on the stack.
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
    /// Like strings and bigints. The index oprand is used to index into the `literals`
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
    /// Stack: **=>** object
    PushEmptyObject,

    /// Push an empty array value on the stack.
    ///
    /// Stack: **=>** `array`
    PushNewArray,

    /// Push a value to an array.
    ///
    /// Stack: `array`, `value` **=>** `array`
    PushValueToArray,

    /// Push all iterator values to an array.
    ///
    /// Stack: `array`, `iterator`, `next_function` **=>** `array`
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
    /// This is a short-circit operator, if the `lhs` value is `false`, then it jumps to `exit` address.
    ///
    /// Operands: exit: `u32`
    ///
    /// Stack: lhs, rhs **=>** (lhs && rhs)
    LogicalAnd,

    /// Binary logical `||` operator.
    ///
    /// This is a short-circit operator, if the `lhs` value is `true`, then it jumps to `exit` address.
    ///
    /// Operands: exit: `u32`
    ///
    /// Stack: lhs, rhs **=>** (lhs || rhs)
    LogicalOr,

    /// Binary `??` operator.
    ///
    /// This is a short-circit operator, if the `lhs` value is **not** `null` or `undefined`,
    /// then it jumps to `exit` address.
    ///
    /// Operands: exit: `u32`
    ///
    /// Stack: lhs, rhs **=>** (lhs && rhs)
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

    /// Unary `--` operator.
    ///
    /// Operands:
    ///
    /// Stack: value **=>** (value - 1)
    Dec,

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

    /// Sets a property by name of an object.
    ///
    /// Like `object.name = value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    SetPropertyByName,

    /// Sets a property by value of an object.
    ///
    /// Like `object[key] = value`
    ///
    /// Operands:
    ///
    /// Stack: value, key, object **=>**
    SetPropertyByValue,

    /// Sets a getter property by name of an object.
    ///
    /// Like `get name() value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    SetPropertyGetterByName,

    /// Sets a getter property by value of an object.
    ///
    /// Like `get [key]() value`
    ///
    /// Operands:
    ///
    /// Stack: value, key, object **=>**
    SetPropertyGetterByValue,

    /// Sets a setter property by name of an object.
    ///
    /// Like `set name() value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: value, object **=>**
    SetPropertySetterByName,

    /// Sets a setter property by value of an object.
    ///
    /// Like `set [key]() value`
    ///
    /// Operands:
    ///
    /// Stack: value, key, object **=>**
    SetPropertySetterByValue,

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
    /// Operands: number of excluded keys: `u32`
    ///
    /// Stack: object, rest_object, excluded_key_0 ... excluded_key_n **=>** object
    CopyDataProperties,

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
    /// Stack: `exc` **=>**
    Throw,

    /// Start of a try block.
    ///
    /// Operands: address: `u32`
    TryStart,

    /// End of a try block.
    TryEnd,

    /// Start of a finally block.
    FinallyStart,

    /// End of a finally block.
    FinallyEnd,

    /// Set the address for a finally jump.
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
    /// Stack: **=>** `this`
    This,

    /// Pop the two values of the stack, strict equal compares the two values,
    /// if true jumps to address, otherwise push the second poped value.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: `value`, `cond` **=>** `cond` (if `cond !== value`).
    Case,

    /// Pops the top of stack and jump to address.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: `value` **=>**
    Default,

    /// Get function from the precompiled inner functions.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>** `func`
    GetFunction,

    /// Call a function.
    ///
    /// Operands: argc: `u32`
    ///
    /// Stack: `func`, `this`, `arg1`, `arg2`,...`argn` **=>**
    Call,

    /// Call a function where the last argument is a rest parameter.
    ///
    /// Operands: argc: `u32`
    ///
    /// Stack: `func`, `this`, `arg1`, `arg2`,...`argn` **=>**
    CallWithRest,

    /// Call construct on a function.
    ///
    /// Operands: argc: `u32`
    ///
    /// Stack: `func`, `arg1`, `arg2`,...`argn` **=>**
    New,

    /// Call construct on a function where the last argument is a rest parameter.
    ///
    /// Operands: argc: `u32`
    ///
    /// Stack: `func`, `arg1`, `arg2`,...`argn` **=>**
    NewWithRest,

    /// Return from a function.
    Return,

    /// Push a declarative environment.
    PushDeclarativeEnvironment,

    /// Push a function environment.
    PushFunctionEnvironment,

    /// Pop the current environment.
    PopEnvironment,

    /// Initialize the iterator for a for..in loop or jump to after the loop if object is null or undefined.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: `object` **=>** `for_in_iterator`, `next_function`
    ForInLoopInitIterator,

    /// Initialize an iterator.
    ///
    /// Stack: `object` **=>** `iterator`, `next_function`
    InitIterator,

    /// Advance the iterator by one and put the value on the stack.
    ///
    /// Stack: `iterator`, `next_function` **=>** `for_of_iterator`, `next_function`, `next_value`
    IteratorNext,

    /// Advance the iterator by one and put done and value on the stack.
    ///
    /// Stack: `iterator`, `next_function` **=>** `for_of_iterator`, `next_function`, `done`, `next_value`
    IteratorNextFull,

    /// Close an iterator.
    ///
    /// Stack: `iterator`, `next_function` **=>**
    IteratorClose,

    /// Consume the iterator and construct and array with all the values.
    ///
    /// Stack: `iterator`, `next_function` **=>** `for_of_iterator`, `next_function`, `array`
    IteratorToArray,

    /// Move to the next value in a for..in loop or jump to exit of the loop if done.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: `for_in_iterator`, `next_function` **=>** `for_in_iterator`, `next_function`, `next_result` (if not done)
    ForInLoopNext,

    /// Concat multiple stack objects into a string.
    ///
    /// Operands: number of stack objects: `u32`
    ///
    /// Stack: `value1`,...`valuen` **=>** `string`
    ConcatToString,

    /// Call RequireObjectCoercible on the stack value.
    ///
    /// Stack: `value` **=>** `value`
    RequireObjectCoercible,

    /// Require the stack value to be neither null nor undefined.
    ///
    /// Stack: `value` **=>** `value`
    ValueNotNullOrUndefined,

    /// Initialize the rest parameter value of a function from the remaining arguments.
    ///
    /// Stack: `argument_0` .. `argument_n` **=>** `rest_value`
    RestParameterInit,

    /// Pop the remaining arguments of a function.
    ///
    /// Stack: `argument_0` .. `argument_n` **=>**
    RestParameterPop,

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
            Opcode::Pop => "Pop",
            Opcode::Dup => "Dup",
            Opcode::Swap => "Swap",
            Opcode::Swap2 => "Swap2",
            Opcode::PushZero => "PushZero",
            Opcode::PushOne => "PushOne",
            Opcode::PushInt8 => "PushInt8",
            Opcode::PushInt16 => "PushInt16",
            Opcode::PushInt32 => "PushInt32",
            Opcode::PushRational => "PushRational",
            Opcode::PushNaN => "PushNaN",
            Opcode::PushPositiveInfinity => "PushPositiveInfinity",
            Opcode::PushNegativeInfinity => "PushNegativeInfinity",
            Opcode::PushNull => "PushNull",
            Opcode::PushTrue => "PushTrue",
            Opcode::PushFalse => "PushFalse",
            Opcode::PushUndefined => "PushUndefined",
            Opcode::PushLiteral => "PushLiteral",
            Opcode::PushEmptyObject => "PushEmptyObject",
            Opcode::PushNewArray => "PushNewArray",
            Opcode::PushValueToArray => "PushValueToArray",
            Opcode::PushIteratorToArray => "PushIteratorToArray",
            Opcode::Add => "Add",
            Opcode::Sub => "Sub",
            Opcode::Div => "Div",
            Opcode::Mul => "Mul",
            Opcode::Mod => "Mod",
            Opcode::Pow => "Pow",
            Opcode::ShiftRight => "ShiftRight",
            Opcode::ShiftLeft => "ShiftLeft",
            Opcode::UnsignedShiftRight => "UnsignedShiftRight",
            Opcode::BitOr => "BitOr",
            Opcode::BitAnd => "BitAnd",
            Opcode::BitXor => "BitXor",
            Opcode::BitNot => "BitNot",
            Opcode::In => "In",
            Opcode::Eq => "Eq",
            Opcode::StrictEq => "StrictEq",
            Opcode::NotEq => "NotEq",
            Opcode::StrictNotEq => "StrictNotEq",
            Opcode::GreaterThan => "GreaterThan",
            Opcode::GreaterThanOrEq => "GreaterThanOrEq",
            Opcode::LessThan => "LessThan",
            Opcode::LessThanOrEq => "LessThanOrEq",
            Opcode::InstanceOf => "InstanceOf",
            Opcode::TypeOf => "TypeOf",
            Opcode::Void => "Void",
            Opcode::LogicalNot => "LogicalNot",
            Opcode::LogicalAnd => "LogicalAnd",
            Opcode::LogicalOr => "LogicalOr",
            Opcode::Coalesce => "Coalesce",
            Opcode::Pos => "Pos",
            Opcode::Neg => "Neg",
            Opcode::Inc => "Inc",
            Opcode::Dec => "Dec",
            Opcode::DefInitArg => "DefInitArg",
            Opcode::DefVar => "DefVar",
            Opcode::DefInitVar => "DefInitVar",
            Opcode::DefLet => "DefLet",
            Opcode::DefInitLet => "DefInitLet",
            Opcode::DefInitConst => "DefInitConst",
            Opcode::GetName => "GetName",
            Opcode::GetNameOrUndefined => "GetNameOrUndefined",
            Opcode::SetName => "SetName",
            Opcode::GetPropertyByName => "GetPropertyByName",
            Opcode::GetPropertyByValue => "GetPropertyByValue",
            Opcode::SetPropertyByName => "SetPropertyByName",
            Opcode::SetPropertyByValue => "SetPropertyByValue",
            Opcode::SetPropertyGetterByName => "SetPropertyGetterByName",
            Opcode::SetPropertyGetterByValue => "SetPropertyGetterByValue",
            Opcode::SetPropertySetterByName => "SetPropertySetterByName",
            Opcode::SetPropertySetterByValue => "SetPropertySetterByValue",
            Opcode::DeletePropertyByName => "DeletePropertyByName",
            Opcode::DeletePropertyByValue => "DeletePropertyByValue",
            Opcode::CopyDataProperties => "CopyDataProperties",
            Opcode::Jump => "Jump",
            Opcode::JumpIfFalse => "JumpIfFalse",
            Opcode::JumpIfNotUndefined => "JumpIfNotUndefined",
            Opcode::Throw => "Throw",
            Opcode::TryStart => "TryStart",
            Opcode::TryEnd => "TryEnd",
            Opcode::FinallyStart => "FinallyStart",
            Opcode::FinallyEnd => "FinallyEnd",
            Opcode::FinallySetJump => "FinallySetJump",
            Opcode::ToBoolean => "ToBoolean",
            Opcode::This => "This",
            Opcode::Case => "Case",
            Opcode::Default => "Default",
            Opcode::GetFunction => "GetFunction",
            Opcode::Call => "Call",
            Opcode::CallWithRest => "CallWithRest",
            Opcode::New => "New",
            Opcode::NewWithRest => "NewWithRest",
            Opcode::Return => "Return",
            Opcode::PushDeclarativeEnvironment => "PushDeclarativeEnvironment",
            Opcode::PushFunctionEnvironment => "PushFunctionEnvironment",
            Opcode::PopEnvironment => "PopEnvironment",
            Opcode::ForInLoopInitIterator => "ForInLoopInitIterator",
            Opcode::InitIterator => "InitIterator",
            Opcode::IteratorNext => "IteratorNext",
            Opcode::IteratorNextFull => "IteratorNextFull",
            Opcode::IteratorClose => "IteratorClose",
            Opcode::IteratorToArray => "IteratorToArray",
            Opcode::ForInLoopNext => "ForInLoopNext",
            Opcode::ConcatToString => "ConcatToString",
            Opcode::RequireObjectCoercible => "RequireObjectCoercible",
            Opcode::ValueNotNullOrUndefined => "ValueNotNullOrUndefined",
            Opcode::RestParameterInit => "FunctionRestParameter",
            Opcode::RestParameterPop => "RestParameterPop",
            Opcode::Nop => "Nop",
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
