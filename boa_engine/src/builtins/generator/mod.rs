//! Boa's 3tion of ECMAScript's global `Generator` object.
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
    builtins::iterable::create_iter_result_object,
    context::intrinsics::Intrinsics,
    environments::{BindingLocator, DeclarativeEnvironmentStack},
    error::JsNativeError,
    object::{JsObject, CONSTRUCTOR},
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
    value::JsValue,
    vm::{CallFrame, CompletionRecord, GeneratorResumeKind},
    Context, JsArgs, JsError, JsResult,
};
use boa_gc::{custom_trace, Finalize, Trace};
use boa_profiler::Profiler;

use super::{BuiltInBuilder, IntrinsicObject};

/// Indicates the state of a generator.
#[derive(Debug, Clone, Finalize)]
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
    custom_trace!(this, {
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
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct GeneratorContext {
    pub(crate) environments: DeclarativeEnvironmentStack,
    pub(crate) stack: Vec<JsValue>,
    pub(crate) active_function: Option<JsObject>,
    pub(crate) call_frame: Option<CallFrame>,
    pub(crate) realm: Realm,
    pub(crate) bindings_stack: Vec<BindingLocator>,
}

impl GeneratorContext {
    /// Creates a new `GeneratorContext` from the raw `Context` state components.
    pub(crate) fn new(
        environments: DeclarativeEnvironmentStack,
        stack: Vec<JsValue>,
        active_function: Option<JsObject>,
        call_frame: CallFrame,
        realm: Realm,
        bindings_stack: Vec<BindingLocator>,
    ) -> Self {
        Self {
            environments,
            stack,
            active_function,
            call_frame: Some(call_frame),
            realm,
            bindings_stack,
        }
    }

    /// Creates a new `GeneratorContext` from the current `Context` state.
    pub(crate) fn from_current(context: &mut Context<'_>) -> Self {
        Self {
            environments: context.vm.environments.clone(),
            call_frame: Some(context.vm.frame().clone()),
            stack: context.vm.stack.clone(),
            active_function: context.vm.active_function.clone(),
            realm: context.realm().clone(),
            bindings_stack: context.vm.bindings_stack.clone(),
        }
    }

    /// Resumes execution with `GeneratorContext` as the current execution context.
    pub(crate) fn resume(
        &mut self,
        value: Option<JsValue>,
        resume_kind: GeneratorResumeKind,
        context: &mut Context<'_>,
    ) -> CompletionRecord {
        std::mem::swap(&mut context.vm.environments, &mut self.environments);
        std::mem::swap(&mut context.vm.stack, &mut self.stack);
        std::mem::swap(&mut context.vm.active_function, &mut self.active_function);
        std::mem::swap(&mut context.vm.bindings_stack, &mut self.bindings_stack);
        context.swap_realm(&mut self.realm);
        context
            .vm
            .push_frame(self.call_frame.take().expect("should have a call frame"));
        context.vm.frame_mut().generator_resume_kind = resume_kind;
        if let Some(value) = value {
            context.vm.push(value);
        }

        let result = context.run();

        std::mem::swap(&mut context.vm.environments, &mut self.environments);
        std::mem::swap(&mut context.vm.stack, &mut self.stack);
        std::mem::swap(&mut context.vm.active_function, &mut self.active_function);
        std::mem::swap(&mut context.vm.bindings_stack, &mut self.bindings_stack);
        context.swap_realm(&mut self.realm);
        self.call_frame = context.vm.pop_frame();
        assert!(self.call_frame.is_some());
        result
    }
}

/// The internal representation of a `Generator` object.
#[derive(Debug, Finalize, Trace)]
pub struct Generator {
    /// The `[[GeneratorState]]` internal slot.
    pub(crate) state: GeneratorState,
}

impl IntrinsicObject for Generator {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .iterator(),
            )
            .static_method(Self::next, "next", 1)
            .static_method(Self::r#return, "return", 1)
            .static_method(Self::throw, "throw", 1)
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
    const NAME: &str = "Generator";

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
    pub(crate) fn next(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
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
        context: &mut Context<'_>,
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
    pub(crate) fn throw(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
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
        gen: &JsValue,
        value: JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let state be ? GeneratorValidate(generator, generatorBrand).
        let Some(generator_obj) = gen.as_object() else {
            return Err(
                JsNativeError::typ()
                    .with_message("Generator method called on non generator")
                    .into()
            );
        };
        let mut generator_obj_mut = generator_obj.borrow_mut();
        let Some(generator) = generator_obj_mut.as_generator_mut() else {
            return Err(
                JsNativeError::typ()
                    .with_message("generator resumed on non generator object")
                    .into()
            );
        };

        // 4. Let genContext be generator.[[GeneratorContext]].
        // 5. Let methodContext be the running execution context.
        // 6. Suspend methodContext.
        // 7. Set generator.[[GeneratorState]] to executing.
        let (mut generator_context, first_execution) =
            match std::mem::replace(&mut generator.state, GeneratorState::Executing) {
                GeneratorState::Executing => {
                    return Err(JsNativeError::typ()
                        .with_message("Generator should not be executing")
                        .into());
                }
                // 2. If state is completed, return CreateIterResultObject(undefined, true).
                GeneratorState::Completed => {
                    generator.state = GeneratorState::Completed;
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

        drop(generator_obj_mut);

        let record = generator_context.resume(
            (!first_execution).then_some(value),
            GeneratorResumeKind::Normal,
            context,
        );

        let mut generator_obj_mut = generator_obj.borrow_mut();
        let generator = generator_obj_mut
            .as_generator_mut()
            .expect("already checked this object type");

        // 8. Push genContext onto the execution context stack; genContext is now the running execution context.
        // 9. Resume the suspended evaluation of genContext using NormalCompletion(value) as the result of the operation that suspended it. Let result be the value returned by the resumed computation.
        // 10. Assert: When we return here, genContext has already been removed from the execution context stack and methodContext is the currently running execution context.
        // 11. Return Completion(result).
        match record {
            CompletionRecord::Return(value) => {
                generator.state = GeneratorState::SuspendedYield {
                    context: generator_context,
                };
                Ok(value)
            }
            CompletionRecord::Normal(value) => {
                generator.state = GeneratorState::Completed;
                Ok(create_iter_result_object(value, true, context))
            }
            CompletionRecord::Throw(err) => {
                generator.state = GeneratorState::Completed;
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
        gen: &JsValue,
        abrupt_completion: JsResult<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let state be ? GeneratorValidate(generator, generatorBrand).
        let Some(generator_obj) = gen.as_object() else {
            return Err(
                JsNativeError::typ()
                    .with_message("Generator method called on non generator")
                    .into()
            );
        };
        let mut generator_obj_mut = generator_obj.borrow_mut();
        let Some(generator) = generator_obj_mut.as_generator_mut() else {
            return Err(
                JsNativeError::typ()
                    .with_message("generator resumed on non generator object")
                    .into()
            );
        };

        // 4. Assert: state is suspendedYield.
        // 5. Let genContext be generator.[[GeneratorContext]].
        // 6. Let methodContext be the running execution context.
        // 7. Suspend methodContext.
        // 8. Set generator.[[GeneratorState]] to executing.
        let mut generator_context =
            match std::mem::replace(&mut generator.state, GeneratorState::Executing) {
                GeneratorState::Executing => {
                    return Err(JsNativeError::typ()
                        .with_message("Generator should not be executing")
                        .into());
                }
                // 2. If state is suspendedStart, then
                // 3. If state is completed, then
                GeneratorState::SuspendedStart { .. } | GeneratorState::Completed => {
                    // a. Set generator.[[GeneratorState]] to completed.
                    generator.state = GeneratorState::Completed;

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
        drop(generator_obj_mut);

        let (value, resume_kind) = match abrupt_completion {
            Ok(value) => (value, GeneratorResumeKind::Return),
            Err(err) => (err.to_opaque(context), GeneratorResumeKind::Throw),
        };

        let record = generator_context.resume(Some(value), resume_kind, context);

        let mut generator_obj_mut = generator_obj.borrow_mut();
        let generator = generator_obj_mut
            .as_generator_mut()
            .expect("already checked this object type");

        match record {
            CompletionRecord::Return(value) => {
                generator.state = GeneratorState::SuspendedYield {
                    context: generator_context,
                };
                Ok(value)
            }
            CompletionRecord::Normal(value) => {
                generator.state = GeneratorState::Completed;
                Ok(create_iter_result_object(value, true, context))
            }
            CompletionRecord::Throw(err) => {
                generator.state = GeneratorState::Completed;
                Err(err)
            }
        }
    }
}
