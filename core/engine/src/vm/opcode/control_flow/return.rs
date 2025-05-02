use std::ops::ControlFlow;

use crate::{
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionRecord, Registers,
    },
    Context, JsNativeError,
};

/// `Return` implements the Opcode Operation for `Opcode::Return`
///
/// Operation:
///  - Return from a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Return;

impl Return {
    #[inline(always)]
    pub(crate) fn operation(
        (): (),
        registers: &mut Registers,
        context: &mut Context,
    ) -> ControlFlow<CompletionRecord> {
        context.handle_return(registers)
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
    pub(crate) fn operation(
        (): (),
        registers: &mut Registers,
        context: &mut Context,
    ) -> ControlFlow<CompletionRecord> {
        let frame = context.vm.frame();
        if !frame.construct() {
            return ControlFlow::Continue(());
        }
        let this = frame.this(&context.vm);
        let result = context.vm.take_return_value();

        let result = if result.is_object() {
            result
        } else if !this.is_undefined() {
            this
        } else if !result.is_undefined() {
            let realm = context.vm.frame().realm.clone();
            context.vm.pending_exception = Some(
                JsNativeError::typ()
                    .with_message("derived constructor can only return an Object or undefined")
                    .with_realm(realm)
                    .into(),
            );
            return context.handle_thow(registers);
        } else {
            let frame = context.vm.frame();
            if frame.has_this_value_cached() {
                this
            } else {
                let realm = frame.realm.clone();

                match context.vm.environments.get_this_binding() {
                    Err(err) => {
                        let err = err.inject_realm(realm);
                        context.vm.pending_exception = Some(err);
                        return context.handle_thow(registers);
                    }
                    Ok(Some(this)) => this,
                    Ok(None) => context.realm().global_this().clone().into(),
                }
            }
        };

        context.vm.set_return_value(result);
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
    pub(crate) fn operation(
        register: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let value = registers.get(register.into());
        context.vm.set_return_value(value.clone());
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
    pub(crate) fn operation(
        (dst, src): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        _: &mut Context,
    ) {
        let value = registers.get(src.into());
        registers.set(dst.into(), value.clone());
    }
}

impl Operation for Move {
    const NAME: &'static str = "Move";
    const INSTRUCTION: &'static str = "INST - Move";
    const COST: u8 = 2;
}

/// TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopIntoRegister;

impl PopIntoRegister {
    #[inline(always)]
    pub(crate) fn operation(dst: VaryingOperand, registers: &mut Registers, context: &mut Context) {
        registers.set(dst.into(), context.vm.pop());
    }
}

impl Operation for PopIntoRegister {
    const NAME: &'static str = "PopIntoRegister";
    const INSTRUCTION: &'static str = "INST - PopIntoRegister";
    const COST: u8 = 2;
}

/// TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushFromRegister;

impl PushFromRegister {
    #[inline(always)]
    pub(crate) fn operation(dst: VaryingOperand, registers: &mut Registers, context: &mut Context) {
        let value = registers.get(dst.into());
        context.vm.push(value.clone());
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
    pub(crate) fn operation(
        register: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) {
        registers.set(register.into(), context.vm.get_return_value());
    }
}

impl Operation for SetRegisterFromAccumulator {
    const NAME: &'static str = "SetRegisterFromAccumulator";
    const INSTRUCTION: &'static str = "INST - SetRegisterFromAccumulator";
    const COST: u8 = 2;
}
