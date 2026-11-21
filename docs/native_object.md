# `NativeObject` Design Document

## Overview

In the Boa JavaScript engine, a lot of built-in objects (like Maps, Sets, or Date) and user-defined host objects require holding underlying Rust data structures.

The `NativeObject` trait serves as a bridge between the JavaScript object system and raw Rust types. It allows any arbitrary piece of Rust data to be securely passed around, managed, and garbage-collected as a JavaScript object.

## The `JsData` Trait

For a Rust type to be treated as a `NativeObject`, it must implement the `JsData` trait, alongside `Any` and `Trace`.

The `JsData` trait acts as a marker for types that can be stored securely inside a `JsObject`. Furthermore, the `Trace` and `Finalize` traits (from the `boa_gc` crate) allow the Garbage Collector to manage the memory of these Rust objects seamlessly.

Many standard Rust types have blanket implementations of `JsData` provided out of the box (e.g., `String`, `Vec<T>`, numerical types, `HashMap`, etc.), allowing developers to easily wrap basic data easily.

## Memory Management and Alignment (`ObjectData<T>`)

When a `JsData` object is wrapped into a JavaScript object, it is stored inside the `ObjectData<T>` struct.

```rust
#[derive(Debug, Finalize, Trace)]
#[boa_gc(unsafe_no_drop)]
#[repr(C, align(8))]
pub(crate) struct ObjectData<T: ?Sized> {
    data: T,
}
```

A crucial detail here is the `#[repr(C, align(8))]` annotation. It forces the compiler to align the wrapped Rust data to exactly 8 bytes. `ObjectData<T>` includes a compile-time static assertion (`static_assertions::const_assert!(align_of::<Box<()>>() <= 8)`) which enforces that any `JsData` type must have an alignment of 8 bytes or less (if it's larger, it's recommended to wrap the data in a `Box<T>`).

This strict alignment is required to ensure memory safety when casting pointers back and forth inside the garbage-collected environment and ensuring the GC metadata pointers maintain 8-byte alignment requirements on all architectures.

## Integration with `JsObject`

Creating and interacting with native objects involves a few core `JsObject` methods:

### Creating a Native Object

A native object is typically constructed by combining an ECMAScript prototype and the raw Rust data using `JsObject::from_proto_and_data`:

```rust
let my_data = CustomStruct { ... };
let object = JsObject::from_proto_and_data(Some(prototype), my_data);
```

### Retrieving and Mutating the Data

Once wrapped, the underlying Rust data can be accessed safely via downcasting:

1. `object.downcast::<T>()` -> Returns `Result<JsObject<T>, Self>`, converting the object into a typed `JsObject<T>` that can be borrowed later without losing the type information.
2. `object.downcast_ref::<T>()` -> Returns `Option<&T>` representing an immutable reference to the native object data if the types match.
3. `object.downcast_mut::<T>()` -> Returns `Option<&mut T>` representing a mutable reference to the native object data.
4. `object.borrow_mut()` -> Often used when working closely with garbage collected `NativeObject`s that need interior mutability inside contexts.

By using `NativeObject`, Boa provides an ergonomic, type-safe, and memory-safe interface for extending ECMAScript objects with powerful native Rust functionality.
