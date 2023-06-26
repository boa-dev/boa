use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsError, JsNativeError, JsResult,
};

/// `Throw` implements the Opcode Operation for `Opcode::Throw`
///
/// Operation:
///  - Throw exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Throw;

impl Operation for Throw {
    const NAME: &'static str = "Throw";
    const INSTRUCTION: &'static str = "INST - Throw";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let error = JsError::from_opaque(context.vm.pop());
        context.vm.pending_exception = Some(error);

        // Note: -1 because we increment after fetching the opcode.
        let pc = context.vm.frame().pc - 1;
        if let Some(handler) = context.vm.frame().code_block().find_handler(pc).copied() {
            let env_fp = context.vm.frame().env_fp;

            let catch_address = handler.handler();
            let env_fp = (env_fp + handler.env_fp) as usize;
            // TODO: fp
            // let fp = try_entry.fp() as usize;

            context.vm.frame_mut().pc = catch_address;
            context.vm.environments.truncate(env_fp);
            // context.vm.stack.truncate(fp);
            return Ok(CompletionType::Normal);
        }

        Ok(CompletionType::Throw)
    }
}

/// `ReThrow` implements the Opcode Operation for `Opcode::ReThrow`
///
/// Operation:
///  - Rethrow the pending exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ReThrow;

impl Operation for ReThrow {
    const NAME: &'static str = "ReThrow";
    const INSTRUCTION: &'static str = "INST - ReThrow";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        // Note: -1 because we increment after fetching the opcode.
        let pc = context.vm.frame().pc.saturating_sub(1);
        if let Some(handler) = context.vm.frame().code_block().find_handler(pc).copied() {
            let env_fp = context.vm.frame().env_fp;

            let catch_address = handler.handler();
            let env_fp = (env_fp + handler.env_fp) as usize;
            // TODO: fp
            // let fp = try_entry.fp() as usize;

            context.vm.frame_mut().pc = catch_address;
            context.vm.environments.truncate(env_fp);
            // context.vm.stack.truncate(fp);
            return Ok(CompletionType::Normal);
        }

        // Note: If we are rethowing and there is no pending error,
        //       this means that return was called on the generator.
        //
        // Note: If we reached this stage then we there is no handler to handle this,
        //       so return (only for generators).
        if context.vm.pending_exception.is_none() {
            return Ok(CompletionType::Return);
        }

        Ok(CompletionType::Throw)
    }
}

/// `Exception` implements the Opcode Operation for `Opcode::Exception`
///
/// Operation:
///  - Get the thrown exception and push on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Exception;

impl Operation for Exception {
    const NAME: &'static str = "Exception";
    const INSTRUCTION: &'static str = "INST - Exception";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        if let Some(error) = context.vm.pending_exception.take() {
            let error = error.to_opaque(context);
            context.vm.push(error);
            return Ok(CompletionType::Normal);
        }

        // If there is no pending error, this means that `return()` was called
        // on the generator, so we rethrow this (empty) error until there is no handler to handle it.
        // This is done to run the finally code.
        //
        // This should be unreachable for regular functions.
        ReThrow::execute(context)
    }
}

/// `ThrowNewTypeError` implements the Opcode Operation for `Opcode::ThrowNewTypeError`
///
/// Operation:
///  - Throws a `TypeError` exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowNewTypeError;

impl Operation for ThrowNewTypeError {
    const NAME: &'static str = "ThrowNewTypeError";
    const INSTRUCTION: &'static str = "INST - ThrowNewTypeError";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let msg = context.vm.frame().code_block.literals[index as usize]
            .as_string()
            .expect("throw message must be a string")
            .clone();
        let msg = msg
            .to_std_string()
            .expect("throw message must be an ASCII string");
        Err(JsNativeError::typ().with_message(msg).into())
    }
}
