//! Boa's implementation of ECMAScript's `WeakRef` object and related helpers.

mod weak_ref;

pub(crate) use weak_ref::WeakRef;

use crate::{
    builtins::symbol::is_registered_symbol, object::JsObject, symbol::JsSymbol, value::JsValue,
};

/// A value that has passed the [`CanBeHeldWeakly`][spec] check.
///
/// This enum is produced by [`can_be_held_weakly`] and ensures callers
/// always route object vs. symbol storage without repeated `if/else` chains.
///
/// [spec]: https://tc39.es/ecma262/#sec-canbeheldweakly
pub(crate) enum WeakKey {
    Object(JsObject),
    Symbol(JsSymbol),
}

/// Abstract operation [`CanBeHeldWeakly ( v )`][spec].
///
/// If `v` is suitable for use as a weak reference key, returns a
/// [`WeakKey`] wrapping the concrete object or symbol.  Returns `None`
/// for registered symbols, primitives, and other non-weakly-holdable values.
///
/// Per the spec:
/// 1. If `v` is an Object, return **true**.
/// 2. If `v` is a Symbol and `KeyForSymbol(v)` is **undefined**, return **true**.
/// 3. Return **false**.
///
/// [spec]: https://tc39.es/ecma262/#sec-canbeheldweakly
pub(crate) fn can_be_held_weakly(v: &JsValue) -> Option<WeakKey> {
    // 1. If v is an Object, return true.
    if let Some(obj) = v.as_object() {
        return Some(WeakKey::Object(obj.clone()));
    }

    // 2. If v is a Symbol and KeyForSymbol(v) is undefined, return true.
    if let Some(weak) = v
        .as_symbol()
        .filter(|sym| !is_registered_symbol(sym))
        .map(WeakKey::Symbol)
    {
        return Some(weak);
    }

    // 3. Return false.
    None
}
