# `JsObject` Internal Architecture and Behavior

This document provides a design overview of `JsObject` within the Boa JavaScript engine. It explains its role, internal representation, property storage model, and interactions with other parts of the engine.

## 1. Introduction and Role

In Boa, `JsObject` is the fundamental building block representing all JavaScript objects (including arrays, functions, and standard objects). It provides the interface through which the engine interacts with objects, encapsulating their data, properties, and internal methods (such as those defined by the ECMAScript specification). 

Within the engine, `JsObject` acts as a smart pointer to the actual object data allocated on the garbage-collected heap. This allows objects to be easily passed around, cloned, and mutated according to JavaScript's reference semantics.

## 2. Internal Representation

The `JsObject` structure is essentially a thin wrapper around a garbage-collected pointer (`Gc<T>`).

### `JsObject` and `VTableObject`

```rust
pub struct JsObject<T: NativeObject = ErasedObjectData> {
    inner: Gc<VTableObject<T>>,
}
```

The `inner` pointer points to a `VTableObject<T>`, which holds two critical pieces of information:
- **`vtable`**: A static reference to `InternalObjectMethods`, providing the object's essential internal methods (e.g., `[[Get]]`, `[[Set]]`, `[[Delete]]`). This allows different object types (like ordinary objects, arrays, or functions) to have custom internal behaviors.
- **`object`**: A `GcRefCell<Object<T>>`, wrapping the object data in interior mutability to allow mutable access across the engine safely.

### The `Object` Structure

The core data of the object lives here:

```rust
pub struct Object<T> {
    pub data: ObjectData<T>,
    pub properties: PropertyMap,
    pub extensible: bool,
    pub private_elements: ThinVec<PrivateElement>,
}
```

- **`data`**: Contains Rust-specific data for native objects (e.g., `OrdinaryObject` or custom `T: NativeObject` implementations).
- **`properties`**: The map of properties that the object owns.
- **`extensible`**: A boolean corresponding to the `[[Extensible]]` internal slot.
- **`private_elements`**: Storage for private class fields and methods.

## 3. Property Storage Model

Properties in a Boa `JsObject` are primarily managed through the `PropertyMap`. `PropertyMap` optimizes property access by separating how different types of properties are stored over time.

- **Shapes (Hidden Classes)**: The keys and attributes (like `Enumerable`, `Configurable`, `Writable`) of string-keyed properties are not stored directly in the object. Instead, they are stored in a `Shape`. The object's `PropertyMap` holds a reference to this shape and a flat `Vec<JsValue>` array of values. This guarantees that objects created in the same way share the same layout descriptions, improving memory efficiency and property access speed. 
- **Array-like Elements**: Number keys (array indices) are fast-tracked and handled through optimized dense storage.
- **Internal Slots**: Specification-defined internal slots are generally handled by fields on the `Object` struct itself or within the specific `data` variants (e.g., `[[Prototype]]` mapping into the internal representations).

## 4. Interactions with Other Components

### Prototypes
Prototype delegation relies heavily on the `JsObject` type structure. Finding a property involves checking the object's own `PropertyMap`, and if missing, traversing the prototype chain using the internal prototype reference. Because prototypes are themselves `JsObject`s, the engine can recursively or iteratively search up the chain, dynamically observing the `prototype` references attached to the shapes.

### Shapes (Hidden Classes)
`JsObject`s seamlessly interact with the Shapes system to provide fast property lookup properties. When a property is added, modified, or removed, the object transitions to a new shape. The `PropertyMap` updates its internal values array to match the new shape's layout offsets. For more details on the shape interactions, see the [Shapes documentation](./shapes.md).

### Garbage Collection
Boa utilizes a tracing garbage collector (`boa_gc`). Since `JsObject` wraps a `Gc` pointer, memory leaks from cyclic object references are mitigated. `VTableObject` and `Object` safely implement `Trace` and `Finalize`, allowing the garbage collector to traverse all values stored within the `PropertyMap`, `ObjectData`, and `private_elements`. Regular passes of the GC trace through the variables, and unreachable `JsObject`s are safely disposed of without developer intervention.
