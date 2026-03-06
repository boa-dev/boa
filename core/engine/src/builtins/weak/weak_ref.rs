use boa_gc::{Finalize, Trace, WeakGc};

use crate::{
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        weak_map::can_be_held_weakly,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{ErasedVTableObject, JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::{JsSymbol, WeakJsSymbol},
};
use boa_macros::JsData;

/// Boa's implementation of ECMAScript's `WeakRef` builtin object.
///
/// The `WeakRef` is a way to refer to a target object without rooting the target and thus preserving it in garbage
/// collection. A `WeakRef` will allow the user to dereference the target as long as the target object has not been
/// collected by the garbage collector.
///
/// More Information:
///  - [ECMAScript Reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-weak-ref-objects
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct WeakRef;

#[derive(Clone, Trace, Finalize, JsData)]
pub(crate) enum WeakRefTarget {
    Object(WeakGc<ErasedVTableObject>),
    #[boa_gc(unsafe_empty_trace)]
    Symbol(WeakJsSymbol),
}

impl std::fmt::Debug for WeakRefTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object(_) => f.debug_tuple("Object").finish(),
            Self::Symbol(sym) => f.debug_tuple("Symbol").field(sym).finish(),
        }
    }
}

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
        // If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WeakRef: cannot call constructor without `new`")
                .into());
        }

        // 2. If CanBeHeldWeakly(target) is false, throw a TypeError exception.
        let target = args.get_or_undefined(0);
        if !can_be_held_weakly(target) {
            return Err(JsNativeError::typ().with_message(format!(
                "WeakRef: expected target argument of type `object` or non-registered symbol, got target of type `{}`",
                target.type_of()
            ))
            .into());
        }

        let target_val = if let Some(obj) = target.as_object() {
            WeakRefTarget::Object(WeakGc::new(obj.inner()))
        } else if let Some(sym) = target.as_symbol() {
            WeakRefTarget::Symbol(sym.downgrade())
        } else {
            unreachable!("can_be_held_weakly ensures it is an object or a non-registered symbol")
        };

        // 3. Let weakRef be ? OrdinaryCreateFromConstructor(NewTarget, "%WeakRef.prototype%", « [[WeakRefTarget]] »).
        // 5. Set weakRef.[[WeakRefTarget]] to target.
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::weak_ref, context)?;
        let weak_ref = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            target_val,
        );

        // 4. Perform AddToKeptObjects(target).
        context.kept_alive.push(target.clone());

        // 6. Return weakRef.
        Ok(weak_ref.into())
    }
}

impl WeakRef {
    /// Method [`WeakRef.prototype.deref ( )`][spec].
    ///
    /// If the referenced object hasn't been collected, this method promotes a `WeakRef` into a
    /// proper [`JsObject`], or returns `undefined` otherwise.
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

        // `WeakRefDeref`
        // https://tc39.es/ecma262/multipage/managing-memory.html#sec-weakrefderef
        // 1. Let target be weakRef.[[WeakRefTarget]].
        // 2. If target is not empty, then
        match &*weak_ref {
            WeakRefTarget::Object(weak_gc_obj) => {
                if let Some(object) = weak_gc_obj.upgrade() {
                    let object = JsObject::from(object);

                    // a. Perform AddToKeptObjects(target).
                    context.kept_alive.push(object.clone().into());

                    // b. Return target.
                    Ok(object.into())
                } else {
                    // 3. Return undefined.
                    Ok(JsValue::undefined())
                }
            }
            WeakRefTarget::Symbol(weak_sym) => {
                if let Some(symbol) = weak_sym.upgrade() {
                    let val = JsValue::new(symbol);
                    context.kept_alive.push(val.clone());
                    Ok(val)
                } else {
                    Ok(JsValue::undefined())
                }
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
    fn weak_ref_symbol_collected() {
        run_test_actions([
            TestAction::assert_with_op(
                indoc! {r#"
                    var ptr;
                    {
                        let sym = Symbol("test");
                        ptr = new WeakRef(sym);
                    }
                    ptr.deref()
                "#},
                |v, _| v.is_symbol(),
            ),
            TestAction::inspect_context(|context| {
                context.clear_kept_objects();
                boa_gc::force_collect();
            }),
            TestAction::assert_eq("ptr.deref()", JsValue::undefined()),
            // well known symbols cannot be collected
            TestAction::assert_with_op(
                indoc! {r#"
                    var ptr;
                    {
                        ptr = new WeakRef(Symbol.iterator);
                    }
                    ptr.deref()
                "#},
                |v, _| v.is_symbol(),
            ),
            TestAction::inspect_context(|context| {
                context.clear_kept_objects();
                boa_gc::force_collect();
            }),
            TestAction::assert_with_op("ptr.deref()", |v, _| v.is_symbol()),
        ]);
    }

    #[test]
    fn weak_ref_invalid_target() {
        run_test_actions([
            TestAction::assert_native_error(
                "new WeakRef(Symbol.for('registered'))",
                crate::JsNativeErrorKind::Type,
                "WeakRef: expected target argument of type `object` or non registered symbol, got target of type `symbol`",
            ),
            TestAction::assert_native_error(
                "new WeakRef(1)",
                crate::JsNativeErrorKind::Type,
                "WeakRef: expected target argument of type `object` or non registered symbol, got target of type `number`",
            ),
        ]);
    }
}
