//! This module implements the global `AsyncDisposableStack` object.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-asyncdisposablestack-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AsyncDisposableStack

use boa_gc::{Finalize, Trace};

use crate::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsSymbol, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        disposable_stack::{DisposableResource, DisposableState},
        promise::PromiseCapability,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    native_function::NativeFunction,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
};

/// The `AsyncDisposableStack` builtin object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub(crate) struct AsyncDisposableStack {
    state: DisposableState,
    resources: Vec<DisposableResource>,
}

impl IntrinsicObject for AsyncDisposableStack {
    fn init(realm: &Realm) {
        let get_disposed = BuiltInBuilder::callable(realm, Self::get_disposed)
            .name(js_string!("get disposed"))
            .build();

        let dispose_async_fn = BuiltInBuilder::callable(realm, Self::dispose_async)
            .name(js_string!("disposeAsync"))
            .length(0)
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("disposed"),
                Some(get_disposed),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .method(Self::use_, js_string!("use"), 1)
            .method(Self::adopt, js_string!("adopt"), 2)
            .method(Self::defer, js_string!("defer"), 1)
            .property(
                js_string!("disposeAsync"),
                dispose_async_fn.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(Self::r#move, js_string!("move"), 0)
            .property(
                JsSymbol::async_dispose(),
                dispose_async_fn,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for AsyncDisposableStack {
    const NAME: JsString = StaticJsStrings::ASYNC_DISPOSABLE_STACK;
}

impl BuiltInConstructor for AsyncDisposableStack {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 10;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::async_disposable_stack;

    /// [`AsyncDisposableStack ()`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncdisposablestack
    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("AsyncDisposableStack constructor requires 'new'")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::async_disposable_stack,
            context,
        )?;

        let stack = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            Self {
                state: DisposableState::Pending,
                resources: Vec::new(),
            },
        );

        Ok(stack.upcast().into())
    }
}

impl AsyncDisposableStack {
    /// Helper: downcast `this` to `AsyncDisposableStack` or throw `TypeError`.
    fn require_internal_slot(this: &JsValue) -> JsResult<JsObject<AsyncDisposableStack>> {
        this.as_object()
            .and_then(|o| o.clone().downcast::<Self>().ok())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not an AsyncDisposableStack")
                    .into()
            })
    }

    /// Helper: require pending state or throw `ReferenceError`.
    fn require_pending(stack: &JsObject<AsyncDisposableStack>) -> JsResult<()> {
        if stack.borrow().data().state == DisposableState::Disposed {
            return Err(JsNativeError::reference()
                .with_message("AsyncDisposableStack has already been disposed")
                .into());
        }
        Ok(())
    }

    /// [`get AsyncDisposableStack.prototype.disposed`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-asyncdisposablestack.prototype.disposed
    fn get_disposed(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let stack = Self::require_internal_slot(this)?;
        Ok((stack.borrow().data().state == DisposableState::Disposed).into())
    }

    /// [`AsyncDisposableStack.prototype.use ( value )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncdisposablestack.prototype.use
    fn use_(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let stack = Self::require_internal_slot(this)?;
        Self::require_pending(&stack)?;

        let value = args.get_or_undefined(0);

        // If value is null or undefined, return value.
        if value.is_null_or_undefined() {
            return Ok(value.clone());
        }

        // Value must be an object.
        let obj = value
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("value must be an object"))?;

        // Try Symbol.asyncDispose first, then fallback to Symbol.dispose.
        let method = obj.get(JsSymbol::async_dispose(), context)?;
        let method = if method.is_undefined() {
            let sync_method = obj.get(JsSymbol::dispose(), context)?;
            if sync_method.is_undefined() {
                return Err(JsNativeError::typ()
                    .with_message(
                        "value does not have a [Symbol.asyncDispose] or [Symbol.dispose] method",
                    )
                    .into());
            }
            sync_method
        } else {
            method
        };

        if !method.as_object().is_some_and(|o| o.is_callable()) {
            return Err(JsNativeError::typ()
                .with_message("dispose method is not callable")
                .into());
        }

        stack
            .borrow_mut()
            .data_mut()
            .resources
            .push(DisposableResource {
                value: value.clone(),
                method,
            });

        Ok(value.clone())
    }

    /// [`AsyncDisposableStack.prototype.adopt ( value, onDispose )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncdisposablestack.prototype.adopt
    fn adopt(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let stack = Self::require_internal_slot(this)?;
        Self::require_pending(&stack)?;

        let value = args.get_or_undefined(0).clone();
        let on_dispose = args.get_or_undefined(1);

        let on_dispose_obj = on_dispose
            .as_object()
            .filter(JsObject::is_callable)
            .ok_or_else(|| JsNativeError::typ().with_message("onDispose is not callable"))?
            .clone();

        let captured_value = value.clone();
        let closure = NativeFunction::from_copy_closure_with_captures(
            |_this, _args, captures, context| {
                let (val, dispose_fn) = captures;
                dispose_fn.call(&JsValue::undefined(), std::slice::from_ref(val), context)?;
                Ok(JsValue::undefined())
            },
            (captured_value, on_dispose_obj),
        );

        let f: JsValue = closure.to_js_function(context.realm()).into();

        stack
            .borrow_mut()
            .data_mut()
            .resources
            .push(DisposableResource {
                value: JsValue::undefined(),
                method: f,
            });

        Ok(value)
    }

    /// [`AsyncDisposableStack.prototype.defer ( onDispose )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncdisposablestack.prototype.defer
    fn defer(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let stack = Self::require_internal_slot(this)?;
        Self::require_pending(&stack)?;

        let on_dispose = args.get_or_undefined(0);

        if !on_dispose.as_object().is_some_and(|o| o.is_callable()) {
            return Err(JsNativeError::typ()
                .with_message("onDispose is not callable")
                .into());
        }

        stack
            .borrow_mut()
            .data_mut()
            .resources
            .push(DisposableResource {
                value: JsValue::undefined(),
                method: on_dispose.clone(),
            });

        Ok(JsValue::undefined())
    }

    /// [`AsyncDisposableStack.prototype.disposeAsync ()`][spec]
    ///
    /// Returns a Promise that resolves after all resources are disposed.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncdisposablestack.prototype.disposeAsync
    fn dispose_async(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Create a PromiseCapability first so we can reject it on type errors.
        let promise_capability = PromiseCapability::new(
            &context
                .intrinsics()
                .constructors()
                .promise()
                .constructor(),
            context,
        )?;

        // 1. Let asyncDisposableStack be the this value.
        // 2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
        let stack = match Self::require_internal_slot(this) {
            Ok(s) => s,
            Err(err) => {
                let error_value = err.into_opaque(context).unwrap_or(JsValue::undefined());
                promise_capability
                    .reject()
                    .call(&JsValue::undefined(), &[error_value], context)?;
                return Ok(promise_capability.promise().clone().into());
            }
        };

        // 3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, return
        //    ! PromiseResolve(%Promise%, undefined).
        if stack.borrow().data().state == DisposableState::Disposed {
            promise_capability.resolve().call(
                &JsValue::undefined(),
                &[JsValue::undefined()],
                context,
            )?;
            return Ok(promise_capability.promise().clone().into());
        }

        // 4. Set asyncDisposableStack.[[AsyncDisposableState]] to disposed.
        stack.borrow_mut().data_mut().state = DisposableState::Disposed;

        // Take the resources out.
        let resources: Vec<DisposableResource> =
            std::mem::take(&mut stack.borrow_mut().data_mut().resources);

        // DisposeResources: iterate in reverse, call each method.
        let result = super::disposable_stack::dispose_resources(resources, context);

        // Resolve or reject the promise based on the result.
        match result {
            Ok(_) => {
                promise_capability.resolve().call(
                    &JsValue::undefined(),
                    &[JsValue::undefined()],
                    context,
                )?;
            }
            Err(error) => {
                let error_value = error.into_opaque(context).unwrap_or(JsValue::undefined());
                promise_capability
                    .reject()
                    .call(&JsValue::undefined(), &[error_value], context)?;
            }
        }

        Ok(promise_capability.promise().clone().into())
    }

    /// [`AsyncDisposableStack.prototype.move ()`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncdisposablestack.prototype.move
    fn r#move(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let stack = Self::require_internal_slot(this)?;
        Self::require_pending(&stack)?;

        let resources = std::mem::take(&mut stack.borrow_mut().data_mut().resources);
        stack.borrow_mut().data_mut().state = DisposableState::Disposed;

        let new_prototype = context
            .intrinsics()
            .constructors()
            .async_disposable_stack()
            .prototype();

        let new_stack = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            new_prototype,
            Self {
                state: DisposableState::Pending,
                resources,
            },
        );

        Ok(new_stack.upcast().into())
    }
}
