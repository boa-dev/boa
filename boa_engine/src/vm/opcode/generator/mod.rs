use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        iterable::IteratorRecord,
    },
    error::JsNativeError,
    string::utf16,
    vm::{
        call_frame::{AbruptCompletionRecord, GeneratorResumeKind},
        ok_or_throw_completion,
        opcode::Operation,
        throw_completion, CompletionType,
    },
    Context, JsError, JsValue,
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        match context.vm.frame().generator_resume_kind {
            GeneratorResumeKind::Normal => CompletionType::Normal,
            GeneratorResumeKind::Throw => CompletionType::Throw,
            GeneratorResumeKind::Return => {
                // TODO: Determine GeneratorResumeKind::Return can be called in a finally, in which case we would need to skip the first value.
                let finally_entries = context
                    .vm
                    .frame()
                    .env_stack
                    .iter()
                    .filter(|entry| entry.is_finally_env());
                if let Some(next_finally) = finally_entries.rev().next() {
                    context.vm.frame_mut().pc = next_finally.start_address() as usize;
                    let return_record = AbruptCompletionRecord::new_return();
                    context.vm.frame_mut().abrupt_completion = Some(return_record);
                    return CompletionType::Normal;
                }

                CompletionType::Return
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        if context.vm.frame().generator_resume_kind == GeneratorResumeKind::Throw {
            return CompletionType::Throw;
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
        AsyncGenerator::complete_step(&next, completion, false, context);

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
                context.vm.push(true);
            } else {
                context
                    .vm
                    .push(ok_or_throw_completion!(completion.clone(), context));
                context.vm.push(false);
            }

            context.vm.push(false);
        } else {
            gen.state = AsyncGeneratorState::SuspendedYield;
            context.vm.push(true);
            context.vm.push(true);
        }
        CompletionType::Normal
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let done_address = context.vm.read::<u32>();
        let received = context.vm.pop();
        let done = context
            .vm
            .pop()
            .as_boolean()
            .expect("iterator [[Done]] was not a boolean");
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let iterator = iterator.as_object().expect("iterator was not an object");

        match context.vm.frame().generator_resume_kind {
            GeneratorResumeKind::Normal => {
                let result = ok_or_throw_completion!(
                    next_method.call(&iterator.clone().into(), &[received], context),
                    context
                );
                let result = ok_or_throw_completion!(
                    result.as_object().ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("generator next method returned non-object")
                    }),
                    context
                );
                let done = ok_or_throw_completion!(result.get(utf16!("done"), context), context)
                    .to_boolean();
                if done {
                    context.vm.frame_mut().pc = done_address as usize;
                    let value =
                        ok_or_throw_completion!(result.get(utf16!("value"), context), context);
                    context.vm.push(value);
                    return CompletionType::Normal;
                }
                let value = ok_or_throw_completion!(result.get(utf16!("value"), context), context);
                context.vm.push(iterator.clone());
                context.vm.push(next_method.clone());
                context.vm.push(done);
                context.vm.push(value);
                CompletionType::Return
            }
            GeneratorResumeKind::Throw => {
                let throw =
                    ok_or_throw_completion!(iterator.get_method(utf16!("throw"), context), context);
                if let Some(throw) = throw {
                    let result = ok_or_throw_completion!(
                        throw.call(&iterator.clone().into(), &[received], context),
                        context
                    );
                    let result_object = ok_or_throw_completion!(
                        result.as_object().ok_or_else(|| {
                            JsNativeError::typ()
                                .with_message("generator throw method returned non-object")
                        }),
                        context
                    );
                    let done = ok_or_throw_completion!(
                        result_object.get(utf16!("done"), context),
                        context
                    )
                    .to_boolean();
                    if done {
                        context.vm.frame_mut().pc = done_address as usize;
                        let value = ok_or_throw_completion!(
                            result_object.get(utf16!("value"), context),
                            context
                        );
                        context.vm.push(value);
                        return CompletionType::Normal;
                    }
                    let value = ok_or_throw_completion!(
                        result_object.get(utf16!("value"), context),
                        context
                    );
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(done);
                    context.vm.push(value);
                    return CompletionType::Return;
                }
                context.vm.frame_mut().pc = done_address as usize;
                let iterator_record = IteratorRecord::new(iterator.clone(), next_method, done);
                ok_or_throw_completion!(
                    iterator_record.close(Ok(JsValue::Undefined), context),
                    context
                );

                throw_completion!(
                    JsNativeError::typ()
                        .with_message("iterator does not have a throw method")
                        .into(),
                    JsError,
                    context
                );
            }
            GeneratorResumeKind::Return => {
                let r#return = ok_or_throw_completion!(
                    iterator.get_method(utf16!("return"), context),
                    context
                );
                if let Some(r#return) = r#return {
                    let result = ok_or_throw_completion!(
                        r#return.call(&iterator.clone().into(), &[received], context),
                        context
                    );
                    let result_object = ok_or_throw_completion!(
                        result.as_object().ok_or_else(|| {
                            JsNativeError::typ()
                                .with_message("generator return method returned non-object")
                        }),
                        context
                    );
                    let done = ok_or_throw_completion!(
                        result_object.get(utf16!("done"), context),
                        context
                    )
                    .to_boolean();
                    if done {
                        context.vm.frame_mut().pc = done_address as usize;
                        let value = ok_or_throw_completion!(
                            result_object.get(utf16!("value"), context),
                            context
                        );
                        context.vm.push(value);
                        return CompletionType::Return;
                    }
                    let value = ok_or_throw_completion!(
                        result_object.get(utf16!("value"), context),
                        context
                    );
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(done);
                    context.vm.push(value);
                    return CompletionType::Return;
                }
                context.vm.frame_mut().pc = done_address as usize;
                context.vm.push(received);
                CompletionType::Return
            }
        }
    }
}
