use boa_gc::{Finalize, Trace, WeakGc};
use boa_profiler::Profiler;

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData},
    property::Attribute,
    symbol::JsSymbol,
    Context, JsArgs, JsNativeError, JsResult, JsValue,
};

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

    fn init(intrinsics: &Intrinsics) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");
        BuiltInBuilder::from_standard_constructor::<Self>(intrinsics)
            .property(
                JsSymbol::to_string_tag(),
                "WeakRef",
                Attribute::CONFIGURABLE,
            )
            .method(Self::deref, "deref", 0)
            .build();
    }
}

impl BuiltInObject for WeakRef {
    const NAME: &'static str = "WeakRef";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE.union(Attribute::CONFIGURABLE);
}

impl BuiltInConstructor for WeakRef {
    /// The amount of arguments the `WeakRef` constructor takes.
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::weak_ref;

    /// Constructor [`WeakRef ( target )`][cons]
    ///
    /// [cons]: https://tc39.es/ecma262/#sec-weak-ref-target
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WeakRef: cannot call constructor without `new`")
                .into());
        }

        // 2. If target is not an Object, throw a TypeError exception.
        let target = args.get(0).and_then(JsValue::as_object).ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "WeakRef: expected target argument of type `object`, got target of type `{}`",
                args.get_or_undefined(0).type_of()
            ))
        })?;

        // 3. Let weakRef be ? OrdinaryCreateFromConstructor(NewTarget, "%WeakRef.prototype%", « [[WeakRefTarget]] »).
        // 5. Set weakRef.[[WeakRefTarget]] to target.
        let weak_ref = JsObject::from_proto_and_data(
            get_prototype_from_constructor(new_target, StandardConstructors::weak_ref, context)?,
            ObjectData::weak_ref(WeakGc::new(target.inner())),
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
    pub(crate) fn deref(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let weakRef be the this value.
        // 2. Perform ? RequireInternalSlot(weakRef, [[WeakRefTarget]]).
        let weak_ref = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "WeakRef.prototype.deref: expected `this` to be an `object`, got value of type `{}`",
                this.type_of()
            ))
        })?;

        let weak_ref = weak_ref.as_weak_ref().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WeakRef.prototype.deref: expected `this` to be a `WeakRef` object")
        })?;

        // 3. Return WeakRefDeref(weakRef).

        // `WeakRefDeref`
        // https://tc39.es/ecma262/multipage/managing-memory.html#sec-weakrefderef
        // 1. Let target be weakRef.[[WeakRefTarget]].
        // 2. If target is not empty, then
        if let Some(object) = weak_ref.upgrade() {
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
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::{run_test, JsValue, TestAction};

    #[test]
    fn weak_ref_collected() {
        run_test([
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
            TestAction::inspect_context(|_| boa_gc::force_collect()),
            TestAction::assert_eq("ptr.deref()", JsValue::undefined()),
        ]);
    }
}
