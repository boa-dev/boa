//! Boa's implementation of ECMAScript's `WeakMap` builtin object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-weakmap-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap

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
use rustc_hash::FxHashMap;

#[derive(Trace, Finalize, JsData)]
// Symbols are ECMAScript primitives and not GC-managed objects.
// Because of this they cannot participate in the GC weak semantics used
// for object keys. Symbol keys are therefore stored in a strong map/set
// while object keys remain weak.
pub(crate) struct NativeWeakMap {
    pub(crate) objects: boa_gc::WeakMap<ErasedVTableObject, JsValue>,
    pub(crate) symbols: FxHashMap<JsSymbol, JsValue>,
}

impl Default for NativeWeakMap {
    fn default() -> Self {
        Self {
            objects: boa_gc::WeakMap::new(),
            symbols: FxHashMap::default(),
        }
    }
}

impl NativeWeakMap {
    pub(crate) fn new() -> Self {
        Self::default()
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
        let mut map = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<NativeWeakMap>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("WeakMap.delete: called with non-object value")
            })?;

        // 3. Let entries be M.[[WeakMapData]].
        // 4. If key is not an Object, return false.
        let key = args.get_or_undefined(0);
        if !key.can_be_held_weakly() {
            return Ok(false.into());
        }

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // ...
        let has_removed = if let Some(obj) = key.as_object() {
            map.objects.remove(obj.inner()).is_some()
        } else if let Some(sym) = key.as_symbol() {
            map.symbols.remove(&sym).is_some()
        } else {
            unreachable!("key.can_be_held_weakly() returned true for non-object, non-symbol key")
        };
        Ok(has_removed.into())
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
        // 4. If key is not an Object, return undefined.
        let key = args.get_or_undefined(0);
        if !key.can_be_held_weakly() {
            return Ok(JsValue::undefined());
        }

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // ...
        let val = if let Some(obj) = key.as_object() {
            map.objects.get(obj.inner())
        } else if let Some(sym) = key.as_symbol() {
            map.symbols.get(&sym).cloned()
        } else {
            unreachable!("key.can_be_held_weakly() returned true for non-object, non-symbol key")
        };
        Ok(val.unwrap_or_default())
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
        // 4. If key is not an Object, return false.
        let key = args.get_or_undefined(0);
        if !key.can_be_held_weakly() {
            return Ok(false.into());
        }

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // ...
        let has = if let Some(obj) = key.as_object() {
            map.objects.contains_key(obj.inner())
        } else if let Some(sym) = key.as_symbol() {
            map.symbols.contains_key(&sym)
        } else {
            unreachable!("key.can_be_held_weakly() returned true for non-object, non-symbol key")
        };
        Ok(has.into())
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
        // 4. If key is not an Object, throw a TypeError exception.
        let key = args.get_or_undefined(0);
        if !key.can_be_held_weakly() {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakMap.set: expected target argument of type `object` or non-registered symbol, got target of type `{}`",
                    key.type_of()
                ))
                .into());
        }

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // ...
        let value = args.get_or_undefined(1).clone();
        if let Some(obj) = key.as_object() {
            map.objects.insert(obj.inner(), value);
        } else if let Some(sym) = key.as_symbol() {
            map.symbols.insert(sym, value);
        } else {
            unreachable!("key.can_be_held_weakly() returned true for non-object, non-symbol key")
        }

        // 8. Return M.
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
        if !key_val.can_be_held_weakly() {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakMap.getOrInsert: expected target argument of type `object` or non-registered symbol, got target of type `{}`",
                    key_val.type_of()
                ))
                .into());
        }

        // 4. For each Record { [[Key]], [[Value]] } p of M.[[WeakMapData]]
        if let Some(obj) = key_val.as_object() {
            if let Some(existing) = map.borrow().data().objects.get(obj.inner()) {
                return Ok(existing);
            }
            let value = args.get_or_undefined(1).clone();
            map.borrow_mut()
                .data_mut()
                .objects
                .insert(obj.inner(), value.clone());
            Ok(value)
        } else if let Some(sym) = key_val.as_symbol() {
            if let Some(existing) = map.borrow().data().symbols.get(&sym).cloned() {
                return Ok(existing);
            }
            let value = args.get_or_undefined(1).clone();
            map.borrow_mut()
                .data_mut()
                .symbols
                .insert(sym, value.clone());
            Ok(value)
        } else {
            unreachable!("key.can_be_held_weakly() returned true for non-object, non-symbol key")
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
        if !key_value.can_be_held_weakly() {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakMap.getOrInsertComputed: expected target argument of type `object` or non-registered symbol, got target of type `{}`",
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

        // 5. For each Record { [[Key]], [[Value]] } p of M.[[WeakMapData]]
        if let Some(obj) = key_value.as_object() {
            if let Some(existing) = map.borrow().data().objects.get(obj.inner()) {
                return Ok(existing);
            }
        } else if let Some(sym) = key_value.as_symbol()
            && let Some(existing) = map.borrow().data().symbols.get(&sym).cloned()
        {
            return Ok(existing);
        }

        // 6. Let value be ? Call(callback, undefined, « key »).
        // 7. NOTE: The WeakMap may have been modified during execution of callback.
        let value = callback_fn.call(
            &JsValue::undefined(),
            std::slice::from_ref(&key_value),
            context,
        )?;

        // 8-10. Insert or update the entry and return value.
        if let Some(obj) = key_value.as_object() {
            map.borrow_mut()
                .data_mut()
                .objects
                .insert(obj.inner(), value.clone());
        } else if let Some(sym) = key_value.as_symbol() {
            map.borrow_mut()
                .data_mut()
                .symbols
                .insert(sym, value.clone());
        } else {
            unreachable!("key.can_be_held_weakly() returned true for non-object, non-symbol key")
        }
        Ok(value)
    }
}
