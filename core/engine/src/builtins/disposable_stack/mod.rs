//! This module implements the global `DisposableStack` object.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-disposablestack-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DisposableStack

use boa_gc::{Finalize, Trace};

use crate::{
    Context, JsArgs, JsData, JsError, JsNativeError, JsResult, JsString, JsSymbol, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    native_function::NativeFunction,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
};

/// Internal state for `DisposableStack`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Trace, Finalize)]
#[boa_gc(empty_trace)]
pub(crate) enum DisposableState {
    Pending,
    Disposed,
}

/// A single disposable resource record.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct DisposableResource {
    pub(crate) value: JsValue,
    pub(crate) method: JsValue,
}

/// The `DisposableStack` builtin object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub(crate) struct DisposableStack {
    state: DisposableState,
    resources: Vec<DisposableResource>,
}

impl IntrinsicObject for DisposableStack {
    fn init(realm: &Realm) {
        let get_disposed = BuiltInBuilder::callable(realm, Self::get_disposed)
            .name(js_string!("get disposed"))
            .build();

        let dispose_fn = BuiltInBuilder::callable(realm, Self::dispose)
            .name(js_string!("dispose"))
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
                js_string!("dispose"),
                dispose_fn.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(Self::r#move, js_string!("move"), 0)
            .property(
                JsSymbol::dispose(),
                dispose_fn,
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

impl BuiltInObject for DisposableStack {
    const NAME: JsString = StaticJsStrings::DISPOSABLE_STACK;
}

impl BuiltInConstructor for DisposableStack {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 10;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::disposable_stack;

    /// [`DisposableStack ()`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-disposablestack
    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("DisposableStack constructor requires 'new'")
                .into());
        }

        // 2. Let disposableStack be ? OrdinaryCreateFromConstructor(NewTarget,
        //    "%DisposableStack.prototype%", « [[DisposableState]], [[DisposeCapability]] »).
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::disposable_stack,
            context,
        )?;

        // 3. Set disposableStack.[[DisposableState]] to pending.
        // 4. Set disposableStack.[[DisposeCapability]] to NewDisposeCapability().
        let stack = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            Self {
                state: DisposableState::Pending,
                resources: Vec::new(),
            },
        );

        // 5. Return disposableStack.
        Ok(stack.upcast().into())
    }
}

impl DisposableStack {
    /// Helper: downcast `this` to `DisposableStack` or throw `TypeError`.
    fn require_internal_slot(this: &JsValue) -> JsResult<JsObject<DisposableStack>> {
        this.as_object()
            .and_then(|o| o.clone().downcast::<Self>().ok())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a DisposableStack")
                    .into()
            })
    }

    /// Helper: require pending state or throw `ReferenceError`.
    fn require_pending(stack: &JsObject<DisposableStack>) -> JsResult<()> {
        if stack.borrow().data().state == DisposableState::Disposed {
            return Err(JsNativeError::reference()
                .with_message("DisposableStack has already been disposed")
                .into());
        }
        Ok(())
    }

    /// [`get DisposableStack.prototype.disposed`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-disposablestack.prototype.disposed
    fn get_disposed(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let stack = Self::require_internal_slot(this)?;
        Ok((stack.borrow().data().state == DisposableState::Disposed).into())
    }

    /// [`DisposableStack.prototype.use ( value )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-disposablestack.prototype.use
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

        // Get the @@dispose method.
        let method = obj.get(JsSymbol::dispose(), context)?;
        if method.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("value does not have a [Symbol.dispose] method")
                .into());
        }
        if !method.as_object().is_some_and(|o| o.is_callable()) {
            return Err(JsNativeError::typ()
                .with_message("[Symbol.dispose] is not callable")
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

    /// [`DisposableStack.prototype.adopt ( value, onDispose )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-disposablestack.prototype.adopt
    fn adopt(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let stack = Self::require_internal_slot(this)?;
        Self::require_pending(&stack)?;

        let value = args.get_or_undefined(0).clone();
        let on_dispose = args.get_or_undefined(1);

        // If IsCallable(onDispose) is false, throw a TypeError exception.
        let on_dispose_obj = on_dispose
            .as_object()
            .filter(JsObject::is_callable)
            .ok_or_else(|| JsNativeError::typ().with_message("onDispose is not callable"))?
            .clone();

        // Create a closure that captures value and onDispose.
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

    /// [`DisposableStack.prototype.defer ( onDispose )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-disposablestack.prototype.defer
    fn defer(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let stack = Self::require_internal_slot(this)?;
        Self::require_pending(&stack)?;

        let on_dispose = args.get_or_undefined(0);

        // If IsCallable(onDispose) is false, throw a TypeError exception.
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

    /// [`DisposableStack.prototype.dispose ()`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-disposablestack.prototype.dispose
    fn dispose(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let stack = Self::require_internal_slot(this)?;

        // If disposableStack.[[DisposableState]] is disposed, return undefined.
        if stack.borrow().data().state == DisposableState::Disposed {
            return Ok(JsValue::undefined());
        }

        // Set disposableStack.[[DisposableState]] to disposed.
        stack.borrow_mut().data_mut().state = DisposableState::Disposed;

        // Take the resources out.
        let resources: Vec<DisposableResource> =
            std::mem::take(&mut stack.borrow_mut().data_mut().resources);

        // DisposeResources: iterate in reverse, call each method.
        dispose_resources(resources, context)
    }

    /// [`DisposableStack.prototype.move ()`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-disposablestack.prototype.move
    fn r#move(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let stack = Self::require_internal_slot(this)?;
        Self::require_pending(&stack)?;

        // Transfer resources from old stack to new stack.
        let resources = std::mem::take(&mut stack.borrow_mut().data_mut().resources);

        // Set old stack state to disposed.
        stack.borrow_mut().data_mut().state = DisposableState::Disposed;

        // Create a new DisposableStack.
        let new_prototype = context
            .intrinsics()
            .constructors()
            .disposable_stack()
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

/// Implements `DisposeResources` abstract operation.
///
/// Iterates resources in reverse order, calling each dispose method.
/// Aggregates errors using `SuppressedError`.
pub(crate) fn dispose_resources(
    resources: Vec<DisposableResource>,
    context: &mut Context,
) -> JsResult<JsValue> {
    let mut completion: Option<JsError> = None;

    for resource in resources.into_iter().rev() {
        if resource.method.is_undefined() {
            continue;
        }

        let result = resource
            .method
            .as_object()
            .map(|m| m.call(&resource.value, &[], context));

        if let Some(Err(err)) = result {
            completion = Some(match completion {
                Some(prev_error) => {
                    // Create SuppressedError(error, suppressed)
                    let suppressed_error_ctor = context
                        .intrinsics()
                        .constructors()
                        .suppressed_error()
                        .constructor();
                    let error_value = err.into_opaque(context).unwrap_or(JsValue::undefined());
                    let prev_value = prev_error
                        .into_opaque(context)
                        .unwrap_or(JsValue::undefined());
                    match suppressed_error_ctor.call(
                        &suppressed_error_ctor.clone().into(),
                        &[error_value, prev_value, JsValue::undefined()],
                        context,
                    ) {
                        Ok(se) => JsError::from_opaque(se),
                        Err(e) => e,
                    }
                }
                None => err,
            });
        }
    }

    match completion {
        Some(error) => Err(error),
        None => Ok(JsValue::undefined()),
    }
}
