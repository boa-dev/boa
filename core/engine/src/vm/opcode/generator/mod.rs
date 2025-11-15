pub(crate) mod yield_stm;

use super::VaryingOperand;
use crate::{
    Context, JsError, JsObject, JsResult,
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        generator::{GeneratorContext, GeneratorState},
    },
    js_string,
    object::PROTOTYPE,
    vm::{
        CompletionRecord,
        call_frame::GeneratorResumeKind,
        opcode::{Operation, ReThrow},
    },
};
use std::{collections::VecDeque, ops::ControlFlow};

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
        context: &mut Context,
    ) -> ControlFlow<CompletionRecord> {
        let r#async = u32::from(r#async) != 0;

        let active_function = context.vm.stack.get_function(context.vm.frame());
        let this_function_object =
            active_function.expect("active function should be set to the generator");

        let proto = this_function_object
            .get(PROTOTYPE, context)
            .expect("generator must have a prototype property")
            .as_object()
            .unwrap_or_else(|| {
                if r#async {
                    context.intrinsics().objects().async_generator()
                } else {
                    context.intrinsics().objects().generator()
                }
            });

        let generator = if r#async {
            let generator = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                AsyncGenerator {
                    state: AsyncGeneratorState::SuspendedStart,
                    context: None,
                    queue: VecDeque::new(),
                },
            );
            let gen_ctx = GeneratorContext::from_current(context, Some(generator.clone().upcast()));
            generator.borrow_mut().data_mut().context = Some(gen_ctx);
            generator.upcast()
        } else {
            let generator = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                proto,
                crate::builtins::generator::Generator {
                    state: GeneratorState::Completed,
                },
            );

            let gen_ctx = GeneratorContext::from_current(context, None);

            generator.borrow_mut().data_mut().state =
                GeneratorState::SuspendedStart { context: gen_ctx };
            generator.upcast()
        };

        context.vm.set_return_value(generator.into());
        context.handle_yield()
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
    pub(super) fn operation((): (), context: &mut Context) -> JsResult<()> {
        // Step 3.e-g in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart)
        let generator = context
            .vm
            .stack
            .async_generator_object(&context.vm.frame)
            .expect("There should be a object")
            .downcast::<AsyncGenerator>()
            .expect("must be async generator");

        let mut r#gen = generator.borrow_mut();

        // e. Assert: If we return here, the async generator either threw an exception or performed either an implicit or explicit return.
        // f. Remove acGenContext from the execution context stack and restore the execution context that is at the top of the execution context stack as the running execution context.

        // g. Set acGenerator.[[AsyncGeneratorState]] to draining-queue.
        r#gen.data_mut().state = AsyncGeneratorState::DrainingQueue;

        // h. If result is a normal completion, set result to NormalCompletion(undefined).
        // i. If result is a return completion, set result to NormalCompletion(result.[[Value]]).
        let return_value = context.vm.take_return_value();

        let result = context
            .vm
            .pending_exception
            .take()
            .map_or(Ok(return_value), Err);

        drop(r#gen);

        // j. Perform AsyncGeneratorCompleteStep(acGenerator, result, true).
        AsyncGenerator::complete_step(&generator, result, true, None, context)?;
        // k. Perform AsyncGeneratorDrainQueue(acGenerator).
        AsyncGenerator::drain_queue(&generator, context)?;

        // l. Return undefined.
        Ok(())
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
        context: &mut Context,
    ) -> ControlFlow<CompletionRecord> {
        let resume_kind = context
            .vm
            .get_register(resume_kind.into())
            .to_generator_resume_kind();
        match resume_kind {
            GeneratorResumeKind::Normal => ControlFlow::Continue(()),
            GeneratorResumeKind::Throw => context.handle_error(JsError::from_opaque(
                context.vm.get_register(value.into()).clone(),
            )),
            GeneratorResumeKind::Return => {
                assert!(context.vm.pending_exception.is_none());
                let value = context.vm.get_register(value.into());
                context.vm.set_return_value(value.clone());
                ReThrow::operation((), context)
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
    #[inline(always)]
    pub(super) fn operation(
        (exit, expected, value): (u32, VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) {
        let resume_kind = context
            .vm
            .get_register(value.into())
            .to_generator_resume_kind();
        if resume_kind as u8 != u32::from(expected) as u8 {
            context.vm.frame_mut().pc = exit;
        }
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
        context: &mut Context,
    ) -> JsResult<()> {
        let resume_kind = context
            .vm
            .get_register(resume_kind.into())
            .to_generator_resume_kind();
        let received = context.vm.get_register(value.into()).clone();

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
                    std::slice::from_ref(&received),
                    context,
                )?;
                context.vm.set_register(is_return.into(), false.into());
                context.vm.set_register(value.into(), result);
            }
            GeneratorResumeKind::Throw => {
                let throw = iterator_record
                    .iterator()
                    .get_method(js_string!("throw"), context)?;
                if let Some(throw) = throw {
                    let result = throw.call(
                        &iterator_record.iterator().clone().into(),
                        std::slice::from_ref(&received),
                        context,
                    )?;
                    context.vm.set_register(is_return.into(), false.into());
                    context.vm.set_register(value.into(), result);
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
                        std::slice::from_ref(&received),
                        context,
                    )?;
                    context.vm.set_register(is_return.into(), true.into());
                    context.vm.set_register(value.into(), result);
                } else {
                    context.vm.frame_mut().pc = return_method_undefined;

                    // The current iterator didn't have a cleanup `return` method, so we can
                    // skip pushing it to the iterator stack for cleanup.
                    return Ok(());
                }
            }
        }

        context.vm.frame_mut().iterators.push(iterator_record);

        Ok(())
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
        context: &mut Context,
    ) -> JsResult<()> {
        let resume_kind = context
            .vm
            .get_register(resume_kind.into())
            .to_generator_resume_kind();
        let result = context.vm.get_register(value.into()).clone();
        let is_return = context.vm.get_register(is_return.into()).to_boolean();

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
            context.vm.set_register(value.into(), result);
            context.vm.frame_mut().pc = if is_return { return_gen } else { exit };
            return Ok(());
        }

        context.vm.frame_mut().iterators.push(iterator);

        Ok(())
    }
}

impl Operation for GeneratorDelegateResume {
    const NAME: &'static str = "GeneratorDelegateResume";
    const INSTRUCTION: &'static str = "INST - GeneratorDelegateResume";
    const COST: u8 = 7;
}
