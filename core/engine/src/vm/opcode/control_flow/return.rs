use std::ops::ControlFlow;

use crate::{
    Context, JsNativeError,
    vm::{
        CompletionRecord,
        opcode::{Operation, VaryingOperand},
    },
};

/// `Return` implements the Opcode Operation for `Opcode::Return`
///
/// Operation:
///  - Return from a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Return;

impl Return {
    #[inline(always)]
    pub(crate) fn operation((): (), context: &Context) -> ControlFlow<CompletionRecord> {
        context.handle_return()
    }
}

impl Operation for Return {
    const NAME: &'static str = "Return";
    const INSTRUCTION: &'static str = "INST - Return";
    const COST: u8 = 4;
}

/// `CheckReturn` implements the Opcode Operation for `Opcode::CheckReturn`
///
/// Operation:
///  - Check return from a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CheckReturn;

impl CheckReturn {
    #[inline(always)]
    pub(crate) fn operation((): (), context: &Context) -> ControlFlow<CompletionRecord> {
        if !context.with_vm(|vm| vm.frame().construct()) {
            return ControlFlow::Continue(());
        }

        let (this, result) = context.with_vm_mut(|vm| {
            let this = vm.stack.get_this(&vm.frame);
            let result = vm.take_return_value();
            (this, result)
        });

        let result = if result.is_object() {
            result
        } else if !this.is_undefined() {
            this.clone()
        } else if !result.is_undefined() {
            context.set_pending_exception(
                // Avoid setting the realm here, since it needs to be set by the parent
                // execution context.
                JsNativeError::typ()
                    .with_message("derived constructor can only return an Object or undefined")
                    .into(),
            );
            return context.handle_throw();
        } else if context.with_vm(|vm| vm.frame().has_this_value_cached()) {
            this
        } else {
            match context.with_vm(|vm| vm.frame.environments.get_this_binding()) {
                Err(err) => {
                    // Avoid setting the realm here, since it needs to be set by the parent
                    // execution context.
                    context.set_pending_exception(err);
                    return context.handle_throw();
                }
                Ok(Some(this)) => this,
                Ok(None) => context.realm().global_this().clone().into(),
            }
        };

        context.set_return_value(result);
        ControlFlow::Continue(())
    }
}

impl Operation for CheckReturn {
    const NAME: &'static str = "CheckReturn";
    const INSTRUCTION: &'static str = "INST - CheckReturn";
    const COST: u8 = 3;
}

/// `SetAccumulator` implements the Opcode Operation for `Opcode::SetAccumulator`
///
/// Operation:
///  - Sets the accumulator value, which is the implicit return value of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetAccumulator;

impl SetAccumulator {
    #[inline(always)]
    pub(crate) fn operation(register: VaryingOperand, context: &Context) {
        let value = context.get_register(register.into());
        context.set_return_value(value);
    }
}

impl Operation for SetAccumulator {
    const NAME: &'static str = "SetAccumulator";
    const INSTRUCTION: &'static str = "INST - SetAccumulator";
    const COST: u8 = 2;
}

/// `Move` implements the Opcode Operation for `Opcode::Move`
///
/// Operation:
///  - Sets the accumulator value, which is the implicit return value of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Move;

impl Move {
    #[inline(always)]
    pub(crate) fn operation((dst, src): (VaryingOperand, VaryingOperand), context: &Context) {
        context.with_vm_mut(|vm| {
            let value = vm.get_register(src.into()).clone();
            vm.set_register(dst.into(), value);
        });
    }
}

impl Operation for Move {
    const NAME: &'static str = "Move";
    const INSTRUCTION: &'static str = "INST - Move";
    const COST: u8 = 2;
}

/// `PopIntoRegister` implements the Opcode Operation for `Opcode::PopIntoRegister`.
///
/// Operation:
///  - Pop a value from the stack and store it in a register.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopIntoRegister;

impl PopIntoRegister {
    #[inline(always)]
    pub(crate) fn operation(dst: VaryingOperand, context: &Context) {
        let value = context.stack_pop();
        context.set_register(dst.into(), value);
    }
}

impl Operation for PopIntoRegister {
    const NAME: &'static str = "PopIntoRegister";
    const INSTRUCTION: &'static str = "INST - PopIntoRegister";
    const COST: u8 = 2;
}

/// `PushFromRegister` implements the Opcode Operation for `Opcode::PushFromRegister`.
///
/// Operation:
///  - Read a value from a register and push it onto the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushFromRegister;

impl PushFromRegister {
    #[inline(always)]
    pub(crate) fn operation(dst: VaryingOperand, context: &Context) {
        let value = context.get_register(dst.into());
        context.stack_push(value);
    }
}

impl Operation for PushFromRegister {
    const NAME: &'static str = "PushFromRegister";
    const INSTRUCTION: &'static str = "INST - PushFromRegister";
    const COST: u8 = 2;
}

/// `SetRegisterFromAccumulator` implements the Opcode Operation for `Opcode::SetRegisterFromAccumulator`
///
/// Operation:
///  - Sets the accumulator value, which is the implicit return value of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetRegisterFromAccumulator;

impl SetRegisterFromAccumulator {
    #[inline(always)]
    pub(crate) fn operation(register: VaryingOperand, context: &Context) {
        let return_value = context.get_return_value();
        context.set_register(register.into(), return_value);
    }
}

impl Operation for SetRegisterFromAccumulator {
    const NAME: &'static str = "SetRegisterFromAccumulator";
    const INSTRUCTION: &'static str = "INST - SetRegisterFromAccumulator";
    const COST: u8 = 2;
}
