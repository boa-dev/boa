//! Boa's implementation of ECMAScript's `WeakMap` builtin object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-weakmap-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap

use std::{cell::RefCell, collections::HashMap, fmt, sync::Weak};

use crate::{
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        map::add_entries_from_iterable,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{
        ErasedVTableObject, JsData, JsObject, internal_methods::get_prototype_from_constructor,
    },
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::{JsSymbol, RawJsSymbol},
};
use boa_gc::{Finalize, Gc, Trace, custom_trace};

/// Marker type that acts as a GC-managed identity token for a unique symbol.
///
/// Each unique symbol that is used as a key in a [`NativeWeakMap`] gets exactly
/// one `Gc<SymbolToken>` stored in that map's `symbol_tokens` table.
/// Because [`boa_gc::WeakMap`] uses the GC's native ephemeron algorithm, keeping
/// a token's `Gc` alive causes the associated value to be kept alive, and dropping
/// the `Gc` lets the GC release the value in the same collection cycle.
#[derive(Trace, Finalize)]
struct SymbolToken;

/// A map that holds GC-object keys (via [`boa_gc::WeakMap`]), unique-symbol keys,
/// and well-known symbol keys (via a plain [`HashMap`]).
///
/// For unique symbols we use a two-layer design that composes correctly with the GC:
///
/// - `symbol_tokens`: a `RefCell<HashMap>` mapping each symbol hash to a
///   `(Weak<RawJsSymbol>, Gc<SymbolToken>)` pair.  The `Weak` detects Arc-level
///   death from any thread; the `Gc<SymbolToken>` is the GC-visible identity.
///
/// - `symbol_data`: a [`boa_gc::WeakMap<SymbolToken, JsValue>`] that stores values
///   using the GC's native ephemeron algorithm.  A value is marked only while its
///   token key remains reachable.
///
/// During [`Trace`], dead `symbol_tokens` entries are drained (dropping their
/// `Gc<SymbolToken>`, making the token unreachable), and live tokens are marked
/// (keeping their ephemeron keys alive so the GC marks the associated values).
/// Well-known symbol entries are permanent (those symbols are never collected).
pub(crate) struct NativeWeakMap {
    objects: boa_gc::WeakMap<ErasedVTableObject, JsValue>,
    /// Lookup table: symbol hash → (Arc-weak for liveness detection, GC identity token).
    /// `RefCell` allows draining dead entries inside `Trace`, which has `&self`.
    symbol_tokens: RefCell<HashMap<u64, (Weak<RawJsSymbol>, Gc<SymbolToken>)>>,
    /// Actual data store for symbol-keyed entries, using GC ephemeron semantics.
    symbol_data: boa_gc::WeakMap<SymbolToken, JsValue>,
    /// Permanent entries for well-known symbols (e.g. `Symbol.iterator`).
    well_known: HashMap<u64, JsValue>,
}

impl fmt::Debug for NativeWeakMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeWeakMap")
            .field("symbols_len", &self.symbol_tokens.borrow().len())
            .field("well_known_len", &self.well_known.len())
            .finish_non_exhaustive()
    }
}

impl Finalize for NativeWeakMap {}

// SAFETY: `objects` and `symbol_data` are proper GC weak-maps; marking their
// containers makes the GC aware of them.  For unique symbols we use a two-step
// protocol in the trace body:
//
//  1. Mark `symbol_data` (the WeakMap container — NOT the values inside ephemerons;
//     those are handled by the GC's ephemeron algorithm after the trace phase).
//  2. Drain dead entries from `symbol_tokens`, dropping each `Gc<SymbolToken>`.
//     Dropping a `Gc` only decrements a root counter — it does not free memory;
//     deallocation happens during sweep, after all tracing completes.
//  3. Mark the `Gc<SymbolToken>` for each live entry.  The GC's ephemeron algorithm
//     then marks values whose token key is reachable and releases the rest.
//
// No other borrow of `symbol_tokens` is active while the GC trace runs (single-
// threaded engine, GC fires only at safe points), so `borrow_mut()` cannot panic.
unsafe impl Trace for NativeWeakMap {
    custom_trace!(this, mark, {
        mark(&this.objects);
        mark(&this.symbol_data);

        let mut tokens = this.symbol_tokens.borrow_mut();
        tokens.retain(
            |_, (weak, token): &mut (Weak<RawJsSymbol>, Gc<SymbolToken>)| {
                if weak.upgrade().is_some() {
                    mark(token);
                    true
                } else {
                    false // Gc<SymbolToken> dropped; GC ephemeron will not mark the value
                }
            },
        );

        this.well_known.values().for_each(|value| mark(value));
    });
}

impl JsData for NativeWeakMap {}

impl NativeWeakMap {
    pub(crate) fn new() -> Self {
        Self {
            objects: boa_gc::WeakMap::new(),
            symbol_tokens: RefCell::new(HashMap::new()),
            symbol_data: boa_gc::WeakMap::new(),
            well_known: HashMap::new(),
        }
    }

    // ── Object-keyed helpers ──────────────────────────────────────────────

    fn remove_object(&mut self, key: &Gc<ErasedVTableObject>) -> Option<JsValue> {
        self.objects.remove(key)
    }

    fn get_object(&self, key: &Gc<ErasedVTableObject>) -> Option<JsValue> {
        self.objects.get(key)
    }

    fn contains_key_object(&self, key: &Gc<ErasedVTableObject>) -> bool {
        self.objects.contains_key(key)
    }

    fn insert_object(&mut self, key: &Gc<ErasedVTableObject>, value: JsValue) {
        self.objects.insert(key, value);
    }

    // ── Symbol-keyed helpers ──────────────────────────────────────────────

    /// Remove a symbol entry.
    fn remove_symbol(&mut self, sym: &JsSymbol) -> bool {
        if sym.is_well_known() {
            self.well_known.remove(&sym.hash()).is_some()
        } else {
            let removed = self.symbol_tokens.borrow_mut().remove(&sym.hash());
            if let Some((_, token)) = removed {
                self.symbol_data.remove(&token).is_some()
            } else {
                false
            }
        }
    }

    /// Look up the value for a symbol key; returns `None` if absent or dead.
    fn get_symbol(&self, sym: &JsSymbol) -> Option<JsValue> {
        if sym.is_well_known() {
            self.well_known.get(&sym.hash()).cloned()
        } else {
            let tokens = self.symbol_tokens.borrow();
            let (weak, token) = tokens.get(&sym.hash())?;
            if weak.upgrade().is_none() {
                return None;
            }
            let token = token.clone();
            drop(tokens);
            self.symbol_data.get(&token)
        }
    }

    /// Returns `true` if the symbol key is present **and** still alive.
    fn contains_key_symbol(&self, sym: &JsSymbol) -> bool {
        if sym.is_well_known() {
            self.well_known.contains_key(&sym.hash())
        } else {
            let tokens = self.symbol_tokens.borrow();
            let Some((weak, token)) = tokens.get(&sym.hash()) else {
                return false;
            };
            if weak.upgrade().is_none() {
                return false;
            }
            let token = token.clone();
            drop(tokens);
            self.symbol_data.contains_key(&token)
        }
    }

    /// Insert or update a symbol entry.
    fn insert_symbol(&mut self, sym: &JsSymbol, value: JsValue) {
        if sym.is_well_known() {
            self.well_known.insert(sym.hash(), value);
        } else if let Some(weak) = sym.as_weak() {
            let token = {
                let mut tokens = self.symbol_tokens.borrow_mut();
                let (_, token) = tokens
                    .entry(sym.hash())
                    .or_insert_with(|| (weak, Gc::new(SymbolToken)));
                token.clone()
            };
            self.symbol_data.insert(&token, value);
        }
    }
}

#[derive(Debug, Trace, Finalize)]
pub(crate) struct WeakMap;

#[cfg(test)]
mod tests;

impl IntrinsicObject for WeakMap {
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

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for WeakMap {
    const NAME: JsString = StaticJsStrings::WEAK_MAP;

    const ATTRIBUTE: Attribute = Attribute::WRITABLE.union(Attribute::CONFIGURABLE);
}

impl BuiltInConstructor for WeakMap {
    const PROTOTYPE_STORAGE_SLOTS: usize = 7;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    /// The amount of arguments the `WeakMap` constructor takes.
    const CONSTRUCTOR_ARGUMENTS: usize = 0;

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
        // 4. If CanBeHeldWeakly(key) is false, return false.
        let key = args.get_or_undefined(0);
        if !can_be_held_weakly(key) {
            return Ok(false.into());
        }

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, then
        // i. Set p.[[Key]] to empty.
        // ii. Set p.[[Value]] to empty.
        // iii. Return true.
        // 6. Return false.
        if let Some(obj_key) = key.as_object() {
            Ok(map.remove_object(obj_key.inner()).is_some().into())
        } else if let Some(sym) = key.as_symbol() {
            Ok(map.remove_symbol(&sym).into())
        } else {
            Ok(false.into())
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
        if !can_be_held_weakly(key) {
            return Ok(JsValue::undefined());
        }

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
        // 6. Return undefined.
        if let Some(obj_key) = key.as_object() {
            Ok(map.get_object(obj_key.inner()).unwrap_or_default())
        } else if let Some(sym) = key.as_symbol() {
            Ok(map.get_symbol(&sym).unwrap_or_else(JsValue::undefined))
        } else {
            Ok(JsValue::undefined())
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
        if !can_be_held_weakly(key) {
            return Ok(false.into());
        }

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return true.
        // 6. Return false.
        if let Some(obj_key) = key.as_object() {
            Ok(map.contains_key_object(obj_key.inner()).into())
        } else if let Some(sym) = key.as_symbol() {
            Ok(map.contains_key_symbol(&sym).into())
        } else {
            Ok(false.into())
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
        if !can_be_held_weakly(key) {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakMap.set: expected target argument of type `object` or non-registered \
                     `symbol`, got target of type `{}`",
                    describe_key_type(key)
                ))
                .into());
        }

        // 5. For each Record { [[Key]], [[Value]] } p of entries, do
        // a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, then
        // i. Set p.[[Value]] to value.
        // ii. Return M.
        // 6. Let p be the Record { [[Key]]: key, [[Value]]: value }.
        // 7. Append p to entries.
        let value = args.get_or_undefined(1).clone();
        if let Some(obj_key) = key.as_object() {
            map.insert_object(obj_key.inner(), value);
        } else if let Some(sym) = key.as_symbol() {
            map.insert_symbol(&sym, value);
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
        if !can_be_held_weakly(key_val) {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakMap.getOrInsert: expected target argument of type `object` or \
                     non-registered `symbol`, got target of type `{}`",
                    describe_key_type(key_val)
                ))
                .into());
        }

        let value = args.get_or_undefined(1).clone();

        if let Some(key_obj) = key_val.as_object() {
            // 4. For each Record { [[Key]], [[Value]] } p of M.[[WeakMapData]]
            if let Some(existing) = map.borrow().data().get_object(key_obj.inner()) {
                // a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
                return Ok(existing);
            }
            // 5-6. Insert the new record with provided value and return it.
            map.borrow_mut()
                .data_mut()
                .insert_object(key_obj.inner(), value.clone());
        } else if let Some(sym) = key_val.as_symbol() {
            if let Some(existing) = map.borrow().data().get_symbol(&sym) {
                return Ok(existing);
            }
            map.borrow_mut()
                .data_mut()
                .insert_symbol(&sym, value.clone());
        }

        Ok(value)
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
        if !can_be_held_weakly(&key_value) {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "WeakMap.getOrInsertComputed: expected target argument of type `object` or \
                     non-registered `symbol`, got target of type `{}`",
                    describe_key_type(&key_value)
                ))
                .into());
        }

        // 4. If IsCallable(callback) is false, throw a TypeError exception.
        let Some(callback_fn) = args.get_or_undefined(1).as_callable() else {
            return Err(JsNativeError::typ()
                .with_message("Method WeakMap.prototype.getOrInsertComputed called with non-callable callback function")
                .into());
        };

        if let Some(key_obj) = key_value.as_object() {
            // 5. For each Record { [[Key]], [[Value]] } p of M.[[WeakMapData]]
            if let Some(existing) = map.borrow().data().get_object(key_obj.inner()) {
                // a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
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
            map.borrow_mut()
                .data_mut()
                .insert_object(key_obj.inner(), value.clone());
            Ok(value)
        } else if let Some(sym) = key_value.as_symbol() {
            if let Some(existing) = map.borrow().data().get_symbol(&sym) {
                return Ok(existing);
            }

            let value = callback_fn.call(
                &JsValue::undefined(),
                std::slice::from_ref(&key_value),
                context,
            )?;

            map.borrow_mut()
                .data_mut()
                .insert_symbol(&sym, value.clone());
            Ok(value)
        } else {
            Ok(JsValue::undefined())
        }
    }
}

/// Abstract operation `CanBeHeldWeakly ( v )`
///
/// Returns `true` if `v` may be used as a `WeakMap`/`WeakSet` key or `WeakRef` target.
/// Objects are always eligible.  All symbols are eligible **except** those created via
/// `Symbol.for()` (registered symbols), because registered symbols are unlimited in
/// number and can never be GC'd, which would cause unbounded memory growth.
/// Well-known symbols (e.g. `Symbol.iterator`) are finite in number and therefore
/// allowed, even though they are also never GC'd.
///
/// See: <https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-canbeheldweakly>
#[inline]
fn can_be_held_weakly(value: &JsValue) -> bool {
    value.is_object() || value.as_symbol().map_or(false, |sym| !sym.is_registered())
}

/// Returns a human-readable type description for use in `WeakMap`/`WeakSet` error messages.
///
/// Distinguishes registered symbols (`Symbol.for(...)`) from the generic `"symbol"` type
/// so that error messages are more actionable.
fn describe_key_type(value: &JsValue) -> String {
    if let Some(sym) = value.as_symbol() {
        if sym.is_registered() {
            return "registered symbol".to_string();
        }
        return "symbol".to_string();
    }
    format!("{}", value.type_of())
}
