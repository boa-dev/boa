use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        iterable::IteratorRecord,
    },
    error::JsNativeError,
    vm::{
        call_frame::{FinallyReturn, GeneratorResumeKind},
        opcode::Operation,
        ShouldExit,
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        match context.vm.frame().generator_resume_kind {
            GeneratorResumeKind::Normal => Ok(ShouldExit::False),
            GeneratorResumeKind::Throw => {
                let received = context.vm.pop();
                Err(JsError::from_opaque(received))
            }
            GeneratorResumeKind::Return => {
                let mut finally_left = false;

                while let Some(catch_addresses) = context.vm.frame().catch.last() {
                    if let Some(finally_address) = catch_addresses.finally {
                        let frame = context.vm.frame_mut();
                        frame.pc = finally_address as usize;
                        frame.finally_return = FinallyReturn::Ok;
                        frame.catch.pop();
                        finally_left = true;
                        break;
                    }
                    context.vm.frame_mut().catch.pop();
                }

                if finally_left {
                    return Ok(ShouldExit::False);
                }
                Ok(ShouldExit::True)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();

        if context.vm.frame().generator_resume_kind == GeneratorResumeKind::Throw {
            return Err(JsError::from_opaque(value));
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
                context.vm.push(completion.clone()?);
                context.vm.push(false);
            }

            context.vm.push(false);
        } else {
            gen.state = AsyncGeneratorState::SuspendedYield;
            context.vm.push(true);
            context.vm.push(true);
        }
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
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
                let result = context.call(&next_method, &iterator.clone().into(), &[received])?;
                let result_object = result.as_object().ok_or_else(|| {
                    JsNativeError::typ().with_message("generator next method returned non-object")
                })?;
                let done = result_object.get("done", context)?.to_boolean();
                if done {
                    context.vm.frame_mut().pc = done_address as usize;
                    let value = result_object.get("value", context)?;
                    context.vm.push(value);
                    return Ok(ShouldExit::False);
                }
                let value = result_object.get("value", context)?;
                context.vm.push(iterator.clone());
                context.vm.push(next_method.clone());
                context.vm.push(done);
                context.vm.push(value);
                Ok(ShouldExit::Yield)
            }
            GeneratorResumeKind::Throw => {
                let throw = iterator.get_method("throw", context)?;
                if let Some(throw) = throw {
                    let result = throw.call(&iterator.clone().into(), &[received], context)?;
                    let result_object = result.as_object().ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("generator throw method returned non-object")
                    })?;
                    let done = result_object.get("done", context)?.to_boolean();
                    if done {
                        context.vm.frame_mut().pc = done_address as usize;
                        let value = result_object.get("value", context)?;
                        context.vm.push(value);
                        return Ok(ShouldExit::False);
                    }
                    let value = result_object.get("value", context)?;
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(done);
                    context.vm.push(value);
                    return Ok(ShouldExit::Yield);
                }
                context.vm.frame_mut().pc = done_address as usize;
                let iterator_record = IteratorRecord::new(iterator.clone(), next_method, done);
                iterator_record.close(Ok(JsValue::Undefined), context)?;

                Err(JsNativeError::typ()
                    .with_message("iterator does not have a throw method")
                    .into())
            }
            GeneratorResumeKind::Return => {
                let r#return = iterator.get_method("return", context)?;
                if let Some(r#return) = r#return {
                    let result = r#return.call(&iterator.clone().into(), &[received], context)?;
                    let result_object = result.as_object().ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("generator return method returned non-object")
                    })?;
                    let done = result_object.get("done", context)?.to_boolean();
                    if done {
                        context.vm.frame_mut().pc = done_address as usize;
                        let value = result_object.get("value", context)?;
                        context.vm.push(value);
                        return Ok(ShouldExit::True);
                    }
                    let value = result_object.get("value", context)?;
                    context.vm.push(iterator.clone());
                    context.vm.push(next_method.clone());
                    context.vm.push(done);
                    context.vm.push(value);
                    return Ok(ShouldExit::Yield);
                }
                context.vm.frame_mut().pc = done_address as usize;
                context.vm.push(received);
                Ok(ShouldExit::True)
            }
        }
    }
}
