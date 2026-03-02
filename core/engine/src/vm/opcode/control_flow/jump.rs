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
    pub(crate) fn operation(address: u32, context: &Context) {
        context.set_pc(address);
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
    pub(crate) fn operation((address, value): (u32, VaryingOperand), context: &Context) {
        if unsafe {
            (*context.vm_const_ptr())
                .get_register(value.into())
                .to_boolean()
        } {
            context.set_pc(address);
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
    pub(crate) fn operation((address, value): (u32, VaryingOperand), context: &Context) {
        if !unsafe {
            (*context.vm_const_ptr())
                .get_register(value.into())
                .to_boolean()
        } {
            context.set_pc(address);
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
    pub(crate) fn operation((address, value): (u32, VaryingOperand), context: &Context) {
        if !unsafe {
            (*context.vm_const_ptr())
                .get_register(value.into())
                .is_undefined()
        } {
            context.set_pc(address);
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
    pub(crate) fn operation((address, value): (u32, VaryingOperand), context: &Context) {
        if unsafe {
            (*context.vm_const_ptr())
                .get_register(value.into())
                .is_null_or_undefined()
        } {
            context.set_pc(address);
        }
    }
}

impl Operation for JumpIfNullOrUndefined {
    const NAME: &'static str = "JumpIfNullOrUndefined";
    const INSTRUCTION: &'static str = "INST - JumpIfNullOrUndefined";
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
    pub(crate) fn operation(
        (index, default, addresses): (u32, u32, ThinVec<u32>),
        context: &Context,
    ) {
        let target = unsafe {
            let vm = &*context.vm_const_ptr();
            let value = vm.get_register(index as usize);
            if let Some(value) = value.as_i32() {
                let value = value as usize;
                let mut target = None;
                for (i, address) in addresses.iter().enumerate() {
                    if i + 1 == value {
                        target = Some(*address);
                    }
                }
                target.unwrap_or(default)
            } else {
                unreachable!("expected positive integer, got {value:?}")
            }
        };
        context.set_pc(target);
    }
}

impl Operation for JumpTable {
    const NAME: &'static str = "JumpTable";
    const INSTRUCTION: &'static str = "INST - JumpTable";
    const COST: u8 = 5;
}
