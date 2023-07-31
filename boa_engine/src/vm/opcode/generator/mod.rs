pub(crate) mod yield_stm;

use std::collections::VecDeque;

use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        generator::{GeneratorContext, GeneratorState},
    },
    environments::EnvironmentStack,
    error::JsNativeError,
    object::{ObjectData, PROTOTYPE},
    string::utf16,
    vm::{
        call_frame::GeneratorResumeKind,
        opcode::{Operation, ReThrow},
        CallFrame, CompletionType,
    },
    Context, JsError, JsObject, JsResult,
};

pub(crate) use yield_stm::*;

use super::SetReturnValue;

/// `Generator` implements the Opcode Operation for `Opcode::Generator`
///
/// Operation:
///  - Creates the generator object and yields.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Generator;

impl Operation for Generator {
    const NAME: &'static str = "Generator";
    const INSTRUCTION: &'static str = "INST - Generator";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let r#async = context.vm.read::<u8>() != 0;

        let code_block = context.vm.frame().code_block().clone();
        let pc = context.vm.frame().pc;
        let mut dummy_call_frame = CallFrame::new(code_block);
        dummy_call_frame.pc = pc;
        let call_frame = std::mem::replace(context.vm.frame_mut(), dummy_call_frame);

        let this_function_object = context
            .vm
            .active_function
            .clone()
            .expect("active function should be set to the generator");

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

        let global_environement = context.vm.environments.global();
        let environments = std::mem::replace(
            &mut context.vm.environments,
            EnvironmentStack::new(global_environement),
        );
        let stack = std::mem::take(&mut context.vm.stack);

        let data = if r#async {
            ObjectData::async_generator(AsyncGenerator {
                state: AsyncGeneratorState::SuspendedStart,
                context: Some(GeneratorContext::new(
                    environments,
                    stack,
                    context.vm.active_function.clone(),
                    call_frame,
                    context.realm().clone(),
                )),
                queue: VecDeque::new(),
            })
        } else {
            ObjectData::generator(crate::builtins::generator::Generator {
                state: GeneratorState::SuspendedStart {
                    context: GeneratorContext::new(
                        environments,
                        stack,
                        context.vm.active_function.clone(),
                        call_frame,
                        context.realm().clone(),
                    ),
                },
            })
        };

        let generator =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), proto, data);

        if r#async {
            let gen_clone = generator.clone();
            let mut generator_mut = generator.borrow_mut();
            let gen = generator_mut
                .as_async_generator_mut()
                .expect("must be object here");
            let gen_context = gen.context.as_mut().expect("must exist");
            // TODO: try to move this to the context itself.
            gen_context
                .call_frame
                .as_mut()
                .expect("should have a call frame initialized")
                .async_generator = Some(gen_clone);
        }

        context.vm.push(generator);
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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        // Step 3.e-g in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart)
        let generator_object = context
            .vm
            .frame()
            .async_generator
            .clone()
            .expect("There should be a object");

        let mut generator_object_mut = generator_object.borrow_mut();
        let generator = generator_object_mut
            .as_async_generator_mut()
            .expect("must be async generator");

        generator.state = AsyncGeneratorState::Completed;
        generator.context = None;

        let next = generator
            .queue
            .pop_front()
            .expect("must have item in queue");
        drop(generator_object_mut);

        let return_value = std::mem::take(&mut context.vm.frame_mut().return_value);

        if let Some(error) = context.vm.pending_exception.take() {
            AsyncGenerator::complete_step(&next, Err(error), true, None, context);
        } else {
            AsyncGenerator::complete_step(&next, Ok(return_value), true, None, context);
        }
        AsyncGenerator::drain_queue(&generator_object, context);

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
