//! Boa's implementation of ECMAScript's `WeakMap` builtin object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-weakmap-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap

use std::collections::HashMap;

use crate::JsData;
use crate::{
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        map::add_entries_from_iterable,
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
use boa_gc::{Finalize, Trace};

/// Native data for `WeakMap` objects with dual storage:
/// - `objects`: GC-backed weak map for object keys (uses ephemerons)
/// - `symbols`: regular `HashMap` for symbol keys (symbols use `Arc`, not GC)
#[derive(Trace, Finalize, JsData)]
pub(crate) struct NativeWeakMap {
    objects: boa_gc::WeakMap<ErasedVTableObject, JsValue>,
    #[unsafe_ignore_trace]
    symbols: HashMap<u64, (JsSymbol, JsValue)>,
}

impl NativeWeakMap {
    pub(crate) fn new() -> Self {
        Self {
            objects: boa_gc::WeakMap::new(),
            symbols: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for NativeWeakMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeWeakMap")
            .field("symbols", &self.symbols)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Trace, Finalize)]
pub(crate) struct WeakMap;

#[cfg(test)]
mod tests;

impl IntrinsicObject for WeakMap {
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
            .method(Self::delete, js_string!("delete"), 1)
            .method(Self::get, js_string!("get"), 1)
            .method(Self::has, js_string!("has"), 1)
            .method(Self::set, js_string!("set"), 2)
            .method(Self::get_or_insert, js_string!("getOrInsert"), 2)
            .method(
                Self::get_or_insert_computed,
                js_string!("getOrInsertComputed"),
                2,
            )
            .build();
    }
}

impl BuiltInObject for WeakMap {
    const NAME: JsString = StaticJsStrings::WEAK_MAP;

    const ATTRIBUTE: Attribute = Attribute::WRITABLE.union(Attribute::CONFIGURABLE);
}

impl BuiltInConstructor for WeakMap {
    /// The amount of arguments the `WeakMap` constructor takes.
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 7;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

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
        )
        .upcast();

        // 4. If iterable is either undefined or null, return map.
        let iterable = args.get_or_undefined(0);
        if iterable.is_null_or_undefined() {
            return Ok(map.into());
        }

        // 5. Let adder be ? Get(map, "set").
        // 6. If IsCallable(adder) is false, throw a TypeError exception.
        let adder = map
            .get(js_string!("set"), context)?
            .as_function()
            .ok_or_else(|| JsNativeError::typ().with_message("WeakMap: 'set' is not a function"))?;

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
        let object = this.as_object();
        let mut map = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<NativeWeakMap>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.delete: called with non-object value")
            })?;

        // 3. Let entries be M.[[WeakMapData]].
        // 4. If CanBeHeldWeakly(key) is false, return false.
        let key = args.get_or_undefined(0);
        let Some(weak_key) = can_be_held_weakly(key) else {
            return Ok(false.into());
        };

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        //   a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, then
        //     i. Set p.[[Key]] to empty.
        //     ii. Set p.[[Value]] to empty.
        //     iii. Return true.
        // 6. Return false.
        match weak_key {
            WeakKey::Object(obj) => Ok(map.objects.remove(obj.inner()).is_some().into()),
            WeakKey::Symbol(sym) => Ok(map.symbols.remove(&sym.hash()).is_some().into()),
        }
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
        let object = this.as_object();
        let map = object
            .as_ref()
            .and_then(JsObject::downcast_ref::<NativeWeakMap>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.get: called with non-object value")
            })?;

        // 3. Let entries be M.[[WeakMapData]].
        // 4. If CanBeHeldWeakly(key) is false, return undefined.
        let key = args.get_or_undefined(0);
        let Some(weak_key) = can_be_held_weakly(key) else {
            return Ok(JsValue::undefined());
        };

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        //   a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true,
        //      return p.[[Value]].
        // 6. Return undefined.
        match weak_key {
            WeakKey::Object(obj) => Ok(map.objects.get(obj.inner()).unwrap_or_default()),
            WeakKey::Symbol(sym) => Ok(map
                .symbols
                .get(&sym.hash())
                .map(|(_, v)| v.clone())
                .unwrap_or_default()),
        }
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
        let object = this.as_object();
        let map = object
            .as_ref()
            .and_then(JsObject::downcast_ref::<NativeWeakMap>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.has: called with non-object value")
            })?;

        // 3. Let entries be M.[[WeakMapData]].
        // 4. If CanBeHeldWeakly(key) is false, return false.
        let key = args.get_or_undefined(0);
        let Some(weak_key) = can_be_held_weakly(key) else {
            return Ok(false.into());
        };

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        //   a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true,
        //      return true.
        // 6. Return false.
        match weak_key {
            WeakKey::Object(obj) => Ok(map.objects.contains_key(obj.inner()).into()),
            WeakKey::Symbol(sym) => Ok(map.symbols.contains_key(&sym.hash()).into()),
        }
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
        let object = this.as_object();
        let mut map = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<NativeWeakMap>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.set: called with non-object value")
            })?;

        // 3. Let entries be M.[[WeakMapData]].
        // 4. If CanBeHeldWeakly(key) is false, throw a TypeError exception.
        let key = args.get_or_undefined(0);
        let weak_key = can_be_held_weakly(key).ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "WeakMap.set: invalid key type `{}`: cannot be held weakly",
                key.type_of()
            ))
        })?;

        let value = args.get_or_undefined(1).clone();

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        //   a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, then
        //     i. Set p.[[Value]] to value.
        //     ii. Return M.
        // 6. Let p be the Record { [[Key]]: key, [[Value]]: value }.
        // 7. Append p to entries.
        // 8. Return M.
        match weak_key {
            WeakKey::Object(obj) => {
                map.objects.insert(obj.inner(), value);
            }
            WeakKey::Symbol(sym) => {
                map.symbols.insert(sym.hash(), (sym, value));
            }
        }

        Ok(this.clone())
    }

    /// `WeakMap.prototype.getOrInsert ( key, value )`
    ///
    /// Given a key and a value, returns the existing value if it exists; otherwise inserts the
    /// provided default value and returns that value.
    ///
    /// More information:
    ///  - [Upsert proposal reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/proposal-upsert/#sec-weakmap.prototype.getOrInsert
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/getOrInsert
    pub(crate) fn get_or_insert(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Perform ? RequireInternalSlot(M, [[WeakMapData]]).
        let object = this.as_object();
        let map = object
            .and_then(|obj| obj.clone().downcast::<NativeWeakMap>().ok())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("WeakMap.getOrInsert: called with non-object value")
            })?;

        // 3. If CanBeHeldWeakly(key) is false, throw a TypeError exception.
        let key_val = args.get_or_undefined(0);
        let weak_key = can_be_held_weakly(key_val).ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "WeakMap.getOrInsert: invalid key type `{}`: cannot be held weakly",
                key_val.type_of()
            ))
        })?;

        // 4. For each Record { [[Key]], [[Value]] } p of M.[[WeakMapData]], do
        //   a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true,
        //      return p.[[Value]].
        // 5. Let p be the Record { [[Key]]: key, [[Value]]: value }.
        // 6. Append p to M.[[WeakMapData]].
        // 7. Return value.
        match weak_key {
            WeakKey::Object(obj) => {
                if let Some(existing) = map.borrow().data().objects.get(obj.inner()) {
                    return Ok(existing);
                }
                let value = args.get_or_undefined(1).clone();
                map.borrow_mut()
                    .data_mut()
                    .objects
                    .insert(obj.inner(), value.clone());
                Ok(value)
            }
            WeakKey::Symbol(sym) => {
                if let Some((_, existing)) = map.borrow().data().symbols.get(&sym.hash()) {
                    return Ok(existing.clone());
                }
                let value = args.get_or_undefined(1).clone();
                map.borrow_mut()
                    .data_mut()
                    .symbols
                    .insert(sym.hash(), (sym, value.clone()));
                Ok(value)
            }
        }
    }

    /// `WeakMap.prototype.getOrInsertComputed ( key, callback )`
    ///
    /// If the key exists, returns the existing value. Otherwise computes a new value by calling
    /// `callback` with the key, inserts it into the `WeakMap`, and returns it.
    ///
    /// More information:
    ///  - [Upsert proposal reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/proposal-upsert/#sec-weakmap.prototype.getOrInsertComputed
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/getOrInsertComputed
    pub(crate) fn get_or_insert_computed(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let M be the this value.
        // 2. Perform ? RequireInternalSlot(M, [[WeakMapData]]).
        let object = this.as_object();
        let map = object
            .and_then(|obj| obj.clone().downcast::<NativeWeakMap>().ok())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("WeakMap.getOrInsertComputed: called with non-object value")
            })?;

        // 3. If CanBeHeldWeakly(key) is false, throw a TypeError exception.
        let key_value = args.get_or_undefined(0).clone();
        let weak_key = can_be_held_weakly(&key_value).ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "WeakMap.getOrInsertComputed: invalid key type `{}`: cannot be held weakly",
                key_value.type_of()
            ))
        })?;

        // 4. If IsCallable(callback) is false, throw a TypeError exception.
        let Some(callback_fn) = args.get_or_undefined(1).as_callable() else {
            return Err(JsNativeError::typ()
                .with_message("WeakMap.getOrInsertComputed: callback is not a function")
                .into());
        };

        // 5. For each Record { [[Key]], [[Value]] } p of M.[[WeakMapData]], do
        //   a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true,
        //      return p.[[Value]].
        // 6. Let value be ? Call(callback, undefined, « key »).
        // 7. NOTE: The callback may have already inserted an entry for key.
        // 8. For each Record { [[Key]], [[Value]] } p of M.[[WeakMapData]], do
        //   a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, then
        //     i. Set p.[[Value]] to value.
        //     ii. Return value.
        // 9. Let p be the Record { [[Key]]: key, [[Value]]: value }.
        // 10. Append p to M.[[WeakMapData]].
        // 11. Return value.
        match weak_key {
            WeakKey::Object(obj) => {
                if let Some(existing) = map.borrow().data().objects.get(obj.inner()) {
                    return Ok(existing);
                }

                let value = callback_fn.call(
                    &JsValue::undefined(),
                    std::slice::from_ref(&key_value),
                    context,
                )?;

                map.borrow_mut()
                    .data_mut()
                    .objects
                    .insert(obj.inner(), value.clone());
                Ok(value)
            }
            WeakKey::Symbol(sym) => {
                if let Some((_, existing)) = map.borrow().data().symbols.get(&sym.hash()) {
                    return Ok(existing.clone());
                }

                let value = callback_fn.call(
                    &JsValue::undefined(),
                    std::slice::from_ref(&key_value),
                    context,
                )?;

                map.borrow_mut()
                    .data_mut()
                    .symbols
                    .insert(sym.hash(), (sym, value.clone()));
                Ok(value)
            }
        }
    }
}
