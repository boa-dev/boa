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

    /// Push array object `{}` value on the stack.
    ///
    /// Operands: n: `u32`
    ///
    /// Stack: v1, v1, ... vn **=>** [v1, v2, ..., vn]
    PushNewArray,

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

    /// Declate `var` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>**
    DefVar,

    /// Declate `let` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>**
    DefLet,

    /// Declate `const` type variable.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>**
    DefConst,

    /// Initialize a lexical binding.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>**
    InitLexical,

    /// Find a binding on the environment chain and push its value.
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: **=>** value
    GetName,

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

    /// Unconditional jump to address.
    ///
    /// Operands: address: `u32`
    /// Stack: **=>**
    Jump,

    /// Constional jump to address.
    ///
    /// If the value popped is [`falsy`][falsy] then jump to `address`.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: cond **=>**
    ///
    /// [falsy]: https://developer.mozilla.org/en-US/docs/Glossary/Falsy
    JumpIfFalse,

    /// Constional jump to address.
    ///
    /// If the value popped is [`truthy`][truthy] then jump to `address`.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: cond **=>**
    ///
    /// [truthy]: https://developer.mozilla.org/en-US/docs/Glossary/Truthy
    JumpIfTrue,

    /// Throw exception
    ///
    /// Operands:
    ///
    /// Stack: `exc` **=>**
    Throw,

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
            Opcode::DefVar => "DefVar",
            Opcode::DefLet => "DefLet",
            Opcode::DefConst => "DefConst",
            Opcode::InitLexical => "InitLexical",
            Opcode::GetName => "GetName",
            Opcode::SetName => "SetName",
            Opcode::GetPropertyByName => "GetPropertyByName",
            Opcode::GetPropertyByValue => "GetPropertyByValue",
            Opcode::SetPropertyByName => "SetPropertyByName",
            Opcode::SetPropertyByValue => "SetPropertyByValue",
            Opcode::Jump => "Jump",
            Opcode::JumpIfFalse => "JumpIfFalse",
            Opcode::JumpIfTrue => "JumpIfTrue",
            Opcode::Throw => "Throw",
            Opcode::ToBoolean => "ToBoolean",
            Opcode::This => "This",
            Opcode::Case => "Case",
            Opcode::Default => "Default",
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
