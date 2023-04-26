//! Boa's implementation of ECMAScript's `WeakSet` builtin object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-weakset-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData},
    property::Attribute,
    realm::Realm,
    string::utf16,
    symbol::JsSymbol,
    Context, JsArgs, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace, WeakMap};
use boa_profiler::Profiler;

#[derive(Debug, Trace, Finalize)]
pub(crate) struct WeakSet;

impl IntrinsicObject for WeakSet {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }

    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(Self::add, "add", 1)
            .method(Self::delete, "delete", 1)
            .method(Self::has, "has", 1)
            .build();
    }
}

impl BuiltInObject for WeakSet {
    const NAME: &'static str = "WeakSet";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE.union(Attribute::CONFIGURABLE);
}

impl BuiltInConstructor for WeakSet {
    /// The amount of arguments the `WeakSet` constructor takes.
    const LENGTH: usize = 0;

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
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WeakSet: cannot call constructor without `new`")
                .into());
        }

        // 2. Let set be ? OrdinaryCreateFromConstructor(NewTarget, "%WeakSet.prototype%", « [[WeakSetData]] »).
        // 3. Set set.[[WeakSetData]] to a new empty List.
        let weak_set = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            get_prototype_from_constructor(new_target, StandardConstructors::weak_set, context)?,
            ObjectData::weak_set(WeakMap::new()),
        );

        // 4. If iterable is either undefined or null, return set.
        let iterable = args.get_or_undefined(0);
        if iterable.is_null_or_undefined() {
            return Ok(weak_set.into());
        }

        // 5. Let adder be ? Get(set, "add").
        let adder = weak_set.get(utf16!("add"), context)?;

        // 6. If IsCallable(adder) is false, throw a TypeError exception.
        let adder = adder
            .as_callable()
            .ok_or_else(|| JsNativeError::typ().with_message("WeakSet: 'add' is not a function"))?;

        // 7. Let iteratorRecord be ? GetIterator(iterable).
        let iterator_record = iterable.clone().get_iterator(context, None, None)?;

        // 8. Repeat,
        // a. Let next be ? IteratorStep(iteratorRecord).
        while let Some(next) = iterator_record.step(context)? {
            // c. Let nextValue be ? IteratorValue(next).
            let next_value = next.value(context)?;

            // d. Let status be Completion(Call(adder, set, « nextValue »)).
            // e. IfAbruptCloseIterator(status, iteratorRecord).
            if let Err(status) = adder.call(&weak_set.clone().into(), &[next_value], context) {
                return iterator_record.close(Err(status), context);
            }
        }

        // b. If next is false, return set.
        Ok(weak_set.into())
    }
}

impl WeakSet {
    /// `WeakSet.prototype.add( value )`
    ///
    /// The add() method appends a new object to the end of a `WeakSet` object.
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
        _context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[WeakSetData]]).
        let Some(obj) = this.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("WeakSet.add: called with non-object value")
                .into());
        };
        let mut obj_borrow = obj.borrow_mut();
        let o = obj_borrow.as_weak_set_mut().ok_or_else(|| {
            JsNativeError::typ().with_message("WeakSet.add: called with non-object value")
        })?;

        // 3. If Type(value) is not Object, throw a TypeError exception.
        let value = args.get_or_undefined(0);
        let Some(value) = args.get_or_undefined(0).as_object() else {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakSet.add: expected target argument of type `object`, got target of type `{}`",
                    value.type_of()
                )).into());
        };

        // 4. Let entries be the List that is S.[[WeakSetData]].
        // 5. For each element e of entries, do
        if o.contains_key(value.inner()) {
            // a. If e is not empty and SameValue(e, value) is true, then
            // i. Return S.
            return Ok(this.clone());
        }

        // 6. Append value as the last element of entries.
        o.insert(value.inner(), ());

        // 7. Return S.
        Ok(this.clone())
    }

    /// `WeakSet.prototype.delete( value )`
    ///
    /// The delete() method removes the specified element from a `WeakSet` object.  
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
        _context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[WeakSetData]]).
        let Some(obj) = this.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("WeakSet.delete: called with non-object value")
                .into());
        };
        let mut obj_borrow = obj.borrow_mut();
        let o = obj_borrow.as_weak_set_mut().ok_or_else(|| {
            JsNativeError::typ().with_message("WeakSet.delete: called with non-object value")
        })?;

        // 3. If Type(value) is not Object, return false.
        let value = args.get_or_undefined(0);
        let Some(value) = value.as_object() else {
            return Ok(false.into());
        };

        // 4. Let entries be the List that is S.[[WeakSetData]].
        // 5. For each element e of entries, do
        // a. If e is not empty and SameValue(e, value) is true, then
        // i. Replace the element of entries whose value is e with an element whose value is empty.
        // ii. Return true.
        // 6. Return false.
        Ok(o.remove(value.inner()).is_some().into())
    }

    /// `WeakSet.prototype.has( value )`
    ///
    /// The has() method returns a boolean indicating whether an object exists in a `WeakSet` or not.   
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
        _context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[WeakSetData]]).
        let Some(obj) = this.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("WeakSet.has: called with non-object value")
                .into());
        };
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_weak_set().ok_or_else(|| {
            JsNativeError::typ().with_message("WeakSet.has: called with non-object value")
        })?;

        // 3. Let entries be the List that is S.[[WeakSetData]].
        // 4. If Type(value) is not Object, return false.
        let value = args.get_or_undefined(0);
        let Some(value) = value.as_object() else {
            return Ok(false.into());
        };

        // 5. For each element e of entries, do
        // a. If e is not empty and SameValue(e, value) is true, return true.
        // 6. Return false.
        Ok(o.contains_key(value.inner()).into())
    }
}
