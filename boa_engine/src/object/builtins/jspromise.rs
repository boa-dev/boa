//! A Rust API wrapper for Boa's promise Builtin ECMAScript Object

use std::{future::Future, pin::Pin, task};

use super::{JsArray, JsFunction};
use crate::{
    builtins::{
        promise::{PromiseState, ResolvingFunctions},
        Promise,
    },
    context::intrinsics::StandardConstructors,
    job::NativeJob,
    object::{FunctionObjectBuilder, JsObject, JsObjectType, ObjectData},
    value::TryFromJs,
    Context, JsArgs, JsError, JsNativeError, JsResult, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Gc, GcRefCell, Trace};

/// An ECMAScript [promise] object.
///
/// Known as the concurrency primitive of ECMAScript, this is the main struct used to manipulate,
/// chain and inspect `Promises` from Rust code.
///
/// # Examples
///
/// ```
/// # use boa_engine::{
/// #     builtins::promise::PromiseState,
/// #     js_string,
/// #     object::{builtins::JsPromise, FunctionObjectBuilder},
/// #     property::Attribute,
/// #     Context, JsArgs, JsError, JsValue, NativeFunction,
/// # };
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let context = &mut Context::default();
///
/// context.register_global_property("finally", false, Attribute::all());
///
/// let promise = JsPromise::new(
///     |resolvers, context| {
///         let result = js_string!("hello world!").into();
///         resolvers
///             .resolve
///             .call(&JsValue::undefined(), &[result], context)?;
///         Ok(JsValue::undefined())
///     },
///     context,
/// )?;
///
/// let promise = promise
///     .then(
///         Some(
///             FunctionObjectBuilder::new(
///                 context,
///                 NativeFunction::from_fn_ptr(|_, args, _| {
///                     Err(JsError::from_opaque(args.get_or_undefined(0).clone()).into())
///                 }),
///             )
///             .build(),
///         ),
///         None,
///         context,
///     )?
///     .catch(
///         FunctionObjectBuilder::new(
///             context,
///             NativeFunction::from_fn_ptr(|_, args, _| {
///                 Ok(args.get_or_undefined(0).clone())
///             }),
///         )
///         .build(),
///         context,
///     )?
///     .finally(
///         FunctionObjectBuilder::new(
///             context,
///             NativeFunction::from_fn_ptr(|_, _, context| {
///                 context
///                     .global_object()
///                     .clone()
///                     .set("finally", JsValue::from(true), true, context)?;
///                 Ok(JsValue::undefined())
///             }),
///         )
///         .build(),
///         context,
///     )?;
///
/// context.run_jobs();
///
/// assert_eq!(
///     promise.state()?,
///     PromiseState::Fulfilled(js_string!("hello world!").into())
/// );
///
/// assert_eq!(
///     context.global_object().clone().get("finally", context)?,
///     JsValue::from(true)
/// );
///
/// # Ok(())
/// # }
/// ```
///
/// [promise]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsPromise {
    inner: JsObject,
}

impl JsPromise {
    /// Creates a new promise object from an executor function.
    ///
    /// It is equivalent to calling the [`Promise()`] constructor, which makes it share the same
    /// execution semantics as the constructor:
    /// - The executor function `executor` is called synchronously just after the promise is created.
    /// - The executor return value is ignored.
    /// - Any error thrown within the execution of `executor` will call the `reject` function
    /// of the newly created promise, unless either `resolve` or `reject` were already called
    /// beforehand.
    ///
    /// `executor` receives as an argument the [`ResolvingFunctions`] needed to settle the promise,
    /// which can be done by either calling the `resolve` function or the `reject` function.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, JsValue, js_string
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::new(|resolvers, context| {
    ///     let result = js_string!("hello world").into();
    ///     resolvers.resolve.call(&JsValue::undefined(), &[result], context)?;
    ///     Ok(JsValue::undefined())
    /// }, context)?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(promise.state()?, PromiseState::Fulfilled(js_string!("hello world").into()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/Promise
    pub fn new<F>(executor: F, context: &mut Context<'_>) -> JsResult<Self>
    where
        F: FnOnce(&ResolvingFunctions, &mut Context<'_>) -> JsResult<JsValue>,
    {
        let promise = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().promise().prototype(),
            ObjectData::promise(Promise::new()),
        );
        let resolvers = Promise::create_resolving_functions(&promise, context);

        if let Err(e) = executor(&resolvers, context) {
            let e = e.to_opaque(context);
            resolvers
                .reject
                .call(&JsValue::undefined(), &[e], context)?;
        }

        Ok(Self { inner: promise })
    }

    /// Creates a new pending promise and returns it and its associated `ResolvingFunctions`.
    ///
    /// This can be useful when you want to manually settle a promise from Rust code, instead of
    /// running an `executor` function that automatically settles the promise on creation
    /// (see [`JsPromise::new`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, JsValue
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let (promise, resolvers) = JsPromise::new_pending(context);
    ///
    /// assert_eq!(promise.state()?, PromiseState::Pending);
    ///
    /// resolvers.reject.call(&JsValue::undefined(), &[5.into()], context)?;
    ///
    /// assert_eq!(promise.state()?, PromiseState::Rejected(5.into()));
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new_pending(context: &mut Context<'_>) -> (Self, ResolvingFunctions) {
        let promise = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().promise().prototype(),
            ObjectData::promise(Promise::new()),
        );
        let resolvers = Promise::create_resolving_functions(&promise, context);
        let promise =
            Self::from_object(promise).expect("this shouldn't fail with a newly created promise");

        (promise, resolvers)
    }

    /// Wraps an existing object with the `JsPromise` interface, returning `Err` if the object
    /// is not a valid promise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, JsObject, JsValue, Source
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = context.eval(Source::from_bytes("new Promise((resolve, reject) => resolve())"))?;
    /// let promise = promise.as_object().cloned().unwrap();
    ///
    /// let promise = JsPromise::from_object(promise)?;
    ///
    /// assert_eq!(promise.state()?, PromiseState::Fulfilled(JsValue::undefined()));
    ///
    /// assert!(JsPromise::from_object(JsObject::with_null_proto()).is_err());
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if !object.is_promise() {
            return Err(JsNativeError::typ()
                .with_message("`object` is not a Promise")
                .into());
        }
        Ok(Self { inner: object })
    }

    /// Creates a new `JsPromise` from a [`Future`]-like.
    ///
    /// If you want to convert a Rust async function into an ECMAScript async function, see
    /// [`NativeFunction::from_async_fn`][async_fn].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, JsResult, JsValue
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// async fn f() -> JsResult<JsValue> {
    ///     Ok(JsValue::null())
    /// }
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::from_future(f(), context);
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(promise.state()?, PromiseState::Fulfilled(JsValue::null()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [async_fn]: crate::native_function::NativeFunction::from_async_fn
    pub fn from_future<Fut>(future: Fut, context: &mut Context<'_>) -> Self
    where
        Fut: std::future::IntoFuture<Output = JsResult<JsValue>> + 'static,
    {
        let (promise, resolvers) = Self::new_pending(context);

        let future = async move {
            let result = future.await;

            NativeJob::new(move |context| match result {
                Ok(v) => resolvers.resolve.call(&JsValue::undefined(), &[v], context),
                Err(e) => {
                    let e = e.to_opaque(context);
                    resolvers.reject.call(&JsValue::undefined(), &[e], context)
                }
            })
        };

        context
            .job_queue()
            .enqueue_future_job(Box::pin(future), context);

        promise
    }

    /// Resolves a `JsValue` into a `JsPromise`.
    ///
    /// Equivalent to the [`Promise.resolve()`] static method.
    ///
    /// This function is mainly used to wrap a plain `JsValue` into a fulfilled promise, but it can
    /// also flatten nested layers of [thenables], which essentially converts them into native
    /// promises.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, js_string
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::resolve(js_string!("resolved!"), context)?;
    ///
    /// assert_eq!(promise.state()?, PromiseState::Fulfilled(js_string!("resolved!").into()));
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.resolve()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/resolve
    /// [thenables]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise#thenables
    pub fn resolve<V: Into<JsValue>>(value: V, context: &mut Context<'_>) -> JsResult<Self> {
        Promise::promise_resolve(
            &context.intrinsics().constructors().promise().constructor(),
            value.into(),
            context,
        )
        .and_then(Self::from_object)
    }

    /// Creates a `JsPromise` that is rejected with the reason `error`.
    ///
    /// Equivalent to the [`Promise.reject`] static method.
    ///
    /// `JsPromise::reject` is pretty similar to [`JsPromise::resolve`], with the difference that
    /// it always wraps `error` into a rejected promise, even if `error` is a promise or a [thenable].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, js_string, JsError
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::reject(JsError::from_opaque(js_string!("oops!").into()), context)?;
    ///
    /// assert_eq!(promise.state()?, PromiseState::Rejected(js_string!("oops!").into()));
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.reject`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/reject
    /// [thenable]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise#thenables
    pub fn reject<E: Into<JsError>>(error: E, context: &mut Context<'_>) -> JsResult<Self> {
        Promise::promise_reject(
            &context.intrinsics().constructors().promise().constructor(),
            &error.into(),
            context,
        )
        .and_then(Self::from_object)
    }

    /// Gets the current state of the promise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::new_pending(context).0;
    ///
    /// assert_eq!(promise.state()?, PromiseState::Pending);
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn state(&self) -> JsResult<PromiseState> {
        // TODO: if we can guarantee that objects cannot change type after creation,
        // we can remove this throw.
        let promise = self.inner.borrow();
        let promise = promise
            .as_promise()
            .ok_or_else(|| JsNativeError::typ().with_message("object is not a Promise"))?;

        Ok(promise.state().clone())
    }

    /// Schedules callback functions to run when the promise settles.
    ///
    /// Equivalent to the [`Promise.prototype.then`] method.
    ///
    /// The return value is a promise that is always pending on return, regardless of the current
    /// state of the original promise. Two handlers can be provided as callbacks to be executed when
    /// the original promise settles:
    ///
    /// - If the original promise is fulfilled, `on_fulfilled` is called with the fulfillment value
    /// of the original promise.
    /// - If the original promise is rejected, `on_rejected` is called with the rejection reason
    /// of the original promise.
    ///
    /// The return value of the handlers can be used to mutate the state of the created promise. If
    /// the callback:
    ///
    /// - returns a value: the created promise gets fulfilled with the returned value.
    /// - doesn't return: the created promise gets fulfilled with undefined.
    /// - throws: the created promise gets rejected with the thrown error as its value.
    /// - returns a fulfilled promise: the created promise gets fulfilled with that promise's value as its value.
    /// - returns a rejected promise: the created promise gets rejected with that promise's value as its value.
    /// - returns another pending promise: the created promise remains pending but becomes settled with that
    /// promise's value as its value immediately after that promise becomes settled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #     builtins::promise::PromiseState,
    /// #     js_string,
    /// #     object::{builtins::JsPromise, FunctionObjectBuilder},
    /// #     Context, JsArgs, JsError, JsValue, NativeFunction,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::new(
    ///     |resolvers, context| {
    ///         resolvers
    ///             .resolve
    ///             .call(&JsValue::undefined(), &[255.255.into()], context)?;
    ///         Ok(JsValue::undefined())
    ///     },
    ///     context,
    /// )?.then(
    ///     Some(
    ///         FunctionObjectBuilder::new(
    ///             context,
    ///             NativeFunction::from_fn_ptr(|_, args, context| {
    ///                 args.get_or_undefined(0).to_string(context).map(JsValue::from)
    ///             }),
    ///         )
    ///         .build(),
    ///     ),
    ///     None,
    ///     context,
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(
    ///     promise.state()?,
    ///     PromiseState::Fulfilled(js_string!("255.255").into())
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.prototype.then`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/then
    #[inline]
    pub fn then(
        &self,
        on_fulfilled: Option<JsFunction>,
        on_rejected: Option<JsFunction>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let result_promise = Promise::inner_then(self, on_fulfilled, on_rejected, context)?;
        Self::from_object(result_promise)
    }

    /// Schedules a callback to run when the promise is rejected.
    ///
    /// Equivalent to the [`Promise.prototype.catch`] method.
    ///
    /// This is essentially a shortcut for calling [`promise.then(None, Some(function))`][then], which
    /// only handles the error case and leaves the fulfilled case untouched.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #     js_string,
    /// #     builtins::promise::PromiseState,
    /// #     object::{builtins::JsPromise, FunctionObjectBuilder},
    /// #     Context, JsArgs, JsNativeError, JsValue, NativeFunction,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::new(
    ///     |resolvers, context| {
    ///         let error = JsNativeError::typ().with_message("thrown");
    ///         let error = error.to_opaque(context);
    ///         resolvers
    ///             .reject
    ///             .call(&JsValue::undefined(), &[error.into()], context)?;
    ///         Ok(JsValue::undefined())
    ///     },
    ///     context,
    /// )?.catch(
    ///     FunctionObjectBuilder::new(
    ///         context,
    ///         NativeFunction::from_fn_ptr(|_, args, context| {
    ///             args.get_or_undefined(0).to_string(context).map(JsValue::from)
    ///         }),
    ///     )
    ///     .build(),
    ///     context,
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(
    ///     promise.state()?,
    ///     PromiseState::Fulfilled(js_string!("TypeError: thrown").into())
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.prototype.catch`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/catch
    /// [then]: JsPromise::then
    #[inline]
    pub fn catch(&self, on_rejected: JsFunction, context: &mut Context<'_>) -> JsResult<Self> {
        self.then(None, Some(on_rejected), context)
    }

    /// Schedules a callback to run when the promise is rejected.
    ///
    /// Equivalent to the [`Promise.prototype.finally()`] method.
    ///
    /// While this could be seen as a shortcut for calling [`promise.then(Some(function), Some(function))`][then],
    /// it has slightly different semantics than `then`:
    /// - `on_finally` doesn't receive any argument, unlike `on_fulfilled` and `on_rejected`.
    /// - `finally()` is transparent; a call like `Promise.resolve("first").finally(() => "second")`
    /// returns a promise fulfilled with the value `"first"`, which would return `"second"` if `finally`
    /// was a shortcut of `then`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #     object::{builtins::JsPromise, FunctionObjectBuilder},
    /// #     property::Attribute,
    /// #     Context, JsNativeError, JsValue, NativeFunction,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// context.register_global_property("finally", false, Attribute::all());
    ///
    /// let promise = JsPromise::new(
    ///     |resolvers, context| {
    ///         let error = JsNativeError::typ().with_message("thrown");
    ///         let error = error.to_opaque(context);
    ///         resolvers
    ///             .reject
    ///             .call(&JsValue::undefined(), &[error.into()], context)?;
    ///         Ok(JsValue::undefined())
    ///     },
    ///     context,
    /// )?.finally(
    ///     FunctionObjectBuilder::new(
    ///         context,
    ///         NativeFunction::from_fn_ptr(|_, _, context| {
    ///             context
    ///                 .global_object()
    ///                 .clone()
    ///                 .set("finally", JsValue::from(true), true, context)?;
    ///             Ok(JsValue::undefined())
    ///         }),
    ///     )
    ///     .build(),
    ///     context,
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(
    ///     context.global_object().clone().get("finally", context)?,
    ///     JsValue::from(true)
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.prototype.finally()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/finally
    /// [then]: JsPromise::then
    #[inline]
    pub fn finally(&self, on_finally: JsFunction, context: &mut Context<'_>) -> JsResult<Self> {
        let c = self.species_constructor(StandardConstructors::promise, context)?;
        let (then, catch) = Promise::then_catch_finally_closures(c, on_finally, context);
        self.then(Some(then), Some(catch), context)
    }

    /// Waits for a list of promises to settle with fulfilled values, rejecting the aggregate promise
    /// when any of the inner promises is rejected.
    ///
    /// Equivalent to the [`Promise.all`] static method.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #     js_string,
    /// #     object::builtins::{JsArray, JsPromise},
    /// #     Context, JsNativeError, JsValue,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise1 = JsPromise::all(
    ///     [
    ///         JsPromise::resolve(0, context)?,
    ///         JsPromise::resolve(2, context)?,
    ///         JsPromise::resolve(4, context)?,
    ///     ],
    ///     context,
    /// )?;
    ///
    /// let promise2 = JsPromise::all(
    ///     [
    ///         JsPromise::resolve(1, context)?,
    ///         JsPromise::reject(JsNativeError::typ(), context)?,
    ///         JsPromise::resolve(3, context)?,
    ///     ],
    ///     context,
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// let array = promise1.state()?.as_fulfilled().and_then(JsValue::as_object).unwrap().clone();
    /// let array = JsArray::from_object(array)?;
    /// assert_eq!(array.at(0, context)?, 0.into());
    /// assert_eq!(array.at(1, context)?, 2.into());
    /// assert_eq!(array.at(2, context)?, 4.into());
    ///
    /// let error = promise2.state()?.as_rejected().unwrap().clone();
    /// assert_eq!(error.to_string(context)?, js_string!("TypeError"));
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.all`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/all
    pub fn all<I>(promises: I, context: &mut Context<'_>) -> JsResult<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let promises = JsArray::from_iter(promises.into_iter().map(JsValue::from), context);

        let c = &context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .into();

        let value = Promise::all(c, &[promises.into()], context)?;
        let value = value
            .as_object()
            .expect("Promise.all always returns an object on success");

        Self::from_object(value.clone())
    }

    /// Waits for a list of promises to settle, fulfilling with an array of the outcomes of every
    /// promise.
    ///
    /// Equivalent to the [`Promise.allSettled`] static method.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #     js_string,
    /// #     object::builtins::{JsArray, JsPromise},
    /// #     Context, JsNativeError, JsValue,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::all_settled(
    ///     [
    ///         JsPromise::resolve(1, context)?,
    ///         JsPromise::reject(JsNativeError::typ(), context)?,
    ///         JsPromise::resolve(3, context)?,
    ///     ],
    ///     context,
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// let array = promise.state()?.as_fulfilled().and_then(JsValue::as_object).unwrap().clone();
    /// let array = JsArray::from_object(array)?;
    ///
    /// let a = array.at(0, context)?.as_object().unwrap().clone();
    /// assert_eq!(a.get("status", context)?, js_string!("fulfilled").into());
    /// assert_eq!(a.get("value", context)?, 1.into());
    ///
    /// let b = array.at(1, context)?.as_object().unwrap().clone();
    /// assert_eq!(b.get("status", context)?, js_string!("rejected").into());
    /// assert_eq!(b.get("reason", context)?.to_string(context)?, js_string!("TypeError"));
    ///
    /// let c = array.at(2, context)?.as_object().unwrap().clone();
    /// assert_eq!(c.get("status", context)?, js_string!("fulfilled").into());
    /// assert_eq!(c.get("value", context)?, 3.into());
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.allSettled`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/allSettled
    pub fn all_settled<I>(promises: I, context: &mut Context<'_>) -> JsResult<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let promises = JsArray::from_iter(promises.into_iter().map(JsValue::from), context);

        let c = &context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .into();

        let value = Promise::all_settled(c, &[promises.into()], context)?;
        let value = value
            .as_object()
            .expect("Promise.allSettled always returns an object on success");

        Self::from_object(value.clone())
    }

    /// Returns the first promise that fulfills from a list of promises.
    ///
    /// Equivalent to the [`Promise.any`] static method.
    ///
    /// If after settling all promises in `promises` there isn't a fulfilled promise, the returned
    /// promise will be rejected with an `AggregatorError` containing the rejection values of every
    /// promise; this includes the case where `promises` is an empty iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #     builtins::promise::PromiseState,
    /// #     js_string,
    /// #     object::builtins::JsPromise,
    /// #     Context, JsNativeError,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    ///
    /// let promise = JsPromise::any(
    ///     [
    ///         JsPromise::reject(JsNativeError::syntax(), context)?,
    ///         JsPromise::reject(JsNativeError::typ(), context)?,
    ///         JsPromise::resolve(js_string!("fulfilled"), context)?,
    ///         JsPromise::reject(JsNativeError::range(), context)?,
    ///     ],
    ///     context,
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(promise.state()?, PromiseState::Fulfilled(js_string!("fulfilled").into()));
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.any`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/any
    pub fn any<I>(promises: I, context: &mut Context<'_>) -> JsResult<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let promises = JsArray::from_iter(promises.into_iter().map(JsValue::from), context);

        let c = &context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .into();

        let value = Promise::any(c, &[promises.into()], context)?;
        let value = value
            .as_object()
            .expect("Promise.any always returns an object on success");

        Self::from_object(value.clone())
    }

    /// Returns the first promise that settles from a list of promises.
    ///
    /// Equivalent to the [`Promise.race`] static method.
    ///
    /// If the provided iterator is empty, the returned promise will remain on the pending state
    /// forever.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #     builtins::promise::PromiseState,
    /// #     js_string,
    /// #     object::builtins::JsPromise,
    /// #     Context, JsValue,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let (a, resolvers_a) = JsPromise::new_pending(context);
    /// let (b, resolvers_b) = JsPromise::new_pending(context);
    /// let (c, resolvers_c) = JsPromise::new_pending(context);
    ///
    /// let promise = JsPromise::race([a, b, c], context)?;
    ///
    /// resolvers_b.reject.call(&JsValue::undefined(), &[], context);
    /// resolvers_a.resolve.call(&JsValue::undefined(), &[5.into()], context);
    /// resolvers_c.reject.call(&JsValue::undefined(), &[js_string!("c error").into()], context);
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(
    ///     promise.state()?,
    ///     PromiseState::Rejected(JsValue::undefined())
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.race`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/race
    pub fn race<I>(promises: I, context: &mut Context<'_>) -> JsResult<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let promises = JsArray::from_iter(promises.into_iter().map(JsValue::from), context);

        let c = &context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .into();

        let value = Promise::race(c, &[promises.into()], context)?;
        let value = value
            .as_object()
            .expect("Promise.race always returns an object on success");

        Self::from_object(value.clone())
    }

    /// Creates a `JsFuture` from this `JsPromise`.
    ///
    /// The returned `JsFuture` implements [`Future`], which means it can be `await`ed within Rust's
    /// async contexts (async functions and async blocks).
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use boa_engine::{
    /// #     builtins::promise::PromiseState,
    /// #     object::builtins::JsPromise,
    /// #     Context, JsValue, JsError
    /// # };
    /// # use futures_lite::future;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let (promise, resolvers) = JsPromise::new_pending(context);
    /// let promise_future = promise.into_js_future(context)?;
    ///
    /// let future1 = async move {
    ///     promise_future.await
    /// };
    ///
    /// let future2 = async move {
    ///     resolvers.resolve.call(&JsValue::undefined(), &[10.into()], context)?;
    ///     context.run_jobs();
    ///     Ok::<(), JsError>(())
    /// };
    ///
    /// let (result1, result2) = future::block_on(future::zip(future1, future2));
    ///
    /// assert_eq!(result1, Ok(JsValue::from(10)));
    /// assert_eq!(result2, Ok(()));
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_js_future(self, context: &mut Context<'_>) -> JsResult<JsFuture> {
        // Mostly based from:
        // https://docs.rs/wasm-bindgen-futures/0.4.37/src/wasm_bindgen_futures/lib.rs.html#109-168

        fn finish(state: &GcRefCell<Inner>, val: JsResult<JsValue>) {
            let task = {
                let mut state = state.borrow_mut();

                // The engine ensures both `resolve` and `reject` are called only once,
                // and only one of them.
                debug_assert!(state.result.is_none());

                // Store the received value into the state shared by the resolving functions
                // and the `JsFuture` itself. This will be accessed when the executor polls
                // the `JsFuture` again.
                state.result = Some(val);
                state.task.take()
            };

            // `task` could be `None` if the `JsPromise` was already fulfilled before polling
            // the `JsFuture`.
            if let Some(task) = task {
                task.wake();
            }
        }

        let state = Gc::new(GcRefCell::new(Inner {
            result: None,
            task: None,
        }));

        let resolve = {
            let state = state.clone();

            FunctionObjectBuilder::new(
                context,
                NativeFunction::from_copy_closure_with_captures(
                    move |_, args, state, _| {
                        finish(state, Ok(args.get_or_undefined(0).clone()));
                        Ok(JsValue::undefined())
                    },
                    state,
                ),
            )
            .build()
        };

        let reject = {
            let state = state.clone();

            FunctionObjectBuilder::new(
                context,
                NativeFunction::from_copy_closure_with_captures(
                    move |_, args, state, _| {
                        let err = JsError::from_opaque(args.get_or_undefined(0).clone());
                        finish(state, Err(err));
                        Ok(JsValue::undefined())
                    },
                    state,
                ),
            )
            .build()
        };

        drop(self.then(Some(resolve), Some(reject), context)?);

        Ok(JsFuture { inner: state })
    }
}

impl From<JsPromise> for JsObject {
    #[inline]
    fn from(o: JsPromise) -> Self {
        o.inner.clone()
    }
}

impl From<JsPromise> for JsValue {
    #[inline]
    fn from(o: JsPromise) -> Self {
        o.inner.clone().into()
    }
}

impl std::ops::Deref for JsPromise {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsPromise {}

impl TryFromJs for JsPromise {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Self::from_object(o.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("value is not a Promise object")
                .into()),
        }
    }
}

/// A Rust's `Future` that becomes ready when a `JsPromise` fulfills.
///
/// This type allows `await`ing `JsPromise`s inside Rust's async contexts, which makes interfacing
/// between promises and futures a bit easier.
///
/// The only way to construct an instance of `JsFuture` is by calling [`JsPromise::into_js_future`].
pub struct JsFuture {
    inner: Gc<GcRefCell<Inner>>,
}

impl std::fmt::Debug for JsFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsFuture").finish_non_exhaustive()
    }
}

#[derive(Trace, Finalize)]
struct Inner {
    result: Option<JsResult<JsValue>>,
    #[unsafe_ignore_trace]
    task: Option<task::Waker>,
}

// Taken from:
// https://docs.rs/wasm-bindgen-futures/0.4.37/src/wasm_bindgen_futures/lib.rs.html#171-187
impl Future for JsFuture {
    type Output = JsResult<JsValue>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let mut inner = self.inner.borrow_mut();

        if let Some(result) = inner.result.take() {
            return task::Poll::Ready(result);
        }

        inner.task = Some(cx.waker().clone());
        task::Poll::Pending
    }
}
