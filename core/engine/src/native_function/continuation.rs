use boa_gc::{Finalize, Gc, Trace};

use crate::{Context, JsResult, JsValue};

#[derive(Trace, Finalize)]
#[boa_gc(unsafe_no_drop)]
pub(crate) enum CoroutineState {
    Yielded(JsValue),
    Done,
}

trait TraceableCoroutine: Trace {
    fn call(&self, value: JsResult<JsValue>, context: &mut Context) -> JsResult<CoroutineState>;
}

#[derive(Trace, Finalize)]
struct Coroutine<F, T>
where
    F: Fn(JsResult<JsValue>, &T, &mut Context) -> JsResult<CoroutineState>,
    T: Trace,
{
    // SAFETY: `NativeCoroutine`'s safe API ensures only `Copy` closures are stored; its unsafe API,
    // on the other hand, explains the invariants to hold in order for this to be safe, shifting
    // the responsibility to the caller.
    #[unsafe_ignore_trace]
    f: F,
    captures: T,
}

impl<F, T> TraceableCoroutine for Coroutine<F, T>
where
    F: Fn(JsResult<JsValue>, &T, &mut Context) -> JsResult<CoroutineState>,
    T: Trace,
{
    fn call(&self, result: JsResult<JsValue>, context: &mut Context) -> JsResult<CoroutineState> {
        (self.f)(result, &self.captures, context)
    }
}

/// A callable Rust coroutine that can be used to await promises.
///
/// # Caveats
///
/// By limitations of the Rust language, the garbage collector currently cannot inspect closures
/// in order to trace their captured variables. This means that only [`Copy`] closures are 100% safe
/// to use. All other closures can also be stored in a `NativeCoroutine`, albeit by using an `unsafe`
/// API, but note that passing closures implicitly capturing traceable types could cause
/// **Undefined Behaviour**.
#[derive(Clone, Trace, Finalize)]
pub(crate) struct NativeCoroutine {
    inner: Gc<dyn TraceableCoroutine>,
}

impl std::fmt::Debug for NativeCoroutine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeCoroutine").finish_non_exhaustive()
    }
}

impl NativeCoroutine {
    /// Creates a `NativeCoroutine` from a `Copy` closure and a list of traceable captures.
    pub(crate) fn from_copy_closure_with_captures<F, T>(closure: F, captures: T) -> Self
    where
        F: Fn(JsResult<JsValue>, &T, &mut Context) -> JsResult<CoroutineState> + Copy + 'static,
        T: Trace + 'static,
    {
        // SAFETY: The `Copy` bound ensures there are no traceable types inside the closure.
        unsafe { Self::from_closure_with_captures(closure, captures) }
    }

    /// Create a new `NativeCoroutine` from a closure and a list of traceable captures.
    ///
    /// # Safety
    ///
    /// Passing a closure that contains a captured variable that needs to be traced by the garbage
    /// collector could cause an use after free, memory corruption or other kinds of **Undefined
    /// Behaviour**. See <https://github.com/Manishearth/rust-gc/issues/50> for a technical explanation
    /// on why that is the case.
    pub(crate) unsafe fn from_closure_with_captures<F, T>(closure: F, captures: T) -> Self
    where
        F: Fn(JsResult<JsValue>, &T, &mut Context) -> JsResult<CoroutineState> + 'static,
        T: Trace + 'static,
    {
        // Hopefully, this unsafe operation will be replaced by the `CoerceUnsized` API in the
        // future: https://github.com/rust-lang/rust/issues/18598
        let ptr = Gc::into_raw(Gc::new(Coroutine {
            f: closure,
            captures,
        }));
        // SAFETY: The pointer returned by `into_raw` is only used to coerce to a trait object,
        // meaning this is safe.
        unsafe {
            Self {
                inner: Gc::from_raw(ptr),
            }
        }
    }

    /// Calls this `NativeCoroutine`, forwarding the arguments to the corresponding function.
    #[inline]
    pub(crate) fn call(
        &self,
        result: JsResult<JsValue>,
        context: &mut Context,
    ) -> JsResult<CoroutineState> {
        self.inner.call(result, context)
    }
}
