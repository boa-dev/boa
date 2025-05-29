use std::ops::ControlFlow;

use crate::{
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionRecord,
    },
    Context, JsError, JsNativeError,
};

/// `Throw` implements the Opcode Operation for `Opcode::Throw`
///
/// Operation:
///  - Throw exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Throw;

impl Throw {
    #[inline(always)]
    pub(crate) fn operation(
        value: VaryingOperand,
        context: &mut Context,
    ) -> ControlFlow<CompletionRecord> {
        let value = context.vm.get_register(value.into());
        let error = JsError::from_opaque(value.clone());
        context.vm.pending_exception = Some(error);

        // Note: -1 because we increment after fetching the opcode.
        let pc = context.vm.frame().pc - 1;
        if context.vm.handle_exception_at(pc) {
            return ControlFlow::Continue(());
        }

        context.handle_thow()
    }
}

impl Operation for Throw {
    const NAME: &'static str = "Throw";
    const INSTRUCTION: &'static str = "INST - Throw";
    const COST: u8 = 6;
}

/// `ReThrow` implements the Opcode Operation for `Opcode::ReThrow`
///
/// Operation:
///  - Rethrow the pending exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ReThrow;

impl ReThrow {
    #[inline(always)]
    pub(crate) fn operation((): (), context: &mut Context) -> ControlFlow<CompletionRecord> {
        // Note: -1 because we increment after fetching the opcode.
        let pc = context.vm.frame().pc.saturating_sub(1);
        if context.vm.handle_exception_at(pc) {
            return ControlFlow::Continue(());
        }

        // Note: If we are rethowing and there is no pending error,
        //       this means that return was called on the generator.
        //
        // Note: If we reached this stage then we there is no handler to handle this,
        //       so return (only for generators).
        if context.vm.pending_exception.is_none() {
            return context.handle_return();
        }

        context.handle_thow()
    }
}

impl Operation for ReThrow {
    const NAME: &'static str = "ReThrow";
    const INSTRUCTION: &'static str = "INST - ReThrow";
    const COST: u8 = 2;
}

/// `Exception` implements the Opcode Operation for `Opcode::Exception`
///
/// Operation:
///  - Get the thrown exception and push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Exception;

impl Exception {
    #[inline(always)]
    pub(crate) fn operation(
        dst: VaryingOperand,
        context: &mut Context,
    ) -> ControlFlow<CompletionRecord> {
        if let Some(error) = context.vm.pending_exception.take() {
            let error = error.to_opaque(context);
            context.vm.set_register(dst.into(), error);
            return ControlFlow::Continue(());
        }

        // If there is no pending error, this means that `return()` was called
        // on the generator, so we rethrow this (empty) error until there is no handler to handle it.
        // This is done to run the finally code.
        //
        // This should be unreachable for regular functions.
        ReThrow::operation((), context)
    }
}

impl Operation for Exception {
    const NAME: &'static str = "Exception";
    const INSTRUCTION: &'static str = "INST - Exception";
    const COST: u8 = 2;
}

/// `MaybeException` implements the Opcode Operation for `Opcode::MaybeException`
///
/// Operation:
///  - Get the thrown pending exception if it's set and push `true`, otherwise push only `false`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MaybeException;

impl MaybeException {
    #[inline(always)]
    pub(crate) fn operation(
        (has_exception, exception): (VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) {
        if let Some(error) = context.vm.pending_exception.take() {
            let error = error.to_opaque(context);
            context.vm.set_register(exception.into(), error);
            context.vm.set_register(has_exception.into(), true.into());
        } else {
            context.vm.set_register(has_exception.into(), false.into());
        }
    }
}

impl Operation for MaybeException {
    const NAME: &'static str = "MaybeException";
    const INSTRUCTION: &'static str = "INST - MaybeException";
    const COST: u8 = 3;
}

/// `ThrowNewTypeError` implements the Opcode Operation for `Opcode::ThrowNewTypeError`
///
/// Operation:
///  - Throws a `TypeError` exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowNewTypeError;

impl ThrowNewTypeError {
    #[inline(always)]
    pub(crate) fn operation(index: VaryingOperand, context: &mut Context) -> JsError {
        let msg = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into());
        let msg = msg
            .to_std_string()
            .expect("throw message must be an ASCII string");
        JsNativeError::typ().with_message(msg).into()
    }
}

impl Operation for ThrowNewTypeError {
    const NAME: &'static str = "ThrowNewTypeError";
    const INSTRUCTION: &'static str = "INST - ThrowNewTypeError";
    const COST: u8 = 2;
}

/// `ThrowNewSyntaxError` implements the Opcode Operation for `Opcode::ThrowNewSyntaxError`
///
/// Operation:
///  - Throws a `SyntaxError` exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowNewSyntaxError;

impl ThrowNewSyntaxError {
    #[inline(always)]
    pub(crate) fn operation(index: VaryingOperand, context: &mut Context) -> JsError {
        let msg = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into());
        let msg = msg
            .to_std_string()
            .expect("throw message must be an ASCII string");
        JsNativeError::syntax().with_message(msg).into()
    }
}

impl Operation for ThrowNewSyntaxError {
    const NAME: &'static str = "ThrowNewSyntaxError";
    const INSTRUCTION: &'static str = "INST - ThrowNewSyntaxError";
    const COST: u8 = 2;
}
