pub(crate) mod yield_stm;

use std::collections::VecDeque;

use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        generator::{GeneratorContext, GeneratorState},
    },
    error::JsNativeError,
    js_str,
    object::PROTOTYPE,
    vm::{
        call_frame::GeneratorResumeKind,
        opcode::{Operation, ReThrow},
        CallFrame, CompletionType,
    },
    Context, JsError, JsObject, JsResult, JsValue,
};

pub(crate) use yield_stm::*;

use super::SetAccumulatorFromStack;

/// `Generator` implements the Opcode Operation for `Opcode::Generator`
///
/// Operation:
///  - Creates the generator object and yields.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Generator;

impl Operation for Generator {
    const NAME: &'static str = "Generator";
    const INSTRUCTION: &'static str = "INST - Generator";
    const COST: u8 = 8;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let r#async = context.vm.read::<u8>() != 0;

        let active_function = context.vm.frame().function(&context.vm);
        let this_function_object =
            active_function.expect("active function should be set to the generator");

        let mut frame = GeneratorContext::from_current(context);

        let proto = this_function_object
            .get(PROTOTYPE, context)
            .expect("generator must have a prototype property")
            .as_object()
            .map_or_else(
                || {
                    if r#async {
                        context.intrinsics().objects().async_generator()
                    } else {
                        context.intrinsics().objects().generator()
                    }
                },
                Clone::clone,
            );

        let generator = if r#async {
            JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                AsyncGenerator {
                    state: AsyncGeneratorState::SuspendedStart,
                    context: None,
                    queue: VecDeque::new(),
                },
            )
        } else {
            JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                crate::builtins::generator::Generator {
                    state: GeneratorState::Completed,
                },
            )
        };

        if r#async {
            let rp = frame
                .call_frame
                .as_ref()
                .map_or(0, |frame| frame.rp as usize);
            frame.stack[rp + CallFrame::ASYNC_GENERATOR_OBJECT_REGISTER_INDEX as usize] =
                generator.clone().into();

            let mut gen = generator
                .downcast_mut::<AsyncGenerator>()
                .expect("must be object here");

            gen.context = Some(frame);
        } else {
            let mut gen = generator
                .downcast_mut::<crate::builtins::generator::Generator>()
                .expect("must be object here");

            gen.state = GeneratorState::SuspendedStart { context: frame };
        }

        context.vm.set_return_value(generator.into());
        Ok(CompletionType::Yield)
    }
}

/// `AsyncGeneratorClose` implements the Opcode Operation for `Opcode::AsyncGeneratorClose`
///
/// Operation:
///  - Close an async generator function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AsyncGeneratorClose;

impl Operation for AsyncGeneratorClose {
    const NAME: &'static str = "AsyncGeneratorClose";
    const INSTRUCTION: &'static str = "INST - AsyncGeneratorClose";
    const COST: u8 = 8;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        // Step 3.e-g in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart)
        let generator = context
            .vm
            .frame()
            .async_generator_object(&context.vm.stack)
            .expect("There should be a object")
            .downcast::<AsyncGenerator>()
            .expect("must be async generator");

        let mut gen = generator.borrow_mut();

        gen.data.state = AsyncGeneratorState::Completed;
        gen.data.context = None;

        let next = gen.data.queue.pop_front().expect("must have item in queue");

        let return_value = context.vm.get_return_value();
        context.vm.set_return_value(JsValue::undefined());

        let completion = context
            .vm
            .pending_exception
            .take()
            .map_or(Ok(return_value), Err);

        drop(gen);

        AsyncGenerator::complete_step(&next, completion, true, None, context);
        // TODO: Upgrade to the latest spec when the problem is fixed.
        AsyncGenerator::resume_next(&generator, context);

        Ok(CompletionType::Normal)
    }
}

/// `GeneratorNext` implements the Opcode Operation for `Opcode::GeneratorNext`
///
/// Operation:
///  - Resumes the current generator function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorNext;

impl Operation for GeneratorNext {
    const NAME: &'static str = "GeneratorNext";
    const INSTRUCTION: &'static str = "INST - GeneratorNext";
    const COST: u8 = 1;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let generator_resume_kind = context.vm.pop().to_generator_resume_kind();
        match generator_resume_kind {
            GeneratorResumeKind::Normal => Ok(CompletionType::Normal),
            GeneratorResumeKind::Throw => Err(JsError::from_opaque(context.vm.pop())),
            GeneratorResumeKind::Return => {
                assert!(context.vm.pending_exception.is_none());

                SetAccumulatorFromStack::execute(context)?;
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
    const COST: u8 = 1;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
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
    const COST: u8 = 18;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
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
                    .get_method(js_str!("throw"), context)?;
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
                    .get_method(js_str!("return"), context)?;
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
    const COST: u8 = 7;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
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
