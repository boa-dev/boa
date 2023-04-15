use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        iterable::IteratorRecord,
    },
    error::JsNativeError,
    string::utf16,
    vm::{
        call_frame::{AbruptCompletionRecord, GeneratorResumeKind},
        opcode::Operation,
        CompletionType,
    },
    Context, JsError, JsResult, JsValue,
};

pub(crate) mod yield_stm;

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
                    if context.vm.frame().pc < next_finally.start_address() as usize {
                        context.vm.frame_mut().pc = next_finally.start_address() as usize;
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
                context.vm.frame_mut().pc = skip_yield as usize;
            } else {
                context.vm.push(completion.clone()?);
                context.vm.frame_mut().pc = skip_yield_await as usize;
            }
        } else {
            gen.state = AsyncGeneratorState::SuspendedYield;
        }
        Ok(CompletionType::Normal)
    }
}

/// `GeneratorNextDelegate` implements the Opcode Operation for `Opcode::GeneratorNextDelegate`
///
/// Operation:
///  - Delegates the current generator function another generator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorNextDelegate;

impl Operation for GeneratorNextDelegate {
    const NAME: &'static str = "GeneratorNextDelegate";
    const INSTRUCTION: &'static str = "INST - GeneratorNextDelegate";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let done_address = context.vm.read::<u32>();
        let received = context.vm.pop();
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let iterator = iterator.as_object().expect("iterator was not an object");

        match context.vm.frame().generator_resume_kind {
            GeneratorResumeKind::Normal => {
                let result = next_method.call(&iterator.clone().into(), &[received], context)?;
                let result = result.as_object().ok_or_else(|| {
                    JsNativeError::typ().with_message("generator next method returned non-object")
                })?;
                // TODO: This is wrong for async generators, since we need to await the result first.
                let done = result.get(utf16!("done"), context)?.to_boolean();
                if done {
                    context.vm.frame_mut().pc = done_address as usize;
                    let value = result.get(utf16!("value"), context)?;
                    context.vm.push(value);
                    return Ok(CompletionType::Normal);
                }
                let value = result.get(utf16!("value"), context)?;
                context.vm.push(iterator.clone());
                context.vm.push(next_method.clone());
                context.vm.push(value);
                context.vm.frame_mut().r#yield = true;
                Ok(CompletionType::Return)
            }
            GeneratorResumeKind::Throw => {
                let throw = iterator.get_method(utf16!("throw"), context)?;
                if let Some(throw) = throw {
                    let result = throw.call(&iterator.clone().into(), &[received], context)?;
                    let result_object = result.as_object().ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("generator throw method returned non-object")
                    })?;
                    let done = result_object.get(utf16!("done"), context)?.to_boolean();
                    if done {
                        context.vm.frame_mut().pc = done_address as usize;
                        let value = result_object.get(utf16!("value"), context)?;
                        context.vm.push(value);
                        return Ok(CompletionType::Normal);
                    }
                    let value = result_object.get(utf16!("value"), context)?;
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(value);
                    context.vm.frame_mut().r#yield = true;
                    return Ok(CompletionType::Return);
                }
                context.vm.frame_mut().pc = done_address as usize;
                let iterator_record = IteratorRecord::new(iterator.clone(), next_method, false);
                iterator_record.close(Ok(JsValue::Undefined), context)?;

                Err(JsNativeError::typ()
                    .with_message("iterator does not have a throw method")
                    .into())
            }
            GeneratorResumeKind::Return => {
                let r#return = iterator.get_method(utf16!("return"), context)?;
                if let Some(r#return) = r#return {
                    let result = r#return.call(&iterator.clone().into(), &[received], context)?;
                    let result_object = result.as_object().ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("generator return method returned non-object")
                    })?;
                    let done = result_object.get(utf16!("done"), context)?.to_boolean();
                    if done {
                        context.vm.frame_mut().pc = done_address as usize;
                        let value = result_object.get(utf16!("value"), context)?;
                        context.vm.push(value);
                        return Ok(CompletionType::Return);
                    }
                    let value = result_object.get(utf16!("value"), context)?;
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(value);
                    context.vm.frame_mut().r#yield = true;
                    return Ok(CompletionType::Return);
                }
                context.vm.frame_mut().pc = done_address as usize;
                context.vm.push(received);
                Ok(CompletionType::Return)
            }
        }
    }
}
