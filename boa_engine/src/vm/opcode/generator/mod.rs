pub(crate) mod yield_stm;

use crate::{
    error::JsNativeError,
    string::utf16,
    vm::{
        call_frame::GeneratorResumeKind,
        opcode::{control_flow::Return, Operation},
        CompletionType,
    },
    Context, JsError, JsResult,
};

pub(crate) use yield_stm::*;

/// `GeneratorNext` implements the Opcode Operation for `Opcode::GeneratorNext`
///
/// Operation:
///  - Resumes the current generator function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorNext;

impl Operation for GeneratorNext {
    const NAME: &'static str = "GeneratorNext";
    const INSTRUCTION: &'static str = "INST - GeneratorNext";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        match context.vm.frame().generator_resume_kind {
            GeneratorResumeKind::Normal => Ok(CompletionType::Normal),
            GeneratorResumeKind::Throw => Err(JsError::from_opaque(context.vm.pop())),
            GeneratorResumeKind::Return => Return::execute(context),
        }
    }
}

/// `GeneratorJumpOnResumeKind` implements the Opcode Operation for
/// `Opcode::GeneratorJumpOnResumeKind`
///
/// Operation:
///  - Jumps to the specified instruction if the current resume kind is `Return`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorJumpOnResumeKind;

impl Operation for GeneratorJumpOnResumeKind {
    const NAME: &'static str = "GeneratorJumpOnResumeKind";
    const INSTRUCTION: &'static str = "INST - GeneratorJumpOnResumeKind";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let normal = context.vm.read::<u32>();
        let throw = context.vm.read::<u32>();
        let r#return = context.vm.read::<u32>();
        match context.vm.frame().generator_resume_kind {
            GeneratorResumeKind::Normal => context.vm.frame_mut().pc = normal,
            GeneratorResumeKind::Throw => context.vm.frame_mut().pc = throw,
            GeneratorResumeKind::Return => context.vm.frame_mut().pc = r#return,
        }
        Ok(CompletionType::Normal)
    }
}

/// `GeneratorSetReturn` implements the Opcode Operation for `Opcode::GeneratorSetReturn`
///
/// Operation:
///  - Sets the current generator resume kind to `Return`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorSetReturn;

impl Operation for GeneratorSetReturn {
    const NAME: &'static str = "GeneratorSetReturn";
    const INSTRUCTION: &'static str = "INST - GeneratorSetReturn";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        context.vm.frame_mut().generator_resume_kind = GeneratorResumeKind::Return;
        Ok(CompletionType::Normal)
    }
}

/// `GeneratorResumeReturn` implements the Opcode Operation for `Opcode::GeneratorResumeReturn`
///
/// Operation:
///  - Resumes a generator with a return completion.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorResumeReturn;

impl Operation for GeneratorResumeReturn {
    const NAME: &'static str = "GeneratorResumeReturn";
    const INSTRUCTION: &'static str = "INST - GeneratorResumeReturn";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        if context.vm.frame().generator_resume_kind == GeneratorResumeKind::Throw {
            return Err(JsError::from_opaque(context.vm.pop()));
        }
        Return::execute(context)
    }
}

/// `GeneratorDelegateNext` implements the Opcode Operation for `Opcode::GeneratorDelegateNext`
///
/// Operation:
///  - Delegates the current generator function to another iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorDelegateNext;

impl Operation for GeneratorDelegateNext {
    const NAME: &'static str = "GeneratorDelegateNext";
    const INSTRUCTION: &'static str = "INST - GeneratorDelegateNext";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let throw_method_undefined = context.vm.read::<u32>();
        let return_method_undefined = context.vm.read::<u32>();
        let received = context.vm.pop();

        // Preemptively popping removes the iterator from the iterator stack if any operation
        // throws, which avoids calling cleanup operations on the poisoned iterator.
        let iterator_record = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");

        match std::mem::take(&mut context.vm.frame_mut().generator_resume_kind) {
            GeneratorResumeKind::Normal => {
                let result = iterator_record.next_method().call(
                    &iterator_record.iterator().clone().into(),
                    &[received],
                    context,
                )?;
                context.vm.push(false);
                context.vm.push(result);
            }
            GeneratorResumeKind::Throw => {
                let throw = iterator_record
                    .iterator()
                    .get_method(utf16!("throw"), context)?;
                if let Some(throw) = throw {
                    let result = throw.call(
                        &iterator_record.iterator().clone().into(),
                        &[received],
                        context,
                    )?;
                    context.vm.push(false);
                    context.vm.push(result);
                } else {
                    let error = JsNativeError::typ()
                        .with_message("iterator does not have a throw method")
                        .to_opaque(context);
                    context.vm.push(error);
                    context.vm.frame_mut().pc = throw_method_undefined;
                }
            }
            GeneratorResumeKind::Return => {
                let r#return = iterator_record
                    .iterator()
                    .get_method(utf16!("return"), context)?;
                if let Some(r#return) = r#return {
                    let result = r#return.call(
                        &iterator_record.iterator().clone().into(),
                        &[received],
                        context,
                    )?;
                    context.vm.push(true);
                    context.vm.push(result);
                } else {
                    context.vm.push(received);
                    context.vm.frame_mut().pc = return_method_undefined;

                    // The current iterator didn't have a cleanup `return` method, so we can
                    // skip pushing it to the iterator stack for cleanup.
                    return Ok(CompletionType::Normal);
                }
            }
        }

        context.vm.frame_mut().iterators.push(iterator_record);

        Ok(CompletionType::Normal)
    }
}

/// `GeneratorDelegateResume` implements the Opcode Operation for `Opcode::GeneratorDelegateResume`
///
/// Operation:
///  - Resume the generator with yield delegate logic after it awaits a value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorDelegateResume;

impl Operation for GeneratorDelegateResume {
    const NAME: &'static str = "GeneratorDelegateResume";
    const INSTRUCTION: &'static str = "INST - GeneratorDelegateResume";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let return_gen = context.vm.read::<u32>();
        let exit = context.vm.read::<u32>();

        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");

        let result = context.vm.pop();
        let is_return = context.vm.pop().to_boolean();

        if context.vm.frame().generator_resume_kind == GeneratorResumeKind::Throw {
            return Err(JsError::from_opaque(result));
        }

        iterator.update_result(result, context)?;

        if iterator.done() {
            let value = iterator.value(context)?;
            context.vm.push(value);

            context.vm.frame_mut().pc = if is_return { return_gen } else { exit };

            return Ok(CompletionType::Normal);
        }

        context.vm.frame_mut().iterators.push(iterator);

        Ok(CompletionType::Normal)
    }
}
