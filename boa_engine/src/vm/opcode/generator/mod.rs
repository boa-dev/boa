pub(crate) mod yield_stm;

use crate::{
    error::JsNativeError,
    string::utf16,
    vm::{
        call_frame::GeneratorResumeKind,
        opcode::{Operation, ReThrow},
        CompletionType,
    },
    Context, JsError, JsResult,
};

pub(crate) use yield_stm::*;

use super::SetReturnValue;

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
        let generator_resume_kind = context.vm.pop().to_generator_resume_kind();
        match generator_resume_kind {
            GeneratorResumeKind::Normal => Ok(CompletionType::Normal),
            GeneratorResumeKind::Throw => Err(JsError::from_opaque(context.vm.pop())),
            GeneratorResumeKind::Return => {
                assert!(context.vm.pending_exception.is_none());

                SetReturnValue::execute(context)?;
                ReThrow::execute(context)
            }
        }
    }
}

/// `JumpIfNotResumeKind` implements the Opcode Operation for `Opcode::JumpIfNotResumeKind`
///
/// Operation:
///  - Jumps to the specified address if the resume kind is not equal.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfNotResumeKind;

impl Operation for JumpIfNotResumeKind {
    const NAME: &'static str = "JumpIfNotResumeKind";
    const INSTRUCTION: &'static str = "INST - JumpIfNotResumeKind";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let exit = context.vm.read::<u32>();
        let resume_kind = context.vm.read::<u8>();

        let generator_resume_kind = context.vm.pop().to_generator_resume_kind();
        context.vm.push(generator_resume_kind);

        if generator_resume_kind as u8 != resume_kind {
            context.vm.frame_mut().pc = exit;
        }

        Ok(CompletionType::Normal)
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

        let generator_resume_kind = context.vm.pop().to_generator_resume_kind();
        let received = context.vm.pop();

        // Preemptively popping removes the iterator from the iterator stack if any operation
        // throws, which avoids calling cleanup operations on the poisoned iterator.
        let iterator_record = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");

        match generator_resume_kind {
            GeneratorResumeKind::Normal => {
                let result = iterator_record.next_method().call(
                    &iterator_record.iterator().clone().into(),
                    &[received],
                    context,
                )?;
                context.vm.push(false);
                context.vm.push(result);
                context.vm.push(GeneratorResumeKind::Normal);
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
                    context.vm.push(GeneratorResumeKind::Normal);
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
                    context.vm.push(GeneratorResumeKind::Normal);
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

        let generator_resume_kind = context.vm.pop().to_generator_resume_kind();

        let result = context.vm.pop();
        let is_return = context.vm.pop().to_boolean();

        if generator_resume_kind == GeneratorResumeKind::Throw {
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
