pub(crate) mod yield_stm;

use crate::{
    builtins::async_generator::{AsyncGenerator, AsyncGeneratorState},
    error::JsNativeError,
    string::utf16,
    vm::{
        call_frame::{AbruptCompletionRecord, GeneratorResumeKind},
        opcode::{control_flow::Return, Operation},
        CompletionType,
    },
    Context, JsError, JsResult, JsValue,
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
            GeneratorResumeKind::Return => {
                let finally_entries = context
                    .vm
                    .frame()
                    .env_stack
                    .iter()
                    .filter(|entry| entry.is_finally_env());
                if let Some(next_finally) = finally_entries.rev().next() {
                    if context.vm.frame().pc < next_finally.start_address() {
                        context.vm.frame_mut().pc = next_finally.start_address();
                        let return_record = AbruptCompletionRecord::new_return();
                        context.vm.frame_mut().abrupt_completion = Some(return_record);
                        return Ok(CompletionType::Normal);
                    }
                }

                let return_record = AbruptCompletionRecord::new_return();
                context.vm.frame_mut().abrupt_completion = Some(return_record);
                Ok(CompletionType::Return)
            }
        }
    }
}

/// `AsyncGeneratorNext` implements the Opcode Operation for `Opcode::AsyncGeneratorNext`
///
/// Operation:
///  - Resumes the current generator function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AsyncGeneratorNext;

impl Operation for AsyncGeneratorNext {
    const NAME: &'static str = "AsyncGeneratorNext";
    const INSTRUCTION: &'static str = "INST - AsyncGeneratorNext";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let skip_yield = context.vm.read::<u32>();
        let skip_yield_await = context.vm.read::<u32>();

        if context.vm.frame().generator_resume_kind == GeneratorResumeKind::Throw {
            return Err(JsError::from_opaque(context.vm.pop()));
        }

        let value = context.vm.pop();

        let completion = Ok(value);
        let generator_object = context
            .vm
            .frame()
            .async_generator
            .as_ref()
            .expect("must be in generator context here")
            .clone();
        let next = generator_object
            .borrow_mut()
            .as_async_generator_mut()
            .expect("must be async generator object")
            .queue
            .pop_front()
            .expect("must have item in queue");
        AsyncGenerator::complete_step(&next, completion, false, None, context);

        let mut generator_object_mut = generator_object.borrow_mut();
        let gen = generator_object_mut
            .as_async_generator_mut()
            .expect("must be async generator object");

        if let Some(next) = gen.queue.front() {
            let (completion, r#return) = &next.completion;
            if *r#return {
                let value = match completion {
                    Ok(value) => value.clone(),
                    Err(e) => e.clone().to_opaque(context),
                };
                context.vm.push(value);
                context.vm.frame_mut().pc = skip_yield;
            } else {
                context.vm.push(completion.clone()?);
                context.vm.frame_mut().pc = skip_yield_await;
            }
        } else {
            gen.state = AsyncGeneratorState::SuspendedYield;
        }
        Ok(CompletionType::Normal)
    }
}

/// `GeneratorAsyncResumeYield` implements the Opcode Operation for `Opcode::GeneratorAsyncResumeYield`
///
/// Operation:
///  - Resumes the current async generator function after a yield.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorAsyncResumeYield;

impl Operation for GeneratorAsyncResumeYield {
    const NAME: &'static str = "GeneratorAsyncResumeYield";
    const INSTRUCTION: &'static str = "INST - GeneratorAsyncResumeYield";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let normal_completion = context.vm.read::<u32>();

        match context.vm.frame().generator_resume_kind {
            GeneratorResumeKind::Normal => {
                context.vm.frame_mut().pc = normal_completion;
                Ok(CompletionType::Normal)
            }
            GeneratorResumeKind::Throw => Err(JsError::from_opaque(context.vm.pop())),
            GeneratorResumeKind::Return => Ok(CompletionType::Normal),
        }
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

        iterator.update_result(result.clone(), context)?;

        if iterator.done() {
            let value = iterator.value(context)?;
            context.vm.push(value);

            context.vm.frame_mut().pc = if is_return { return_gen } else { exit };

            return Ok(CompletionType::Normal);
        }

        let Some(async_gen) = context.vm.frame().async_generator.clone() else {
            context.vm.frame_mut().iterators.push(iterator);
            context.vm.push(result);
            context.vm.frame_mut().r#yield = true;
            return Ok(CompletionType::Return);
        };

        let value = iterator.value(context)?;
        context.vm.frame_mut().iterators.push(iterator);

        let completion = Ok(value);
        let next = async_gen
            .borrow_mut()
            .as_async_generator_mut()
            .expect("must be async generator object")
            .queue
            .pop_front()
            .expect("must have item in queue");
        AsyncGenerator::complete_step(&next, completion, false, None, context);

        let mut generator_object_mut = async_gen.borrow_mut();
        let gen = generator_object_mut
            .as_async_generator_mut()
            .expect("must be async generator object");

        if let Some(next) = gen.queue.front() {
            let (completion, r#return) = &next.completion;
            if *r#return {
                let value = match completion {
                    Ok(value) => value.clone(),
                    Err(e) => e.clone().to_opaque(context),
                };
                context.vm.push(value);
            } else {
                context.vm.push(completion.clone()?);
            }
            Ok(CompletionType::Normal)
        } else {
            gen.state = AsyncGeneratorState::SuspendedYield;
            context.vm.push(JsValue::undefined());
            context.vm.frame_mut().r#yield = true;
            Ok(CompletionType::Return)
        }
    }
}
