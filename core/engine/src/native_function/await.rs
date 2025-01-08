use boa_gc::{Finalize, Gc, Trace};

use crate::{Context, JsResult, JsValue};

trait TraceableContinuation: Trace {
    fn call(&self, value: JsResult<JsValue>, context: &mut Context);
}

#[derive(Trace, Finalize)]
struct Continuation<F, T>
where
    F: Fn(JsResult<JsValue>, &T, &mut Context),
    T: Trace,
{
    // SAFETY: `NativeFunction`'s safe API ensures only `Copy` closures are stored; its unsafe API,
    // on the other hand, explains the invariants to hold in order for this to be safe, shifting
    // the responsibility to the caller.
    #[unsafe_ignore_trace]
    f: F,
    captures: T,
}

impl<F, T> TraceableContinuation for Continuation<F, T>
where
    F: Fn(JsResult<JsValue>, &T, &mut Context),
    T: Trace,
{
    fn call(&self, result: JsResult<JsValue>, context: &mut Context) {
        (self.f)(result, &self.captures, context)
    }
}

/// A callable Rust continuation that can be used to await promises.
///
/// # Caveats
///
/// By limitations of the Rust language, the garbage collector currently cannot inspect closures
/// in order to trace their captured variables. This means that only [`Copy`] closures are 100% safe
/// to use. All other closures can also be stored in a `NativeContinuation`, albeit by using an `unsafe`
/// API, but note that passing closures implicitly capturing traceable types could cause
/// **Undefined Behaviour**.
#[derive(Trace, Finalize)]
pub(crate) struct NativeContinuation {
    inner: Gc<dyn TraceableContinuation>,
}

impl std::fmt::Debug for NativeContinuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeContinuation").finish_non_exhaustive()
    }
}

impl NativeContinuation {
    /// Creates a `NativeFunction` from a `Copy` closure.
    pub(crate) fn from_copy_closure<F>(closure: F) -> Self
    where
        F: Fn(JsResult<JsValue>, &mut Context) + Copy + 'static,
    {
        // SAFETY: The `Copy` bound ensures there are no traceable types inside the closure.
        unsafe { Self::from_closure(closure) }
    }

    /// Creates a `NativeFunction` from a `Copy` closure and a list of traceable captures.
    pub(crate) fn from_copy_closure_with_captures<F, T>(closure: F, captures: T) -> Self
    where
        F: Fn(JsResult<JsValue>, &T, &mut Context) + Copy + 'static,
        T: Trace + 'static,
    {
        // SAFETY: The `Copy` bound ensures there are no traceable types inside the closure.
        unsafe { Self::from_closure_with_captures(closure, captures) }
    }

    /// Creates a new `NativeFunction` from a closure.
    ///
    /// # Safety
    ///
    /// Passing a closure that contains a captured variable that needs to be traced by the garbage
    /// collector could cause an use after free, memory corruption or other kinds of **Undefined
    /// Behaviour**. See <https://github.com/Manishearth/rust-gc/issues/50> for a technical explanation
    /// on why that is the case.
    pub(crate) unsafe fn from_closure<F>(closure: F) -> Self
    where
        F: Fn(JsResult<JsValue>, &mut Context) + 'static,
    {
        // SAFETY: The caller must ensure the invariants of the closure hold.
        unsafe {
            Self::from_closure_with_captures(
                move |result, (), context| closure(result, context),
                (),
            )
        }
    }

    /// Create a new `NativeFunction` from a closure and a list of traceable captures.
    ///
    /// # Safety
    ///
    /// Passing a closure that contains a captured variable that needs to be traced by the garbage
    /// collector could cause an use after free, memory corruption or other kinds of **Undefined
    /// Behaviour**. See <https://github.com/Manishearth/rust-gc/issues/50> for a technical explanation
    /// on why that is the case.
    pub(crate) unsafe fn from_closure_with_captures<F, T>(closure: F, captures: T) -> Self
    where
        F: Fn(JsResult<JsValue>, &T, &mut Context) + 'static,
        T: Trace + 'static,
    {
        // Hopefully, this unsafe operation will be replaced by the `CoerceUnsized` API in the
        // future: https://github.com/rust-lang/rust/issues/18598
        let ptr = Gc::into_raw(Gc::new(Continuation {
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

    /// Calls this `NativeFunction`, forwarding the arguments to the corresponding function.
    #[inline]
    pub(crate) fn call(&self, result: JsResult<JsValue>, context: &mut Context) {
        self.inner.call(result, context)
    }
}
