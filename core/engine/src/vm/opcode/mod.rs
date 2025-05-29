#![allow(clippy::inline_always)]

use crate::{
    vm::{completion_record::CompletionRecord, completion_record::IntoCompletionRecord},
    Context,
};
use args::{read, Argument};
use std::ops::ControlFlow;
use thin_vec::ThinVec;

mod args;

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
}

/// The compile time representation of bytecode instructions.
#[derive(Debug)]
pub(crate) struct ByteCodeEmitter {
    bytecode: Vec<u8>,
}

impl ByteCodeEmitter {
    /// Create a new [`ByteCodeEmitter`] instance.
    pub(crate) fn new() -> Self {
        Self {
            bytecode: Vec::new(),
        }
    }

    /// Convert the [`ByteCodeEmitter`] into a [`ByteCode`] instance.
    pub(crate) fn into_bytecode(self) -> ByteCode {
        ByteCode {
            bytecode: self.bytecode.into_boxed_slice(),
        }
    }

    /// Get the location of the next opcode in the bytecode.
    pub(crate) fn next_opcode_location(&self) -> u32 {
        self.bytecode.len() as u32
    }

    /// Patch the jump instruction at the given label with the given address.
    pub(crate) fn patch_jump(&mut self, label: u32, patch: u32) {
        let pos = label as usize;
        let bytes = patch.to_le_bytes();
        self.bytecode[pos + 1] = bytes[0];
        self.bytecode[pos + 2] = bytes[1];
        self.bytecode[pos + 3] = bytes[2];
        self.bytecode[pos + 4] = bytes[3];
    }

    /// Patch the jump instruction at the given label with two addresses.
    pub(crate) fn patch_jump_two_addresses(&mut self, label: u32, patch: (u32, u32)) {
        let pos = label as usize;
        // Write first patched value
        let bytes = patch.0.to_le_bytes();
        self.bytecode[pos + 1] = bytes[0];
        self.bytecode[pos + 2] = bytes[1];
        self.bytecode[pos + 3] = bytes[2];
        self.bytecode[pos + 4] = bytes[3];

        // Write second patched value
        let bytes = patch.1.to_le_bytes();
        self.bytecode[pos + 5] = bytes[0];
        self.bytecode[pos + 6] = bytes[1];
        self.bytecode[pos + 7] = bytes[2];
        self.bytecode[pos + 8] = bytes[3];
    }

    /// Patch the jump instruction at the given label with jump table addresses.
    pub(crate) fn patch_jump_table(&mut self, label: u32, patch: (u32, &[u32])) {
        let pos = label as usize;

        let (total_len, pos) = read::<u16>(&self.bytecode, pos + 1);
        let arg_count = total_len as usize;
        assert_eq!(arg_count, patch.1.len());

        // Write first patched value (default address)
        let bytes = patch.0.to_le_bytes();
        self.bytecode[pos] = bytes[0];
        self.bytecode[pos + 1] = bytes[1];
        self.bytecode[pos + 2] = bytes[2];
        self.bytecode[pos + 3] = bytes[3];

        // Write remaining patched values
        for (i, value) in patch.1.iter().enumerate() {
            let offset = pos + 4 + (i) * 4;
            let bytes = value.to_le_bytes();
            self.bytecode[offset] = bytes[0];
            self.bytecode[offset + 1] = bytes[1];
            self.bytecode[offset + 2] = bytes[2];
            self.bytecode[offset + 3] = bytes[3];
        }
    }
}

#[derive(Clone, Debug, Default)]
/// The bytecode representation of a codeblock.
pub(crate) struct ByteCode {
    pub(crate) bytecode: Box<[u8]>,
}

/// The enum representation of [`VaryingOperand`] values.
enum VaryingOperandVariant {
    U8(u8),
    U16(u16),
    U32(u32),
}

#[derive(Debug, Clone, Copy)]
/// A varying operand is a value that can be either a u8, u16 or u32.
pub(crate) struct VaryingOperand {
    value: u32,
}

impl VaryingOperand {
    /// Create a new [`VaryingOperand`] from a u32 value.
    pub(crate) fn new(value: u32) -> Self {
        Self { value }
    }

    /// Return the variant of the [`VaryingOperand`].
    fn variant(self) -> VaryingOperandVariant {
        if let Ok(value) = u8::try_from(self.value) {
            VaryingOperandVariant::U8(value)
        } else if let Ok(value) = u16::try_from(self.value) {
            VaryingOperandVariant::U16(value)
        } else {
            VaryingOperandVariant::U32(self.value)
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

impl std::fmt::Display for VaryingOperand {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Opcode {
    fn encode(self) -> u8 {
        self as u8
    }

    pub(crate) fn decode(instruction: u8) -> Self {
        Self::from(instruction)
    }
}

fn encode_instruction<A: Argument>(opcode: Opcode, args: A, bytes: &mut Vec<u8>) {
    bytes.push(opcode.encode());
    args.encode(bytes);
}

macro_rules! generate_opcodes {
    (
        $(
            $(#[$comment:ident $($args:tt)*])*
            $Variant:ident $({
                $(
                    $(#[$fieldinner:ident $($fieldargs:tt)*])*
                    $FieldName:ident : $FieldType:ty
                ),*
            })? $(=> $mapping:ident)?
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

        impl Opcode {
            pub(crate) fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$Variant => $Variant::NAME),*
                }
            }
        }

        impl ByteCodeEmitter {
            $(
                paste::paste! {
                    #[allow(unused)]
                    pub(crate) fn [<emit_ $Variant:snake>](&mut self $( $(, $FieldName: $FieldType)* )? ) {
                        encode_instruction(
                            Opcode::$Variant,
                            ($($($FieldName),*)?),
                            &mut self.bytecode,
                        );
                    }
                }
            )*
        }

        type OpcodeHandler = fn(&mut Context, usize) -> ControlFlow<CompletionRecord>;

        const OPCODE_HANDLERS: [OpcodeHandler; 256] = {
            [
                $(
                    paste::paste! { [<handle_ $Variant:snake>] },
                )*
            ]
        };

        type OpcodeHandlerBudget = fn(&mut Context, usize, &mut u32) -> ControlFlow<CompletionRecord>;

        const OPCODE_HANDLERS_BUDGET: [OpcodeHandlerBudget; 256] = {
            [
                $(
                    paste::paste! { [<handle_ $Variant:snake _budget>] },
                )*
            ]
        };

        $(
            paste::paste! {
                #[inline(always)]
                #[allow(unused_parens)]
                fn [<handle_ $Variant:snake>](context: &mut Context, pc: usize) -> ControlFlow<CompletionRecord> {
                    let bytes = &context.vm.frame.code_block.bytecode.bytecode;
                    let (args, next_pc) = <($($($FieldType),*)?)>::decode(bytes, pc + 1);
                    context.vm.frame_mut().pc = next_pc as u32;
                    let result = $Variant::operation(args, context);
                    IntoCompletionRecord::into_completion_record(result, context)
                }
            }
        )*

        $(
            paste::paste! {
                #[inline(always)]
                #[allow(unused_parens)]
                fn [<handle_ $Variant:snake _budget>](context: &mut Context, pc: usize, budget: &mut u32) -> ControlFlow<CompletionRecord> {
                    *budget = budget.saturating_sub(u32::from($Variant::COST));
                    let bytes = &context.vm.frame.code_block.bytecode.bytecode;
                    let (args, next_pc) = <($($($FieldType),*)?)>::decode(bytes, pc + 1);
                    context.vm.frame_mut().pc = next_pc as u32;
                    let result = $Variant::operation(args, context);
                    IntoCompletionRecord::into_completion_record(result, context)
                }
            }
        )*

        $(
            $(
                struct $Variant {}
                impl $Variant {
                    #[allow(unused_parens)]
                    #[allow(unused_variables)]
                    #[inline(always)]
                    fn operation(args: (), context: &mut Context) {
                        $mapping::operation(args, context)
                    }
                }

                impl Operation for $Variant {
                    const NAME: &'static str = "Reserved";
                    const INSTRUCTION: &'static str = "INST - Reserved";
                    const COST: u8 = 0;
                }
            )?
        )*

        pub(crate) enum Instruction {
            $(
                $Variant $({
                    $(
                        $(#[$fieldinner $($fieldargs)*])*
                        $FieldName : $FieldType
                    ),*
                })?
            ),*
        }

        impl ByteCode {
            #[allow(unused_parens)]
            pub(crate) fn next_instruction(&self, pc: usize) -> (Instruction, usize) {
                let bytes = &self.bytecode;
                let opcode = Opcode::decode(bytes[pc]);

                match opcode {
                    $(
                        Opcode::$Variant => {
                            let (($($($FieldName),*)?), read_size) = <($($($FieldType),*)?)>::decode(bytes, pc + 1);
                            (Instruction::$Variant $({
                                $(
                                    $FieldName: $FieldName
                                ),*
                            })?, read_size)
                        }
                    ),*
                }
            }
        }
    }
}

impl Context {
    pub(crate) fn execute_bytecode_instruction(
        &mut self,
        opcode: Opcode,
    ) -> ControlFlow<CompletionRecord> {
        let frame = self.vm.frame_mut();
        let pc = frame.pc as usize;

        OPCODE_HANDLERS[opcode as usize](self, pc)
    }

    pub(crate) fn execute_bytecode_instruction_with_budget(
        &mut self,
        budget: &mut u32,
        opcode: Opcode,
    ) -> ControlFlow<CompletionRecord> {
        let frame = self.vm.frame_mut();
        let pc = frame.pc as usize;

        OPCODE_HANDLERS_BUDGET[opcode as usize](self, pc, budget)
    }
}

/// Iterator over the instructions in the compact bytecode.
// #[derive(Debug, Clone)]
pub(crate) struct InstructionIterator<'bytecode> {
    bytes: &'bytecode ByteCode,
    pc: usize,
}

// TODO: see if this can be exposed on all features.
// #[allow(unused)]
impl<'bytecode> InstructionIterator<'bytecode> {
    /// Create a new [`InstructionIterator`] from bytecode array.
    #[inline]
    #[must_use]
    pub(crate) const fn new(bytes: &'bytecode ByteCode) -> Self {
        Self { bytes, pc: 0 }
    }

    /// Get the current program counter.
    #[inline]
    #[must_use]
    pub(crate) const fn pc(&self) -> usize {
        self.pc
    }
}

impl Iterator for InstructionIterator<'_> {
    type Item = (usize, Opcode, Instruction);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let start_pc = self.pc;
        if self.pc >= self.bytes.bytecode.len() {
            return None;
        }

        let bytes = &self.bytes.bytecode;
        let opcode = Opcode::decode(bytes[self.pc]);
        // Get instruction and determine how much to advance pc
        let (instruction, read_size) = self.bytes.next_instruction(self.pc);
        self.pc = read_size;
        Some((start_pc, opcode, instruction))
    }
}

generate_opcodes! {
    /// Pop the top value from the stack.
    ///
    /// - Stack: value **=>**
    Pop,

    /// Push integer `0` on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushZero { dst: VaryingOperand },

    /// Push integer `1` on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushOne { dst: VaryingOperand },

    /// Push `i8` value on the stack.
    ///
    /// - Operands:
    ///   - value: `i8`
    /// - Registers:
    ///   - Output: dst
    PushInt8 { dst: VaryingOperand, value: i8 },

    /// Push i16 value on the stack.
    ///
    /// - Operands:
    ///   - value: `i16`
    /// - Registers:
    ///   - Output: dst
    PushInt16 { dst: VaryingOperand, value: i16 },

    /// Push i32 value on the stack.
    ///
    /// - Operands:
    ///   - value: `i32`
    /// - Registers:
    ///   - Output: dst
    PushInt32 { dst: VaryingOperand, value: i32 },

    /// Push `f32` value on the stack.
    ///
    /// - Operands:
    ///   - value: `f32`
    /// - Registers:
    ///   - Output: dst
    PushFloat { dst: VaryingOperand, value: f32 },

    /// Push `f64` value on the stack.
    ///
    /// - Operands:
    ///   - value: `f64`
    /// - Registers:
    ///   - Output: dst
    PushDouble { dst: VaryingOperand, value: f64 },

    /// Push `NaN` integer on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushNan { dst: VaryingOperand },

    /// Push `Infinity` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushPositiveInfinity { dst: VaryingOperand },

    /// Push `-Infinity` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushNegativeInfinity { dst: VaryingOperand },

    /// Push `null` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushNull { dst: VaryingOperand },

    /// Push `true` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushTrue { dst: VaryingOperand },

    /// Push `false` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushFalse { dst: VaryingOperand },

    /// Push `undefined` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushUndefined { dst: VaryingOperand },

    /// Push literal value on the stack.
    ///
    /// Like strings and bigints. The index operand is used to index into the `literals`
    /// array to get the value.
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    PushLiteral { dst: VaryingOperand, index: VaryingOperand },

    /// Push regexp value on the stack.
    ///
    /// - Operands:
    ///   - pattern_index: `VaryingOperand`
    ///   - flags: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    PushRegexp { dst: VaryingOperand, pattern_index: VaryingOperand, flags_index: VaryingOperand },

    /// Push empty object `{}` value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushEmptyObject { dst: VaryingOperand },

    /// Get the prototype of a superclass and push it on the stack.
    ///
    /// Additionally this sets the `[[prototype]]` of the class and the `DERIVED` flag.
    ///
    /// - Registers:
    ///   - Input: class, superclass
    ///   - Output: dst
    PushClassPrototype {
        dst: VaryingOperand,
        class: VaryingOperand,
        superclass: VaryingOperand
    },

    /// Set the prototype of a class object.
    ///
    /// - Registers:
    ///   - Input: class, prototype
    ///   - Output: dst
    SetClassPrototype {
        dst: VaryingOperand,
        prototype: VaryingOperand,
        class: VaryingOperand
    },

    /// Set home object internal slot of an object literal method.
    ///
    /// - Registers:
    ///   - Input: function, home
    SetHomeObject {
        function: VaryingOperand,
        home: VaryingOperand
    },

    /// Set the prototype of an object if the value is an object or null.
    ///
    /// - Registers:
    ///   - Input: object, prototype
    SetPrototype {
        object: VaryingOperand,
        prototype: VaryingOperand
    },

    /// Push an empty array value on the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    PushNewArray { dst: VaryingOperand },

    /// Push a value to an array.
    ///
    /// - Registers:
    ///   - Input: array, value
    PushValueToArray { value: VaryingOperand, array: VaryingOperand },

    /// Push an empty element/hole to an array.
    ///
    /// - Registers:
    ///   - Input: array
    PushElisionToArray { array: VaryingOperand },

    /// Push all iterator values to an array.
    ///
    /// - Registers:
    ///   - Input: array
    PushIteratorToArray { array: VaryingOperand },

    /// Binary `+` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Add { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `-` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Sub { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `/` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Div { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `*` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Mul { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `%` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Mod { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `**` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Pow { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `>>` operator.
    ///
    /// - Registers
    ///   - Input: lhs, rhs
    ///   - Output: dst
    ShiftRight { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `<<` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    ShiftLeft { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `>>>` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    UnsignedShiftRight { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary bitwise `|` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    BitOr { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary bitwise `&` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    BitAnd { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary bitwise `^` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    BitXor { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Unary bitwise `~` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    BitNot { value: VaryingOperand },

    /// Binary `in` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    In { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `in` operator for private names.
    ///
    /// - Operands: index: `u32`
    /// - Registers:
    ///   - Input: rhs
    ///   - Output: dst
    InPrivate { dst: VaryingOperand, index: VaryingOperand, rhs: VaryingOperand },

    /// Binary `==` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    Eq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `===` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    StrictEq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `!=` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    NotEq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `!==` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    StrictNotEq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `>` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    GreaterThan { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `>=` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    GreaterThanOrEq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `<` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    LessThan { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `<=` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    LessThanOrEq { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary `instanceof` operator.
    ///
    /// - Registers:
    ///   - Input: lhs, rhs
    ///   - Output: dst
    InstanceOf { dst: VaryingOperand, lhs: VaryingOperand, rhs: VaryingOperand },

    /// Binary logical `&&` operator.
    ///
    /// This is a short-circuit operator, if the `lhs` value is `false`, then it jumps to `exit` address.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    LogicalAnd { address: u32, value: VaryingOperand },

    /// Binary logical `||` operator.
    ///
    /// This is a short-circuit operator, if the `lhs` value is `true`, then it jumps to `exit` address.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    LogicalOr { address: u32, value: VaryingOperand },

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
    Coalesce { address: u32, value: VaryingOperand },

    /// Unary `typeof` operator.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    TypeOf { value: VaryingOperand },

    /// Unary logical `!` operator.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    LogicalNot { value: VaryingOperand },

    /// Unary `+` operator.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    Pos { value: VaryingOperand },

    /// Unary `-` operator.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    Neg { value: VaryingOperand },

    /// Unary `++` operator.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    Inc { dst: VaryingOperand, src: VaryingOperand },

    /// Unary `--` operator.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    Dec { dst: VaryingOperand, src: VaryingOperand },

    /// Declare `var` type variable.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    DefVar { binding_index: VaryingOperand },

    /// Declare and initialize `var` type variable.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: src
    DefInitVar { src: VaryingOperand, binding_index: VaryingOperand },

    /// Initialize a lexical binding.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: src
    PutLexicalValue { src: VaryingOperand, binding_index: VaryingOperand },

    /// Throws an error because the binding access is illegal.
    ///
    /// - Operands:
    ///   -index: `VaryingOperand`
    ThrowMutateImmutable { index: VaryingOperand },

    /// Get i-th argument of the current frame.
    ///
    /// Returns `undefined` if `arguments.len()` < `index`.
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetArgument { index: VaryingOperand, dst: VaryingOperand },

    /// Find a binding on the environment chain and push its value.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetName { dst: VaryingOperand, binding_index: VaryingOperand },

    /// Find a binding in the global object and push its value.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    ///   - ic_index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetNameGlobal { dst: VaryingOperand, binding_index: VaryingOperand, ic_index: VaryingOperand },

    /// Find a binding on the environment and set the `current_binding` of the current frame.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    GetLocator { binding_index: VaryingOperand },

    /// Find a binding on the environment chain and push its value to the stack and its
    /// `BindingLocator` to the `bindings_stack`.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetNameAndLocator { dst: VaryingOperand, binding_index: VaryingOperand },

    /// Find a binding on the environment chain and push its value. If the binding does not exist push undefined.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetNameOrUndefined { dst: VaryingOperand, binding_index: VaryingOperand },

    /// Find a binding on the environment chain and assign its value.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: src
    SetName { src: VaryingOperand, binding_index: VaryingOperand },

    /// Assigns a value to the binding pointed by the top of the `bindings_stack`.
    ///
    /// - Registers:
    ///   - Input: src
    SetNameByLocator { src: VaryingOperand },

    /// Deletes a property of the global object.
    ///
    /// - Operands:
    ///   - binding_index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    DeleteName { dst: VaryingOperand, binding_index: VaryingOperand },

    /// Get a property by name from an object an push it on the stack.
    ///
    /// Like `object.name`
    ///
    /// - Operands:
    ///   - ic_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: receiver, value
    ///   - Output: dst
    GetPropertyByName {
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
    GetPropertyByValue {
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
    GetPropertyByValuePush {
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
    SetPropertyByName {
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
    SetFunctionName { function: VaryingOperand, name: VaryingOperand, prefix: VaryingOperand },

    /// Defines a own property of an object by name.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineOwnPropertyByName { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Defines a static class method by name.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineClassStaticMethodByName {
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
    DefineClassMethodByName {
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
    SetPropertyByValue {
        value: VaryingOperand,
        key: VaryingOperand,
        receiver: VaryingOperand,
        object: VaryingOperand
    },

    /// Defines a own property of an object by value.
    ///
    /// - Registers:
    ///   - Input: object, key, value
    DefineOwnPropertyByValue {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Defines a static class method by value.
    ///
    /// - Registers:
    ///   - Input: object, key, value
    DefineClassStaticMethodByValue {
        value: VaryingOperand,
        key: VaryingOperand,
        object: VaryingOperand
    },

    /// Defines a class method by value.
    ///
    /// - Registers:
    ///   - Input: object, key, value
    DefineClassMethodByValue {
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
    SetPropertyGetterByName { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Defines a static getter class method by name.
    ///
    /// Like `static get name() value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineClassStaticGetterByName {
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
    DefineClassGetterByName {
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
    SetPropertyGetterByValue {
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
    DefineClassStaticGetterByValue {
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
    DefineClassGetterByValue {
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
    SetPropertySetterByName { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Defines a static setter class method by name.
    ///
    /// Like `static set name() value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefineClassStaticSetterByName {
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
    DefineClassSetterByName {
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
    SetPropertySetterByValue {
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
    DefineClassStaticSetterByValue {
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
    DefineClassSetterByValue {
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
    SetPrivateField { value: VaryingOperand, object: VaryingOperand, name_index: VaryingOperand },

    /// Define a private property of a class constructor by it's name.
    ///
    /// Like `#name = value`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    DefinePrivateField { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Set a private method of a class constructor by it's name.
    ///
    /// Like `#name() {}`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    SetPrivateMethod { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Set a private setter property of a class constructor by it's name.
    ///
    /// Like `set #name() {}`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    SetPrivateSetter { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Set a private getter property of a class constructor by it's name.
    ///
    /// Like `get #name() {}`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    SetPrivateGetter { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Get a private property by name from an object an push it on the stack.
    ///
    /// Like `object.#name`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object
    ///   - Output: dst
    GetPrivateField { dst: VaryingOperand, object: VaryingOperand, name_index: VaryingOperand },

    /// Push a field to a class.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    ///   - is_anonymous_function: `bool`
    /// - Registers:
    ///   - Input: object, value
    PushClassField { object: VaryingOperand, name_index: VaryingOperand, value: VaryingOperand, is_anonymous_function: VaryingOperand },

    /// Push a private field to the class.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    PushClassFieldPrivate { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Push a private getter to the class.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    PushClassPrivateGetter { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Push a private setter to the class.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, value
    PushClassPrivateSetter { object: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Push a private method to the class.
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object, proto, value
    PushClassPrivateMethod { object: VaryingOperand, proto: VaryingOperand, value: VaryingOperand, name_index: VaryingOperand },

    /// Deletes a property by name of an object.
    ///
    /// Like `delete object.key`
    ///
    /// - Operands:
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: object
    DeletePropertyByName { object: VaryingOperand, name_index: VaryingOperand },

    /// Deletes a property by value of an object.
    ///
    /// Like `delete object[key]`
    ///
    /// - Registers:
    ///   - Input: object, key
    DeletePropertyByValue { object: VaryingOperand, key: VaryingOperand },

    /// Throws an error when trying to delete a property of `super`
    DeleteSuperThrow,

    /// Copy all properties of one object to another object.
    ///
    /// - Registers:
    ///   - Input: object, source, excluded_keys
    CopyDataProperties { object: VaryingOperand, source: VaryingOperand, excluded_keys: ThinVec<VaryingOperand> },

    /// Call ToPropertyKey on the value on the stack.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    ToPropertyKey { src: VaryingOperand, dst: VaryingOperand },

    /// Unconditional jump to address.
    ///
    /// - Operands:
    ///   - address: `u32`
    Jump { address: u32 },

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
    JumpIfTrue { address: u32, value: VaryingOperand },

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
    JumpIfFalse { address: u32, value: VaryingOperand },

    /// Conditional jump to address.
    ///
    /// If the value popped is not undefined jump to `address`.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Output: value
    JumpIfNotUndefined { address: u32, value: VaryingOperand },

    /// Conditional jump to address.
    ///
    /// If the value popped is undefined jump to `address`.
    ///
    /// - Operands:
    ///   - address: `u32`
    /// - Registers:
    ///   - Output: value
    JumpIfNullOrUndefined { address: u32, value: VaryingOperand },

    /// Jump table that jumps depending on top value of the stack.
    ///
    /// This is used to handle special cases when we call `continue`, `break` or `return` in a try block,
    /// that has finally block.
    ///
    /// Operands: default: `u32`, count: `u32`, address: `u32` * count
    ///
    /// Stack: value: [`i32`] **=>**
    JumpTable { default: u32, addresses: ThinVec<u32> },

    /// Throw exception.
    ///
    /// This sets pending exception and searches for an exception handler.
    ///
    /// - Registers:
    ///   - Input: src
    Throw { src: VaryingOperand },

    /// Rethrow thrown exception.
    ///
    /// This is also used to handle generator `return()` call, we throw an empty exception, by setting pending exception to [`None`],
    /// propagating it and calling finally code until there is no exception handler left, in that case we consume the empty exception and return
    /// from the generator.
    ReThrow,

    /// Get the thrown pending exception (if it's set) and push on the stack.
    ///
    /// If there is no pending exception, which can happend if we are handling `return()` call on generator,
    /// then we rethrow the empty exception. See [`Opcode::ReThrow`].
    ///
    /// - Registers:
    ///   - Output: dst
    Exception { dst: VaryingOperand },

    /// Get the thrown pending exception if it's set and push `true`, otherwise push only `false`.
    ///
    /// - Registers:
    ///   - Output: exception, has_exception
    MaybeException { has_exception: VaryingOperand, exception: VaryingOperand },

    /// Throw a new `TypeError` exception
    ///
    /// - Operands:
    ///   - message: `VaryingOperand`
    ThrowNewTypeError { message: VaryingOperand },

    /// Throw a new `SyntaxError` exception
    ///
    /// - Operands:
    ///   - message: `VaryingOperand`
    ThrowNewSyntaxError { message: VaryingOperand },

    /// Pushes `this` value
    ///
    /// - Registers:
    ///   - Output: dst
    This { dst: VaryingOperand },

    /// Pushes `this` value that is related to the object environment of the given binding
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    ThisForObjectEnvironmentName { dst: VaryingOperand, index: VaryingOperand },

    /// Pushes the current `super` value to the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    Super { dst: VaryingOperand },

    /// Get the super constructor and the new target of the current environment.
    ///
    /// - Registers:
    ///   - Output: dst
    SuperCallPrepare { dst: VaryingOperand },

    /// Execute the `super()` method.
    ///
    /// - Operands:
    ///   - argument_count: `VaryingOperand`
    /// - Stack: super_constructor, argument_1, ... argument_n **=>**
    SuperCall { argument_count: VaryingOperand },

    /// Execute the `super()` method where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: super_constructor, arguments_array **=>**
    SuperCallSpread,

    /// Execute the `super()` method when no constructor of the class is defined.
    ///
    /// Operands:
    ///
    /// Stack: argument_n, ... argument_1 **=>**
    SuperCallDerived,

    /// Binds `this` value and initializes the instance elements.
    ///
    /// Performs steps 7-12 of [`SuperCall: super Arguments`][spec]
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-super-keyword-runtime-semantics-evaluation
    BindThisValue { value: VaryingOperand },

    /// Dynamically import a module.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    ImportCall { value: VaryingOperand },

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
    Case { address: u32, value: VaryingOperand, condition: VaryingOperand },

    /// Get function from the pre-compiled inner functions.
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    GetFunction { dst: VaryingOperand, index: VaryingOperand },

    /// Call a function named "eval".
    ///
    /// - Operands:
    ///   - argument_count: `VaryingOperand`
    ///   - scope_index: `VaryingOperand`
    /// - Stack: this, func, argument_1, ... argument_n **=>** result
    CallEval { argument_count: VaryingOperand, scope_index: VaryingOperand },

    /// Call a function named "eval" where the arguments contain spreads.
    ///
    /// - Operands:
    ///   - scope_index: `VaryingOperand`
    /// - Stack: Stack: this, func, arguments_array **=>** result
    CallEvalSpread { scope_index: VaryingOperand },

    /// Call a function.
    ///
    /// - Operands:
    ///   - argument_count: `VaryingOperand`
    /// - Stack: this, func, argument_1, ... argument_n **=>** result
    Call { argument_count: VaryingOperand },

    /// Call a function where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: this, func, arguments_array **=>** result
    CallSpread,

    /// Call construct on a function.
    ///
    /// - Operands:
    ///   - argument_count: `VaryingOperand`
    /// - Stack: this, func, argument_1, ... argument_n **=>** result
    New { argument_count: VaryingOperand },

    /// Call construct on a function where the arguments contain spreads.
    ///
    /// Operands:
    ///
    /// Stack: arguments_array, func **=>** result
    NewSpread,

    /// Check return from a function.
    CheckReturn,

    /// Return from a function.
    Return,

    /// Close an async generator function.
    AsyncGeneratorClose,

    /// Creates the generator object and yields.
    ///
    /// - Operands:
    ///   - async: `bool`
    /// - Stack: **=>** resume_kind
    Generator { r#async: VaryingOperand },

    /// Set return value of a function.
    ///
    /// - Registers:
    ///   - Input: src
    SetAccumulator { src: VaryingOperand },

    // Set return value of a function.
    ///
    /// - Registers:
    ///   - Output: dst
    SetRegisterFromAccumulator { dst: VaryingOperand },

    /// Move value of operand `src` to register `dst`.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    Move { dst: VaryingOperand, src: VaryingOperand },

    /// Pop value from the stack and push to register `dst`
    ///
    /// - Registers:
    ///   - Output: dst
    PopIntoRegister { dst: VaryingOperand },

    /// Copy value at register `src` and push it on the stack.
    ///
    /// - Registers:
    ///   - Input: src
    PushFromRegister { src: VaryingOperand },

    /// Pop value from the stack and push to a local binding register `dst`.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    PopIntoLocal { src: VaryingOperand, dst: VaryingOperand },

    /// Copy value at local binding register `src` and push it into the stack.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: dst
    PushFromLocal { src: VaryingOperand, dst: VaryingOperand },

    /// Push a declarative environment.
    ///
    /// - Operands:
    ///   - scope_index: `VaryingOperand`
    PushScope { scope_index: VaryingOperand },

    /// Push an object environment.
    ///
    /// - Registers:
    ///   - Input: src
    PushObjectEnvironment { src: VaryingOperand },

    /// Pop the current environment.
    PopEnvironment,

    /// Increment loop iteration count.
    ///
    /// Used for limiting the loop iteration.
    IncrementLoopIteration,

    /// Creates the ForInIterator of an object.
    ///
    /// - Registers:
    ///   - Input: src
    CreateForInIterator { src: VaryingOperand },

    /// Gets the iterator of an object.
    ///
    /// - Registers:
    ///   - Input: src
    /// - Iterator Stack: **=>** `iterator`
    GetIterator { src: VaryingOperand },

    /// Gets the async iterator of an object.
    ///
    /// - Registers:
    ///   - Input: src
    /// - Iterator Stack: **=>** `iterator`
    GetAsyncIterator { src: VaryingOperand },

    /// Calls the `next` method of `iterator`, updating its record with the next value.
    ///
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorNext,

    /// Returns `true` if the current iterator is done, or `false` otherwise
    ///
    /// - Registers:
    ///   - Output: dst
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorDone { dst: VaryingOperand },

    /// Finishes the call to `Opcode::IteratorNext` within a `for await` loop by setting the current
    /// result of the current iterator.
    ///
    /// - Registers:
    ///   - Input: resume_kind, value
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorFinishAsyncNext { resume_kind: VaryingOperand, value: VaryingOperand },

    /// Gets the `value` property of the current iterator record.
    ///
    /// - Registers:
    ///   - Output: dst
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorValue { dst: VaryingOperand },

    /// Gets the last iteration result of the current iterator record.
    ///
    /// - Registers:
    ///   - Output: dst
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorResult { dst: VaryingOperand },

    /// Consume the iterator and construct and array with all the values.
    ///
    /// - Registers:
    ///   - Output: dst
    /// - Iterator Stack: `iterator` **=>** `iterator`
    IteratorToArray { dst: VaryingOperand },

    /// Pushes `true` to the stack if the iterator stack is empty.
    ///
    /// - Registers:
    ///   - Output: dst
    /// - Iterator Stack: **=>**
    IteratorStackEmpty { dst: VaryingOperand },

    /// Creates a new iterator result object.
    ///
    /// - Operands:
    ///   - done: `bool` (codified as u8 with `0` -> `false` and `!0` -> `true`)
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    CreateIteratorResult { value: VaryingOperand, done: VaryingOperand },

    /// Calls `return` on the current iterator and returns the result.
    ///
    /// - Registers:
    ///   - Output: value, called
    /// - Iterator Stack: `iterator` **=>**
    IteratorReturn { value: VaryingOperand, called: VaryingOperand },

    /// Concat multiple stack objects into a string.
    ///
    /// - Registers:
    ///   - Input: values
    ///   - Output: dst
    ConcatToString { dst: VaryingOperand, values: ThinVec<VaryingOperand> },

    /// Require the stack value to be neither null nor undefined.
    ///
    /// - Registers:
    ///   - Input: src
    ValueNotNullOrUndefined { src: VaryingOperand },

    /// Initialize the rest parameter value of a function from the remaining arguments.
    ///
    /// - Stack: `argument_1` .. `argument_n` **=>**
    /// - Registers:
    ///   - Output: dst
    RestParameterInit { dst: VaryingOperand },

    /// Yields from the current generator execution.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: resume_kind, received
    GeneratorYield { src: VaryingOperand },

    /// Resumes the current generator function.
    ///
    /// If the `resume_kind` is `Throw`, then the value is poped and thrown, otherwise if `Return`
    /// we pop the value, set it as the return value and throw and empty exception. See [`Opcode::ReThrow`].
    ///
    /// - Registers:
    ///   - Input: resume_kind, value
    GeneratorNext { resume_kind: VaryingOperand, value: VaryingOperand },

    /// Yields from the current async generator execution.
    ///
    /// - Registers:
    ///   - Input: src
    ///   - Output: resume_kind, received
    AsyncGeneratorYield { src: VaryingOperand },

    /// Create a promise capacity for an async function, if not already set.
    CreatePromiseCapability,

    /// Resolves or rejects the promise capability of an async function.
    ///
    /// If the pending exception is set, reject and rethrow the exception, otherwise resolve.
    CompletePromiseCapability,

    /// Jumps to the specified address if the resume kind is not equal.
    ///
    /// - Operands:
    ///   - address: `u32`
    ///   - resume_kind: `GeneratorResumeKind`
    /// - Registers:
    ///   - Input: src
    JumpIfNotResumeKind { address: u32, resume_kind: VaryingOperand, src: VaryingOperand },

    /// Delegates the current async generator function to another iterator.
    ///
    /// - Operands:
    ///   - throw_method_undefined: `u32`,
    ///   - return_method_undefined: `u32`
    /// - Registers:
    ///   - Input: value, resume_kind
    ///   - Output: value, is_return
    GeneratorDelegateNext {
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
    GeneratorDelegateResume {
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
    Await { src: VaryingOperand },

    /// Push the current new target to the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    NewTarget { dst: VaryingOperand },

    /// Push the current `import.meta` to the stack.
    ///
    /// - Registers:
    ///   - Output: dst
    ImportMeta { dst: VaryingOperand },

    /// Pushes `true` to the stack if the top stack value is an object, or `false` otherwise.
    ///
    /// - Registers:
    ///   - Input: value
    ///   - Output: value
    IsObject { value: VaryingOperand },

    /// Lookup if a tagged template object is cached and skip the creation if it is.
    ///
    /// - Operands:
    ///   - address: `u32`
    ///   - site: `u64`
    /// - Registers:
    ///   - Output: dst
    TemplateLookup { address: u32, site: u64, dst: VaryingOperand },

    /// Create a new tagged template object and cache it.
    ///
    /// - Operands:
    ///   - site: `u64`
    /// - Registers:
    ///   - Inputs: values
    ///   - Output: dst
    TemplateCreate { site: u64, dst: VaryingOperand, values: ThinVec<u32> },

    /// Push a private environment.
    ///
    /// Operands: count: `u32`, count * name_index: `u32`
    ///
    /// - Registers:
    ///   - Input: class, name_indices
    PushPrivateEnvironment { class: VaryingOperand, name_indices: ThinVec<u32> },

    /// Pop a private environment.
    PopPrivateEnvironment,

    /// Creates a mapped `arguments` object.
    ///
    /// Performs [`10.4.4.7 CreateMappedArgumentsObject ( func, formals, argumentsList, env )`]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createunmappedargumentsobject
    ///
    /// - Registers:
    ///   - Output: dst
    CreateMappedArgumentsObject { dst: VaryingOperand },

    /// Creates an unmapped `arguments` object.
    ///
    /// Performs: [`10.4.4.6 CreateUnmappedArgumentsObject ( argumentsList )`]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createmappedargumentsobject
    ///
    /// - Registers:
    ///   - Output: dst
    CreateUnmappedArgumentsObject { dst: VaryingOperand },

    /// Performs [`HasRestrictedGlobalProperty ( N )`][spec]
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasrestrictedglobalproperty
    HasRestrictedGlobalProperty { dst: VaryingOperand, index: VaryingOperand },

    /// Performs [`CanDeclareGlobalFunction ( N )`][spec]
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalfunction
    CanDeclareGlobalFunction { dst: VaryingOperand, index: VaryingOperand },

    /// Performs [`CanDeclareGlobalVar ( N )`][spec]
    ///
    /// - Operands:
    ///   - index: `VaryingOperand`
    /// - Registers:
    ///   - Output: dst
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalvar
    CanDeclareGlobalVar { dst: VaryingOperand, index: VaryingOperand },

    /// Performs [`CreateGlobalFunctionBinding ( N, V, D )`][spec]
    ///
    /// - Operands:
    ///   - configurable: `bool`
    ///   - name_index: `VaryingOperand`
    /// - Registers:
    ///   - Input: src
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createglobalfunctionbinding
    CreateGlobalFunctionBinding { src: VaryingOperand, configurable: VaryingOperand, name_index: VaryingOperand },

    /// Performs [`CreateGlobalVarBinding ( N, V, D )`][spec]
    ///
    /// - Operands:
    ///   - configurable: `bool`
    ///   - name_index: `VaryingOperand`
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createglobalvarbinding
    CreateGlobalVarBinding { configurable: VaryingOperand, name_index: VaryingOperand },

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
