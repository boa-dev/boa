# Symbol as WeakMap/WeakSet Keys

TC39 proposal: allow unique + registered symbols as WeakMap/WeakSet keys (well-known symbols are NOT allowed).

## Done
- [x] `is_registered_symbol(sym)` - true if created via `Symbol.for()`
- [x] `is_well_known_symbol(sym)` - true if one of the 13 spec-defined symbols
- [x] `is_unique_symbol(sym)` - true if not registered and not well-known
- [x] Tests for all three functions

## Next Steps

### 1. Add `can_be_held_weakly(value)` helper
A value can be held weakly if it is:
- An object, OR
- A symbol that is NOT well-known (i.e. unique or registered)

```rust
// in builtins/symbol/mod.rs or a shared util
pub(crate) fn can_be_held_weakly(value: &JsValue) -> bool {
    value.is_object()
        || value.as_symbol().map_or(false, |s| !is_well_known_symbol(&s))
}
```

### 2. Update WeakMap.set() validation
File: `core/engine/src/builtins/weak_map/mod.rs` ~line 259
- Replace `key.as_object()` guard with `can_be_held_weakly(&key)`
- Error message should match spec: "WeakMap.set: target must be an object or a non-registered symbol"

### 3. Update WeakSet.add() validation
File: `core/engine/src/builtins/weak_set/mod.rs` ~line 155
- Same replacement as WeakMap.set()

### 4. Update WeakMap.getOrInsert() and getOrInsertComputed()
File: `core/engine/src/builtins/weak_map/mod.rs` ~lines 305-317, 359-371
- These already have TODO comments noting the need for `CanBeHeldWeakly`
- Apply same fix as WeakMap.set()

### 5. Refactor internal storage to support symbol keys
**This is the major task.**
- `NativeWeakMap` currently uses `boa_gc::WeakMap<ErasedVTableObject, JsValue>`
- `NativeWeakSet` uses `boa_gc::WeakSet<ErasedVTableObject>`
- Symbols cannot be stored in these directly - they're not `ErasedVTableObject`
- Options:
  - A) Wrap symbol in a `JsObject` internally (simpler but hacky)
  - B) Change the storage to an enum key: `WeakKey::Object(ErasedVTableObject) | WeakKey::Symbol(JsSymbol)`
  - C) Use a separate `DashMap<JsSymbol, JsValue>` alongside the existing WeakMap (but this won't be truly weak for symbols)

### 6. Add tests
- `WeakMap.set()` with a unique symbol key
- `WeakMap.set()` with a registered symbol key
- `WeakMap.set()` with a well-known symbol (should throw TypeError)
- `WeakSet.add()` equivalents
- Round-trip: `set` then `get`/`has`/`delete` with symbol keys
