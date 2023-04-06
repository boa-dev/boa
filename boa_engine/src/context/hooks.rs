use crate::{
    builtins::promise::OperationType,
    job::JobCallback,
    object::{JsFunction, JsObject},
    realm::Realm,
    Context, JsResult, JsValue,
};

use super::intrinsics::Intrinsics;

/// [`Host Hooks`] customizable by the host code or engine.
///
/// Every hook contains on its `Requirements` section the spec requirements
/// that the hook must abide to for spec compliance.
///
/// # Usage
///
/// Implement the trait for a custom struct (maybe with additional state), overriding the methods that
/// need to be redefined:
///
/// ```
/// use boa_engine::{JsNativeError, JsResult, context::{Context, ContextBuilder, HostHooks}, Source};
///
/// struct Hooks;
///
/// impl HostHooks for Hooks {
///     fn ensure_can_compile_strings(
///         &self,
///         context: &mut Context<'_>,
///     ) -> JsResult<()> {
///         Err(JsNativeError::typ().with_message("eval calls not available").into())
///     }
/// }
/// let hooks = Hooks; // Can have additional state.
/// let context = &mut ContextBuilder::new().host_hooks(&hooks).build().unwrap();
/// let result = context.eval_script(Source::from_bytes(r#"eval("let a = 5")"#));
/// assert_eq!(result.unwrap_err().to_string(), "TypeError: eval calls not available");
/// ```
///
/// [`Host Hooks`]: https://tc39.es/ecma262/#sec-host-hooks-summary
pub trait HostHooks {
    /// [`HostMakeJobCallback ( callback )`][spec]
    ///
    /// # Requirements
    ///
    /// - It must return a `JobCallback` Record whose `[[Callback]]` field is `callback`.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hostmakejobcallback
    fn make_job_callback(&self, callback: JsFunction, _context: &mut Context<'_>) -> JobCallback {
        // The default implementation of HostMakeJobCallback performs the following steps when called:

        // 1. Return the JobCallback Record { [[Callback]]: callback, [[HostDefined]]: empty }.
        JobCallback::new(callback, ())
    }

    /// [`HostCallJobCallback ( jobCallback, V, argumentsList )`][spec]
    ///
    /// # Requirements
    ///
    /// - It must perform and return the result of `Call(jobCallback.[[Callback]], V, argumentsList)`.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hostcalljobcallback
    fn call_job_callback(
        &self,
        job: JobCallback,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // The default implementation of HostCallJobCallback performs the following steps when called:

        // 1. Assert: IsCallable(jobCallback.[[Callback]]) is true.
        // already asserted by `Call`.
        // 2. Return ? Call(jobCallback.[[Callback]], V, argumentsList).
        job.callback().call(this, args, context)
    }

    /// [`HostPromiseRejectionTracker ( promise, operation )`][spec]
    ///
    /// # Requirements
    ///
    /// - It must complete normally (i.e. not return an abrupt completion). This is already
    /// ensured by the return type.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-host-promise-rejection-tracker
    fn promise_rejection_tracker(
        &self,
        _promise: &JsObject,
        _operation: OperationType,
        _context: &mut Context<'_>,
    ) {
        // The default implementation of HostPromiseRejectionTracker is to return unused.
    }

    /// [`HostEnsureCanCompileStrings ( calleeRealm )`][spec]
    ///
    /// # Requirements
    ///
    /// - If the returned Completion Record is a normal completion, it must be a normal completion
    /// containing unused. This is already ensured by the return type.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hostensurecancompilestrings
    // TODO: Track https://github.com/tc39/ecma262/issues/938
    fn ensure_can_compile_strings(
        &self,
        _realm: Realm,
        _context: &mut Context<'_>,
    ) -> JsResult<()> {
        // The default implementation of HostEnsureCanCompileStrings is to return NormalCompletion(unused).
        Ok(())
    }

    /// [`HostHasSourceTextAvailable ( func )`][spec]
    ///
    /// # Requirements
    ///
    /// - It must be deterministic with respect to its parameters. Each time it is called with a
    /// specific `func` as its argument, it must return the same result.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hosthassourcetextavailable
    fn has_source_text_available(
        &self,
        _function: &JsFunction,
        _context: &mut Context<'_>,
    ) -> bool {
        // The default implementation of HostHasSourceTextAvailable is to return true.
        true
    }

    /// [`HostEnsureCanAddPrivateElement ( O )`][spec]
    ///
    /// # Requirements
    ///
    /// - If `O` is not a host-defined exotic object, this abstract operation must return
    /// `NormalCompletion(unused)` and perform no other steps.
    /// - Any two calls of this abstract operation with the same argument must return the same kind
    /// of *Completion Record*.
    /// - This abstract operation should only be overriden by ECMAScript hosts that are web browsers.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hostensurecanaddprivateelement
    fn ensure_can_add_private_element(
        &self,
        _o: &JsObject,
        _context: &mut Context<'_>,
    ) -> JsResult<()> {
        Ok(())
    }

    /// Creates the global object of a new [`Context`] from the initial intrinsics.
    ///
    /// Equivalent to the step 7 of [`InitializeHostDefinedRealm ( )`][ihdr].
    ///
    /// [ihdr]: https://tc39.es/ecma262/#sec-initializehostdefinedrealm
    fn create_global_object(&self, intrinsics: &Intrinsics) -> JsObject {
        JsObject::with_object_proto(intrinsics)
    }

    /// Creates the global `this` of a new [`Context`] from the initial intrinsics.
    ///
    /// Equivalent to the step 8 of [`InitializeHostDefinedRealm ( )`][ihdr].
    ///
    /// [ihdr]: https://tc39.es/ecma262/#sec-initializehostdefinedrealm
    fn create_global_this(&self, _intrinsics: &Intrinsics) -> Option<JsObject> {
        None
    }
}

/// Default implementation of [`HostHooks`], which doesn't carry any state.
#[derive(Debug, Clone, Copy)]
pub struct DefaultHooks;

impl HostHooks for DefaultHooks {}
