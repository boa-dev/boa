use crate::{
    Context,
    vm::opcode::{Operation, VaryingOperand},
};
use thin_vec::ThinVec;

/// `Jump` implements the Opcode Operation for `Opcode::Jump`
///
/// Operation:
///  - Unconditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Jump;

impl Jump {
    #[inline(always)]
    pub(crate) fn operation(address: u32, context: &mut Context) {
        context.vm.frame_mut().pc = address;
    }
}

impl Operation for Jump {
    const NAME: &'static str = "Jump";
    const INSTRUCTION: &'static str = "INST - Jump";
    const COST: u8 = 1;
}

// `JumpIfTrue` implements the Opcode Operation for `Opcode::JumpIfTrue`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfTrue;

impl JumpIfTrue {
    #[inline(always)]
    pub(crate) fn operation((address, value): (u32, VaryingOperand), context: &mut Context) {
        let value = context.vm.get_register(value.into());
        if value.to_boolean() {
            context.vm.frame_mut().pc = address;
        }
    }
}

impl Operation for JumpIfTrue {
    const NAME: &'static str = "JumpIfTrue";
    const INSTRUCTION: &'static str = "INST - JumpIfTrue";
    const COST: u8 = 1;
}

/// `JumpIfFalse` implements the Opcode Operation for `Opcode::JumpIfFalse`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfFalse;

impl JumpIfFalse {
    #[inline(always)]
    pub(crate) fn operation((address, value): (u32, VaryingOperand), context: &mut Context) {
        let value = context.vm.get_register(value.into());
        if !value.to_boolean() {
            context.vm.frame_mut().pc = address;
        }
    }
}

impl Operation for JumpIfFalse {
    const NAME: &'static str = "JumpIfFalse";
    const INSTRUCTION: &'static str = "INST - JumpIfFalse";
    const COST: u8 = 1;
}

/// `JumpIfNotUndefined` implements the Opcode Operation for `Opcode::JumpIfNotUndefined`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfNotUndefined;

impl JumpIfNotUndefined {
    #[inline(always)]
    pub(crate) fn operation((address, value): (u32, VaryingOperand), context: &mut Context) {
        let value = context.vm.get_register(value.into());
        if !value.is_undefined() {
            context.vm.frame_mut().pc = address;
        }
    }
}

impl Operation for JumpIfNotUndefined {
    const NAME: &'static str = "JumpIfNotUndefined";
    const INSTRUCTION: &'static str = "INST - JumpIfNotUndefined";
    const COST: u8 = 1;
}

/// `JumpIfNullOrUndefined` implements the Opcode Operation for `Opcode::JumpIfNullOrUndefined`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfNullOrUndefined;

impl JumpIfNullOrUndefined {
    #[inline(always)]
    pub(crate) fn operation((address, value): (u32, VaryingOperand), context: &mut Context) {
        let value = context.vm.get_register(value.into());
        if value.is_null_or_undefined() {
            context.vm.frame_mut().pc = address;
        }
    }
}

impl Operation for JumpIfNullOrUndefined {
    const NAME: &'static str = "JumpIfNullOrUndefined";
    const INSTRUCTION: &'static str = "INST - JumpIfNullOrUndefined";
    const COST: u8 = 1;
}

/// `JumpIfNotEqual` implements the Opcode Operation for `Opcode::JumpIfNotEqual`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfNotEqual;

impl JumpIfNotEqual {
    #[inline(always)]
    pub(crate) fn operation(
        (address, lhs, rhs): (u32, VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) {
        let lhs = context.vm.get_register(lhs.into());
        let rhs = context.vm.get_register(rhs.into());
        if lhs != rhs {
            context.vm.frame_mut().pc = address;
        }
    }
}

impl Operation for JumpIfNotEqual {
    const NAME: &'static str = "JumpIfNotEqual";
    const INSTRUCTION: &'static str = "INST - JumpIfNotEqual";
    const COST: u8 = 1;
}

/// `JumpTable` implements the Opcode Operation for `Opcode::JumpTable`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpTable;

impl JumpTable {
    #[inline(always)]
    pub(crate) fn operation((index, addresses): (u32, ThinVec<u32>), context: &mut Context) {
        let value = context.vm.get_register(index as usize);
        let Some(offset) = value.as_i32().map(|i| i as usize) else {
            return;
        };

        let Some(pc) = addresses.get(offset).copied() else {
            return;
        };

        context.vm.frame_mut().pc = pc;
    }
}

impl Operation for JumpTable {
    const NAME: &'static str = "JumpTable";
    const INSTRUCTION: &'static str = "INST - JumpTable";
    const COST: u8 = 5;
}
