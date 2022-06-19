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
    /// Stack: superclass **=>** superclass.prototype
    PushClassPrototype,

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

    /// Set a private property by name from an object.
    ///
    /// Like `#name = value`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    SetPrivateValue,

    /// Set a private setter property by name from an object.
    ///
    /// Like `set #name() {}`
    ///
    /// Operands: name_index: `u32`
    ///
    /// Stack: object, value **=>**
    SetPrivateSetter,

    /// Set a private getter property by name from an object.
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

    /// Push a computed class field name to a class constructor object.
    ///
    /// Operands:
    ///
    /// Stack: value, object **=>**
    PushClassComputedFieldName,

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
    /// Operands: excluded_key_count: `u32`
    ///
    /// Stack: source, value, excluded_key_0 ... excluded_key_n **=>** value
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

    /// Get generator function from the pre-compiled inner functions.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: **=>** func
    GetGenerator,

    /// Call a function named "eval".
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, this, argument_1, ... argument_n **=>** result
    CallEval,

    /// Call a function named "eval" where the last argument is a rest parameter.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, this, argument_1, ... argument_n **=>** result
    CallEvalWithRest,

    /// Call a function.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, this, argument_1, ... argument_n **=>** result
    Call,

    /// Call a function where the last argument is a rest parameter.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, this, argument_1, ... argument_n **=>** result
    CallWithRest,

    /// Call construct on a function.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, argument_1, ... argument_n **=>** result
    New,

    /// Call construct on a function where the last argument is a rest parameter.
    ///
    /// Operands: argument_count: `u32`
    ///
    /// Stack: func, argument_1, ... argument_n **=>** result
    NewWithRest,

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

    /// Delegates the current generator function another generator.
    ///
    /// Operands: done_address: `u32`
    ///
    /// Stack: iterator, next_method, done, received **=>** iterator, next_method, done
    GeneratorNextDelegate,

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
            Self::Pop => "Pop",
            Self::PopIfThrown => "PopIfThrown",
            Self::Dup => "Dup",
            Self::Swap => "Swap",
            Self::PushZero => "PushZero",
            Self::PushOne => "PushOne",
            Self::PushInt8 => "PushInt8",
            Self::PushInt16 => "PushInt16",
            Self::PushInt32 => "PushInt32",
            Self::PushRational => "PushRational",
            Self::PushNaN => "PushNaN",
            Self::PushPositiveInfinity => "PushPositiveInfinity",
            Self::PushNegativeInfinity => "PushNegativeInfinity",
            Self::PushNull => "PushNull",
            Self::PushTrue => "PushTrue",
            Self::PushFalse => "PushFalse",
            Self::PushUndefined => "PushUndefined",
            Self::PushLiteral => "PushLiteral",
            Self::PushEmptyObject => "PushEmptyObject",
            Self::PushClassPrototype => "PushClassPrototype",
            Self::PushNewArray => "PushNewArray",
            Self::PushValueToArray => "PushValueToArray",
            Self::PushElisionToArray => "PushElisionToArray",
            Self::PushIteratorToArray => "PushIteratorToArray",
            Self::Add => "Add",
            Self::Sub => "Sub",
            Self::Div => "Div",
            Self::Mul => "Mul",
            Self::Mod => "Mod",
            Self::Pow => "Pow",
            Self::ShiftRight => "ShiftRight",
            Self::ShiftLeft => "ShiftLeft",
            Self::UnsignedShiftRight => "UnsignedShiftRight",
            Self::BitOr => "BitOr",
            Self::BitAnd => "BitAnd",
            Self::BitXor => "BitXor",
            Self::BitNot => "BitNot",
            Self::In => "In",
            Self::Eq => "Eq",
            Self::StrictEq => "StrictEq",
            Self::NotEq => "NotEq",
            Self::StrictNotEq => "StrictNotEq",
            Self::GreaterThan => "GreaterThan",
            Self::GreaterThanOrEq => "GreaterThanOrEq",
            Self::LessThan => "LessThan",
            Self::LessThanOrEq => "LessThanOrEq",
            Self::InstanceOf => "InstanceOf",
            Self::TypeOf => "TypeOf",
            Self::Void => "Void",
            Self::LogicalNot => "LogicalNot",
            Self::LogicalAnd => "LogicalAnd",
            Self::LogicalOr => "LogicalOr",
            Self::Coalesce => "Coalesce",
            Self::Pos => "Pos",
            Self::Neg => "Neg",
            Self::Inc => "Inc",
            Self::IncPost => "IncPost",
            Self::Dec => "Dec",
            Self::DecPost => "DecPost",
            Self::DefInitArg => "DefInitArg",
            Self::DefVar => "DefVar",
            Self::DefInitVar => "DefInitVar",
            Self::DefLet => "DefLet",
            Self::DefInitLet => "DefInitLet",
            Self::DefInitConst => "DefInitConst",
            Self::GetName => "GetName",
            Self::GetNameOrUndefined => "GetNameOrUndefined",
            Self::SetName => "SetName",
            Self::GetPropertyByName => "GetPropertyByName",
            Self::GetPropertyByValue => "GetPropertyByValue",
            Self::SetPropertyByName => "SetPropertyByName",
            Self::DefineOwnPropertyByName => "DefineOwnPropertyByName",
            Self::DefineClassMethodByName => "DefineClassMethodByName",
            Self::SetPropertyByValue => "SetPropertyByValue",
            Self::DefineOwnPropertyByValue => "DefineOwnPropertyByValue",
            Self::DefineClassMethodByValue => "DefineClassMethodByValue",
            Self::SetPropertyGetterByName => "SetPropertyGetterByName",
            Self::DefineClassGetterByName => "DefineClassGetterByName",
            Self::SetPropertyGetterByValue => "SetPropertyGetterByValue",
            Self::DefineClassGetterByValue => "DefineClassGetterByValue",
            Self::SetPropertySetterByName => "SetPropertySetterByName",
            Self::DefineClassSetterByName => "DefineClassSetterByName",
            Self::SetPropertySetterByValue => "SetPropertySetterByValue",
            Self::DefineClassSetterByValue => "DefineClassSetterByValue",
            Self::SetPrivateValue => "SetPrivateValue",
            Self::SetPrivateSetter => "SetPrivateSetter",
            Self::SetPrivateGetter => "SetPrivateGetter",
            Self::GetPrivateField => "GetPrivateByName",
            Self::PushClassComputedFieldName => "PushClassComputedFieldName",
            Self::DeletePropertyByName => "DeletePropertyByName",
            Self::DeletePropertyByValue => "DeletePropertyByValue",
            Self::CopyDataProperties => "CopyDataProperties",
            Self::ToPropertyKey => "ToPropertyKey",
            Self::Jump => "Jump",
            Self::JumpIfFalse => "JumpIfFalse",
            Self::JumpIfNotUndefined => "JumpIfNotUndefined",
            Self::Throw => "Throw",
            Self::TryStart => "TryStart",
            Self::TryEnd => "TryEnd",
            Self::CatchStart => "CatchStart",
            Self::CatchEnd => "CatchEnd",
            Self::CatchEnd2 => "CatchEnd2",
            Self::FinallyStart => "FinallyStart",
            Self::FinallyEnd => "FinallyEnd",
            Self::FinallySetJump => "FinallySetJump",
            Self::ToBoolean => "ToBoolean",
            Self::This => "This",
            Self::Case => "Case",
            Self::Default => "Default",
            Self::GetFunction => "GetFunction",
            Self::GetGenerator => "GetGenerator",
            Self::CallEval => "CallEval",
            Self::CallEvalWithRest => "CallEvalWithRest",
            Self::Call => "Call",
            Self::CallWithRest => "CallWithRest",
            Self::New => "New",
            Self::NewWithRest => "NewWithRest",
            Self::Return => "Return",
            Self::PushDeclarativeEnvironment => "PushDeclarativeEnvironment",
            Self::PushFunctionEnvironment => "PushFunctionEnvironment",
            Self::PopEnvironment => "PopEnvironment",
            Self::LoopStart => "LoopStart",
            Self::LoopContinue => "LoopContinue",
            Self::LoopEnd => "LoopEnd",
            Self::ForInLoopInitIterator => "ForInLoopInitIterator",
            Self::InitIterator => "InitIterator",
            Self::IteratorNext => "IteratorNext",
            Self::IteratorClose => "IteratorClose",
            Self::IteratorToArray => "IteratorToArray",
            Self::ForInLoopNext => "ForInLoopNext",
            Self::ConcatToString => "ConcatToString",
            Self::RequireObjectCoercible => "RequireObjectCoercible",
            Self::ValueNotNullOrUndefined => "ValueNotNullOrUndefined",
            Self::RestParameterInit => "FunctionRestParameter",
            Self::RestParameterPop => "RestParameterPop",
            Self::PopOnReturnAdd => "PopOnReturnAdd",
            Self::PopOnReturnSub => "PopOnReturnSub",
            Self::Yield => "Yield",
            Self::GeneratorNext => "GeneratorNext",
            Self::GeneratorNextDelegate => "GeneratorNextDelegate",
            Self::Nop => "Nop",
        }
    }

    /// Name of the profiler event for this opcode
    pub fn as_instruction_str(self) -> &'static str {
        match self {
            Self::Pop => "INST - Pop",
            Self::PopIfThrown => "INST - PopIfThrown",
            Self::Dup => "INST - Dup",
            Self::Swap => "INST - Swap",
            Self::PushZero => "INST - PushZero",
            Self::PushOne => "INST - PushOne",
            Self::PushInt8 => "INST - PushInt8",
            Self::PushInt16 => "INST - PushInt16",
            Self::PushInt32 => "INST - PushInt32",
            Self::PushRational => "INST - PushRational",
            Self::PushNaN => "INST - PushNaN",
            Self::PushPositiveInfinity => "INST - PushPositiveInfinity",
            Self::PushNegativeInfinity => "INST - PushNegativeInfinity",
            Self::PushNull => "INST - PushNull",
            Self::PushTrue => "INST - PushTrue",
            Self::PushFalse => "INST - PushFalse",
            Self::PushUndefined => "INST - PushUndefined",
            Self::PushLiteral => "INST - PushLiteral",
            Self::PushEmptyObject => "INST - PushEmptyObject",
            Self::PushNewArray => "INST - PushNewArray",
            Self::PushValueToArray => "INST - PushValueToArray",
            Self::PushElisionToArray => "INST - PushElisionToArray",
            Self::PushIteratorToArray => "INST - PushIteratorToArray",
            Self::Add => "INST - Add",
            Self::Sub => "INST - Sub",
            Self::Div => "INST - Div",
            Self::Mul => "INST - Mul",
            Self::Mod => "INST - Mod",
            Self::Pow => "INST - Pow",
            Self::ShiftRight => "INST - ShiftRight",
            Self::ShiftLeft => "INST - ShiftLeft",
            Self::UnsignedShiftRight => "INST - UnsignedShiftRight",
            Self::BitOr => "INST - BitOr",
            Self::BitAnd => "INST - BitAnd",
            Self::BitXor => "INST - BitXor",
            Self::BitNot => "INST - BitNot",
            Self::In => "INST - In",
            Self::Eq => "INST - Eq",
            Self::StrictEq => "INST - StrictEq",
            Self::NotEq => "INST - NotEq",
            Self::StrictNotEq => "INST - StrictNotEq",
            Self::GreaterThan => "INST - GreaterThan",
            Self::GreaterThanOrEq => "INST - GreaterThanOrEq",
            Self::LessThan => "INST - LessThan",
            Self::LessThanOrEq => "INST - LessThanOrEq",
            Self::InstanceOf => "INST - InstanceOf",
            Self::TypeOf => "INST - TypeOf",
            Self::Void => "INST - Void",
            Self::LogicalNot => "INST - LogicalNot",
            Self::LogicalAnd => "INST - LogicalAnd",
            Self::LogicalOr => "INST - LogicalOr",
            Self::Coalesce => "INST - Coalesce",
            Self::Pos => "INST - Pos",
            Self::Neg => "INST - Neg",
            Self::Inc => "INST - Inc",
            Self::IncPost => "INST - IncPost",
            Self::Dec => "INST - Dec",
            Self::DecPost => "INST - DecPost",
            Self::DefInitArg => "INST - DefInitArg",
            Self::DefVar => "INST - DefVar",
            Self::DefInitVar => "INST - DefInitVar",
            Self::DefLet => "INST - DefLet",
            Self::DefInitLet => "INST - DefInitLet",
            Self::DefInitConst => "INST - DefInitConst",
            Self::GetName => "INST - GetName",
            Self::GetNameOrUndefined => "INST - GetNameOrUndefined",
            Self::SetName => "INST - SetName",
            Self::GetPropertyByName => "INST - GetPropertyByName",
            Self::GetPropertyByValue => "INST - GetPropertyByValue",
            Self::SetPropertyByName => "INST - SetPropertyByName",
            Self::DefineOwnPropertyByName => "INST - DefineOwnPropertyByName",
            Self::SetPropertyByValue => "INST - SetPropertyByValue",
            Self::DefineOwnPropertyByValue => "INST - DefineOwnPropertyByValue",
            Self::SetPropertyGetterByName => "INST - SetPropertyGetterByName",
            Self::SetPropertyGetterByValue => "INST - SetPropertyGetterByValue",
            Self::SetPropertySetterByName => "INST - SetPropertySetterByName",
            Self::SetPropertySetterByValue => "INST - SetPropertySetterByValue",
            Self::DeletePropertyByName => "INST - DeletePropertyByName",
            Self::DeletePropertyByValue => "INST - DeletePropertyByValue",
            Self::CopyDataProperties => "INST - CopyDataProperties",
            Self::Jump => "INST - Jump",
            Self::JumpIfFalse => "INST - JumpIfFalse",
            Self::JumpIfNotUndefined => "INST - JumpIfNotUndefined",
            Self::Throw => "INST - Throw",
            Self::TryStart => "INST - TryStart",
            Self::TryEnd => "INST - TryEnd",
            Self::CatchStart => "INST - CatchStart",
            Self::CatchEnd => "INST - CatchEnd",
            Self::CatchEnd2 => "INST - CatchEnd2",
            Self::FinallyStart => "INST - FinallyStart",
            Self::FinallyEnd => "INST - FinallyEnd",
            Self::FinallySetJump => "INST - FinallySetJump",
            Self::ToBoolean => "INST - ToBoolean",
            Self::This => "INST - This",
            Self::Case => "INST - Case",
            Self::Default => "INST - Default",
            Self::GetFunction => "INST - GetFunction",
            Self::GetGenerator => "INST - GetGenerator",
            Self::CallEval => "INST - CallEval",
            Self::CallEvalWithRest => "INST - CallEvalWithRest",
            Self::Call => "INST - Call",
            Self::CallWithRest => "INST - CallWithRest",
            Self::New => "INST - New",
            Self::NewWithRest => "INST - NewWithRest",
            Self::Return => "INST - Return",
            Self::PushDeclarativeEnvironment => "INST - PushDeclarativeEnvironment",
            Self::PushFunctionEnvironment => "INST - PushFunctionEnvironment",
            Self::PopEnvironment => "INST - PopEnvironment",
            Self::LoopStart => "INST - LoopStart",
            Self::LoopContinue => "INST - LoopContinue",
            Self::LoopEnd => "INST - LoopEnd",
            Self::ForInLoopInitIterator => "INST - ForInLoopInitIterator",
            Self::InitIterator => "INST - InitIterator",
            Self::IteratorNext => "INST - IteratorNext",
            Self::IteratorClose => "INST - IteratorClose",
            Self::IteratorToArray => "INST - IteratorToArray",
            Self::ForInLoopNext => "INST - ForInLoopNext",
            Self::ConcatToString => "INST - ConcatToString",
            Self::RequireObjectCoercible => "INST - RequireObjectCoercible",
            Self::ValueNotNullOrUndefined => "INST - ValueNotNullOrUndefined",
            Self::RestParameterInit => "INST - FunctionRestParameter",
            Self::RestParameterPop => "INST - RestParameterPop",
            Self::PopOnReturnAdd => "INST - PopOnReturnAdd",
            Self::PopOnReturnSub => "INST - PopOnReturnSub",
            Self::Yield => "INST - Yield",
            Self::GeneratorNext => "INST - GeneratorNext",
            Self::GeneratorNextDelegate => "INST - GeneratorNextDelegate",
            Self::Nop => "INST - Nop",
            Self::PushClassPrototype => "INST - PushClassPrototype",
            Self::DefineClassMethodByName => "INST - DefineClassMethodByName",
            Self::DefineClassMethodByValue => "INST - DefineClassMethodByValue",
            Self::DefineClassGetterByName => "INST - DefineClassGetterByName",
            Self::DefineClassGetterByValue => "INST - DefineClassGetterByValue",
            Self::DefineClassSetterByName => "INST - DefineClassSetterByName",
            Self::DefineClassSetterByValue => "INST - DefineClassSetterByValue",
            Self::SetPrivateValue => "INST - SetPrivateValue",
            Self::SetPrivateSetter => "INST - SetPrivateSetter",
            Self::SetPrivateGetter => "INST - SetPrivateGetter",
            Self::GetPrivateField => "INST - GetPrivateField",
            Self::PushClassComputedFieldName => "INST - PushClassComputedFieldName",
            Self::ToPropertyKey => "INST - ToPropertyKey",
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
