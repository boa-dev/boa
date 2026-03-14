# `JsString` Design Document

## Overview

In the Boa JavaScript engine, a [`JsString`] is a reference-counted, immutable string that represents strings in ECMAScript. To optimize memory usage, `JsString` can store data in either Latin-1 (1 byte per character) or UTF-16 (2 bytes per character) encodings. 

`JsString` is designed to have a small memory footprint, being exactly the size of a single thin pointer (e.g., 8 bytes on 64-bit systems).

## Memory Layout (`vtable`)

A key aspect of `JsString` is its internal representation. The `ptr` field inside `JsString` always points to a heap allocation where the very first field is a `JsStringVTable` struct. 

```rust
pub struct JsString {
    ptr: NonNull<JsStringVTable>,
}

pub(crate) struct JsStringVTable {
    pub clone: fn(NonNull<JsStringVTable>) -> JsString,
    pub drop: fn(NonNull<JsStringVTable>),
    pub as_str: fn(NonNull<JsStringVTable>) -> JsStr<'static>,
    pub code_points: fn(NonNull<JsStringVTable>) -> CodePointsIter<'static>,
    pub refcount: fn(NonNull<JsStringVTable>) -> Option<usize>,
    pub len: usize,
    pub kind: JsStringKind,
}
```

This inline `vtable` design allows uniform dispatch for all string operations (like cloning, dropping, or converting to a slice) without branching. Because the `vtable` pointer itself is embedded directly at the start of the memory layout of the various `JsString` representations, `JsString` acts as a thin pointer instead of a fat trait object pointer.

## Representations (`JsStringKind`)

Internally, strings can be represented in multiple forms, identified by the `JsStringKind` enum:

1. **`SequenceString<Latin1>` & `SequenceString<Utf16>` (`JsStringKind::Latin1Sequence` / `JsStringKind::Utf16Sequence`)**
   A sequential memory slice of characters. When allocated, the layout includes the `vtable`, the reference count, a `PhantomData` marker to ensure invariance, and finally, a trailing slice representing the actual string data (`data: [u8; 0]`). `Latin1` uses `u8` characters, while `Utf16` uses `u16` characters. This is the most common heap-allocated representation.
2. **`SliceString` (`JsStringKind::Slice`)**
   Created when taking a substring (slice) of an existing `JsString`. Instead of copying the data, a `SliceString` holds a reference to the original (owned) `JsString`, the start/end bounds (`JsStr<'static>`), its own `vtable`, and a reference count. This avoids copying memory for substring operations.
3. **`StaticString` (`JsStringKind::Static`)**
   A static string representation that requires no allocation or reference counting. Boa defines a large number of common JavaScript strings (e.g., `"length"`, `"name"`, `"prototype"`, widely used symbols and built-in object names) at compile-time in `core/string/src/common.rs`. When creating a new `JsString`, Boa checks if it exists in the interned static array. If so, it returns a pointer to the `StaticString` instead of allocating heap memory.

## Reference Counting

For heap-allocated string representations (`SequenceString` and `SliceString`), reference counting is managed internally using a `Cell<usize>`:

```rust
pub(crate) struct SequenceString<T: InternalStringType> {
    vtable: JsStringVTable,
    refcount: Cell<usize>,
    _marker: PhantomData<fn() -> T>,
    pub(crate) data: [u8; 0],
}
```

Since the `JsString` `ptr` points directly at the structure that contains the `vtable`, `refcount`, and string data all in the same allocation, the engine achieves similar functionality to `std::rc::Rc` but completely avoids the length metadata overhead associated with the `Rc` fat pointer structure. The length configuration is cached cleanly on the `vtable` structure.
