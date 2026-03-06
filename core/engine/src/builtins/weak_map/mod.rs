//! Boa's implementation of ECMAScript's `WeakMap` builtin object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-weakmap-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap

use std::collections::HashMap;

use crate::{
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        map::add_entries_from_iterable,
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

pub(crate) type NativeWeakMap = boa_gc::WeakMap<ErasedVTableObject, JsValue>;

#[derive(Trace, Finalize, JsData)]
pub(crate) struct WeakMapData {
    pub(crate) objects: NativeWeakMap,
    // values keyed by the unique hash of a non-registered `JsSymbol`
    //
    // SAFETY:
    // symbol hashes are globally unique and monotonically increasing,
    // so we store only the hash to avoid rooting it over time
    //
    // TODO: this map isn't truly weak, values stay alive until the entire map dies.
    // real weak tracking needs Gc support for symbols
    pub(crate) symbols: HashMap<u64, JsValue>,
}

impl WeakMapData {
    pub(crate) fn new() -> Self {
        Self {
            objects: NativeWeakMap::new(),
            symbols: HashMap::new(),
        }
    }
}

// Returns `true` if `value` is an Object or a non registered Symbol,
// per the `CanBeHeldWeakly` spec: https://tc39.es/ecma262/#sec-canbeheldweakly
#[inline]
pub(crate) fn can_be_held_weakly(value: &JsValue) -> bool {
    // 1. If v is an Object, return true
    if value.is_object() {
        return true;
    }
    // 2. If v is a Symbol and KeyForSymbol(v) is undefined, return true.
    if let Some(sym) = value.as_symbol() {
        return !sym.is_registered();
    }
    // 3. Return false
    false
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
            WeakMapData::new(),
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
            .ok_or_else(|| JsNativeError::typ().with_message("WeakMap: 'add' is not a function"))?;

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
        let mut data = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<WeakMapData>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.delete: called with non-object value")
            })?;

        // 3. Let key be the first argument.
        let key = args.get_or_undefined(0);

        // 4. If key is an Object, remove from GC weak map
        if let Some(key_obj) = key.as_object() {
            return Ok(data.objects.remove(key_obj.inner()).is_some().into());
        }

        // 5. If key is a non-registered Symbol, remove from symbol map
        if let Some(sym) = key.as_symbol() {
            if !sym.is_registered() {
                return Ok(data.symbols.remove(&sym.hash()).is_some().into());
            }
        }

        // 6. Otherwise key cannot be held weakly, return false.
        Ok(false.into())
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
        let data = object
            .as_ref()
            .and_then(JsObject::downcast_ref::<WeakMapData>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.get: called with non-object value")
            })?;

        let key = args.get_or_undefined(0);

        // 3. If key is an Object, look up in GC weak map.
        if let Some(key_obj) = key.as_object() {
            return Ok(data.objects.get(key_obj.inner()).unwrap_or_default());
        }

        // 4. If key is a non-registered Symbol, look up in symbol map.
        if let Some(sym) = key.as_symbol() {
            if !sym.is_registered() {
                return Ok(data.symbols.get(&sym.hash()).cloned().unwrap_or_default());
            }
        }

        // 5. Otherwise return undefined
        Ok(JsValue::undefined())
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
        let data = object
            .as_ref()
            .and_then(JsObject::downcast_ref::<WeakMapData>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.has: called with non-object value")
            })?;

        let key = args.get_or_undefined(0);

        // 3. If key is an Object, check GC weak map.
        if let Some(key_obj) = key.as_object() {
            return Ok(data.objects.contains_key(key_obj.inner()).into());
        }

        // 4. If key is a non-registered Symbol, check symbol map.
        if let Some(sym) = key.as_symbol() {
            if !sym.is_registered() {
                return Ok(data.symbols.contains_key(&sym.hash()).into());
            }
        }

        // 5. Otherwise return false
        Ok(false.into())
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
        let mut data = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<WeakMapData>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.set: called with non-object value")
            })?;

        // 3. If CanBeHeldWeakly(key) is false, throw a TypeError exception.
        let key = args.get_or_undefined(0);
        if !can_be_held_weakly(key) {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakMap.set: invalid key type `{}`, expected an object or non-registered symbol",
                    key.type_of()
                ))
                .into());
        }

        let value = args.get_or_undefined(1).clone();

        // 4. Dispatch to the appropriate backing store
        if let Some(key_obj) = key.as_object() {
            data.objects.insert(key_obj.inner(), value);
        } else if let Some(sym) = key.as_symbol() {
            data.symbols.insert(sym.hash(), value);
        }

        // 5. Return M.
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
        let mut data = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<WeakMapData>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("WeakMap.getOrInsert: called with non-object value")
            })?;

        // 3. If CanBeHeldWeakly(key) is false, throw a TypeError exception.
        let key_val = args.get_or_undefined(0);
        if !can_be_held_weakly(key_val) {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakMap.getOrInsert: invalid key type `{}`, expected an object or non-registered symbol",
                    key_val.type_of()
                ))
                .into());
        }

        // 4. For each Record { [[Key]], [[Value]] }, check if key already exists
        if let Some(key_obj) = key_val.as_object() {
            if let Some(existing) = data.objects.get(key_obj.inner()) {
                return Ok(existing);
            }
            let value = args.get_or_undefined(1).clone();
            data.objects.insert(key_obj.inner(), value.clone());
            return Ok(value);
        }

        if let Some(sym) = key_val.as_symbol() {
            let hash = sym.hash();
            if let Some(existing) = data.symbols.get(&hash).cloned() {
                return Ok(existing);
            }
            let value = args.get_or_undefined(1).clone();
            data.symbols.insert(hash, value.clone());
            return Ok(value);
        }

        unreachable!("can_be_held_weakly ensures key is object or non-registered symbol")
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
        let data = object
            .as_ref()
            .and_then(JsObject::downcast_ref::<WeakMapData>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("WeakMap.getOrInsertComputed: called with non-object value")
            })?;

        // 3. If CanBeHeldWeakly(key) is false, throw a TypeError exception.
        let key_value = args.get_or_undefined(0).clone();
        if !can_be_held_weakly(&key_value) {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakMap.getOrInsertComputed: invalid key type `{}`, expected an object or non-registered symbol",
                    key_value.type_of()
                ))
                .into());
        }

        // 4. If IsCallable(callback) is false, throw a TypeError exception.
        let Some(callback_fn) = args.get_or_undefined(1).as_callable() else {
            return Err(JsNativeError::typ()
                .with_message("Method WeakMap.prototype.getOrInsertComputed called with non-callable callback function")
                .into());
        };

        // 5. Object key path: check if the key already exists otherwise compute it
        if let Some(key_obj) = key_value.as_object() {
            if let Some(existing) = data.objects.get(key_obj.inner()) {
                return Ok(existing);
            }

            // Release the borrow before calling back into the engine
            drop(data);

            // 6. Compute new value
            let value = callback_fn.call(
                &JsValue::undefined(),
                std::slice::from_ref(&key_value),
                context,
            )?;

            let object2 = this.as_object();
            let mut data2 = object2
                .as_ref()
                .and_then(JsObject::downcast_mut::<WeakMapData>)
                .ok_or_else(|| {
                    JsNativeError::typ()
                        .with_message("WeakMap.getOrInsertComputed: called with non-object value")
                })?;
            data2.objects.insert(key_obj.inner(), value.clone());
            return Ok(value);
        }

        // 5. Symbol key path
        if let Some(sym) = key_value.as_symbol() {
            let hash = sym.hash();
            if let Some(existing) = data.symbols.get(&hash).cloned() {
                return Ok(existing);
            }

            // Release the borrow before calling back into the engine
            drop(data);

            // 6. Compute new value
            let value = callback_fn.call(
                &JsValue::undefined(),
                std::slice::from_ref(&key_value),
                context,
            )?;

            let object2 = this.as_object();
            let mut data2 = object2
                .as_ref()
                .and_then(JsObject::downcast_mut::<WeakMapData>)
                .ok_or_else(|| {
                    JsNativeError::typ()
                        .with_message("WeakMap.getOrInsertComputed: called with non-object value")
                })?;
            data2.symbols.insert(hash, value.clone());
            return Ok(value);
        }

        unreachable!("can_be_held_weakly ensures key is object or non-registered symbol")
    }
}
