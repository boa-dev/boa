//! Boa's implementation of ECMAScript's `WeakMap` builtin object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-weakmap-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap

use crate::{
    builtins::{
        map::add_entries_from_iterable, BuiltInBuilder, BuiltInConstructor, BuiltInObject,
        IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, ErasedVTableObject, JsObject},
    property::Attribute,
    realm::Realm,
    string::common::StaticJsStrings,
    symbol::JsSymbol,
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_macros::js_str;
use boa_profiler::Profiler;

type NativeWeakMap = boa_gc::WeakMap<ErasedVTableObject, JsValue>;

#[derive(Debug, Trace, Finalize)]
pub(crate) struct WeakMap;

impl IntrinsicObject for WeakMap {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }

    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(Self::delete, js_str!("delete"), 1)
            .method(Self::get, js_str!("get"), 1)
            .method(Self::has, js_str!("has"), 1)
            .method(Self::set, js_str!("set"), 2)
            .build();
    }
}

impl BuiltInObject for WeakMap {
    const NAME: JsString = StaticJsStrings::WEAK_MAP;

    const ATTRIBUTE: Attribute = Attribute::WRITABLE.union(Attribute::CONFIGURABLE);
}

impl BuiltInConstructor for WeakMap {
    /// The amount of arguments the `WeakMap` constructor takes.
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::weak_map;

    /// `WeakMap ( [ iterable ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakmap-iterable
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/WeakMap
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WeakMap: cannot call constructor without `new`")
                .into());
        }

        // 2. Let map be ? OrdinaryCreateFromConstructor(NewTarget, "%WeakMap.prototype%", « [[WeakMapData]] »).
        // 3. Set map.[[WeakMapData]] to a new empty List.
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::weak_map, context)?;
        let map = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            NativeWeakMap::new(),
        );

        // 4. If iterable is either undefined or null, return map.
        let iterable = args.get_or_undefined(0);
        if iterable.is_null_or_undefined() {
            return Ok(map.into());
        }

        // 5. Let adder be ? Get(map, "set").
        let adder = map.get(js_str!("set"), context)?;

        // 6. If IsCallable(adder) is false, throw a TypeError exception.
        if !adder.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("WeakMap: 'add' is not a function")
                .into());
        }

        // 7. Return ? AddEntriesFromIterable(map, iterable, adder).
        add_entries_from_iterable(&map, iterable, &adder, context)
    }
}

impl WeakMap {
    /// `WeakMap.prototype.delete ( key )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakmap.prototype.delete
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/delete
    pub(crate) fn delete(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Perform ? RequireInternalSlot(M, [[WeakMapData]]).
        let mut map = this
            .as_object()
            .and_then(JsObject::downcast_mut::<NativeWeakMap>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.delete: called with non-object value")
            })?;

        // 3. Let entries be M.[[WeakMapData]].
        // 4. If key is not an Object, return false.
        let Some(key) = args.get_or_undefined(0).as_object() else {
            return Ok(false.into());
        };

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, then
        // i. Set p.[[Key]] to empty.
        // ii. Set p.[[Value]] to empty.
        // iii. Return true.
        // 6. Return false.
        Ok(map.remove(key.inner()).is_some().into())
    }

    /// `WeakMap.prototype.get ( key )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakmap.prototype.get
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/get
    pub(crate) fn get(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Perform ? RequireInternalSlot(M, [[WeakMapData]]).
        let map = this
            .as_object()
            .and_then(JsObject::downcast_ref::<NativeWeakMap>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.get: called with non-object value")
            })?;

        // 3. Let entries be M.[[WeakMapData]].
        // 4. If key is not an Object, return undefined.
        let Some(key) = args.get_or_undefined(0).as_object() else {
            return Ok(JsValue::undefined());
        };

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
        // 6. Return undefined.
        Ok(map.get(key.inner()).unwrap_or_default())
    }

    /// `WeakMap.prototype.has ( key )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakmap.prototype.has
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/has
    pub(crate) fn has(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Perform ? RequireInternalSlot(M, [[WeakMapData]]).
        let map = this
            .as_object()
            .and_then(JsObject::downcast_ref::<NativeWeakMap>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.has: called with non-object value")
            })?;

        // 3. Let entries be M.[[WeakMapData]].
        // 4. If key is not an Object, return false.
        let Some(key) = args.get_or_undefined(0).as_object() else {
            return Ok(false.into());
        };

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return true.
        // 6. Return false.
        Ok(map.contains_key(key.inner()).into())
    }

    /// `WeakMap.prototype.set ( key, value )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakmap.prototype.set
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/set
    pub(crate) fn set(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Perform ? RequireInternalSlot(M, [[WeakMapData]]).
        let mut map = this
            .as_object()
            .and_then(JsObject::downcast_mut::<NativeWeakMap>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.set: called with non-object value")
            })?;

        // 3. Let entries be M.[[WeakMapData]].
        // 4. If key is not an Object, throw a TypeError exception.
        let key = args.get_or_undefined(0);
        let Some(key) = key.as_object() else {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakMap.set: expected target argument of type `object`, got target of type `{}`",
                    key.type_of()
                )).into());
        };

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, then
        // i. Set p.[[Value]] to value.
        // ii. Return M.
        // 6. Let p be the Record { [[Key]]: key, [[Value]]: value }.
        // 7. Append p to entries.
        map.insert(key.inner(), args.get_or_undefined(1).clone());

        // 8. Return M.
        Ok(this.clone())
    }
}
