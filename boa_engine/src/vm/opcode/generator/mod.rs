pub(crate) mod yield_stm;

use std::collections::VecDeque;

use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        generator::{GeneratorContext, GeneratorState},
    },
    error::JsNativeError,
    object::PROTOTYPE,
    string::utf16,
    vm::{
        call_frame::GeneratorResumeKind,
        opcode::{Operation, ReThrow},
        CallFrame, CompletionType,
    },
    Context, JsError, JsObject, JsResult, JsValue,
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
    const COST: u8 = 8;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let r#async = context.vm.read::<u8>() != 0;

        let frame = context.vm.frame();
        let code_block = frame.code_block().clone();
        let active_runnable = frame.active_runnable.clone();
        let active_function = frame.function(&context.vm);
        let environments = frame.environments.clone();
        let realm = frame.realm.clone();
        let pc = frame.pc;

        let mut dummy_call_frame = CallFrame::new(code_block, active_runnable, environments, realm);
        dummy_call_frame.pc = pc;
        let mut call_frame = std::mem::replace(context.vm.frame_mut(), dummy_call_frame);

        context
            .vm
            .frame_mut()
            .set_exit_early(call_frame.exit_early());

        call_frame.environments = context.vm.environments.clone();
        call_frame.realm = context.realm().clone();

        let fp = call_frame.fp as usize;

        let stack = context.vm.stack[fp..].to_vec();
        context.vm.stack.truncate(fp);

        call_frame.fp = 0;

        let this_function_object =
            active_function.expect("active function should be set to the generator");

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
                    context: Some(GeneratorContext::new(stack, call_frame)),
                    queue: VecDeque::new(),
                },
            )
        } else {
            JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                crate::builtins::generator::Generator {
                    state: GeneratorState::SuspendedStart {
                        context: GeneratorContext::new(stack, call_frame),
                    },
                },
            )
        };

        if r#async {
            let gen_clone = generator.clone();
            let mut gen = generator
                .downcast_mut::<AsyncGenerator>()
                .expect("must be object here");
            let gen_context = gen.context.as_mut().expect("must exist");
            // TODO: try to move this to the context itself.
            gen_context
                .call_frame
                .as_mut()
                .expect("should have a call frame initialized")
                .async_generator = Some(gen_clone);
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
        let generator_object = context
            .vm
            .frame()
            .async_generator
            .clone()
            .expect("There should be a object");

        let mut gen = generator_object
            .downcast_mut::<AsyncGenerator>()
            .expect("must be async generator");

        gen.state = AsyncGeneratorState::Completed;
        gen.context = None;

        let next = gen
            .queue
            .pop_front()
            .expect("must have item in queue");
        drop(gen);

        let return_value = context.vm.get_return_value();
        context.vm.set_return_value(JsValue::undefined());

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
    const COST: u8 = 1;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
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
