//! This module implements the global `Generator` object.
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
    builtins::{iterable::create_iter_result_object, BuiltIn, JsArgs},
    environments::DeclarativeEnvironmentStack,
    object::{ConstructorBuilder, JsObject, ObjectData},
    property::{Attribute, PropertyDescriptor},
    symbol::WellKnownSymbols,
    value::JsValue,
    vm::{CallFrame, GeneratorResumeKind, ReturnType},
    Context, JsResult,
};
use boa_gc::{Cell, Finalize, Gc, Trace};
use boa_profiler::Profiler;

/// Indicates the state of a generator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum GeneratorState {
    Undefined,
    SuspendedStart,
    SuspendedYield,
    Executing,
    Completed,
}

/// Holds all information that a generator needs to continue it's execution.
///
/// All of the fields must be changed with those that are currently present in the
/// context/vm before the generator execution starts/resumes and after it has ended/yielded.
#[derive(Debug, Clone, Finalize, Trace)]
pub(crate) struct GeneratorContext {
    pub(crate) environments: DeclarativeEnvironmentStack,
    pub(crate) call_frame: CallFrame,
    pub(crate) stack: Vec<JsValue>,
}

/// The internal representation on a `Generator` object.
#[derive(Debug, Clone, Finalize, Trace)]
pub struct Generator {
    /// The `[[GeneratorState]]` internal slot.
    #[unsafe_ignore_trace]
    pub(crate) state: GeneratorState,

    /// The `[[GeneratorContext]]` internal slot.
    pub(crate) context: Option<Gc<Cell<GeneratorContext>>>,
}

impl BuiltIn for Generator {
    const NAME: &'static str = "Generator";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let iterator_prototype = context
            .intrinsics()
            .objects()
            .iterator_prototypes()
            .iterator_prototype();

        let generator_function_prototype = context
            .intrinsics()
            .constructors()
            .generator_function()
            .prototype();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().generator().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .property(
            WellKnownSymbols::to_string_tag(),
            Self::NAME,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .method(Self::next, "next", 1)
        .method(Self::r#return, "return", 1)
        .method(Self::throw, "throw", 1)
        .inherit(iterator_prototype)
        .build();

        context
            .intrinsics()
            .constructors()
            .generator()
            .prototype
            .insert_property(
                "constructor",
                PropertyDescriptor::builder()
                    .value(generator_function_prototype)
                    .writable(false)
                    .enumerable(false)
                    .configurable(true),
            );

        None
    }
}

impl Generator {
    pub(crate) const LENGTH: usize = 0;

    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn constructor(
        _: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = context.intrinsics().constructors().generator().prototype();

        let this = JsObject::from_proto_and_data(
            prototype,
            ObjectData::generator(Self {
                state: GeneratorState::Undefined,
                context: None,
            }),
        );

        Ok(this.into())
    }

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
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Return ? GeneratorResume(this value, value, empty).
        match this.as_object() {
            Some(obj) if obj.is_generator() => {
                Self::generator_resume(obj, args.get_or_undefined(0), context)
            }
            _ => context.throw_type_error("Generator.prototype.next called on non generator"),
        }
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
        context: &mut Context,
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
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let g be the this value.
        // 2. Let C be ThrowCompletion(exception).
        // 3. Return ? GeneratorResumeAbrupt(g, C, empty).
        Self::generator_resume_abrupt(this, Err(args.get_or_undefined(0).clone()), context)
    }

    /// `27.5.3.3 GeneratorResume ( generator, value, generatorBrand )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorresume
    pub(crate) fn generator_resume(
        generator_obj: &JsObject,
        value: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let state be ? GeneratorValidate(generator, generatorBrand).
        let mut generator_obj_mut = generator_obj.borrow_mut();
        let generator = generator_obj_mut.as_generator_mut().ok_or_else(|| {
            context.construct_type_error("generator resumed on non generator object")
        })?;
        let state = generator.state;

        if state == GeneratorState::Executing {
            return Err(context.construct_type_error("Generator should not be executing"));
        }

        // 2. If state is completed, return CreateIterResultObject(undefined, true).
        if state == GeneratorState::Completed {
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }

        // 3. Assert: state is either suspendedStart or suspendedYield.
        assert!(matches!(
            state,
            GeneratorState::SuspendedStart | GeneratorState::SuspendedYield
        ));

        // 4. Let genContext be generator.[[GeneratorContext]].
        // 5. Let methodContext be the running execution context.
        // 6. Suspend methodContext.
        // 7. Set generator.[[GeneratorState]] to executing.
        generator.state = GeneratorState::Executing;
        let first_execution = matches!(state, GeneratorState::SuspendedStart);

        let generator_context_cell = generator
            .context
            .take()
            .expect("generator context cannot be empty here");
        let mut generator_context = generator_context_cell.borrow_mut();
        drop(generator_obj_mut);

        std::mem::swap(
            &mut context.realm.environments,
            &mut generator_context.environments,
        );
        std::mem::swap(&mut context.vm.stack, &mut generator_context.stack);
        context.vm.push_frame(generator_context.call_frame.clone());
        if !first_execution {
            context.vm.push(value);
        }

        context.vm.frame_mut().generator_resume_kind = GeneratorResumeKind::Normal;

        let result = context.run();

        generator_context.call_frame = *context
            .vm
            .pop_frame()
            .expect("generator call frame must exist");
        std::mem::swap(
            &mut context.realm.environments,
            &mut generator_context.environments,
        );
        std::mem::swap(&mut context.vm.stack, &mut generator_context.stack);

        let mut generator_obj_mut = generator_obj.borrow_mut();
        let generator = generator_obj_mut
            .as_generator_mut()
            .expect("already checked this object type");

        match result {
            Ok((value, ReturnType::Yield)) => {
                generator.state = GeneratorState::SuspendedYield;
                drop(generator_context);
                generator.context = Some(generator_context_cell);
                Ok(create_iter_result_object(value, false, context))
            }
            Ok((value, _)) => {
                generator.state = GeneratorState::Completed;
                Ok(create_iter_result_object(value, true, context))
            }
            Err(value) => {
                generator.state = GeneratorState::Completed;
                Err(value)
            }
        }

        // 8. Push genContext onto the execution context stack; genContext is now the running execution context.
        // 9. Resume the suspended evaluation of genContext using NormalCompletion(value) as the result of the operation that suspended it. Let result be the value returned by the resumed computation.
        // 10. Assert: When we return here, genContext has already been removed from the execution context stack and methodContext is the currently running execution context.
        // 11. Return Completion(result).
    }

    /// `27.5.3.4 GeneratorResumeAbrupt ( generator, abruptCompletion, generatorBrand )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorresumeabrupt
    pub(crate) fn generator_resume_abrupt(
        this: &JsValue,
        abrupt_completion: JsResult<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let state be ? GeneratorValidate(generator, generatorBrand).
        let generator_obj = this.as_object().ok_or_else(|| {
            context.construct_type_error("generator resumed on non generator object")
        })?;
        let mut generator_obj_mut = generator_obj.borrow_mut();
        let generator = generator_obj_mut.as_generator_mut().ok_or_else(|| {
            context.construct_type_error("generator resumed on non generator object")
        })?;
        let mut state = generator.state;

        if state == GeneratorState::Executing {
            return Err(context.construct_type_error("Generator should not be executing"));
        }

        // 2. If state is suspendedStart, then
        if state == GeneratorState::SuspendedStart {
            // a. Set generator.[[GeneratorState]] to completed.
            generator.state = GeneratorState::Completed;
            // b. Once a generator enters the completed state it never leaves it and its associated execution context is never resumed. Any execution state associated with generator can be discarded at this point.
            generator.context = None;
            // c. Set state to completed.
            state = GeneratorState::Completed;
        }

        // 3. If state is completed, then
        if state == GeneratorState::Completed {
            // a. If abruptCompletion.[[Type]] is return, then
            if let Ok(value) = abrupt_completion {
                // i. Return CreateIterResultObject(abruptCompletion.[[Value]], true).
                return Ok(create_iter_result_object(value, true, context));
            }
            // b. Return Completion(abruptCompletion).
            return abrupt_completion;
        }

        // 4. Assert: state is suspendedYield.
        // 5. Let genContext be generator.[[GeneratorContext]].
        // 6. Let methodContext be the running execution context.
        // 7. Suspend methodContext.
        // 8. Set generator.[[GeneratorState]] to executing.
        // 9. Push genContext onto the execution context stack; genContext is now the running execution context.
        // 10. Resume the suspended evaluation of genContext using abruptCompletion as the result of the operation that suspended it. Let result be the completion record returned by the resumed computation.
        // 11. Assert: When we return here, genContext has already been removed from the execution context stack and methodContext is the currently running execution context.
        // 12. Return Completion(result).
        let generator_context_cell = generator
            .context
            .take()
            .expect("generator context cannot be empty here");
        let mut generator_context = generator_context_cell.borrow_mut();

        generator.state = GeneratorState::Executing;
        drop(generator_obj_mut);

        std::mem::swap(
            &mut context.realm.environments,
            &mut generator_context.environments,
        );
        std::mem::swap(&mut context.vm.stack, &mut generator_context.stack);
        context.vm.push_frame(generator_context.call_frame.clone());

        let result = match abrupt_completion {
            Ok(value) => {
                context.vm.push(value);
                context.vm.frame_mut().generator_resume_kind = GeneratorResumeKind::Return;
                context.run()
            }
            Err(value) => {
                context.vm.push(value);
                context.vm.frame_mut().generator_resume_kind = GeneratorResumeKind::Throw;
                context.run()
            }
        };
        generator_context.call_frame = *context
            .vm
            .pop_frame()
            .expect("generator call frame must exist");
        std::mem::swap(
            &mut context.realm.environments,
            &mut generator_context.environments,
        );
        std::mem::swap(&mut context.vm.stack, &mut generator_context.stack);

        let mut generator_obj_mut = generator_obj.borrow_mut();
        let generator = generator_obj_mut
            .as_generator_mut()
            .expect("already checked this object type");

        match result {
            Ok((value, ReturnType::Yield)) => {
                generator.state = GeneratorState::SuspendedYield;
                drop(generator_context);
                generator.context = Some(generator_context_cell);
                Ok(create_iter_result_object(value, false, context))
            }
            Ok((value, _)) => {
                generator.state = GeneratorState::Completed;
                Ok(create_iter_result_object(value, true, context))
            }
            Err(value) => {
                generator.state = GeneratorState::Completed;
                Err(value)
            }
        }
    }
}
