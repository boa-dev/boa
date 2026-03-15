//! Boa's implementation of ECMAScript's `FinalizationRegistry` object.

use std::{cell::Cell, rc::Rc, slice};

use boa_gc::{Ephemeron, Finalize, Gc, Trace, WeakGc};
use boa_profiler::Profiler;

use crate::{
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    job::JobCallback,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, ErasedVTableObject, JsFunction},
    property::Attribute,
    realm::Realm,
    string::common::StaticJsStrings,
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsSymbol, JsValue,
};

use super::{builder::BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject};

// Another possible model would be to have in `Context` a shared set of registries that need
// to be cleaned up, then `CleanupSignaler` would have a reference to its parent registry and
// this same shared set, which should allow it to push the registry to the list.
//
// However, having this shared structure would be really costly in space (each `cell` would have an
// additional pointer to carry, increasing the used memory overall) and time (borrowing several
// times through a `GcRefCell` is more costly than setting a `bool` through a `Cell`).
//
// Since it should be really unusual to have more than 6 or 7 registries live, it's just better
// to track a weak list of all the created registries, then cycle to check which of them need
// cleanup. In addition to that, this could be delayed by enqueueing an individual job for each
// group of 16 registers if any of them need to be cleaned up.
//
// Having said that, I'm leaving this comment here in case we want to reconsider this alternative
// approach in the future.

/// On GG collection, sends a signal to a [`FinalizationRegistry`] indicating that it needs to
/// be collected.
#[derive(Trace)]
#[boa_gc(unsafe_empty_trace)]
struct CleanupSignaler(Cell<Option<Rc<Cell<bool>>>>);

impl Finalize for CleanupSignaler {
    fn finalize(&self) {
        if let Some(signal) = self.0.take() {
            signal.set(true)
        }
    }
}

///  A cell tracked by a [`FinalizationRegistry`].
#[derive(Trace, Finalize)]
pub(crate) struct RegistryCell {
    target: Ephemeron<ErasedVTableObject, CleanupSignaler>,
    held_value: JsValue,
    unregister_token: Option<WeakGc<ErasedVTableObject>>,
}

/// Boa's implementation of ECMAScript's [`FinalizationRegistry`] builtin object.
///
/// FinalizationRegistry provides a way to request that a cleanup callback get called at some point
/// when a value registered with the registry has been reclaimed (garbage-collected).
///
/// [`FinalizationRegistry`]: https://tc39.es/ecma262/#sec-finalization-registry-objects
#[derive(Trace, Finalize, JsData)]
pub(crate) struct FinalizationRegistry {
    pub(crate) realm: Realm,
    pub(crate) callback: JobCallback,
    #[unsafe_ignore_trace]
    pub(crate) needs_cleanup: Rc<Cell<bool>>,
    cells: Vec<RegistryCell>,
}

impl IntrinsicObject for FinalizationRegistry {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }

    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                js_string!("FinalizationRegistry"),
                Attribute::CONFIGURABLE,
            )
            .method(Self::register, js_string!("register"), 2)
            .method(Self::unregister, js_string!("unregister"), 1)
            .build();
    }
}

impl BuiltInObject for FinalizationRegistry {
    const NAME: crate::JsString = StaticJsStrings::FINALIZATION_REGISTRY;

    const ATTRIBUTE: Attribute = Attribute::WRITABLE.union(Attribute::CONFIGURABLE);
}

impl BuiltInConstructor for FinalizationRegistry {
    /// The amount of arguments the `FinalizationRegistry` constructor takes.
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::finalization_registry;

    /// Constructor [`FinalizationRegistry ( cleanupCallback )`][cons]
    ///
    /// [cons]: https://tc39.es/ecma262/#sec-finalization-registry-cleanup-callback
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("FinalizationRegistry: cannot call constructor without `new`")
                .into());
        }

        // 2. If IsCallable(cleanupCallback) is false, throw a TypeError exception.
        let Some(callback) = args
            .get_or_undefined(0)
            .as_object()
            .cloned()
            .and_then(JsFunction::from_object)
        else {
            return Err(JsNativeError::typ()
                .with_message("FinalizationRegistry: cleanup callback of registry must be callable")
                .into());
        };

        // 3. Let finalizationRegistry be ? OrdinaryCreateFromConstructor(NewTarget,
        //    "%FinalizationRegistry.prototype%", « [[Realm]], [[CleanupCallback]], [[Cells]] »).
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::finalization_registry,
            context,
        )?;

        // 4. Let fn be the active function object.
        // 5. Set finalizationRegistry.[[Realm]] to fn.[[Realm]].
        let realm = context.realm().clone();

        // 6. Set finalizationRegistry.[[CleanupCallback]] to HostMakeJobCallback(cleanupCallback).
        let callback = context.host_hooks().make_job_callback(callback, context);

        // 7. Set finalizationRegistry.[[Cells]] to a new empty List.
        let cells = Vec::new();

        let registry = JsObject::from_proto_and_data(
            prototype,
            FinalizationRegistry {
                realm,
                callback,
                cells,
                needs_cleanup: Rc::default(),
            },
        );

        context.registries.push(WeakGc::new(registry.inner()));

        // 8. Return finalizationRegistry.
        Ok(registry.into())
    }
}

impl FinalizationRegistry {
    /// [`FinalizationRegistry.prototype.register ( target, heldValue [ , unregisterToken ] )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/sec-finalization-registry.prototype.register
    fn register(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // 1. Let finalizationRegistry be the this value.
        // 2. Perform ? RequireInternalSlot(finalizationRegistry, [[Cells]]).
        let mut registry = this
            .as_object()
            .and_then(JsObject::downcast_mut::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "FinalizationRegistry.prototype.register: invalid object type for `this`",
                )
            })?;

        let target = args.get_or_undefined(0);
        let held_value = args.get_or_undefined(1);
        let unregister_token = args.get_or_undefined(2);

        // 3. If CanBeHeldWeakly(target) is false, throw a TypeError exception.
        // TODO: support Symbols
        let Some(target_obj) = target.as_object() else {
            return Err(JsNativeError::typ()
                .with_message(
                    "FinalizationRegistry.prototype.register: `target` must be an Object or Symbol",
                )
                .into());
        };

        // 4. If SameValue(target, heldValue) is true, throw a TypeError exception.
        if target == held_value {
            return Err(JsNativeError::typ()
                .with_message(
                    "FinalizationRegistry.prototype.register: `heldValue` cannot be the same as `target`",
                )
                .into());
        }

        // 5. If CanBeHeldWeakly(unregisterToken) is false, then
        // TODO: support Symbols
        let unregister_token = match unregister_token {
            JsValue::Object(obj) => Some(WeakGc::new(obj.inner())),
            // b. Set unregisterToken to empty.
            JsValue::Undefined => None,
            // a. If unregisterToken is not undefined, throw a TypeError exception.
            _ => {
                return Err(JsNativeError::typ()
                .with_message(
                    "FinalizationRegistry.prototype.register: `unregisterToken` must be an Object, a Symbol, or undefined",
                )
                .into());
            }
        };

        // 6. Let cell be the Record { [[WeakRefTarget]]: target, [[HeldValue]]: heldValue, [[UnregisterToken]]: unregisterToken }.
        let cell = RegistryCell {
            target: Ephemeron::new(
                target_obj.inner(),
                CleanupSignaler(Cell::new(Some(registry.needs_cleanup.clone()))),
            ),
            held_value: held_value.clone(),
            unregister_token,
        };

        // 7. Append cell to finalizationRegistry.[[Cells]].
        registry.cells.push(cell);

        // 8. Return undefined.
        Ok(JsValue::undefined())
    }

    /// [`FinalizationRegistry.prototype.unregister ( unregisterToken )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-finalization-registry.prototype.unregister
    fn unregister(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // 1. Let finalizationRegistry be the this value.
        // 2. Perform ? RequireInternalSlot(finalizationRegistry, [[Cells]]).
        let mut registry = this
            .as_object()
            .and_then(JsObject::downcast_mut::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "FinalizationRegistry.prototype.register: invalid object type for `this`",
                )
            })?;

        // 3. If CanBeHeldWeakly(unregisterToken) is false, throw a TypeError exception.\
        // TODO: support Symbols
        let Some(unregister_token) = args.get_or_undefined(0).as_object().map(JsObject::inner)
        else {
            return Err(JsNativeError::typ()
                .with_message(
                    "FinalizationRegistry.prototype.unregister: `unregisterToken` must be an Object or a Symbol.",
                )
                .into());
        };
        // 4. Let removed be false.
        let mut removed = false;
        let mut i = 0;
        // 5. For each Record { [[WeakRefTarget]], [[HeldValue]], [[UnregisterToken]] } cell of finalizationRegistry.[[Cells]], do
        loop {
            if i >= registry.cells.len() {
                break;
            }

            let cell = &registry.cells[i];

            // a. If cell.[[UnregisterToken]] is not empty and SameValue(cell.[[UnregisterToken]], unregisterToken) is true, then
            if cell
                .unregister_token
                .as_ref()
                .and_then(WeakGc::upgrade)
                .is_some_and(|tok| Gc::ptr_eq(&tok, &unregister_token))
            {
                // i. Remove cell from finalizationRegistry.[[Cells]].
                let cell = registry.cells.swap_remove(i);
                if let Some(value) = cell.target.value() {
                    // Remove the inner signaler to avoid notifying a registry that doesn't
                    // have dead entries.
                    value.0.take();
                }

                // ii. Set removed to true.
                removed = true;
            } else {
                i += 1;
            }
        }

        // 6. Return removed.
        Ok(removed.into())
    }

    /// Abstract operation [`CleanupFinalizationRegistry ( finalizationRegistry )`][spec].
    ///
    /// Cleans up all the cells of the finalization registry that are determined to be
    /// unreachable by the garbage collector.
    ///
    /// # Panics
    ///
    /// Panics if `obj` is not a `FinalizationRegistry` object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-cleanup-finalization-registry
    pub(crate) fn cleanup(obj: &JsObject, context: &mut Context) -> JsResult<()> {
        // 1. Assert: finalizationRegistry has [[Cells]] and [[CleanupCallback]] internal slots.
        let mut registry = obj
            .downcast_mut::<FinalizationRegistry>()
            .expect("must be a `FinalizationRegistry");

        let mut cells = std::mem::take(&mut registry.cells);

        // 2. Let callback be finalizationRegistry.[[CleanupCallback]].
        let mut callback = std::mem::replace(
            &mut registry.callback,
            JobCallback::new(context.intrinsics().objects().throw_type_error(), ()),
        );

        drop(registry);

        let mut i = 0;
        let result = loop {
            if i >= cells.len() {
                break Ok(());
            }
            // 3. While finalizationRegistry.[[Cells]] contains a Record cell such that cell.[[WeakRefTarget]] is empty, an implementation may perform the following steps:
            if !cells[i].target.has_value() {
                // a. Choose any such cell.
                // b. Remove cell from finalizationRegistry.[[Cells]].
                let cell = cells.swap_remove(i);
                // c. Perform ? HostCallJobCallback(callback, undefined, « cell.[[HeldValue]] »).
                let result = context.host_hooks().call_job_callback(
                    &mut callback,
                    &JsValue::undefined(),
                    slice::from_ref(&cell.held_value),
                    context,
                );

                if let Err(err) = result {
                    break Err(err);
                }
            } else {
                i += 1;
            }
        };

        let mut registry = obj
            .downcast_mut::<FinalizationRegistry>()
            .expect("must be a `FinalizationRegistry");

        registry.cells = cells;
        registry.callback = callback;

        // 4. Return unused.
        result
    }
}
