//! This module implements the global `WeakSet` objest.
//!
//! The JavaScript `WeakSet` class is a global object that is used in the
//! construction of weak sets; which are high-level, collections of objects.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-weakset-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet

use boa_profiler::Profiler;
use tap::{Conv, Pipe};

use crate::{
    builtins::{set::ordered_set::OrderedSet, JsArgs},
    context::intrinsics::StandardConstructors,
    object::{internal_methods::get_prototype_from_constructor, ConstructorBuilder, ObjectData},
    prelude::JsObject,
    property::Attribute,
    symbol::WellKnownSymbols,
    Context, JsResult, JsValue,
};

use super::BuiltIn;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct WeakSet(OrderedSet<JsValue>);

impl BuiltIn for WeakSet {
    const NAME: &'static str = "WeakSet";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let to_string_tag = WellKnownSymbols::to_string_tag();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().weak_set().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .method(Self::add, "add", 1)
        .method(Self::delete, "delete", 1)
        .method(Self::has, "has", 1)
        .constructor(true)
        .property(
            to_string_tag,
            Self::NAME,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl WeakSet {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 0;

    /// Create a new weak set
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return context.throw_type_error(
                "calling a builtin WeakSet constructor without new is forbidden",
            );
        }

        // 2. Let set be ? OrdinaryCreateFromConstructor(NewTarget, "%WeakSet.prototype%", « [[WeakSetData]] »).
        // 3. Set set.[[WeakSetData]] to a new empty List.
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::weak_set, context)?;
        let weak_set =
            JsObject::from_proto_and_data(prototype, ObjectData::weak_set(OrderedSet::default()));

        // 4. If iterable is either undefined or null, return set.
        let iterable = args.get_or_undefined(0);
        if iterable.is_null_or_undefined() {
            return Ok(weak_set.into());
        }

        // 5. Let adder be ? Get(set, "add").
        let adder = weak_set.get("add", context)?;

        // 6. If IsCallable(adder) is false, throw a TypeError exception.
        let adder = adder.as_callable().ok_or_else(|| {
            context.construct_type_error("'add' of 'newTarget' is not a function")
        })?;

        // 7. Let iteratorRecord be ? GetIterator(iterable).
        let iterator_record = iterable.clone().get_iterator(context, None, None)?;

        // 8. Repeat,
        //     a. Let next be ? IteratorStep(iteratorRecord).
        //     b. If next is false, return set.
        //     c. Let nextValue be ? IteratorValue(next).
        //     d. Let status be Completion(Call(adder, set, « nextValue »)).
        //     e. IfAbruptCloseIterator(status, iteratorRecord).
        while let Some(next) = iterator_record.step(context)? {
            // c
            let next_value = next.value(context)?;

            // d, e
            if let Err(status) = adder.call(&weak_set.clone().into(), &[next_value], context) {
                return iterator_record.close(Err(status), context);
            }
        }

        // 8.b
        Ok(weak_set.into())
    }

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
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[WeakSetData]]).
        let obj = if let Some(obj) = this.as_object() {
            obj
        } else {
            return context.throw_type_error("WeakSet.add called with non-object value");
        };
        let mut obj_borrow = obj.borrow_mut();
        let o = obj_borrow
            .as_weak_set_mut()
            .ok_or_else(|| context.construct_type_error("this is not a weak set object"))?;

        // 3. If Type(value) is not Object, throw a TypeError exception.
        let value = args.get_or_undefined(0);
        if !value.is_object() {
            return context.throw_type_error("value must be an object");
        }

        // 4. Let entries be the List that is S.[[WeakSetData]].
        // 5. For each element e of entries, do

        //     a. If e is not empty and SameValue(e, value) is true, then
        //         i. Return S.
        if o.contains(value) {
            return Ok(this.into());
        }

        // 6. Append value as the last element of entries.
        o.add(value.into());

        // 7. Return S.
        Ok(this.into())
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
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[WeakSetData]]).
        let obj = if let Some(obj) = this.as_object() {
            obj
        } else {
            return context.throw_type_error("WeakSet.delete called with non-object value");
        };
        let mut obj_borrow = obj.borrow_mut();
        let o = obj_borrow
            .as_weak_set_mut()
            .ok_or_else(|| context.construct_type_error("this is not a weak set object"))?;

        // 3. If Type(value) is not Object, return false.
        let value = args.get_or_undefined(0);
        if !value.is_object() {
            return Ok(false.into());
        }

        // 4. Let entries be the List that is S.[[WeakSetData]].
        // 5. For each element e of entries, do
        //     a. If e is not empty and SameValue(e, value) is true, then
        //         i. Replace the element of entries whose value is e with an element whose value is empty.
        //         ii. Return true.
        // 6. Return false.
        Ok(o.delete(value).into())
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
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[WeakSetData]]).
        let obj = if let Some(obj) = this.as_object() {
            obj
        } else {
            return context.throw_type_error("WeakSet.has called with non-object value");
        };
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_weak_set()
            .ok_or_else(|| context.construct_type_error("this is not a weak set object"))?;

        // 3. Let entries be the List that is S.[[WeakSetData]].
        // 4. If Type(value) is not Object, return false.
        let value = args.get_or_undefined(0);
        if !value.is_object() {
            return Ok(false.into());
        }

        // 5. For each element e of entries, do
        //     a. If e is not empty and SameValue(e, value) is true, return true.
        // 6. Return false.
        Ok(o.contains(value).into())
    }
}
