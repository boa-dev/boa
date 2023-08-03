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
        if context.vm.handle_exception_at(pc) {
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
        if context.vm.handle_exception_at(pc) {
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

/// `MaybeException` implements the Opcode Operation for `Opcode::MaybeException`
///
/// Operation:
///  - Get the thrown pending exception if it's set and push `true`, otherwise push only `false`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MaybeException;

impl Operation for MaybeException {
    const NAME: &'static str = "MaybeException";
    const INSTRUCTION: &'static str = "INST - MaybeException";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        if let Some(error) = context.vm.pending_exception.take() {
            let error = error.to_opaque(context);
            context.vm.push(error);
            context.vm.push(true);
            return Ok(CompletionType::Normal);
        }

        context.vm.push(false);
        Ok(CompletionType::Normal)
    }
}

/// `ThrowNewTypeError` implements the Opcode Operation for `Opcode::ThrowNewTypeError`
///
/// Operation:
///  - Throws a `TypeError` exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowNewTypeError;

impl ThrowNewTypeError {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let msg = context.vm.frame().code_block.literals[index]
            .as_string()
            .expect("throw message must be a string")
            .clone();
        let msg = msg
            .to_std_string()
            .expect("throw message must be an ASCII string");
        Err(JsNativeError::typ().with_message(msg).into())
    }
}

impl Operation for ThrowNewTypeError {
    const NAME: &'static str = "ThrowNewTypeError";
    const INSTRUCTION: &'static str = "INST - ThrowNewTypeError";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn half_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn wide_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}
