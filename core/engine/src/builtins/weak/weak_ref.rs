use boa_gc::{Finalize, Trace, WeakGc};
use boa_macros::JsData;

use crate::{
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{ErasedVTableObject, JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
};

#[derive(Clone, Trace, Finalize, JsData)]
pub(crate) enum WeakRefTarget {
    Object(WeakGc<ErasedVTableObject>),
    Symbol(JsSymbol),
}

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

        // 2. If target is not an Object, throw a TypeError exception.
        let target_val = args.get_or_undefined(0);
        if !target_val.can_be_held_weakly() {
            return Err(JsNativeError::typ().with_message(format!(
                "WeakRef: expected target argument of type `object` or non-registered symbol, got target of type `{}`",
                target_val.type_of()
            )).into());
        }

        // 3. Let weakRef be ? OrdinaryCreateFromConstructor(NewTarget, "%WeakRef.prototype%", « [[WeakRefTarget]] »).
        // 5. Set weakRef.[[WeakRefTarget]] to target.
        let inner_target = if let Some(obj) = target_val.as_object() {
            WeakRefTarget::Object(WeakGc::new(obj.inner()))
        } else if let Some(sym) = target_val.as_symbol() {
            WeakRefTarget::Symbol(sym)
        } else {
            unreachable!(
                "target_val.can_be_held_weakly() returned true for non-object, non-symbol key"
            )
        };

        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::weak_ref, context)?;
        let weak_ref = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            inner_target,
        );

        // 4. Perform AddToKeptObjects(target).
        if let Some(obj) = target_val.as_object() {
            context.kept_alive.push(obj);
        }

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
            WeakRefTarget::Object(weak_obj) => {
                if let Some(object) = weak_obj.upgrade() {
                    let object = JsObject::from(object);
                    context.kept_alive.push(object.clone());
                    Ok(object.into())
                } else {
                    Ok(JsValue::undefined())
                }
            }
            WeakRefTarget::Symbol(sym) => {
                let val = JsValue::from(sym.clone());
                Ok(val)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::{JsNativeErrorKind, JsValue, TestAction, run_test_actions};

    #[test]
    fn weak_ref_symbol_support() {
        run_test_actions([
            TestAction::run("const s = Symbol(); const wr = new WeakRef(s);"),
            TestAction::assert("wr.deref() === s"),
        ]);
    }

    #[test]
    fn weak_ref_global_symbol_rejection() {
        run_test_actions([TestAction::assert_native_error(
            "new WeakRef(Symbol.for('sim'))",
            JsNativeErrorKind::Type,
            "WeakRef: expected target argument of type `object` or non-registered symbol, got target of type `symbol`",
        )]);
    }

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
}
