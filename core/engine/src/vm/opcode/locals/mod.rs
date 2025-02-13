use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsNativeError, JsResult,
};

/// `PopIntoLocal` implements the Opcode Operation for `Opcode::PopIntoLocal`
///
/// Operation:
///  - Pop value from the stack and push to a local binding register `dst`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopIntoLocal;

impl PopIntoLocal {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    fn operation(
        src: u32,
        dst: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        context.vm.frame_mut().local_bindings_initialized[dst as usize] = true;
        registers.set(dst, registers.get(src).clone());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PopIntoLocal {
    const NAME: &'static str = "PopIntoLocal";
    const INSTRUCTION: &'static str = "INST - PopIntoLocal";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let src = context.vm.read::<u8>().into();
        let dst = context.vm.read::<u8>().into();
        Self::operation(src, dst, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let src = context.vm.read::<u16>().into();
        let dst = context.vm.read::<u16>().into();
        Self::operation(src, dst, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let src = context.vm.read::<u32>();
        let dst = context.vm.read::<u32>();
        Self::operation(src, dst, registers, context)
    }
}

/// `PushFromLocal` implements the Opcode Operation for `Opcode::PushFromLocal`
///
/// Operation:
///  - Copy value at local binding register `src` and push it into the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushFromLocal;

impl PushFromLocal {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    fn operation(
        src: u32,
        dst: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        if !context.vm.frame().local_bindings_initialized[src as usize] {
            return Err(JsNativeError::reference()
                .with_message("access to uninitialized binding")
                .into());
        }
        registers.set(dst, registers.get(src).clone());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushFromLocal {
    const NAME: &'static str = "PushFromLocal";
    const INSTRUCTION: &'static str = "INST - PushFromLocal";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let src = context.vm.read::<u8>().into();
        let dst = context.vm.read::<u8>().into();
        Self::operation(src, dst, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let src = context.vm.read::<u16>().into();
        let dst = context.vm.read::<u16>().into();
        Self::operation(src, dst, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let src = context.vm.read::<u32>();
        let dst = context.vm.read::<u32>();
        Self::operation(src, dst, registers, context)
    }
}
