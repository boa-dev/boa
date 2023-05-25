pub(crate) mod yield_stm;

use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        iterable::IteratorRecord,
    },
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
                let result_value =
                    next_method.call(&iterator.clone().into(), &[received], context)?;
                let result = result_value.as_object().ok_or_else(|| {
                    JsNativeError::typ().with_message("generator next method returned non-object")
                })?;
                let done = result.get(utf16!("done"), context)?.to_boolean();
                if done {
                    context.vm.frame_mut().pc = done_address;
                    let value = result.get(utf16!("value"), context)?;
                    context.vm.push(value);
                    return Ok(CompletionType::Normal);
                }
                context.vm.push(iterator.clone());
                context.vm.push(next_method.clone());
                context.vm.push(result_value);
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
                        context.vm.frame_mut().pc = done_address;
                        let value = result_object.get(utf16!("value"), context)?;
                        context.vm.push(value);
                        return Ok(CompletionType::Normal);
                    }
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(result);
                    context.vm.frame_mut().r#yield = true;
                    return Ok(CompletionType::Return);
                }
                context.vm.frame_mut().pc = done_address;
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
                        let value = result_object.get(utf16!("value"), context)?;
                        context.vm.push(value);
                        return Return::execute(context);
                    }
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(result);
                    context.vm.frame_mut().r#yield = true;
                    return Ok(CompletionType::Return);
                }
                context.vm.push(received);
                Return::execute(context)
            }
        }
    }
}

/// `GeneratorAsyncDelegateNext` implements the Opcode Operation for `Opcode::GeneratorAsyncDelegateNext`
///
/// Operation:
///  - Delegates the current async generator function to another iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorAsyncDelegateNext;

impl Operation for GeneratorAsyncDelegateNext {
    const NAME: &'static str = "GeneratorAsyncDelegateNext";
    const INSTRUCTION: &'static str = "INST - GeneratorAsyncDelegateNext";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let throw_method_undefined = context.vm.read::<u32>();
        let return_method_undefined = context.vm.read::<u32>();
        let received = context.vm.pop();
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let iterator = iterator.as_object().expect("iterator was not an object");

        match context.vm.frame().generator_resume_kind {
            GeneratorResumeKind::Normal => {
                let result_value =
                    next_method.call(&iterator.clone().into(), &[received], context)?;
                context.vm.push(iterator.clone());
                context.vm.push(next_method.clone());
                context.vm.push(false);
                context.vm.push(result_value);
                Ok(CompletionType::Normal)
            }
            GeneratorResumeKind::Throw => {
                let throw = iterator.get_method(utf16!("throw"), context)?;
                if let Some(throw) = throw {
                    let result = throw.call(&iterator.clone().into(), &[received], context)?;
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(false);
                    context.vm.push(result);
                } else {
                    let error = JsNativeError::typ()
                        .with_message("iterator does not have a throw method")
                        .to_opaque(context);
                    context.vm.push(error);
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(false);
                    context.vm.frame_mut().pc = throw_method_undefined;
                }

                Ok(CompletionType::Normal)
            }
            GeneratorResumeKind::Return => {
                let r#return = iterator.get_method(utf16!("return"), context)?;
                if let Some(r#return) = r#return {
                    let result = r#return.call(&iterator.clone().into(), &[received], context)?;
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(true);
                    context.vm.push(result);
                    return Ok(CompletionType::Normal);
                }
                context.vm.push(iterator.clone());
                context.vm.push(next_method.clone());
                context.vm.push(received);
                context.vm.frame_mut().pc = return_method_undefined;
                Ok(CompletionType::Normal)
            }
        }
    }
}

/// `GeneratorAsyncDelegateResume` implements the Opcode Operation for `Opcode::GeneratorAsyncDelegateResume`
///
/// Operation:
///  - Resume the async generator with yield delegate logic after it awaits a value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorAsyncDelegateResume;

impl Operation for GeneratorAsyncDelegateResume {
    const NAME: &'static str = "GeneratorAsyncDelegateResume";
    const INSTRUCTION: &'static str = "INST - GeneratorAsyncDelegateResume";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let skip_yield = context.vm.read::<u32>();
        let normal_completion = context.vm.read::<u32>();
        let exit = context.vm.read::<u32>();

        if context.vm.frame().generator_resume_kind == GeneratorResumeKind::Throw {
            return Err(JsError::from_opaque(context.vm.pop()));
        }

        let received = context.vm.pop();
        let is_return = context.vm.pop().to_boolean();

        let result = received.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("generator next method returned non-object")
        })?;
        let done = result.get(utf16!("done"), context)?.to_boolean();
        let value = result.get(utf16!("value"), context)?;
        if done {
            context.vm.push(value);

            if is_return {
                return Return::execute(context);
            }

            context.vm.frame_mut().pc = exit;
            return Ok(CompletionType::Normal);
        }

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
                context.vm.frame_mut().pc = normal_completion;
            }
        } else {
            gen.state = AsyncGeneratorState::SuspendedYield;
        }
        Ok(CompletionType::Normal)
    }
}
