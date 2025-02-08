use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsNativeError, JsResult,
};

/// `Return` implements the Opcode Operation for `Opcode::Return`
///
/// Operation:
///  - Return from a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Return;

impl Operation for Return {
    const NAME: &'static str = "Return";
    const INSTRUCTION: &'static str = "INST - Return";
    const COST: u8 = 4;

    fn execute(_: &mut Registers, _: &mut Context) -> JsResult<CompletionType> {
        Ok(CompletionType::Return)
    }
}

/// `CheckReturn` implements the Opcode Operation for `Opcode::CheckReturn`
///
/// Operation:
///  - Check return from a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CheckReturn;

impl Operation for CheckReturn {
    const NAME: &'static str = "CheckReturn";
    const INSTRUCTION: &'static str = "INST - CheckReturn";
    const COST: u8 = 3;

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let frame = context.vm.frame();
        if !frame.construct() {
            return Ok(CompletionType::Normal);
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
            return Ok(CompletionType::Throw);
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
                        return Ok(CompletionType::Throw);
                    }
                    Ok(Some(this)) => this,
                    Ok(None) => context.realm().global_this().clone().into(),
                }
            }
        };

        context.vm.set_return_value(result);
        Ok(CompletionType::Normal)
    }
}

/// `SetAccumulator` implements the Opcode Operation for `Opcode::SetAccumulator`
///
/// Operation:
///  - Sets the accumulator value, which is the implicit return value of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetAccumulator;

impl SetAccumulator {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        register: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(register);
        context.vm.set_return_value(value.clone());
        Ok(CompletionType::Normal)
    }
}

impl Operation for SetAccumulator {
    const NAME: &'static str = "SetAccumulator";
    const INSTRUCTION: &'static str = "INST - SetAccumulator";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let register = u32::from(context.vm.read::<u8>());
        Self::operation(register, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let register = u32::from(context.vm.read::<u16>());
        Self::operation(register, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let register = context.vm.read::<u32>();
        Self::operation(register, registers, context)
    }
}

/// `Move` implements the Opcode Operation for `Opcode::Move`
///
/// Operation:
///  - Sets the accumulator value, which is the implicit return value of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Move;

impl Move {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    fn operation(dst: u32, src: u32, registers: &mut Registers) -> JsResult<CompletionType> {
        let value = registers.get(src);
        registers.set(dst, value.clone());
        Ok(CompletionType::Normal)
    }
}

impl Operation for Move {
    const NAME: &'static str = "Move";
    const INSTRUCTION: &'static str = "INST - Move";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u8>().into();
        let src = context.vm.read::<u8>().into();
        Self::operation(dst, src, registers)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u16>().into();
        let src = context.vm.read::<u16>().into();
        Self::operation(dst, src, registers)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        let src = context.vm.read::<u32>();
        Self::operation(dst, src, registers)
    }
}

/// TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopIntoRegister;

impl PopIntoRegister {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    fn operation(
        dst: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        registers.set(dst, context.vm.pop());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PopIntoRegister {
    const NAME: &'static str = "PopIntoRegister";
    const INSTRUCTION: &'static str = "INST - PopIntoRegister";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u8>());
        Self::operation(dst, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u16>());
        Self::operation(dst, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        Self::operation(dst, registers, context)
    }
}

/// TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushFromRegister;

impl PushFromRegister {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    fn operation(
        dst: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(dst);
        context.vm.push(value.clone());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushFromRegister {
    const NAME: &'static str = "PushFromRegister";
    const INSTRUCTION: &'static str = "INST - PushFromRegister";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u8>());
        Self::operation(dst, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u16>());
        Self::operation(dst, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        Self::operation(dst, registers, context)
    }
}

/// `SetRegisterFromAccumulator` implements the Opcode Operation for `Opcode::SetRegisterFromAccumulator`
///
/// Operation:
///  - Sets the accumulator value, which is the implicit return value of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetRegisterFromAccumulator;

impl SetRegisterFromAccumulator {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        register: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        registers.set(register, context.vm.get_return_value());
        Ok(CompletionType::Normal)
    }
}

impl Operation for SetRegisterFromAccumulator {
    const NAME: &'static str = "SetRegisterFromAccumulator";
    const INSTRUCTION: &'static str = "INST - SetRegisterFromAccumulator";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let register = u32::from(context.vm.read::<u8>());
        Self::operation(register, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let register = u32::from(context.vm.read::<u16>());
        Self::operation(register, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let register = context.vm.read::<u32>();
        Self::operation(register, registers, context)
    }
}
