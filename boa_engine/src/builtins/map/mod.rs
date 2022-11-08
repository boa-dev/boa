//! This module implements the global `Map` object.
//!
//! The JavaScript `Map` class is a global object that is used in the construction of maps; which
//! are high-level, key-value stores.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-map-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map

use self::{map_iterator::MapIterator, ordered_map::OrderedMap};
use super::JsArgs;
use crate::{
    builtins::BuiltIn,
    context::intrinsics::StandardConstructors,
    error::JsNativeError,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        JsObject, ObjectData,
    },
    property::{Attribute, PropertyNameKind},
    symbol::WellKnownSymbols,
    Context, JsResult, JsValue,
};
use boa_profiler::Profiler;
use num_traits::Zero;
use tap::{Conv, Pipe};

pub mod map_iterator;
pub mod ordered_map;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct Map(OrderedMap<JsValue>);

impl BuiltIn for Map {
    const NAME: &'static str = "Map";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let get_species = FunctionBuilder::native(context, Self::get_species)
            .name("get [Symbol.species]")
            .constructor(false)
            .build();

        let get_size = FunctionBuilder::native(context, Self::get_size)
            .name("get size")
            .length(0)
            .constructor(false)
            .build();

        let entries_function = FunctionBuilder::native(context, Self::entries)
            .name("entries")
            .length(0)
            .constructor(false)
            .build();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().map().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .static_accessor(
            WellKnownSymbols::species(),
            Some(get_species),
            None,
            Attribute::CONFIGURABLE,
        )
        .property(
            "entries",
            entries_function.clone(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            WellKnownSymbols::iterator(),
            entries_function,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            WellKnownSymbols::to_string_tag(),
            Self::NAME,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .method(Self::clear, "clear", 0)
        .method(Self::delete, "delete", 1)
        .method(Self::for_each, "forEach", 1)
        .method(Self::get, "get", 1)
        .method(Self::has, "has", 1)
        .method(Self::keys, "keys", 0)
        .method(Self::set, "set", 2)
        .method(Self::values, "values", 0)
        .accessor("size", Some(get_size), None, Attribute::CONFIGURABLE)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl Map {
    pub(crate) const LENGTH: usize = 0;

    /// `Map ( [ iterable ] )`
    ///
    /// Constructor for `Map` objects.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map-iterable
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/Map
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling a builtin Map constructor without new is forbidden")
                .into());
        }

        // 2. Let map be ? OrdinaryCreateFromConstructor(NewTarget, "%Map.prototype%", « [[MapData]] »).
        // 3. Set map.[[MapData]] to a new empty List.
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::map, context)?;
        let map = JsObject::from_proto_and_data(prototype, ObjectData::map(OrderedMap::new()));

        // 4. If iterable is either undefined or null, return map.
        let iterable = match args.get_or_undefined(0) {
            val if !val.is_null_or_undefined() => val,
            _ => return Ok(map.into()),
        };

        // 5. Let adder be ? Get(map, "set").
        let adder = map.get("set", context)?;

        // 6. Return ? AddEntriesFromIterable(map, iterable, adder).
        add_entries_from_iterable(&map, iterable, &adder, context)
    }

    /// `get Map [ @@species ]`
    ///
    /// The `Map [ @@species ]` accessor property returns the Map constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-map-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `Map.prototype.entries()`
    ///
    /// Returns a new Iterator object that contains the [key, value] pairs for each element in the Map object in insertion order.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.entries
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/entries
    pub(crate) fn entries(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Return ? CreateMapIterator(M, key+value).
        MapIterator::create_map_iterator(this, PropertyNameKind::KeyAndValue, context)
    }

    /// `Map.prototype.keys()`
    ///
    /// Returns a new Iterator object that contains the keys for each element in the Map object in insertion order.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.keys
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/keys
    pub(crate) fn keys(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Return ? CreateMapIterator(M, key).
        MapIterator::create_map_iterator(this, PropertyNameKind::Key, context)
    }

    /// `Map.prototype.set( key, value )`
    ///
    /// Inserts a new entry in the Map object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.set
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/set
    pub(crate) fn set(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let key = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);

        // 1. Let M be the this value.
        if let Some(object) = this.as_object() {
            // 2. Perform ? RequireInternalSlot(M, [[MapData]]).
            // 3. Let entries be the List that is M.[[MapData]].
            if let Some(map) = object.borrow_mut().as_map_mut() {
                let key = match key {
                    JsValue::Rational(r) => {
                        // 5. If key is -0𝔽, set key to +0𝔽.
                        if r.is_zero() {
                            JsValue::Rational(0f64)
                        } else {
                            key.clone()
                        }
                    }
                    _ => key.clone(),
                };
                // 4. For each Record { [[Key]], [[Value]] } p of entries, do
                // a. If p.[[Key]] is not empty and SameValueZero(p.[[Key]], key) is true, then
                // i. Set p.[[Value]] to value.
                // 6. Let p be the Record { [[Key]]: key, [[Value]]: value }.
                // 7. Append p as the last element of entries.
                map.insert(key, value.clone());
                // ii. Return M.
                // 8. Return M.
                return Ok(this.clone());
            }
        }
        Err(JsNativeError::typ()
            .with_message("'this' is not a Map")
            .into())
    }

    /// `get Map.prototype.size`
    ///
    /// Obtains the size of the map, filtering empty keys to ensure it updates
    /// while iterating.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-map.prototype.size
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/size
    pub(crate) fn get_size(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        if let Some(object) = this.as_object() {
            // 2. Perform ? RequireInternalSlot(M, [[MapData]]).
            // 3. Let entries be the List that is M.[[MapData]].
            if let Some(map) = object.borrow_mut().as_map_mut() {
                // 4. Let count be 0.
                // 5. For each Record { [[Key]], [[Value]] } p of entries, do
                // a. If p.[[Key]] is not empty, set count to count + 1.
                // 6. Return 𝔽(count).
                return Ok(map.len().into());
            }
        }
        Err(JsNativeError::typ()
            .with_message("'this' is not a Map")
            .into())
    }

    /// `Map.prototype.delete( key )`
    ///
    /// Removes the element associated with the key, if it exists.
    /// Returns true if there was an element, and false otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.delete
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/delete
    pub(crate) fn delete(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let key = args.get_or_undefined(0);

        // 1. Let M be the this value.
        if let Some(object) = this.as_object() {
            // 2. Perform ? RequireInternalSlot(M, [[MapData]]).
            // 3. Let entries be the List that is M.[[MapData]].
            if let Some(map) = object.borrow_mut().as_map_mut() {
                // a. If p.[[Key]] is not empty and SameValueZero(p.[[Key]], key) is true, then
                // i. Set p.[[Key]] to empty.
                // ii. Set p.[[Value]] to empty.
                // iii. Return true.
                // 5. Return false.
                return Ok(map.remove(key).is_some().into());
            }
        }
        Err(JsNativeError::typ()
            .with_message("'this' is not a Map")
            .into())
    }

    /// `Map.prototype.get( key )`
    ///
    /// Returns the value associated with the key, or undefined if there is none.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.get
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/get
    pub(crate) fn get(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        const JS_ZERO: &JsValue = &JsValue::Rational(0f64);

        let key = args.get_or_undefined(0);
        let key = match key {
            JsValue::Rational(r) => {
                if r.is_zero() {
                    JS_ZERO
                } else {
                    key
                }
            }
            _ => key,
        };

        // 1. Let M be the this value.
        if let JsValue::Object(ref object) = this {
            // 2. Perform ? RequireInternalSlot(M, [[MapData]]).
            // 3. Let entries be the List that is M.[[MapData]].
            if let Some(map) = object.borrow().as_map_ref() {
                // 4. For each Record { [[Key]], [[Value]] } p of entries, do
                // a. If p.[[Key]] is not empty and SameValueZero(p.[[Key]], key) is true, return p.[[Value]].
                // 5. Return undefined.
                return Ok(map.get(key).cloned().unwrap_or_default());
            }
        }

        Err(JsNativeError::typ()
            .with_message("'this' is not a Map")
            .into())
    }

    /// `Map.prototype.clear( )`
    ///
    /// Removes all entries from the map.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.clear
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/clear
    pub(crate) fn clear(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Perform ? RequireInternalSlot(M, [[MapData]]).
        if let Some(object) = this.as_object() {
            // 3. Let entries be the List that is M.[[MapData]].
            if let Some(map) = object.borrow_mut().as_map_mut() {
                // 4. For each Record { [[Key]], [[Value]] } p of entries, do
                // a. Set p.[[Key]] to empty.
                // b. Set p.[[Value]] to empty.
                map.clear();

                // 5. Return undefined.
                return Ok(JsValue::undefined());
            }
        }
        Err(JsNativeError::typ()
            .with_message("'this' is not a Map")
            .into())
    }

    /// `Map.prototype.has( key )`
    ///
    /// Checks if the map contains an entry with the given key.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.has
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/has
    pub(crate) fn has(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        const JS_ZERO: &JsValue = &JsValue::Rational(0f64);

        let key = args.get_or_undefined(0);
        let key = match key {
            JsValue::Rational(r) => {
                if r.is_zero() {
                    JS_ZERO
                } else {
                    key
                }
            }
            _ => key,
        };

        // 1. Let M be the this value.
        if let JsValue::Object(ref object) = this {
            // 2. Perform ? RequireInternalSlot(M, [[MapData]]).
            // 3. Let entries be the List that is M.[[MapData]].
            if let Some(map) = object.borrow().as_map_ref() {
                // 4. For each Record { [[Key]], [[Value]] } p of entries, do
                // a. If p.[[Key]] is not empty and SameValueZero(p.[[Key]], key) is true, return true.
                // 5. Return false.
                return Ok(map.contains_key(key).into());
            }
        }

        Err(JsNativeError::typ()
            .with_message("'this' is not a Map")
            .into())
    }

    /// `Map.prototype.forEach( callbackFn [ , thisArg ] )`
    ///
    /// Executes the provided callback function for each key-value pair in the map.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.foreach
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/forEach
    pub(crate) fn for_each(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Perform ? RequireInternalSlot(M, [[MapData]]).
        let map = this
            .as_object()
            .filter(|obj| obj.is_map())
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a Map"))?;

        // 3. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback = args.get_or_undefined(0);
        let callback = callback.as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message(format!("{} is not a function", callback.display()))
        })?;

        let this_arg = args.get_or_undefined(1);

        // NOTE:
        //
        // forEach does not directly mutate the object on which it is called but
        // the object may be mutated by the calls to callbackfn. Each entry of a
        // map's [[MapData]] is only visited once. New keys added after the call
        // to forEach begins are visited. A key will be revisited if it is deleted
        // after it has been visited and then re-added before the forEach call completes.
        // Keys that are deleted after the call to forEach begins and before being visited
        // are not visited unless the key is added again before the forEach call completes.
        let _lock = map
            .borrow_mut()
            .as_map_mut()
            .expect("checked that `this` was a map")
            .lock(map.clone());

        // 4. Let entries be the List that is M.[[MapData]].
        // 5. For each Record { [[Key]], [[Value]] } e of entries, do
        let mut index = 0;
        loop {
            let arguments = {
                let map = map.borrow();
                let map = map.as_map_ref().expect("checked that `this` was a map");
                if index < map.full_len() {
                    map.get_index(index)
                        .map(|(k, v)| [v.clone(), k.clone(), this.clone()])
                } else {
                    // 6. Return undefined.
                    return Ok(JsValue::undefined());
                }
            };

            // a. If e.[[Key]] is not empty, then
            if let Some(arguments) = arguments {
                // i. Perform ? Call(callbackfn, thisArg, « e.[[Value]], e.[[Key]], M »).
                callback.call(this_arg, &arguments, context)?;
            }

            index += 1;
        }
    }

    /// `Map.prototype.values()`
    ///
    /// Returns a new Iterator object that contains the values for each element in the Map object in insertion order.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.values
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/values
    pub(crate) fn values(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Return ? CreateMapIterator(M, value).
        MapIterator::create_map_iterator(this, PropertyNameKind::Value, context)
    }
}

/// `AddEntriesFromIterable`
///
/// Allows adding entries to a map from any object that has a `@@Iterator` field.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-add-entries-from-iterable
pub(crate) fn add_entries_from_iterable(
    target: &JsObject,
    iterable: &JsValue,
    adder: &JsValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. If IsCallable(adder) is false, throw a TypeError exception.
    let adder = adder.as_callable().ok_or_else(|| {
        JsNativeError::typ().with_message("property `set` of `NewTarget` is not callable")
    })?;

    // 2. Let iteratorRecord be ? GetIterator(iterable).
    let iterator_record = iterable.get_iterator(context, None, None)?;

    // 3. Repeat,
    loop {
        // a. Let next be ? IteratorStep(iteratorRecord).
        let next = iterator_record.step(context)?;

        // b. If next is false, return target.
        // c. Let nextItem be ? IteratorValue(next).
        let Some(next_item) = next else {
            return Ok(target.clone().into());
        };

        let next_item = next_item.value(context)?;

        let Some(next_item) = next_item.as_object() else {
            // d. If Type(nextItem) is not Object, then
            // i. Let error be ThrowCompletion(a newly created TypeError object).
            let err = Err(JsNativeError::typ()
                .with_message("cannot get key and value from primitive item of `iterable`")
                .into());

            // ii. Return ? IteratorClose(iteratorRecord, error).
            return iterator_record.close(err, context);
        };

        // e. Let k be Get(nextItem, "0").
        // f. IfAbruptCloseIterator(k, iteratorRecord).
        let key = match next_item.get(0, context) {
            Ok(val) => val,
            err => return iterator_record.close(err, context),
        };

        // g. Let v be Get(nextItem, "1").
        // h. IfAbruptCloseIterator(v, iteratorRecord).
        let value = match next_item.get(1, context) {
            Ok(val) => val,
            err => return iterator_record.close(err, context),
        };

        // i. Let status be Call(adder, target, « k, v »).
        let status = adder.call(&target.clone().into(), &[key, value], context);

        // j. IfAbruptCloseIterator(status, iteratorRecord).
        if status.is_err() {
            return iterator_record.close(status, context);
        }
    }
}
