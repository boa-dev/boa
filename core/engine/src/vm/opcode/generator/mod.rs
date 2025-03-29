pub(crate) mod yield_stm;

use super::VaryingOperand;
use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        generator::{GeneratorContext, GeneratorState},
    },
    js_string,
    object::PROTOTYPE,
    vm::{
        call_frame::GeneratorResumeKind,
        opcode::{Operation, ReThrow},
        CompletionType, Registers,
    },
    Context, JsError, JsObject, JsResult,
};
use std::collections::VecDeque;

pub(crate) use yield_stm::*;

/// `Generator` implements the Opcode Operation for `Opcode::Generator`
///
/// Operation:
///  - Creates the generator object and yields.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Generator;

impl Generator {
    #[inline(always)]
    pub(super) fn operation(
        r#async: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let r#async = u32::from(r#async) != 0;

        let active_function = context.vm.frame().function(&context.vm);
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
            let generator_context = GeneratorContext::from_current(
                context,
                registers.clone_current_frame(),
                Some(generator.clone()),
            );

            let mut gen = generator
                .downcast_mut::<AsyncGenerator>()
                .expect("must be object here");

            gen.context = Some(generator_context);
        } else {
            let generator_context =
                GeneratorContext::from_current(context, registers.clone_current_frame(), None);

            let mut gen = generator
                .downcast_mut::<crate::builtins::generator::Generator>()
                .expect("must be object here");

            gen.state = GeneratorState::SuspendedStart {
                context: generator_context,
            };
        }

        context.vm.set_return_value(generator.into());
        Ok(CompletionType::Yield)
    }
}

impl Operation for Generator {
    const NAME: &'static str = "Generator";
    const INSTRUCTION: &'static str = "INST - Generator";
    const COST: u8 = 8;
}

/// `AsyncGeneratorClose` implements the Opcode Operation for `Opcode::AsyncGeneratorClose`
///
/// Operation:
///  - Close an async generator function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AsyncGeneratorClose;

impl AsyncGeneratorClose {
    #[inline(always)]
    pub(super) fn operation(
        _: (),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        // Step 3.e-g in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart)
        let generator = context
            .vm
            .frame()
            .async_generator_object(registers)
            .expect("There should be a object")
            .downcast::<AsyncGenerator>()
            .expect("must be async generator");

        let mut gen = generator.borrow_mut();

        // e. Assert: If we return here, the async generator either threw an exception or performed either an implicit or explicit return.
        // f. Remove acGenContext from the execution context stack and restore the execution context that is at the top of the execution context stack as the running execution context.

        // g. Set acGenerator.[[AsyncGeneratorState]] to draining-queue.
        gen.data.state = AsyncGeneratorState::DrainingQueue;

        // h. If result is a normal completion, set result to NormalCompletion(undefined).
        // i. If result is a return completion, set result to NormalCompletion(result.[[Value]]).
        let return_value = context.vm.take_return_value();

        let result = context
            .vm
            .pending_exception
            .take()
            .map_or(Ok(return_value), Err);

        drop(gen);

        // j. Perform AsyncGeneratorCompleteStep(acGenerator, result, true).
        AsyncGenerator::complete_step(&generator, result, true, None, context);
        // k. Perform AsyncGeneratorDrainQueue(acGenerator).
        AsyncGenerator::drain_queue(&generator, context);

        // l. Return undefined.
        Ok(CompletionType::Normal)
    }
}

impl Operation for AsyncGeneratorClose {
    const NAME: &'static str = "AsyncGeneratorClose";
    const INSTRUCTION: &'static str = "INST - AsyncGeneratorClose";
    const COST: u8 = 8;
}

/// `GeneratorNext` implements the Opcode Operation for `Opcode::GeneratorNext`
///
/// Operation:
///  - Resumes the current generator function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorNext;

impl GeneratorNext {
    #[inline(always)]
    pub(super) fn operation(
        (resume_kind, value): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let resume_kind = registers.get(resume_kind.into()).to_generator_resume_kind();
        match resume_kind {
            GeneratorResumeKind::Normal => Ok(CompletionType::Normal),
            GeneratorResumeKind::Throw => {
                Err(JsError::from_opaque(registers.get(value.into()).clone()))
            }
            GeneratorResumeKind::Return => {
                assert!(context.vm.pending_exception.is_none());
                let value = registers.get(value.into());
                context.vm.set_return_value(value.clone());
                ReThrow::operation((), registers, context)
            }
        }
    }
}

impl Operation for GeneratorNext {
    const NAME: &'static str = "GeneratorNext";
    const INSTRUCTION: &'static str = "INST - GeneratorNext";
    const COST: u8 = 1;
}

/// `JumpIfNotResumeKind` implements the Opcode Operation for `Opcode::JumpIfNotResumeKind`
///
/// Operation:
///  - Jumps to the specified address if the resume kind is not equal.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JumpIfNotResumeKind;

impl JumpIfNotResumeKind {
    #[allow(clippy::unnecessary_wraps)]
    #[inline(always)]
    pub(super) fn operation(
        (exit, expected, value): (u32, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let resume_kind = registers.get(value.into()).to_generator_resume_kind();
        if resume_kind as u8 != u32::from(expected) as u8 {
            context.vm.frame_mut().pc = exit;
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for JumpIfNotResumeKind {
    const NAME: &'static str = "JumpIfNotResumeKind";
    const INSTRUCTION: &'static str = "INST - JumpIfNotResumeKind";
    const COST: u8 = 1;
}

/// `GeneratorDelegateNext` implements the Opcode Operation for `Opcode::GeneratorDelegateNext`
///
/// Operation:
///  - Delegates the current generator function to another iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorDelegateNext;

impl GeneratorDelegateNext {
    #[inline(always)]
    pub(super) fn operation(
        (throw_method_undefined, return_method_undefined, value, resume_kind, is_return): (
            u32,
            u32,
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
        ),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let resume_kind = registers.get(resume_kind.into()).to_generator_resume_kind();
        let received = registers.get(value.into());

        // Preemptively popping removes the iterator from the iterator stack if any operation
        // throws, which avoids calling cleanup operations on the poisoned iterator.
        let iterator_record = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");

        match resume_kind {
            GeneratorResumeKind::Normal => {
                let result = iterator_record.next_method().call(
                    &iterator_record.iterator().clone().into(),
                    &[received.clone()],
                    context,
                )?;
                registers.set(is_return.into(), false.into());
                registers.set(value.into(), result);
            }
            GeneratorResumeKind::Throw => {
                let throw = iterator_record
                    .iterator()
                    .get_method(js_string!("throw"), context)?;
                if let Some(throw) = throw {
                    let result = throw.call(
                        &iterator_record.iterator().clone().into(),
                        &[received.clone()],
                        context,
                    )?;
                    registers.set(is_return.into(), false.into());
                    registers.set(value.into(), result);
                } else {
                    context.vm.frame_mut().pc = throw_method_undefined;
                }
            }
            GeneratorResumeKind::Return => {
                let r#return = iterator_record
                    .iterator()
                    .get_method(js_string!("return"), context)?;
                if let Some(r#return) = r#return {
                    let result = r#return.call(
                        &iterator_record.iterator().clone().into(),
                        &[received.clone()],
                        context,
                    )?;
                    registers.set(is_return.into(), true.into());
                    registers.set(value.into(), result);
                } else {
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

impl Operation for GeneratorDelegateNext {
    const NAME: &'static str = "GeneratorDelegateNext";
    const INSTRUCTION: &'static str = "INST - GeneratorDelegateNext";
    const COST: u8 = 18;
}

/// `GeneratorDelegateResume` implements the Opcode Operation for `Opcode::GeneratorDelegateResume`
///
/// Operation:
///  - Resume the generator with yield delegate logic after it awaits a value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorDelegateResume;

impl GeneratorDelegateResume {
    #[inline(always)]
    pub(super) fn operation(
        (return_gen, exit, value, resume_kind, is_return): (
            u32,
            u32,
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
        ),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let resume_kind = registers.get(resume_kind.into()).to_generator_resume_kind();
        let result = registers.get(value.into());
        let is_return = registers.get(is_return.into()).to_boolean();

        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");

        if resume_kind == GeneratorResumeKind::Throw {
            return Err(JsError::from_opaque(result.clone()));
        }

        iterator.update_result(result.clone(), context)?;

        if iterator.done() {
            let result = iterator.value(context)?;
            registers.set(value.into(), result);
            context.vm.frame_mut().pc = if is_return { return_gen } else { exit };
            return Ok(CompletionType::Normal);
        }

        context.vm.frame_mut().iterators.push(iterator);

        Ok(CompletionType::Normal)
    }
}

impl Operation for GeneratorDelegateResume {
    const NAME: &'static str = "GeneratorDelegateResume";
    const INSTRUCTION: &'static str = "INST - GeneratorDelegateResume";
    const COST: u8 = 7;
}
