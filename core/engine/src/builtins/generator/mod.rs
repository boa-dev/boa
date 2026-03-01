//! Boa's implementation of ECMAScript's global `Generator` object.
//!
//! A Generator is an instance of a generator function and conforms to both the Iterator and Iterable interfaces.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-generator-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator

use crate::{
    Context, JsArgs, JsData, JsError, JsResult, JsString,
    builtins::iterable::create_iter_result_object,
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::{CONSTRUCTOR, JsObject},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
    value::JsValue,
    vm::{CallFrame, CallFrameFlags, CompletionRecord, GeneratorResumeKind},
};
use boa_gc::{Finalize, Trace, custom_trace};

use super::{BuiltInBuilder, IntrinsicObject};

/// Indicates the state of a generator.
#[derive(Debug, Finalize)]
pub(crate) enum GeneratorState {
    SuspendedStart {
        /// The `[[GeneratorContext]]` internal slot.
        context: GeneratorContext,
    },
    SuspendedYield {
        /// The `[[GeneratorContext]]` internal slot.
        context: GeneratorContext,
    },
    Executing,
    Completed,
}

// Need to manually implement, since `Trace` adds a `Drop` impl which disallows destructuring.
unsafe impl Trace for GeneratorState {
    custom_trace!(this, mark, {
        match &this {
            Self::SuspendedStart { context } | Self::SuspendedYield { context } => mark(context),
            Self::Executing | Self::Completed => {}
        }
    });
}

/// Holds all information that a generator needs to continue it's execution.
///
/// All of the fields must be changed with those that are currently present in the
/// context/vm before the generator execution starts/resumes and after it has ended/yielded.
#[derive(Debug, Trace, Finalize)]
pub(crate) struct GeneratorContext {
    pub(crate) stack: crate::vm::Stack,
    pub(crate) registers: Vec<JsValue>,
    pub(crate) call_frame: Option<CallFrame>,
}

impl GeneratorContext {
    /// Creates a new `GeneratorContext` from the current `Context` state.
    pub(crate) fn from_current(context: &Context, async_generator: Option<JsObject>) -> Self {
        let mut frame = context.vm_mut().frame().clone();
        frame.environments = context.vm_mut().frame.environments.clone();
        frame.realm = context.realm().clone();
        let stack = context.vm_mut().stack.split_off_frame(&frame);

        // Split off registers belonging to this frame from the shared register Vec.
        let register_start = frame.register_start as usize;
        let mut registers = context.vm_mut().registers.split_off(register_start);

        frame.rp = CallFrame::FUNCTION_PROLOGUE + frame.argument_count;
        // Set register_start to 0 — relative to the saved registers Vec.
        frame.register_start = 0;

        // NOTE: Since we get a pre-built call frame with registers, and we reuse them.
        //       So we don't need to allocate registers in subsequent calls.
        frame.flags |= CallFrameFlags::REGISTERS_ALREADY_PUSHED;

        if let Some(async_generator) = async_generator {
            registers[CallFrame::ASYNC_GENERATOR_OBJECT_REGISTER_INDEX] =
                async_generator.into();
        }

        Self {
            call_frame: Some(frame),
            stack,
            registers,
        }
    }

    /// Resumes execution with `GeneratorContext` as the current execution context.
    pub(crate) fn resume(
        &mut self,
        value: Option<JsValue>,
        resume_kind: GeneratorResumeKind,
        context: &Context,
    ) -> CompletionRecord {
        std::mem::swap(&mut context.vm_mut().stack, &mut self.stack);

        // Append saved registers to the Vm register Vec and record where they start.
        let register_start = context.vm_mut().registers.len() as u32;
        context
            .vm_mut()
            .registers
            .append(&mut self.registers);

        let mut frame = self.call_frame.take().expect("should have a call frame");
        let rp = frame.rp;
        frame.register_start = register_start;
        context.vm_mut().push_frame(frame);

        let frame = context.vm_mut().frame_mut();
        frame.rp = rp;
        frame.set_exit_early(true);

        if let Some(value) = value {
            context.vm_mut().stack.push(value);
        }
        context.vm_mut().stack.push(resume_kind);

        let result = context.run();

        // Split off the generator's registers back into our saved Vec.
        self.registers = context
            .vm_mut()
            .registers
            .split_off(register_start as usize);

        std::mem::swap(&mut context.vm_mut().stack, &mut self.stack);
        self.call_frame = context.vm_mut().pop_frame();
        assert!(self.call_frame.is_some());
        // Reset register_start to 0 for next resume.
        if let Some(frame) = &mut self.call_frame {
            frame.register_start = 0;
        }
        result
    }

    /// Returns the async generator object, if the function that this [`GeneratorContext`] is from an async generator, [`None`] otherwise.
    pub(crate) fn async_generator_object(&self) -> Option<JsObject> {
        let frame = self.call_frame.as_ref()?;
        if !frame.code_block().is_async_generator() {
            return None;
        }
        self.registers
            .get(CallFrame::ASYNC_GENERATOR_OBJECT_REGISTER_INDEX)
            .and_then(|v| v.as_object())
    }
}

/// The internal representation of a `Generator` object.
#[derive(Debug, Finalize, Trace, JsData)]
pub struct Generator {
    /// The `[[GeneratorState]]` internal slot.
    pub(crate) state: GeneratorState,
}

impl IntrinsicObject for Generator {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .iterator(),
            )
            .static_method(Self::next, js_string!("next"), 1)
            .static_method(Self::r#return, js_string!("return"), 1)
            .static_method(Self::throw, js_string!("throw"), 1)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .static_property(
                CONSTRUCTOR,
                realm
                    .intrinsics()
                    .constructors()
                    .generator_function()
                    .prototype(),
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().generator()
    }
}

impl Generator {
    const NAME: JsString = StaticJsStrings::GENERATOR;

    /// `Generator.prototype.next ( value )`
    ///
    /// The `next()` method returns an object with two properties done and value.
    /// You can also provide a parameter to the next method to send a value to the generator.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generator.prototype.next
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator/next
    pub(crate) fn next(this: &JsValue, args: &[JsValue], context: &Context) -> JsResult<JsValue> {
        // 1. Return ? GeneratorResume(this value, value, empty).
        Self::generator_resume(this, args.get_or_undefined(0).clone(), context)
    }

    /// `Generator.prototype.return ( value )`
    ///
    /// The `return()` method returns the given value and finishes the generator.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generator.prototype.return
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator/return
    pub(crate) fn r#return(
        this: &JsValue,
        args: &[JsValue],
        context: &Context,
    ) -> JsResult<JsValue> {
        // 1. Let g be the this value.
        // 2. Let C be Completion { [[Type]]: return, [[Value]]: value, [[Target]]: empty }.
        // 3. Return ? GeneratorResumeAbrupt(g, C, empty).
        Self::generator_resume_abrupt(this, Ok(args.get_or_undefined(0).clone()), context)
    }

    /// `Generator.prototype.throw ( exception )`
    ///
    /// The `throw()` method resumes the execution of a generator by throwing an error into it
    /// and returns an object with two properties done and value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generator.prototype.throw
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator/throw
    pub(crate) fn throw(this: &JsValue, args: &[JsValue], context: &Context) -> JsResult<JsValue> {
        // 1. Let g be the this value.
        // 2. Let C be ThrowCompletion(exception).
        // 3. Return ? GeneratorResumeAbrupt(g, C, empty).
        Self::generator_resume_abrupt(
            this,
            Err(JsError::from_opaque(args.get_or_undefined(0).clone())),
            context,
        )
    }

    /// `27.5.3.3 GeneratorResume ( generator, value, generatorBrand )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorresume
    pub(crate) fn generator_resume(
        r#gen: &JsValue,
        value: JsValue,
        context: &Context,
    ) -> JsResult<JsValue> {
        // 1. Let state be ? GeneratorValidate(generator, generatorBrand).
        let Some(generator_obj) = r#gen.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("Generator method called on non generator")
                .into());
        };
        let mut r#gen = generator_obj.downcast_mut::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("generator resumed on non generator object")
        })?;

        // 4. Let genContext be generator.[[GeneratorContext]].
        // 5. Let methodContext be the running execution context.
        // 6. Suspend methodContext.
        // 7. Set generator.[[GeneratorState]] to executing.
        let (mut generator_context, first_execution) =
            match std::mem::replace(&mut r#gen.state, GeneratorState::Executing) {
                GeneratorState::Executing => {
                    return Err(JsNativeError::typ()
                        .with_message("Generator should not be executing")
                        .into());
                }
                // 2. If state is completed, return CreateIterResultObject(undefined, true).
                GeneratorState::Completed => {
                    r#gen.state = GeneratorState::Completed;
                    return Ok(create_iter_result_object(
                        JsValue::undefined(),
                        true,
                        context,
                    ));
                }
                // 3. Assert: state is either suspendedStart or suspendedYield.
                GeneratorState::SuspendedStart { context } => (context, true),
                GeneratorState::SuspendedYield { context } => (context, false),
            };

        drop(r#gen);

        let record = generator_context.resume(
            (!first_execution).then_some(value),
            GeneratorResumeKind::Normal,
            context,
        );

        let mut r#gen = generator_obj
            .downcast_mut::<Self>()
            .expect("already checked this object type");

        // 8. Push genContext onto the execution context stack; genContext is now the running execution context.
        // 9. Resume the suspended evaluation of genContext using NormalCompletion(value) as the result of the operation that suspended it. Let result be the value returned by the resumed computation.
        // 10. Assert: When we return here, genContext has already been removed from the execution context stack and methodContext is the currently running execution context.
        // 11. Return Completion(result).
        match record {
            CompletionRecord::Return(value) => {
                r#gen.state = GeneratorState::SuspendedYield {
                    context: generator_context,
                };
                Ok(value)
            }
            CompletionRecord::Normal(value) => {
                r#gen.state = GeneratorState::Completed;
                Ok(create_iter_result_object(value, true, context))
            }
            CompletionRecord::Throw(err) => {
                r#gen.state = GeneratorState::Completed;
                Err(err)
            }
        }
    }

    /// `27.5.3.4 GeneratorResumeAbrupt ( generator, abruptCompletion, generatorBrand )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorresumeabrupt
    pub(crate) fn generator_resume_abrupt(
        r#gen: &JsValue,
        abrupt_completion: JsResult<JsValue>,
        context: &Context,
    ) -> JsResult<JsValue> {
        // 1. Let state be ? GeneratorValidate(generator, generatorBrand).
        let Some(generator_obj) = r#gen.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("Generator method called on non generator")
                .into());
        };
        let mut r#gen = generator_obj.downcast_mut::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("generator resumed on non generator object")
        })?;

        // 4. Assert: state is suspendedYield.
        // 5. Let genContext be generator.[[GeneratorContext]].
        // 6. Let methodContext be the running execution context.
        // 7. Suspend methodContext.
        // 8. Set generator.[[GeneratorState]] to executing.
        let mut generator_context =
            match std::mem::replace(&mut r#gen.state, GeneratorState::Executing) {
                GeneratorState::Executing => {
                    return Err(JsNativeError::typ()
                        .with_message("Generator should not be executing")
                        .into());
                }
                // 2. If state is suspendedStart, then
                // 3. If state is completed, then
                GeneratorState::SuspendedStart { .. } | GeneratorState::Completed => {
                    // a. Set generator.[[GeneratorState]] to completed.
                    r#gen.state = GeneratorState::Completed;

                    // b. Once a generator enters the completed state it never leaves it and its
                    // associated execution context is never resumed. Any execution state associated
                    // with generator can be discarded at this point.

                    // a. If abruptCompletion.[[Type]] is return, then
                    if let Ok(value) = abrupt_completion {
                        // i. Return CreateIterResultObject(abruptCompletion.[[Value]], true).
                        let value = create_iter_result_object(value, true, context);
                        return Ok(value);
                    }

                    // b. Return Completion(abruptCompletion).
                    return abrupt_completion;
                }
                GeneratorState::SuspendedYield { context } => context,
            };

        // 9. Push genContext onto the execution context stack; genContext is now the running execution context.
        // 10. Resume the suspended evaluation of genContext using abruptCompletion as the result of the operation that suspended it. Let result be the completion record returned by the resumed computation.
        // 11. Assert: When we return here, genContext has already been removed from the execution context stack and methodContext is the currently running execution context.
        // 12. Return Completion(result).
        drop(r#gen);

        let (value, resume_kind) = match abrupt_completion {
            Ok(value) => (value, GeneratorResumeKind::Return),
            Err(err) => (err.into_opaque(context)?, GeneratorResumeKind::Throw),
        };

        let record = generator_context.resume(Some(value), resume_kind, context);

        let mut r#gen = generator_obj.downcast_mut::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("generator resumed on non generator object")
        })?;

        match record {
            CompletionRecord::Return(value) => {
                r#gen.state = GeneratorState::SuspendedYield {
                    context: generator_context,
                };
                Ok(value)
            }
            CompletionRecord::Normal(value) => {
                r#gen.state = GeneratorState::Completed;
                Ok(create_iter_result_object(value, true, context))
            }
            CompletionRecord::Throw(err) => {
                r#gen.state = GeneratorState::Completed;
                Err(err)
            }
        }
    }
}
