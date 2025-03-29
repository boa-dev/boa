#![allow(clippy::inline_always)]


/// The opcodes of the vm.
use crate::{
    vm::{CompletionType, Registers},
    Context, JsResult,
};

mod args;

use args::{ArgumentsFormat, DecodeAndDispatch};

// Operation modules
mod arguments;
mod r#await;
mod binary_ops;
mod call;
mod concat;
mod control_flow;
mod copy;
mod define;
mod delete;
mod environment;
mod generator;
mod get;
mod global;
mod iteration;
mod locals;
mod meta;
mod new;
mod nop;
mod pop;
mod push;
mod rest_parameter;
mod set;
mod switch;
mod templates;
mod to;
mod unary_ops;
mod value;

// Operation structs
#[doc(inline)]
pub(crate) use arguments::*;
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
pub(crate) use environment::*;
#[doc(inline)]
pub(crate) use generator::*;
#[doc(inline)]
pub(crate) use get::*;
#[doc(inline)]
pub(crate) use global::*;
#[doc(inline)]
pub(crate) use iteration::*;
#[doc(inline)]
pub(crate) use locals::*;
#[doc(inline)]
pub(crate) use meta::*;
#[doc(inline)]
pub(crate) use new::*;
#[doc(inline)]
pub(crate) use nop::*;
#[doc(inline)]
pub(crate) use pop::*;
#[doc(inline)]
pub(crate) use push::*;
#[doc(inline)]
pub(crate) use r#await::*;
#[doc(inline)]
pub(crate) use rest_parameter::*;
#[doc(inline)]
pub(crate) use set::*;
#[doc(inline)]
pub(crate) use switch::*;
#[doc(inline)]
pub(crate) use templates::*;
#[doc(inline)]
pub(crate) use to::*;
#[doc(inline)]
pub(crate) use unary_ops::*;
#[doc(inline)]
pub(crate) use value::*;

/// Specific opcodes for bindings.
///
/// This separate enum exists to make matching exhaustive where needed.
#[derive(Clone, Copy, Debug)]
pub(crate) enum BindingOpcode {
    Var,
    InitVar,
    InitLexical,
    SetName,
}

/// The `Operation` trait implements the execution code along with the
/// identifying Name and Instruction value for an Boa Opcode.
///
/// This trait should be implemented for a struct that corresponds with
/// any arm of the `OpCode` enum.
pub(crate) trait Operation {
    const NAME: &'static str;
    const INSTRUCTION: &'static str;
    const COST: u8;

    // /// Execute opcode with [`VaryingOperandKind::U8`] sized [`VaryingOperand`]s.
    // fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType>;
    // /// Execute opcode with [`VaryingOperandKind::U16`] sized [`VaryingOperand`]s.
    // ///
    // /// By default if not implemented will call [`Reserved::execute_u16()`] which panics.
    // fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
    //     Reserved::execute_u16(registers, context)
    // }
    // /// Execute opcode with [`VaryingOperandKind::U32`] sized [`VaryingOperand`]s.
    // ///
    // /// By default if not implemented will call [`Reserved::execute_u32()`] which panics.
    // fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
    //     Reserved::execute_u32(registers, context)
    // }
    // /// Spends the cost of this operation into `budget` and runs `execute`.
    // fn spend_budget_and_execute(
    //     registers: &mut Registers,
    //     context: &mut Context,
    //     budget: &mut u32,
    // ) -> JsResult<CompletionType> {
    //     *budget = budget.saturating_sub(u32::from(Self::COST));
    //     Self::execute(registers, context)
    // }
}

#[derive(Debug)]
struct Label {
    address: u32,
    opcode: Opcode,
}

#[derive(Debug)]
pub(crate) struct ByteCodeEmitter {
    bytecode: Vec<u64>,
    extended: Vec<u32>,
}

impl ByteCodeEmitter {
    pub(crate) fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            extended: Vec::new(),
        }
    }

    pub(crate) fn into_bytecode(self) -> ByteCode {
        ByteCode {
            bytecode: self.bytecode.into_boxed_slice(),
            extended: self.extended.into_boxed_slice(),
        }
    }

    pub(crate) fn next_opcode_location(&self) -> u32 {
        self.bytecode.len() as u32
    }

    // fn patch(&mut self, label: Label)

    pub(crate) fn patch_jump(&mut self, label: u32, patch: u32) {
        let address = label as usize;
        let instruction = self.bytecode[address];
        let opcode = Opcode::decode(instruction);
        let format = ArgumentsFormat::decode(instruction);
        match format {
            ArgumentsFormat::OneArgU32 => {
                let new_instruction = encode_instruction(opcode, patch, &mut self.extended);
                self.bytecode[address] = new_instruction;
            }
            ArgumentsFormat::TwoArgU32
            | ArgumentsFormat::ThreeArgU32
            | ArgumentsFormat::VariableArgsU32 => {
                let index = (instruction >> 16) as u32 as usize;
                self.extended[index] = patch;
            }
            _ => unreachable!("invalid jump format"),
        }
    }

    pub(crate) fn patch_jump_two_addresses(&mut self, label: u32, patch: (u32, u32)) {
        let address = label as usize;
        let instruction = self.bytecode[address];
        let format = ArgumentsFormat::decode(instruction);
        assert_eq!(ArgumentsFormat::VariableArgsU32, format);
        let index = (instruction >> 16) as u32 as usize;
        self.extended[index] = patch.0;
        self.extended[index + 1] = patch.1;
    }

    pub(crate) fn patch_jump_table(&mut self, label: u32, patch: (u32, Vec<u32>)) {
        let address = label as usize;
        let instruction = self.bytecode[address];
        let format = ArgumentsFormat::decode(instruction);
        assert_eq!(ArgumentsFormat::VariableArgsU32, format);
        let index = (instruction >> 16) as u32 as usize;
        self.extended[index] = patch.0;
        for (i, value) in patch.1.iter().enumerate() {
            self.extended[index + i + 1] = *value;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ByteCode {
    bytecode: Box<[u64]>,
    extended: Box<[u32]>,
}

enum VaryingOperandValue {
    U8(u8),
    U16(u16),
    U32(u32),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VaryingOperand {
    value: u32,
}

impl VaryingOperand {
    pub(crate) fn new(value: u32) -> Self {
        Self { value }
    }
}

impl VaryingOperand {
    fn value(&self) -> VaryingOperandValue {
        if let Ok(value) = u8::try_from(self.value) {
            VaryingOperandValue::U8(value)
        } else if let Ok(value) = u16::try_from(self.value) {
            VaryingOperandValue::U16(value)
        } else {
            VaryingOperandValue::U32(self.value)
        }
    }
}

impl From<VaryingOperand> for u32 {
    fn from(value: VaryingOperand) -> Self {
        value.value
    }
}

impl From<VaryingOperand> for usize {
    fn from(value: VaryingOperand) -> Self {
        value.value as usize
    }
}

impl From<bool> for VaryingOperand {
    fn from(value: bool) -> Self {
        Self::new(value.into())
    }
}

impl From<u8> for VaryingOperand {
    fn from(value: u8) -> Self {
        Self::new(value.into())
    }
}

impl From<u16> for VaryingOperand {
    fn from(value: u16) -> Self {
        Self::new(value.into())
    }
}

impl From<u32> for VaryingOperand {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl Opcode {
    fn encode(self) -> u64 {
        (self as u64) << 56
    }

    fn decode(instruction: u64) -> Self {
        Self::from((instruction >> 56) as u8)
    }
}

fn encode_instruction<A: DecodeAndDispatch>(
    opcode: Opcode,
    format: A,
    extended: &mut Vec<u32>,
) -> u64 {
    opcode.encode() | format.encode(extended)
}

macro_rules! generate_opcodes {
    (
        $(
            $(#[$comment:ident $($args:tt)*])*
            $Variant:ident $($EmitFn:ident $({
                $(
                    $(#[$fieldinner:ident $($fieldargs:tt)*])*
                    $FieldName:ident : $FieldType:ty
                ),*
            })? )? $(=> $mapping:ident)?
        ),*
        $(,)?
    ) => {
        /// The opcodes of the vm.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub(crate) enum Opcode {
            $(
                $(#[$comment $($args)*])*
                $Variant
            ),*
        }

        impl From<u8> for Opcode {
            #[inline]
            #[allow(non_upper_case_globals)]
            fn from(value: u8) -> Self {
                $(
                    const $Variant: u8 = Opcode::$Variant as u8;
                )*
                match value {
                    $($Variant => Self::$Variant),*
                }
            }
        }

        impl ByteCodeEmitter {
            $(
                $(
                    pub(crate) fn $EmitFn(&mut self $( $(, $FieldName: $FieldType)* )? ) {
                        self.bytecode.push(encode_instruction(
                            Opcode::$Variant,
                            ($($($FieldName),*)?),
                            &mut self.extended,
                        ));
                    }
                )?
            )*
        }

        type OpcodeHandler = fn(&mut Context, &mut Registers, u64) -> JsResult<CompletionType>;

        static OPCODE_HANDLERS: [OpcodeHandler; 256] = {
            [
                $(
                    paste::paste! {
                        [<handle_ $Variant:lower>]
                    },
                )*
            ]
        };

        $(
            paste::paste! {
            #[inline(always)]
                fn [<handle_ $Variant:lower>](context: &mut Context, registers: &mut Registers, instruction: u64) -> JsResult<CompletionType> {
                    let args = DecodeAndDispatch::decode_and_dispatch(instruction, &context.vm.frame.code_block.bytecode.extended);
                    $Variant::operation(args, registers, context)
                }
            }
        )*

        impl Context {
            pub(crate) fn execute_bytecode_instruction(&mut self, registers: &mut Registers) -> JsResult<CompletionType> {
                let frame = self.vm.frame_mut();
                let pc = frame.pc;
                frame.pc += 1;
                let instruction = self.vm.frame.code_block.bytecode.bytecode[pc as usize];
                let opcode = Opcode::decode(instruction);
                OPCODE_HANDLERS[opcode as usize](self, registers, instruction)
            }
        }

        $(
            $(
                struct $Variant {}
                impl $Variant {
                    #[allow(unused_parens)]
                    #[allow(unused_variables)]
                    #[inline(always)]
                    fn operation(args: (), registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
                        $mapping::operation(args, registers, context)
                    }
                }
            )?
        )*
    }
}

generate_opcodes! {
    /// Pop the top value from the stack.
    ///
    /// - Stack: value **=>**
    Pop emit_pop,

    /// Push integer `0` on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushZero emit_push_zero { dst: VaryingOperand },

    /// Push integer `1` on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushOne emit_push_one { dst: VaryingOperand },

    /// Push `i8` value on the stack.
    ///
    /// - Operands:
    ///   - value: `i8`
    /// - Registers:
    ///   - Output: dst
    PushInt8 emit_push_int8 { dst: VaryingOperand, value: i8 },

    /// Push i16 value on the stack.
    ///
    /// - Operands:
    ///   - value: `i16`
    /// - Registers:
    ///   - Output: dst
    PushInt16 emit_push_int16 { dst: VaryingOperand, value: i16 },

    /// Push i32 value on the stack.
    ///
    /// - Operands:
    ///   - value: `i32`
    /// - Registers:
    ///   - Output: dst
    PushInt32 emit_push_int32 { dst: VaryingOperand, value: i32 },

    /// Push `f32` value on the stack.
    ///
    /// - Operands:
    ///   - value: `f32`
    /// - Registers:
    ///   - Output: dst
    PushFloat emit_push_float { dst: VaryingOperand, value: f32 },

    /// Push `f64` value on the stack.
    ///
    /// - Operands:
    ///   - value: `f64`
    /// - Registers:
    ///   - Output: dst
    PushDouble emit_push_double { dst: VaryingOperand, value: f64 },

    /// Push `NaN` integer on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushNaN emit_push_nan { dst: VaryingOperand },

    /// Push `Infinity` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushPositiveInfinity emit_push_positive_infinity { dst: VaryingOperand },

    /// Push `-Infinity` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushNegativeInfinity emit_push_negative_infinity { dst: VaryingOperand },

    /// Push `null` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushNull emit_push_null { dst: VaryingOperand },

    /// Push `true` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushTrue emit_push_true { dst: VaryingOperand },

    /// Push `false` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushFalse emit_push_false { dst: VaryingOperand },

    /// Push `undefined` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushUndefined emit_push_undefined { dst: VaryingOperand },

    /// Push literal value on the stack.
    ///
    /// Like strings and bigints. The index operand is used to index into the `literals`
    /// array to get the value.
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    PushLiteral emit_push_literal { dst: VaryingOperand, index: VaryingOperand },

    /// Push regexp value on the stack.
    ///
    /// - Operands:
    ///   - pattern_index: `VaryingOperand`
    ///   - flags: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    PushRegExp emit_push_regexp { dst: VaryingOperand, pattern_index: VaryingOperand, flags_index: VaryingOperand },

    /// Push empty object `{}` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushEmptyObject emit_push_empty_object { dst: VaryingOperand },

    /// Get the prototype of a superclass and push it on the stack.
    ///
    /// Additionally this sets the `[[prototype]]` of the class and the `DERIVED` flag.
    ///
    /// - Registers:
    ///   - Input: class, superclass
    ///   - Output: dst
    PushClassPrototype emit_push_class_prototype {
        dst: VaryingOperand,
        class: VaryingOperand,
        superclass: VaryingOperand
    },

    /// Set the prototype of a class object.
    ///
    /// - Registers:
    ///   - Input: class, prototype
    ///   - Output: dst
    SetClassPrototype emit_set_class_prototype {
        dst: VaryingOperand,
        prototype: VaryingOperand,
        class: VaryingOperand
    },

    /// Set home object internal slot of an object literal method.
    ///
    /// - Registers:
    ///   - Input: function, home
    SetHomeObject emit_set_home_object {
        function: VaryingOperand,
        home: VaryingOperand
    },

    /// Set the prototype of an object if the value is an object or null.
    ///
    /// - Registers:
    ///   - Input: object, prototype
    SetPrototype emit_set_prototype {
        object: VaryingOperand,
        prototype: VaryingOperand
    },

    /// Push an empty array value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushNewArray emit_push_new_array { dst: VaryingOperand },

    /// Push a value to an array.
    ///
    /// - Registers:
    ///   - Input: array, value
    PushValueToArray emit_push_value_to_array { value: VaryingOperand, array: VaryingOperand },

    /// Push an empty element/hole to an array.
    ///
    /// - Registers:
    ///   - Input: array
    PushElisionToArray emit_push_elision_to_array { array: VaryingOperand },

    /// Push all iterator values to an array.
    ///
    /// - Registers:
    ///   - Input: array
    PushIteratorToArray emit_push_iterator_to_array { array: VaryingOperand },

    /// Binary `+` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Add emit_add { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `-` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Sub emit_sub { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `/` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Div emit_div { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `*` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Mul emit_mul { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `%` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Mod emit_mod { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `**` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Pow emit_pow { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `>>` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    ShiftRight emit_shift_right { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `<<` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    ShiftLeft emit_shift_left { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `>>>` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    UnsignedShiftRight emit_unsigned_shift_right { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary bitwise `|` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    BitOr emit_bit_or { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary bitwise `&` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    BitAnd emit_bit_and { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary bitwise `^` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    BitXor emit_bit_xor { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Unary bitwise `~` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    BitNot emit_bit_not { value: VaryingOperand },

    /// Binary `in` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    In emit_in { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `in` operator for private names.
    ///
    /// - Operands: index: `u32`
    /// - Registers:
    ///   - Input: rhs
    ///   - Output: dst
    InPrivate emit_in_private { dst: VaryingOperand, index: VaryingOperand, rhs: VaryingOperand },

    /// Binary `==` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Eq emit_eq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `===` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    StrictEq emit_strict_eq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `!=` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    NotEq emit_not_eq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `!==` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    StrictNotEq emit_strict_not_eq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `>` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    GreaterThan emit_greater_than { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `>=` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    GreaterThanOrEq emit_greater_than_or_eq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `<` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    LessThan emit_less_than { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `<=` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    LessThanOrEq emit_less_than_or_eq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `instanceof` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    InstanceOf emit_instance_of { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary logical `&&` operator.
    ///
    /// This is a short-circuit operator, if the `lhs` value is `false`, then it jumps to `exit` address.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    LogicalAnd emit_logical_and { address: u32, value: VaryingOperand },

    /// Binary logical `||` operator.
    ///
    /// This is a short-circuit operator, if the `lhs` value is `true`, then it jumps to `exit` address.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    LogicalOr emit_logical_or { address: u32, value: VaryingOperand },

    /// Binary `??` operator.
    ///
    /// This is a short-circuit operator, if the `lhs` value is **not** `null` or `undefined`,
    /// then it jumps to `exit` address.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    Coalesce emit_coalesce { address: u32, value: VaryingOperand },

    /// Unary `typeof` operator.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    TypeOf emit_type_of { value: VaryingOperand },

    /// Unary logical `!` operator.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    LogicalNot emit_logical_not { value: VaryingOperand },

    /// Unary `+` operator.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    Pos emit_pos { value: VaryingOperand },

    /// Unary `-` operator.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    Neg emit_neg { value: VaryingOperand },

    /// Unary `++` operator.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    Inc emit_inc { dst: VaryingOperand, src: VaryingOperand },

    /// Unary `--` operator.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    Dec emit_dec { dst: VaryingOperand, src: VaryingOperand },

    /// Declare `var` type variable.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    DefVar emit_def_var { binding_index: VaryingOperand },

    /// Declare and initialize `var` type variable.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: src
    DefInitVar emit_def_init_var { src: VaryingOperand, binding_index: VaryingOperand },

    /// Initialize a lexical binding.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: src
    PutLexicalValue emit_put_lexical_value { src: VaryingOperand, binding_index: VaryingOperand },

    /// Throws an error because the binding access is illegal.
    ///
    /// - Operands:
    ///   -index: `VaryingOperand`
    ThrowMutateImmutable emit_throw_mutate_immutable { index: VaryingOperand },

    /// Get i-th argument of the current frame.
    ///
    /// Returns `undefined` if `arguments.len()` < `index`.
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetArgument emit_get_argument { index: VaryingOperand, dst: VaryingOperand },

    /// Find a binding on the environment chain and push its value.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetName emit_get_name { dst: VaryingOperand, binding_index: VaryingOperand },

    /// Find a binding in the global object and push its value.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    ///   - ic_index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetNameGlobal emit_get_name_global { dst: VaryingOperand, binding_index: VaryingOperand, ic_index: VaryingOperand },

    /// Find a binding on the environment and set the `current_binding` of the current frame.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    GetLocator emit_get_locator { binding_index: VaryingOperand },

    /// Find a binding on the environment chain and push its value to the stack and its
    /// `BindingLocator` to the `bindings_stack`.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetNameAndLocator emit_get_name_and_locator { dst: VaryingOperand, binding_index: VaryingOperand },

    /// Find a binding on the environment chain and push its value. If the binding does not exist push undefined.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetNameOrUndefined emit_get_name_or_undefined { dst: VaryingOperand, binding_index: VaryingOperand },

    /// Find a binding on the environment chain and assign its value.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: src
    SetName emit_set_name { src: VaryingOperand, binding_index: VaryingOperand },

    /// Assigns a value to the binding pointed by the top of the `bindings_stack`.
    ///
    /// - Registers:
    ///   - Input: src
    SetNameByLocator emit_set_name_by_locator { src: VaryingOperand },

    /// Deletes a property of the global object.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    DeleteName emit_delete_name { dst: VaryingOperand, binding_index: VaryingOperand },

    /// Get a property by name from an object an push it on the stack.
    ///
    /// Like `object.name`
    ///
    /// - Operands:
    ///   - ic_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: receiver, value
    ///   - Output: dst
    GetPropertyByName emit_get_property_by_name {
        dst: VaryingOperand,
        receiver: VaryingOperand,
        value: VaryingOperand,
        ic_index: VaryingOperand
    },

    /// Get a property by value from an object an push it on the stack.
    ///
    /// Like `object[key]`
    ///
    /// - Registers:
    ///   - Input: object, receiver, key
    ///   - Output: dst
    GetPropertyByValue emit_get_property_by_value {
        dst: VaryingOperand,
        key: VaryingOperand,
        receiver: VaryingOperand,
        object: VaryingOperand
    },

    /// Get a property by value from an object an push the key and value on the stack.
    ///
    /// Like `object[key]`
    ///
    /// - Registers:
    ///   - Input: object, receiver, key
    ///   - Output: dst
    GetPropertyByValuePush emit_get_property_by_value_push {
        dst: VaryingOperand,
        key: VaryingOperand,
        receiver: VaryingOperand,
        object: VaryingOperand
    },

    /// Sets a property by name of an object.
    ///
    /// Like `object.name = value`
    ///
    /// - Operands:
    ///   - ic_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object,receiver, value
    SetPropertyByName emit_set_property_by_name {
        value: VaryingOperand,
        receiver: VaryingOperand,
        object: VaryingOperand,
        ic_index: VaryingOperand
    },

    /// Sets the name of a function object.
    ///
    /// This operation is corresponds to the `SetFunctionName` abstract operation in the [spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-setfunctionname
    ///
    /// - Operands:
    ///   - prefix
    ///     - 0: no prefix
    ///     - 1: "get "
    ///     - 2: "set "
    /// - Registers:
    ///   - Input: function, name
    SetFunctionName emit_set_function_name { function: VaryingOperand, name: VaryingOperand, prefix: VaryingOperand },

    /// Defines a own property of an object by name.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineOwnPropertyByName emit_define_own_property_by_name { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Defines a static class method by name.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineClassStaticMethodByName emit_define_class_static_method_by_name {
        value: VaryingOperand,
        object: VaryingOperand,
        name_index: VaryingOperand
    },

    /// Defines a class method by name.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineClassMethodByName emit_define_class_method_by_name {
        value: VaryingOperand,
        object: VaryingOperand,
        name_index: VaryingOperand
    },

    /// Sets a property by value of an object.
    ///
    /// Like `object[key] = value`
    ///
    /// - Registers:
    ///   - Input: value, key, receiver, object
    SetPropertyByValue emit_set_property_by_value {
        value: VaryingOperand,
        key: VaryingOperand,
        receiver: VaryingOperand,
        object: VaryingOperand
    },

    /// Defines a own property of an object by value.
    ///
    /// - Registers:
    ///   - Input: object, key, value
    DefineOwnPropertyByValue emit_define_own_property_by_value {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Defines a static class method by value.
    ///
    /// - Registers:
    ///   - Input: object, key, value
    DefineClassStaticMethodByValue emit_define_class_static_method_by_value {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Defines a class method by value.
    ///
    /// - Registers:
    ///   - Input: object, key, value
    DefineClassMethodByValue emit_define_class_method_by_value {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Sets a getter property by name of an object.
    ///
    /// Like `get name() value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    SetPropertyGetterByName emit_set_property_getter_by_name { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Defines a static getter class method by name.
    ///
    /// Like `static get name() value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineClassStaticGetterByName emit_define_class_static_getter_by_name {
        value: VaryingOperand,
        object: VaryingOperand,
        name_index: VaryingOperand
    },

    /// Defines a getter class method by name.
    ///
    /// Like `get name() value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineClassGetterByName emit_define_class_getter_by_name {
        value: VaryingOperand,
        object: VaryingOperand,
        name_index: VaryingOperand
    },

    /// Sets a getter property by value of an object.
    ///
    /// Like `get [key]() value`
    ///
    /// - Registers:
    ///   - Input: object, key, value
    SetPropertyGetterByValue emit_set_property_getter_by_value {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Defines a static getter class method by value.
    ///
    /// Like `static get [key]() value`
    ///
    /// - Registers:
    ///   - Input: object, key, value
    DefineClassStaticGetterByValue emit_define_class_static_getter_by_value {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Defines a getter class method by value.
    ///
    /// Like `get [key]() value`
    ///
    /// - Registers:
    ///   - Input: object, key, value
    DefineClassGetterByValue emit_define_class_getter_by_value {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Sets a setter property by name of an object.
    ///
    /// Like `set name() value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    SetPropertySetterByName emit_set_property_setter_by_name { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Defines a static setter class method by name.
    ///
    /// Like `static set name() value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineClassStaticSetterByName emit_define_class_static_setter_by_name {
        value: VaryingOperand,
        object: VaryingOperand,
        name_index: VaryingOperand
    },

    /// Defines a setter class method by name.
    ///
    /// Like `set name() value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineClassSetterByName emit_define_class_setter_by_name {
        value: VaryingOperand,
        object: VaryingOperand,
        name_index: VaryingOperand
    },

    /// Sets a setter property by value of an object.
    ///
    /// Like `set [key]() value`
    ///
    /// - Registers:
    ///   - Input: object, key, value
    SetPropertySetterByValue emit_set_property_setter_by_value {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Defines a static setter class method by value.
    ///
    /// Like `static set [key]() value`
    ///
    /// - Registers:
    ///   - Input: object, key, value
    DefineClassStaticSetterByValue emit_define_class_static_setter_by_value {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Defines a setter class method by value.
    ///
    /// Like `set [key]() value`
    ///
    /// - Registers:
    ///   - Input: object, key, value
    DefineClassSetterByValue emit_define_class_setter_by_value {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Set the value of a private property of an object by it's name.
    ///
    /// Like `obj.#name = value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    SetPrivateField emit_set_private_field { value: VaryingOperand, object: VaryingOperand, name_index: VaryingOperand },

    /// Define a private property of a class constructor by it's name.
    ///
    /// Like `#name = value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefinePrivateField emit_define_private_field { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Set a private method of a class constructor by it's name.
    ///
    /// Like `#name() {}`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    SetPrivateMethod emit_set_private_method { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Set a private setter property of a class constructor by it's name.
    ///
    /// Like `set #name() {}`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    SetPrivateSetter emit_set_private_setter { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Set a private getter property of a class constructor by it's name.
    ///
    /// Like `get #name() {}`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    SetPrivateGetter emit_set_private_getter { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Get a private property by name from an object an push it on the stack.
    ///
    /// Like `object.#name`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object
    ///   - Output: dst
    GetPrivateField emit_get_private_field { dst: VaryingOperand, object: VaryingOperand, name_index: VaryingOperand },

    /// Push a field to a class.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    ///   - is_anonymous_function: `bool`
    /// - Registers:
    ///   - Input: object, value
    PushClassField emit_push_class_field { object: VaryingOperand, name_index: VaryingOperand, value: VaryingOperand, is_anonymous_function: VaryingOperand },

    /// Push a private field to the class.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    PushClassFieldPrivate emit_push_class_field_private { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Push a private getter to the class.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    PushClassPrivateGetter emit_push_class_private_getter { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Push a private setter to the class.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    PushClassPrivateSetter emit_push_class_private_setter { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Push a private method to the class.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, proto, value
    PushClassPrivateMethod emit_push_class_private_method { object: VaryingOperand, proto: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Deletes a property by name of an object.
    ///
    /// Like `delete object.key`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object
    DeletePropertyByName emit_delete_property_by_name { object: VaryingOperand, name_index: VaryingOperand },

    /// Deletes a property by value of an object.
    ///
    /// Like `delete object[key]`
    ///
    /// - Registers:
    ///   - Input: object, key
    DeletePropertyByValue emit_delete_property_by_value { object: VaryingOperand, key: VaryingOperand },

    /// Throws an error when trying to delete a property of `super`
    DeleteSuperThrow emit_delete_super_throw,

    /// Copy all properties of one object to another object.
    ///
    /// - Registers:
    ///   - Input: object, source, excluded_keys
    CopyDataProperties emit_copy_data_properties { object: VaryingOperand, source: VaryingOperand, excluded_keys: Vec<VaryingOperand> },

    /// Call ToPropertyKey on the value on the stack.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    ToPropertyKey emit_to_property_key{ src: VaryingOperand, dst: VaryingOperand },

    /// Unconditional jump to address.
    ///
    /// - Operands:
    ///   - address: `u32`
    Jump emit_jump { address: u32 },

    /// Conditional jump to address.
    ///
    /// If the value popped is [`truthy`][truthy] then jump to `address`.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Output: value
    ///
    /// [truthy]: https://developer.mozilla.org/en-US/docs/Glossary/Truthy
    JumpIfTrue emit_jump_if_true { address: u32, value: VaryingOperand },

    /// Conditional jump to address.
    ///
    /// If the value popped is [`falsy`][falsy] then jump to `address`.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Output: value
    ///
    /// [falsy]: https://developer.mozilla.org/en-US/docs/Glossary/Falsy
    JumpIfFalse emit_jump_if_false { address: u32, value: VaryingOperand },

    /// Conditional jump to address.
    ///
    /// If the value popped is not undefined jump to `address`.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Output: value
    JumpIfNotUndefined emit_jump_if_not_undefined { address: u32, value: VaryingOperand },

    /// Conditional jump to address.
    ///
    /// If the value popped is undefined jump to `address`.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Output: value
    JumpIfNullOrUndefined emit_jump_if_null_or_undefined { address: u32, value: VaryingOperand },

    /// Jump table that jumps depending on top value of the stack.
    ///
    /// This is used to handle special cases when we call `continue`, `break` or `return` in a try block,
    /// that has finally block.
    ///
    /// Operands: default: `u32`, count: `u32`, address: `u32` * count
    ///
    /// Stack: value: [`i32`] **=>**
    JumpTable emit_jump_table { default: u32, addresses: Vec<u32> },

    /// Throw exception.
    ///
    /// This sets pending exception and searches for an exception handler.
    ///
    /// - Registers:
    ///   - Input: src
    Throw emit_throw { src: VaryingOperand },

    /// Rethrow thrown exception.
    ///
    /// This is also used to handle generator `return()` call, we throw an empty exception, by setting pending exception to [`None`],
    /// propagating it and calling finally code until there is no exception handler left, in that case we consume the empty exception and return
    /// from the generator.
    ReThrow emit_re_throw,

    /// Get the thrown pending exception (if it's set) and push on the stack.
    ///
    /// If there is no pending exception, which can happend if we are handling `return()` call on generator,
    /// then we rethrow the empty exception. See [`Opcode::ReThrow`].
    ///
    /// - Registers:
    ///   - Output: dst
    Exception emit_exception { dst: VaryingOperand },

    /// Get the thrown pending exception if it's set and push `true`, otherwise push only `false`.
    ///
    /// - Registers:
    ///   - Output: exception, has_exception
    MaybeException emit_maybe_exception { has_exception: VaryingOperand, exception: VaryingOperand },

    /// Throw a new `TypeError` exception
    ///
    /// - Operands:
    ///   - message: `VaryingOperand`
    ThrowNewTypeError emit_throw_new_type_error { message: VaryingOperand },

    /// Throw a new `SyntaxError` exception
    ///
    /// - Operands:
    ///   - message: `VaryingOperand`
    ThrowNewSyntaxError emit_throw_new_syntax_error { message: VaryingOperand },

    /// Pushes `this` value
    ///
    /// - Registers:
    ///   - Output: dst
    This emit_this { dst: VaryingOperand },

    /// Pushes `this` value that is related to the object environment of the given binding
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    ThisForObjectEnvironmentName emit_this_for_object_environment_name { dst: VaryingOperand, index: VaryingOperand },

    /// Pushes the current `super` value to the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    Super emit_super { dst: VaryingOperand },

    /// Get the super constructor and the new target of the current environment.
    ///
    /// - Registers:
    ///   - Output: dst
    SuperCallPrepare emit_super_call_prepare { dst: VaryingOperand },

    /// Execute the `super()` method.
    ///
    /// - Operands:
    ///   - argument_count: `VaryingOperand`
    /// - Stack: super_constructor, argument_1, ... argument_n **=>**
    SuperCall emit_super_call { argument_count: VaryingOperand },

    /// Execute the `super()` method where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: super_constructor, arguments_array **=>**
    SuperCallSpread emit_super_call_spread,

    /// Execute the `super()` method when no constructor of the class is defined.
    ///
    /// Operands:
    ///
    /// Stack: argument_n, ... argument_1 **=>**
    SuperCallDerived emit_super_call_derived,

    /// Binds `this` value and initializes the instance elements.
    ///
    /// Performs steps 7-12 of [`SuperCall: super Arguments`][spec]
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-super-keyword-runtime-semantics-evaluation
    BindThisValue emit_bind_this_value { value: VaryingOperand },

    /// Dynamically import a module.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    ImportCall emit_import_call { value: VaryingOperand },

    /// Pop the two values of the stack, strict equal compares the two values,
    /// if true jumps to address, otherwise push the second pop'ed value.
    ///
    /// Operands: address: `u32`
    ///
    /// Stack: value, cond **=>** cond (if `cond !== value`).
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Input: value, condition
    Case emit_case { address: u32, value: VaryingOperand, condition: VaryingOperand },

    /// Get function from the pre-compiled inner functions.
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetFunction emit_get_function { dst: VaryingOperand, index: VaryingOperand },

    /// Call a function named "eval".
    ///
    /// - Operands:
    ///   - argument_count: `VaryingOperand`
    ///   - scope_index: `VaryingOperand`
    /// - Stack: this, func, argument_1, ... argument_n **=>** result
    CallEval emit_call_eval { argument_count: VaryingOperand, scope_index: VaryingOperand },

    /// Call a function named "eval" where the arguments contain spreads.
    ///
    /// - Operands:
    ///   - scope_index: `VaryingOperand`
    /// - Stack: Stack: this, func, arguments_array **=>** result
    CallEvalSpread emit_call_eval_spread { scope_index: VaryingOperand },

    /// Call a function.
    ///
    /// - Operands:
    ///   - argument_count: `VaryingOperand`
    /// - Stack: this, func, argument_1, ... argument_n **=>** result
    Call emit_call { argument_count: VaryingOperand },

    /// Call a function where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: this, func, arguments_array **=>** result
    CallSpread emit_call_spread,

    /// Call construct on a function.
    ///
    /// - Operands:
    ///   - argument_count: `VaryingOperand`
    /// - Stack: this, func, argument_1, ... argument_n **=>** result
    New emit_new { argument_count: VaryingOperand },

    /// Call construct on a function where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: arguments_array, func **=>** result
    NewSpread emit_new_spread,

    /// Check return from a function.
    CheckReturn emit_check_return,

    /// Return from a function.
    Return emit_return,

    /// Close an async generator function.
    AsyncGeneratorClose emit_async_generator_close,

    /// Creates the generator object and yields.
    ///
    /// - Operands:
    ///   - async: `bool`
    /// - Stack: **=>** resume_kind
    Generator emit_generator { r#async: VaryingOperand },

    /// Set return value of a function.
    ///
    /// - Registers:
    ///   - Input: src
    SetAccumulator emit_set_accumulator { src: VaryingOperand },

    // Set return value of a function.
    ///
    /// - Registers:
    ///   - Output: dst
    SetRegisterFromAccumulator emit_set_register_from_accumulator { dst: VaryingOperand },

    /// Move value of operand `src` to register `dst`.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    Move emit_move { dst: VaryingOperand, src: VaryingOperand },

    /// Pop value from the stack and push to register `dst`
    ///
    /// - Registers:
    ///   - Output: dst
    PopIntoRegister emit_pop_into_register { dst: VaryingOperand },

    /// Copy value at register `src` and push it on the stack.
    ///
    /// - Registers:
    ///   - Input: src
    PushFromRegister emit_push_from_register { src: VaryingOperand },

    /// Pop value from the stack and push to a local binding register `dst`.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    PopIntoLocal emit_pop_into_local { src: VaryingOperand, dst: VaryingOperand },

    /// Copy value at local binding register `src` and push it into the stack.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    PushFromLocal emit_push_from_local { src: VaryingOperand, dst: VaryingOperand },

    /// Push a declarative environment.
    ///
    /// - Operands:
    ///   - scope_index: `VaryingOperand`
    PushScope emit_push_scope { scope_index: VaryingOperand },

    /// Push an object environment.
    ///
    /// - Registers:
    ///   - Input: src
    PushObjectEnvironment emit_push_object_environment { src: VaryingOperand },

    /// Pop the current environment.
    PopEnvironment emit_pop_environment,

    /// Increment loop iteration count.
    ///
    /// Used for limiting the loop iteration.
    IncrementLoopIteration emit_increment_loop_iteration,

    /// Creates the ForInIterator of an object.
    ///
    /// - Registers:
    ///   - Input: src
    CreateForInIterator emit_create_for_in_iterator { src: VaryingOperand },

    /// Gets the iterator of an object.
    ///
    /// - Registers:
    ///   - Input: src
    /// - Iterator Stack: **=>** `iterator`
    GetIterator emit_get_iterator { src: VaryingOperand },

    /// Gets the async iterator of an object.
    ///
    /// - Registers:
    ///   - Input: src
    /// - Iterator Stack: **=>** `iterator`
    GetAsyncIterator emit_get_async_iterator { src: VaryingOperand },

    /// Calls the `next` method of `iterator`, updating its record with the next value.
    ///
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorNext emit_iterator_next,

    /// Returns `true` if the current iterator is done, or `false` otherwise
    ///
    /// - Registers:
    ///   - Output: dst
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorDone emit_iterator_done { dst: VaryingOperand },

    /// Finishes the call to `Opcode::IteratorNext` within a `for await` loop by setting the current
    /// result of the current iterator.
    ///
    /// - Registers:
    ///   - Input: resume_kind, value
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorFinishAsyncNext emit_iterator_finish_async_next { resume_kind: VaryingOperand, value: VaryingOperand },

    /// Gets the `value` property of the current iterator record.
    ///
    /// - Registers:
    ///   - Output: dst
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorValue emit_iterator_value { dst: VaryingOperand },

    /// Gets the last iteration result of the current iterator record.
    ///
    /// - Registers:
    ///   - Output: dst
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorResult emit_iterator_result { dst: VaryingOperand },

    /// Consume the iterator and construct and array with all the values.
    ///
    /// - Registers:
    ///   - Output: dst
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorToArray emit_iterator_to_array { dst: VaryingOperand },

    /// Pushes `true` to the stack if the iterator stack is empty.
    ///
    /// - Registers:
    ///   - Output: dst
    /// - Iterator Stack: **=>**
    IteratorStackEmpty emit_iterator_stack_empty { dst: VaryingOperand },

    /// Creates a new iterator result object.
    ///
    /// - Operands:
    ///   - done: `bool` (codified as u8 with `0` -> `false` and `!0` -> `true`)
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    CreateIteratorResult emit_create_iterator_result { value: VaryingOperand, done: VaryingOperand },

    /// Calls `return` on the current iterator and returns the result.
    ///
    /// - Registers:
    ///   - Output: value, called
    /// - Iterator Stack: `iterator` **=>**
    IteratorReturn emit_iterator_return { value: VaryingOperand, called: VaryingOperand },

    /// Concat multiple stack objects into a string.
    ///
    /// - Registers:
    ///   - Input: values
    ///   - Output: dst
    ConcatToString emit_concat_to_string { dst: VaryingOperand, values: Vec<VaryingOperand> },

    /// Require the stack value to be neither null nor undefined.
    ///
    /// - Registers:
    ///   - Input: src
    ValueNotNullOrUndefined emit_value_not_null_or_undefined { src: VaryingOperand },

    /// Initialize the rest parameter value of a function from the remaining arguments.
    ///
    /// - Stack: `argument_1` .. `argument_n` **=>**
    /// - Registers:
    ///   - Output: dst
    RestParameterInit emit_rest_parameter_init { dst: VaryingOperand },

    /// Yields from the current generator execution.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: resume_kind, received
    GeneratorYield emit_generator_yield { src: VaryingOperand },

    /// Resumes the current generator function.
    ///
    /// If the `resume_kind` is `Throw`, then the value is poped and thrown, otherwise if `Return`
    /// we pop the value, set it as the return value and throw and empty exception. See [`Opcode::ReThrow`].
    ///
    /// - Registers:
    ///   - Input: resume_kind, value
    GeneratorNext emit_generator_next { resume_kind: VaryingOperand, value: VaryingOperand },

    /// Yields from the current async generator execution.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: resume_kind, received
    AsyncGeneratorYield emit_async_generator_yield { src: VaryingOperand },

    /// Create a promise capacity for an async function, if not already set.
    CreatePromiseCapability emit_create_promise_capability,

    /// Resolves or rejects the promise capability of an async function.
    ///
    /// If the pending exception is set, reject and rethrow the exception, otherwise resolve.
    CompletePromiseCapability emit_complete_promise_capability,

    /// Jumps to the specified address if the resume kind is not equal.
    ///
    /// - Operands:
    ///   - address: `u32`
    ///   - resume_kind: `GeneratorResumeKind`
    /// - Registers:
    ///   - Input: src
    JumpIfNotResumeKind emit_jump_if_not_resume_kind { address: u32, resume_kind: VaryingOperand, src: VaryingOperand },

    /// Delegates the current async generator function to another iterator.
    ///
    /// - Operands:
    ///   - throw_method_undefined: `u32`,
    ///   - return_method_undefined: `u32`
    /// - Registers:
    ///   - Input: value, resume_kind
    ///   - Output: value, is_return
    GeneratorDelegateNext emit_generator_delegate_next {
        throw_method_undefined: u32,
        return_method_undefined: u32,
        value: VaryingOperand,
        resume_kind: VaryingOperand,
        is_return: VaryingOperand
    },

    /// Resume the async generator with yield delegate logic after it awaits a value.
    ///
    /// - Operands:
    ///   - r#return: `u32`,
    ///   - exit: `u32`
    /// - Registers:
    ///   - Input: value, resume_kind, is_return
    ///   - Output: value
    GeneratorDelegateResume emit_generator_delegate_resume {
        r#return: u32,
        exit: u32,
        value: VaryingOperand,
        resume_kind: VaryingOperand,
        is_return: VaryingOperand
    },

    /// Stops the current async function and schedules it to resume later.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: resume_kind, received
    Await emit_await{ src: VaryingOperand },

    /// Push the current new target to the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    NewTarget emit_new_target { dst: VaryingOperand },

    /// Push the current `import.meta` to the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    ImportMeta emit_import_meta { dst: VaryingOperand },

    /// Pushes `true` to the stack if the top stack value is an object, or `false` otherwise.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    IsObject emit_is_object { value: VaryingOperand },

    /// Lookup if a tagged template object is cached and skip the creation if it is.
    ///
    /// - Operands:
    ///   - address: `u32`
    ///   - site: `u64`
    /// - Registers:
    ///   - Output: dst
    TemplateLookup emit_template_lookup { address: u32, site: u64, dst: VaryingOperand },

    /// Create a new tagged template object and cache it.
    ///
    /// - Operands:
    ///   - site: `u64`
    /// - Registers:
    ///   - Inputs: values
    ///   - Output: dst
    TemplateCreate emit_template_create { site: u64, dst: VaryingOperand, values: Vec<u32> },

    /// Push a private environment.
    ///
    /// Operands: count: `u32`, count * name_index: `u32`
    ///
    /// - Registers:
    ///   - Input: class, name_indices
    PushPrivateEnvironment emit_push_private_environment { class: VaryingOperand, name_indices: Vec<u32> },

    /// Pop a private environment.
    PopPrivateEnvironment emit_pop_private_environment,

    /// Creates a mapped `arguments` object.
    ///
    /// Performs [`10.4.4.7 CreateMappedArgumentsObject ( func, formals, argumentsList, env )`]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createunmappedargumentsobject
    ///
    /// - Registers:
    ///   - Output: dst
    CreateMappedArgumentsObject emit_create_mapped_arguments_object { dst: VaryingOperand },

    /// Creates an unmapped `arguments` object.
    ///
    /// Performs: [`10.4.4.6 CreateUnmappedArgumentsObject ( argumentsList )`]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createmappedargumentsobject
    ///
    /// - Registers:
    ///   - Output: dst
    CreateUnmappedArgumentsObject emit_create_unmapped_arguments_object { dst: VaryingOperand },

    /// Performs [`HasRestrictedGlobalProperty ( N )`][spec]
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasrestrictedglobalproperty
    HasRestrictedGlobalProperty emit_has_restricted_global_property { dst: VaryingOperand, index: VaryingOperand },

    /// Performs [`CanDeclareGlobalFunction ( N )`][spec]
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalfunction
    CanDeclareGlobalFunction emit_can_declare_global_function { dst: VaryingOperand, index: VaryingOperand },

    /// Performs [`CanDeclareGlobalVar ( N )`][spec]
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalvar
    CanDeclareGlobalVar emit_can_declare_global_var { dst: VaryingOperand, index: VaryingOperand },

    /// Performs [`CreateGlobalFunctionBinding ( N, V, D )`][spec]
    ///
    /// - Operands:
    ///   - configurable: `bool`
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: src
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createglobalfunctionbinding
    CreateGlobalFunctionBinding emit_create_global_function_binding { src: VaryingOperand, configurable: VaryingOperand, name_index: VaryingOperand },

    /// Performs [`CreateGlobalVarBinding ( N, V, D )`][spec]
    ///
    /// - Operands:
    ///   - configurable: `bool`
    ///   - name_index: `VaryingOperand`
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createglobalvarbinding
    CreateGlobalVarBinding emit_create_global_var_binding { configurable: VaryingOperand, name_index: VaryingOperand },

    /// Reserved [`Opcode`].
    Reserved1 => Reserved,
    /// Reserved [`Opcode`].
    Reserved2 => Reserved,
    /// Reserved [`Opcode`].
    Reserved3 => Reserved,
    /// Reserved [`Opcode`].
    Reserved4 => Reserved,
    /// Reserved [`Opcode`].
    Reserved5 => Reserved,
    /// Reserved [`Opcode`].
    Reserved6 => Reserved,
    /// Reserved [`Opcode`].
    Reserved7 => Reserved,
    /// Reserved [`Opcode`].
    Reserved8 => Reserved,
    /// Reserved [`Opcode`].
    Reserved9 => Reserved,
    /// Reserved [`Opcode`].
    Reserved10 => Reserved,
    /// Reserved [`Opcode`].
    Reserved11 => Reserved,
    /// Reserved [`Opcode`].
    Reserved12 => Reserved,
    /// Reserved [`Opcode`].
    Reserved13 => Reserved,
    /// Reserved [`Opcode`].
    Reserved14 => Reserved,
    /// Reserved [`Opcode`].
    Reserved15 => Reserved,
    /// Reserved [`Opcode`].
    Reserved16 => Reserved,
    /// Reserved [`Opcode`].
    Reserved17 => Reserved,
    /// Reserved [`Opcode`].
    Reserved18 => Reserved,
    /// Reserved [`Opcode`].
    Reserved19 => Reserved,
    /// Reserved [`Opcode`].
    Reserved20 => Reserved,
    /// Reserved [`Opcode`].
    Reserved21 => Reserved,
    /// Reserved [`Opcode`].
    Reserved22 => Reserved,
    /// Reserved [`Opcode`].
    Reserved23 => Reserved,
    /// Reserved [`Opcode`].
    Reserved24 => Reserved,
    /// Reserved [`Opcode`].
    Reserved25 => Reserved,
    /// Reserved [`Opcode`].
    Reserved26 => Reserved,
    /// Reserved [`Opcode`].
    Reserved27 => Reserved,
    /// Reserved [`Opcode`].
    Reserved28 => Reserved,
    /// Reserved [`Opcode`].
    Reserved29 => Reserved,
    /// Reserved [`Opcode`].
    Reserved30 => Reserved,
    /// Reserved [`Opcode`].
    Reserved31 => Reserved,
    /// Reserved [`Opcode`].
    Reserved32 => Reserved,
    /// Reserved [`Opcode`].
    Reserved33 => Reserved,
    /// Reserved [`Opcode`].
    Reserved34 => Reserved,
    /// Reserved [`Opcode`].
    Reserved35 => Reserved,
    /// Reserved [`Opcode`].
    Reserved36 => Reserved,
    /// Reserved [`Opcode`].
    Reserved37 => Reserved,
    /// Reserved [`Opcode`].
    Reserved38 => Reserved,
    /// Reserved [`Opcode`].
    Reserved39 => Reserved,
    /// Reserved [`Opcode`].
    Reserved40 => Reserved,
    /// Reserved [`Opcode`].
    Reserved41 => Reserved,
    /// Reserved [`Opcode`].
    Reserved42 => Reserved,
    /// Reserved [`Opcode`].
    Reserved43 => Reserved,
    /// Reserved [`Opcode`].
    Reserved44 => Reserved,
    /// Reserved [`Opcode`].
    Reserved45 => Reserved,
    /// Reserved [`Opcode`].
    Reserved46 => Reserved,
    /// Reserved [`Opcode`].
    Reserved47 => Reserved,
    /// Reserved [`Opcode`].
    Reserved48 => Reserved,
    /// Reserved [`Opcode`].
    Reserved49 => Reserved,
    /// Reserved [`Opcode`].
    Reserved50 => Reserved,
    /// Reserved [`Opcode`].
    Reserved51 => Reserved,
    /// Reserved [`Opcode`].
    Reserved52 => Reserved,
    /// Reserved [`Opcode`].
    Reserved53 => Reserved,
    /// Reserved [`Opcode`].
    Reserved54 => Reserved,
    /// Reserved [`Opcode`].
    Reserved55 => Reserved,
    /// Reserved [`Opcode`].
    Reserved56 => Reserved,
    /// Reserved [`Opcode`].
    Reserved57 => Reserved,
    /// Reserved [`Opcode`].
    Reserved58 => Reserved,
    /// Reserved [`Opcode`].
    Reserved59 => Reserved,
    /// Reserved [`Opcode`].
    Reserved60 => Reserved,
    /// Reserved [`Opcode`].
    Reserved61 => Reserved,
    /// Reserved [`Opcode`].
    Reserved62 => Reserved,
}
