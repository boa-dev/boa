//! Boa's implementation of ECMAScript's `WeakSet` builtin object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-weakset-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet

use std::collections::HashSet;

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
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};
use boa_macros::JsData;

use super::iterable::IteratorHint;

#[cfg(test)]
mod tests;

pub(crate) type NativeWeakSet = boa_gc::WeakMap<ErasedVTableObject, ()>;

#[derive(Trace, Finalize, JsData)]
pub(crate) struct WeakSetData {
    pub(crate) objects: NativeWeakSet,
    // member set keyed by the unique hash of a non-registered `JsSymbol`
    //
    // TODO: this set isn't truly weak, hashes stay alive until the entire set dies
    // real weak tracking needs GC support for symbols.
    pub(crate) symbols: HashSet<u64>,
}

impl WeakSetData {
    fn new() -> Self {
        Self {
            objects: NativeWeakSet::new(),
            symbols: HashSet::new(),
        }
    }
}

#[derive(Debug, Trace, Finalize)]
pub(crate) struct WeakSet;

impl IntrinsicObject for WeakSet {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }

    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(Self::add, js_string!("add"), 1)
            .method(Self::delete, js_string!("delete"), 1)
            .method(Self::has, js_string!("has"), 1)
            .build();
    }
}

impl BuiltInObject for WeakSet {
    const NAME: JsString = StaticJsStrings::WEAK_SET;

    const ATTRIBUTE: Attribute = Attribute::WRITABLE.union(Attribute::CONFIGURABLE);
}

impl BuiltInConstructor for WeakSet {
    /// The amount of arguments the `WeakSet` constructor takes.
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 4;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::weak_set;

    /// `WeakSet ( [ iterable ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakset-iterable
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet/WeakSet
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WeakSet: cannot call constructor without `new`")
                .into());
        }

        // 2. Let set be ? OrdinaryCreateFromConstructor(NewTarget, "%WeakSet.prototype%", « [[WeakSetData]] »).
        // 3. Set set.[[WeakSetData]] to a new empty List.
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::weak_set, context)?;
        let weak_set = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            WeakSetData::new(),
        )
        .upcast();

        // 4. If iterable is either undefined or null, return set.
        let iterable = args.get_or_undefined(0);
        if iterable.is_null_or_undefined() {
            return Ok(weak_set.into());
        }

        // 5. Let adder be ? Get(set, "add").
        let adder = weak_set.get(js_string!("add"), context)?;

        // 6. If IsCallable(adder) is false, throw a TypeError exception.
        let adder = adder
            .as_callable()
            .ok_or_else(|| JsNativeError::typ().with_message("WeakSet: 'add' is not a function"))?;

        // 7. Let iteratorRecord be ? GetIterator(iterable, sync).
        let mut iterator_record = iterable.clone().get_iterator(IteratorHint::Sync, context)?;

        // 8. Repeat,
        //     a. Let next be ? IteratorStepValue(iteratorRecord).
        while let Some(next) = iterator_record.step_value(context)? {
            //     c. Let status be Completion(Call(adder, set, « next »)).
            if let Err(status) = adder.call(&weak_set.clone().into(), &[next], context) {
                //     d. IfAbruptCloseIterator(status, iteratorRecord).
                return iterator_record.close(Err(status), context);
            }
        }

        //     b. If next is done, return set.
        Ok(weak_set.into())
    }
}

impl WeakSet {
    /// `WeakSet.prototype.add( value )`
    ///
    /// The `add()` method appends a new object to the end of a `WeakSet` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakset.prototype.add
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet/add
    pub(crate) fn add(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[WeakSetData]]).
        let object = this.as_object();
        let mut data = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<WeakSetData>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakSet.add: called with non-object value")
            })?;

        // 3. If CanBeHeldWeakly(value) is false, throw a TypeError exception.
        let value = args.get_or_undefined(0);
        if !can_be_held_weakly(value) {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakSet.add: invalid value type `{}`, expected an object or non-registered symbol",
                    value.type_of()
                ))
                .into());
        }

        // 4. Dispatch to the appropriate backing store.
        if let Some(val_obj) = value.as_object() {
            // If already in the set, return S immediately.
            if data.objects.contains_key(val_obj.inner()) {
                return Ok(this.clone());
            }
            data.objects.insert(val_obj.inner(), ());
        } else if let Some(sym) = value.as_symbol() {
            // non-registered symbol path
            data.symbols.insert(sym.hash());
        }

        // 5. Return S.
        Ok(this.clone())
    }

    /// `WeakSet.prototype.delete( value )`
    ///
    /// The `delete()` method removes the specified element from a `WeakSet` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakset.prototype.delete
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet/delete
    pub(crate) fn delete(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[WeakSetData]]).
        let object = this.as_object();
        let mut data = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<WeakSetData>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakSet.delete: called with non-object value")
            })?;

        let value = args.get_or_undefined(0);

        // 3. If value is an Object, remove from GC weak set
        if let Some(val_obj) = value.as_object() {
            return Ok(data.objects.remove(val_obj.inner()).is_some().into());
        }

        // 4. If value is a non-registered Symbol, remove from symbol set.
        if let Some(sym) = value.as_symbol() {
            if !sym.is_registered() {
                return Ok(data.symbols.remove(&sym.hash()).into());
            }
        }

        // 5. Otherwise return false
        Ok(false.into())
    }

    /// `WeakSet.prototype.has( value )`
    ///
    /// The `has()` method returns a boolean indicating whether an object exists in a `WeakSet` or not.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakset.prototype.has
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet/has
    pub(crate) fn has(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[WeakSetData]]).
        let object = this.as_object();
        let data = object
            .as_ref()
            .and_then(JsObject::downcast_ref::<WeakSetData>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakSet.has: called with non-object value")
            })?;

        let value = args.get_or_undefined(0);

        // 3. If value is an Object, check GC weak set.
        if let Some(val_obj) = value.as_object() {
            return Ok(data.objects.contains_key(val_obj.inner()).into());
        }

        // 4. If value is a non-registered Symbol, check symbol set
        if let Some(sym) = value.as_symbol() {
            if !sym.is_registered() {
                return Ok(data.symbols.contains(&sym.hash()).into());
            }
        }

        // 5. Otherwise return false
        Ok(false.into())
    }
}
