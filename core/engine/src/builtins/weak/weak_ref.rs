use boa_gc::{Finalize, Trace, WeakGc};

use crate::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        weak::{WeakKey, can_be_held_weakly},
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{ErasedVTableObject, JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
};

/// Internal target of a `WeakRef`: either a GC-weak object reference or a symbol.
///
/// Symbols are not GC-traced (they use `Arc`), so they cannot use `WeakGc`.
/// Non-registered symbols hold a strong `JsSymbol` reference; their lifetime
/// is tied to the `WeakRef` instance itself, which matches spec intent since
/// well-known symbols are effectively immortal and user-created symbols are
/// unique identity values.
#[derive(Clone, Trace, Finalize, JsData)]
pub(crate) enum WeakRefTarget {
    Object(WeakGc<ErasedVTableObject>),
    /// Note: `deref()` always succeeds for symbol targets because symbols
    /// are `Arc`-based and cannot be garbage-collected.
    #[unsafe_ignore_trace]
    Symbol(JsSymbol),
}

impl std::fmt::Debug for WeakRefTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object(_) => f.debug_tuple("Object").field(&"<weak>").finish(),
            Self::Symbol(sym) => f.debug_tuple("Symbol").field(sym).finish(),
        }
    }
}

/// Boa's implementation of ECMAScript's `WeakRef` builtin object.
///
/// The `WeakRef` is a way to refer to a target object or symbol without rooting
/// the target and thus preserving it in garbage collection. A `WeakRef` will
/// allow the user to dereference the target as long as the target has not been
/// collected by the garbage collector.
///
/// More Information:
///  - [ECMAScript Reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-weak-ref-objects
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct WeakRef;

impl IntrinsicObject for WeakRef {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }

    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                js_string!("WeakRef"),
                Attribute::CONFIGURABLE,
            )
            .method(Self::deref, js_string!("deref"), 0)
            .build();
    }
}

impl BuiltInObject for WeakRef {
    const NAME: JsString = StaticJsStrings::WEAK_REF;

    const ATTRIBUTE: Attribute = Attribute::WRITABLE.union(Attribute::CONFIGURABLE);
}

impl BuiltInConstructor for WeakRef {
    /// The amount of arguments the `WeakRef` constructor takes.
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 2;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::weak_ref;

    /// Constructor [`WeakRef ( target )`][cons]
    ///
    /// [cons]: https://tc39.es/ecma262/#sec-weak-ref-target
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WeakRef: cannot call constructor without `new`")
                .into());
        }

        // 2. If CanBeHeldWeakly(target) is false, throw a TypeError exception.
        let target = args.get_or_undefined(0);
        let weak_key = can_be_held_weakly(target).ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "WeakRef: invalid target type `{}`: cannot be held weakly",
                target.type_of()
            ))
        })?;

        // 3. Let weakRef be ? OrdinaryCreateFromConstructor(NewTarget, "%WeakRef.prototype%", « [[WeakRefTarget]] »).
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::weak_ref, context)?;

        // 4. Perform AddToKeptObjects(target).
        // 5. Set weakRef.[[WeakRefTarget]] to target.
        // 6. Return weakRef.
        match weak_key {
            WeakKey::Object(obj) => {
                let weak_ref = JsObject::from_proto_and_data_with_shared_shape(
                    context.root_shape(),
                    prototype,
                    WeakRefTarget::Object(WeakGc::new(obj.inner())),
                );

                // Step 4: AddToKeptObjects — prevents GC from collecting `target`
                // before the next synchronous completion.
                context.kept_alive.push(obj.clone());

                Ok(weak_ref.into())
            }
            WeakKey::Symbol(sym) => {
                let weak_ref = JsObject::from_proto_and_data_with_shared_shape(
                    context.root_shape(),
                    prototype,
                    WeakRefTarget::Symbol(sym),
                );

                // Note: AddToKeptObjects is not needed for symbols because they
                // are not GC-managed (`Arc`-based) and cannot be collected.

                Ok(weak_ref.into())
            }
        }
    }
}

impl WeakRef {
    /// Method [`WeakRef.prototype.deref ( )`][spec].
    ///
    /// If the referenced object hasn't been collected, this method promotes a `WeakRef` into a
    /// proper [`JsObject`] or returns the symbol, or returns `undefined` otherwise.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weak-ref.prototype.deref
    pub(crate) fn deref(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let weakRef be the this value.
        // 2. Perform ? RequireInternalSlot(weakRef, [[WeakRefTarget]]).
        let object = this.as_object();
        let weak_ref = object
            .as_ref()
            .and_then(JsObject::downcast_ref::<WeakRefTarget>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "WeakRef.prototype.deref: expected `this` to be a `WeakRef` object",
                )
            })?;

        // 3. Return WeakRefDeref(weakRef).
        // https://tc39.es/ecma262/multipage/managing-memory.html#sec-weakrefderef
        match &*weak_ref {
            WeakRefTarget::Object(weak_gc) => {
                // 1. Let target be weakRef.[[WeakRefTarget]].
                // 2. If target is not empty, then
                if let Some(object) = weak_gc.upgrade() {
                    let object = JsObject::from(object);

                    // a. Perform AddToKeptObjects(target).
                    context.kept_alive.push(object.clone());

                    // b. Return target.
                    Ok(object.into())
                } else {
                    // 3. Return undefined.
                    Ok(JsValue::undefined())
                }
            }
            WeakRefTarget::Symbol(sym) => {
                // Symbols are `Arc`-based and cannot be garbage-collected,
                // so they always resolve successfully.
                Ok(sym.clone().into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::{JsValue, TestAction, run_test_actions};

    #[test]
    fn weak_ref_collected() {
        run_test_actions([
            TestAction::assert_with_op(
                indoc! {r#"
                    var ptr;
                    {
                        let obj = {a: 5, b: 6};
                        ptr = new WeakRef(obj);
                    }
                    ptr.deref()
                "#},
                |v, _| v.is_object(),
            ),
            TestAction::inspect_context(|context| {
                context.clear_kept_objects();
                boa_gc::force_collect();
            }),
            TestAction::assert_eq("ptr.deref()", JsValue::undefined()),
        ]);
    }

    #[test]
    fn weak_ref_symbol_target() {
        run_test_actions([TestAction::assert(indoc! {r#"
                    let sym = Symbol("test");
                    let wr = new WeakRef(sym);
                    wr.deref() === sym
                "#})]);
    }

    #[test]
    fn weak_ref_rejects_registered_symbol() {
        run_test_actions([TestAction::assert_native_error(
            "new WeakRef(Symbol.for('registered'))",
            crate::JsNativeErrorKind::Type,
            "WeakRef: invalid target type `symbol`: cannot be held weakly",
        )]);
    }
}
