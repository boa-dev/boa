use crate::{
    vm::{opcode::Operation, CompletionType},
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
    fn operation(dst: u32, context: &mut Context) -> JsResult<CompletionType> {
        context.vm.frame_mut().local_bindings_initialized[dst as usize] = true;
        let value = context.vm.pop();

        let rp = context.vm.frame().rp;
        context.vm.stack[(rp + dst) as usize] = value;
        Ok(CompletionType::Normal)
    }
}

impl Operation for PopIntoLocal {
    const NAME: &'static str = "PopIntoLocal";
    const INSTRUCTION: &'static str = "INST - PopIntoLocal";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u8>());
        Self::operation(dst, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u16>());
        Self::operation(dst, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        Self::operation(dst, context)
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
    fn operation(dst: u32, context: &mut Context) -> JsResult<CompletionType> {
        if !context.vm.frame().local_bindings_initialized[dst as usize] {
            return Err(JsNativeError::reference()
                .with_message("access to uninitialized binding")
                .into());
        }
        let rp = context.vm.frame().rp;
        let value = context.vm.stack[(rp + dst) as usize].clone();
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushFromLocal {
    const NAME: &'static str = "PushFromLocal";
    const INSTRUCTION: &'static str = "INST - PushFromLocal";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u8>());
        Self::operation(dst, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u16>());
        Self::operation(dst, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        Self::operation(dst, context)
    }
}
