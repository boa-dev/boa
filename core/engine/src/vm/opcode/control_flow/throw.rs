use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsError, JsNativeError, JsResult,
};

/// `Throw` implements the Opcode Operation for `Opcode::Throw`
///
/// Operation:
///  - Throw exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Throw;

impl Throw {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        let error = JsError::from_opaque(value.clone());
        context.vm.pending_exception = Some(error);

        // Note: -1 because we increment after fetching the opcode.
        let pc = context.vm.frame().pc - 1;
        if context.vm.handle_exception_at(pc) {
            return Ok(CompletionType::Normal);
        }

        Ok(CompletionType::Throw)
    }
}

impl Operation for Throw {
    const NAME: &'static str = "Throw";
    const INSTRUCTION: &'static str = "INST - Throw";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        Self::operation(value, registers, context)
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
    const COST: u8 = 2;

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
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
///  - Get the thrown exception and push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Exception;

impl Exception {
    fn operation(
        dst: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        if let Some(error) = context.vm.pending_exception.take() {
            let error = error.to_opaque(context);
            registers.set(dst, error);
            return Ok(CompletionType::Normal);
        }

        // If there is no pending error, this means that `return()` was called
        // on the generator, so we rethrow this (empty) error until there is no handler to handle it.
        // This is done to run the finally code.
        //
        // This should be unreachable for regular functions.
        ReThrow::execute(registers, context)
    }
}

impl Operation for Exception {
    const NAME: &'static str = "Exception";
    const INSTRUCTION: &'static str = "INST - Exception";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u8>().into();
        Self::operation(dst, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u16>().into();
        Self::operation(dst, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        Self::operation(dst, registers, context)
    }
}

/// `MaybeException` implements the Opcode Operation for `Opcode::MaybeException`
///
/// Operation:
///  - Get the thrown pending exception if it's set and push `true`, otherwise push only `false`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MaybeException;

impl MaybeException {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        has_exception: u32,
        exception: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        if let Some(error) = context.vm.pending_exception.take() {
            let error = error.to_opaque(context);
            registers.set(exception, error);
            registers.set(has_exception, true.into());
        } else {
            registers.set(has_exception, false.into());
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for MaybeException {
    const NAME: &'static str = "MaybeException";
    const INSTRUCTION: &'static str = "INST - MaybeException";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let has_exception = context.vm.read::<u8>().into();
        let exception = context.vm.read::<u8>().into();
        Self::operation(has_exception, exception, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let has_exception = context.vm.read::<u16>().into();
        let exception = context.vm.read::<u16>().into();
        Self::operation(has_exception, exception, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let has_exception = context.vm.read::<u32>();
        let exception = context.vm.read::<u32>();
        Self::operation(has_exception, exception, registers, context)
    }
}

/// `ThrowNewTypeError` implements the Opcode Operation for `Opcode::ThrowNewTypeError`
///
/// Operation:
///  - Throws a `TypeError` exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowNewTypeError;

impl ThrowNewTypeError {
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let msg = context.vm.frame().code_block().constant_string(index);
        let msg = msg
            .to_std_string()
            .expect("throw message must be an ASCII string");
        Err(JsNativeError::typ().with_message(msg).into())
    }
}

impl Operation for ThrowNewTypeError {
    const NAME: &'static str = "ThrowNewTypeError";
    const INSTRUCTION: &'static str = "INST - ThrowNewTypeError";
    const COST: u8 = 2;

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_u16(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_u32(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `ThrowNewSyntaxError` implements the Opcode Operation for `Opcode::ThrowNewSyntaxError`
///
/// Operation:
///  - Throws a `SyntaxError` exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowNewSyntaxError;

impl ThrowNewSyntaxError {
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let msg = context.vm.frame().code_block().constant_string(index);
        let msg = msg
            .to_std_string()
            .expect("throw message must be an ASCII string");
        Err(JsNativeError::syntax().with_message(msg).into())
    }
}

impl Operation for ThrowNewSyntaxError {
    const NAME: &'static str = "ThrowNewSyntaxError";
    const INSTRUCTION: &'static str = "INST - ThrowNewSyntaxError";
    const COST: u8 = 2;

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_u16(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_u32(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}
