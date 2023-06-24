//! Boa's wrappers for native Rust functions to be compatible with ECMAScript calls.
//!
//! [`NativeFunction`] is the main type of this module, providing APIs to create native callables
//! from native Rust functions and closures.

use boa_gc::{custom_trace, Finalize, Gc, Trace};

use crate::{object::JsPromise, Context, JsResult, JsValue};

/// The required signature for all native built-in function pointers.
///
/// # Arguments
///
/// - The first argument represents the `this` variable of every ECMAScript function.
///
/// - The second argument represents the list of all arguments passed to the function.
///
/// - The last argument is the engine [`Context`].
pub type NativeFunctionPointer = fn(&JsValue, &[JsValue], &mut Context<'_>) -> JsResult<JsValue>;

trait TraceableClosure: Trace {
    fn call(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
}

#[derive(Trace, Finalize)]
struct Closure<F, T>
where
    F: Fn(&JsValue, &[JsValue], &T, &mut Context<'_>) -> JsResult<JsValue>,
    T: Trace,
{
    // SAFETY: `NativeFunction`'s safe API ensures only `Copy` closures are stored; its unsafe API,
    // on the other hand, explains the invariants to hold in order for this to be safe, shifting
    // the responsibility to the caller.
    #[unsafe_ignore_trace]
    f: F,
    captures: T,
}

impl<F, T> TraceableClosure for Closure<F, T>
where
    F: Fn(&JsValue, &[JsValue], &T, &mut Context<'_>) -> JsResult<JsValue>,
    T: Trace,
{
    fn call(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        (self.f)(this, args, &self.captures, context)
    }
}

/// A callable Rust function that can be invoked by the engine.
///
/// `NativeFunction` functions are divided in two:
/// - Function pointers a.k.a common functions (see [`NativeFunctionPointer`]).
/// - Closure functions that can capture the current environment.
///
/// # Caveats
///
/// By limitations of the Rust language, the garbage collector currently cannot inspect closures
/// in order to trace their captured variables. This means that only [`Copy`] closures are 100% safe
/// to use. All other closures can also be stored in a `NativeFunction`, albeit by using an `unsafe`
/// API, but note that passing closures implicitly capturing traceable types could cause
/// **Undefined Behaviour**.
#[derive(Clone)]
pub struct NativeFunction {
    inner: Inner,
}

#[derive(Clone)]
enum Inner {
    PointerFn(NativeFunctionPointer),
    Closure(Gc<dyn TraceableClosure>),
}

impl Finalize for NativeFunction {
    fn finalize(&self) {
        if let Inner::Closure(c) = &self.inner {
            c.finalize();
        }
    }
}

// Manual implementation because deriving `Trace` triggers the `single_use_lifetimes` lint.
// SAFETY: Only closures can contain `Trace` captures, so this implementation is safe.
unsafe impl Trace for NativeFunction {
    custom_trace!(this, {
        if let Inner::Closure(c) = &this.inner {
            mark(c);
        }
    });
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeFunction").finish_non_exhaustive()
    }
}

impl NativeFunction {
    /// Creates a `NativeFunction` from a function pointer.
    #[inline]
    pub fn from_fn_ptr(function: NativeFunctionPointer) -> Self {
        Self {
            inner: Inner::PointerFn(function),
        }
    }

    /// Creates a `NativeFunction` from a function returning a [`Future`]-like.
    ///
    /// The returned `NativeFunction` will return an ECMAScript `Promise` that will be fulfilled
    /// or rejected when the returned [`Future`] completes.
    ///
    /// If you only need to convert a [`Future`]-like into a [`JsPromise`], see
    /// [`JsPromise::from_future`].
    ///
    /// # Caveats
    ///
    /// Certain async functions need to be desugared for them to be `'static'`. For example, the
    /// following won't compile:
    ///
    /// ```compile_fail
    /// # use boa_engine::{
    /// #   JsValue,
    /// #   Context,
    /// #   JsResult,
    /// #   NativeFunction
    /// # };
    /// async fn test(
    ///     _this: &JsValue,
    ///     args: &[JsValue],
    ///     _context: &mut Context<'_>,
    /// ) -> JsResult<JsValue> {
    ///     let arg = args.get(0).cloned();
    ///     std::future::ready(()).await;
    ///     drop(arg);
    ///     Ok(JsValue::null())
    /// }
    /// NativeFunction::from_async_fn(test);
    /// ```
    ///
    /// Even though `args` is only used before the first await point, Rust's async functions are
    /// fully lazy, which makes `test` equivalent to something like:
    ///
    /// ```
    /// # use std::future::Future;
    /// # use boa_engine::{JsValue, Context, JsResult};
    /// fn test<'a>(
    ///     _this: &JsValue,
    ///     args: &'a [JsValue],
    ///     _context: &mut Context<'_>,
    /// ) -> impl Future<Output = JsResult<JsValue>> + 'a {
    ///     async move {
    ///         let arg = args.get(0).cloned();
    ///         std::future::ready(()).await;
    ///         drop(arg);
    ///         Ok(JsValue::null())
    ///     }
    /// }
    /// ```
    ///
    /// Note that `args` is used inside the `async move`, making the whole future not `'static`.
    ///
    /// In those cases, you can manually restrict the lifetime of the arguments:
    ///
    /// ```
    /// # use std::future::Future;
    /// # use boa_engine::{
    /// #   JsValue,
    /// #   Context,
    /// #   JsResult,
    /// #   NativeFunction
    /// # };
    /// fn test(
    ///     _this: &JsValue,
    ///     args: &[JsValue],
    ///     _context: &mut Context<'_>,
    /// ) -> impl Future<Output = JsResult<JsValue>> {
    ///     let arg = args.get(0).cloned();
    ///     async move {
    ///         std::future::ready(()).await;
    ///         drop(arg);
    ///         Ok(JsValue::null())
    ///     }
    /// }
    /// NativeFunction::from_async_fn(test);
    /// ```
    ///
    /// And this should always return a `'static` future.
    ///
    /// [`Future`]: std::future::Future
    pub fn from_async_fn<Fut>(f: fn(&JsValue, &[JsValue], &mut Context<'_>) -> Fut) -> Self
    where
        Fut: std::future::IntoFuture<Output = JsResult<JsValue>> + 'static,
    {
        Self::from_copy_closure(move |this, args, context| {
            let future = f(this, args, context);

            Ok(JsPromise::from_future(future, context).into())
        })
    }

    /// Creates a `NativeFunction` from a `Copy` closure.
    pub fn from_copy_closure<F>(closure: F) -> Self
    where
        F: Fn(&JsValue, &[JsValue], &mut Context<'_>) -> JsResult<JsValue> + Copy + 'static,
    {
        // SAFETY: The `Copy` bound ensures there are no traceable types inside the closure.
        unsafe { Self::from_closure(closure) }
    }

    /// Creates a `NativeFunction` from a `Copy` closure and a list of traceable captures.
    pub fn from_copy_closure_with_captures<F, T>(closure: F, captures: T) -> Self
    where
        F: Fn(&JsValue, &[JsValue], &T, &mut Context<'_>) -> JsResult<JsValue> + Copy + 'static,
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
    pub unsafe fn from_closure<F>(closure: F) -> Self
    where
        F: Fn(&JsValue, &[JsValue], &mut Context<'_>) -> JsResult<JsValue> + 'static,
    {
        // SAFETY: The caller must ensure the invariants of the closure hold.
        unsafe {
            Self::from_closure_with_captures(
                move |this, args, _, context| closure(this, args, context),
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
    pub unsafe fn from_closure_with_captures<F, T>(closure: F, captures: T) -> Self
    where
        F: Fn(&JsValue, &[JsValue], &T, &mut Context<'_>) -> JsResult<JsValue> + 'static,
        T: Trace + 'static,
    {
        // Hopefully, this unsafe operation will be replaced by the `CoerceUnsized` API in the
        // future: https://github.com/rust-lang/rust/issues/18598
        let (ptr, handle) = Gc::into_raw(Gc::new(Closure {
            f: closure,
            captures,
        }));
        // SAFETY: The pointer returned by `into_raw` is only used to coerce to a trait object,
        // meaning this is safe.
        Self {
            inner: Inner::Closure(Gc::from_raw(ptr, handle)),
        }
    }

    /// Calls this `NativeFunction`, forwarding the arguments to the corresponding function.
    #[inline]
    pub fn call(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        match self.inner {
            Inner::PointerFn(f) => f(this, args, context),
            Inner::Closure(ref c) => c.call(this, args, context),
        }
    }
}
