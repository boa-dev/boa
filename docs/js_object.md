# JsObject Architecture

This document describes the internal architecture of `JsObject` — the core type
representing every JavaScript object in Boa.

## Overview

```
JsObject<T>                   // Handle: GC'd pointer to a heap-allocated object
  │
  └── Gc<VTableObject<T>>     // Garbage-collected reference
        │
        └── GcBox<VTableObject<T>>  // Heap allocation
              │
              ├── GcHeader          // Mark bits, reference count
              ├── VTable            // Trace/finalize/drop function pointers
              └── VTableObject<T>
                    ├── vtable: &'static InternalObjectMethods
                    └── object: GcRefCell<Object<T>>
```

A `JsObject` is a **handle** — it is `Copy + Clone` (it's just a pointer). The
actual object data lives on the GC heap. The default type parameter is
`JsObject<ErasedObjectData>`, which is the representation used inside `JsValue`.

## Key Structs

### `JsObject<T>` — `core/engine/src/object/jsobject.rs`

```rust
pub struct JsObject<T: NativeObject = ErasedObjectData> {
    inner: Gc<VTableObject<T>>,
}
```

The public API surface. All operations on JavaScript objects go through this
type. Key categories:

| Category | Methods |
|----------|---------|
| Construction | `default()`, `with_null_proto()`, `from_proto_and_data()`, `new()` |
| Type erasure | `upcast()`, `downcast::<T>()`, `downcast_unchecked::<T>()` |
| Borrowing | `borrow()`, `borrow_mut()`, `try_borrow()`, `try_borrow_mut()` |
| Prototype | `prototype()`, `set_prototype()` |
| Properties | `insert()`, `get_property()`, `properties()` |
| Type checking | `is::<T>()`, `is_callable()`, `is_constructor()`, `is_ordinary()` |
| Internal methods | `__call__()`, `__construct__()`, `to_primitive()` |
| Identity | `equals()` (pointer equality via `Gc::ptr_eq`) |

### `VTableObject<T>` — Internal wrapper

```rust
pub(crate) struct VTableObject<T: NativeObject + ?Sized> {
    vtable: &'static InternalObjectMethods,
    object: GcRefCell<Object<T>>,
}
```

Separates the object's **internal methods** ([[Get]], [[Set]], [[Delete]], etc.)
from its **data**. The vtable enables exotic objects (arrays, strings, proxies)
to override standard behavior.

### `Object<T>` — `core/engine/src/object/mod.rs`

```rust
#[repr(C)]
pub struct Object<T: ?Sized> {
    properties: PropertyMap,
    extensible: bool,
    private_elements: ThinVec<(PrivateName, PrivateElement)>,
    data: ObjectData<T>,
}
```

`#[repr(C)]` is critical: it guarantees `Object<ErasedObjectData>` and
`Object<T>` have identical field layout, making type-erased pointer casts sound.

| Field | Purpose |
|-------|---------|
| `properties` | All named and indexed properties (see PropertyMap below) |
| `extensible` | The `[[Extensible]]` internal slot |
| `private_elements` | Private class fields, methods, and accessors |
| `data` | The typed native data (e.g., `OrdinaryObject`, `JsArray`, `JsString`) |

### `ObjectData<T>` — Type-erased data storage

```rust
#[repr(C, align(8))]
pub struct ObjectData<T: ?Sized> {
    data: T,
}
```

A zero-cost wrapper that ensures stored types have at most 8-byte alignment
(a compile-time assertion enforces this). When `T = ErasedObjectData`, this
struct is zero-sized.

## PropertyMap & Shape System

Properties are stored in a two-level structure:

```
PropertyMap
  ├── shape: Shape              // Property names, attributes, slot indices
  │     ├── SharedShape         // Multiple objects share this layout
  │     │     └── Inner {
  │     │           property_table: PropertyTable,  // FxHashMap key → Slot
  │     │           prototype: JsPrototype,
  │     │           previous: Option<SharedShape>,   // Transition chain parent
  │     │           forward_transitions: cache       // → next shapes
  │     │     }
  │     └── UniqueShape         // Single-owner (builtins, overflow)
  │
  ├── storage: Vec<JsValue>     // Dense array of property values
  │
  └── indexed_properties: IndexedProperties
        ├── DenseI32(Vec<i32>)
        ├── DenseF64(Vec<f64>)
        ├── DenseElement(Vec<JsValue>)
        ├── SparseElement(HashMap<u32, JsValue>)
        └── SparseProperty(HashMap<u32, PropertyDescriptor>)
```

### Property Lookup Flow

1. If the key is an integer index → search `indexed_properties`
2. Otherwise → look up the key in `shape` to find a `Slot` (index + attributes)
3. Read the value from `storage[slot.index]`

### Shape Transitions

When a property is added, changed, or removed, the shape may transition to a
new shape. Transition chains form a tree rooted at a `SharedShape`:

```
RootSharedShape (prototype: Object.prototype, no properties)
  │
  ├── insert "x" → Shape1
  │     │
  │     ├── insert "y" → Shape2
  │     │     └── change attr → ChangeTransition(Shape2) → Shape3
  │     └── insert "z" → Shape4
  │
  └── insert "a" → Shape5
```

Objects with identical property layouts share shapes, enabling fast inline
caching. Shapes revert to `UniqueShape` when the transition chain exceeds
`TRANSITION_COUNT_MAX` (1024), or for objects that are inherently unique
(e.g., built-in constructors).

## GC Integration

```
JsObject
  └── Gc<T>                                    // GC-traced pointer
        └── GcBox<T>
              ├── GcHeader (mark bits, refcount)
              ├── VTable (trace, finalize, drop function pointers)
              └── value: T                     // e.g., VTableObject<ErasedObjectData>

GcRefCell<T>  ← interior mutability within GC'd memory
  ├── borrow() → GcRef<T>       (like Ref<T>)
  └── borrow_mut() → GcRefMut<T> (like RefMut<T>)
```

- `Gc::new(value)` allocates a `GcBox<T>` through the GC allocator
- `JsObject` derives `Trace` + `Finalize` (auto-traces the inner `Gc` pointer)
- `GcRefCell` provides runtime borrow-checking within GC-managed memory
- The collector is a generational mark-sweep with configurable nursery size

## Type Erasure (Upcast / Downcast)

Boa uses type erasure to store any `JsObject<T>` inside a `JsValue`:

```rust
// Erase: JsObject<JsArray> → JsObject
let obj: JsObject<JsArray> = JsArray::new(...);
let erased: JsObject = obj.upcast();

// Restore: JsObject → JsObject<JsArray>
let restored: JsObject<JsArray> = erased.downcast().expect("not an array");
```

This works because:
1. `Gc<T>` provides `cast_unchecked<U>()` — both `VTableObject<JsArray>` and
   `VTableObject<ErasedObjectData>` have the same layout (the type parameter
   only appears in `ObjectData<T>`, which is zero-sized for `ErasedObjectData`)
2. `Object<T>` is `#[repr(C)]` — ensures `Object<JsArray>` and
   `Object<ErasedObjectData>` share field layout
3. Type identity is checked via `TypeId` at runtime during `downcast()`

## Internal Methods (VTable)

Every `JsObject` carries a `&'static InternalObjectMethods` vtable with 14
function pointers implementing the ECMAScript internal methods:

```rust
pub struct InternalObjectMethods {
    __get_prototype_of__,
    __set_prototype_of__,
    __is_extensible__,
    __prevent_extensions__,
    __get_own_property__,
    __define_own_property__,
    __has_property__,
    __get__,
    __set__,
    __delete__,
    __own_property_keys__,
    __call__,
    __construct__,
}
```

Ordinary objects use `ORDINARY_INTERNAL_METHODS`. Exotic objects override
specific methods — for example, `JsString` overrides `__get_own_property__`
to expose character indices as virtual properties.

Types opt into custom vtables via `JsData::internal_methods()`:

```rust
pub trait JsData {
    fn internal_methods(&self) -> &'static InternalObjectMethods {
        &ORDINARY_INTERNAL_METHODS  // default
    }
}
```

## `JsData` / `NativeObject` Trait Hierarchy

```
JsData                          // Can be stored inside a JsObject
  │
  └── NativeObject: JsData + Any + Trace
        │
        └── OrdinaryObject      // Default ordinary object data
        └── JsArray             // Array exotic
        └── JsString            // String exotic
        └── JsFunction          // Callable
        └── ... (40+ built-in types)
```

`JsData` is designed to be `#[derive(JsData)]` and has a blanket implementation
for many standard library types.

## Construction Patterns

### ObjectInitializer — for ad-hoc objects

```rust
let obj = ObjectInitializer::new(&mut context)
    .property(js_string!("key"), js_string!("value"), Attribute::all())
    .function(NativeFunction::from_fn_ptr(my_fn), js_string!("func"), 0)
    .build();
```

### ConstructorBuilder — for built-in constructors

```rust
let constructor = ConstructorBuilder::new(&mut context, MyStruct::new)
    .name("MyStruct")
    .length(1)
    .method(MyStruct::method, "method", 0)
    .build();
```

### ObjectTemplate — for batched creation

Pre-computes a shared shape chain, then creates many objects with identical
layouts cheaply:

```rust
let template = ObjectTemplate::new(&shape, storage);
let obj1 = template.create(data1);
let obj2 = template.create(data2);  // shares shape with obj1
```

## Key File Reference

| Component | File |
|-----------|------|
| JsObject struct | `core/engine/src/object/jsobject.rs` |
| Object struct | `core/engine/src/object/mod.rs` |
| ObjectInitializer | `core/engine/src/object/mod.rs` |
| ConstructorBuilder | `core/engine/src/object/mod.rs` |
| PropertyMap | `core/engine/src/object/property_map.rs` |
| Shape (shared/unique) | `core/engine/src/object/shape/mod.rs` |
| SharedShape internals | `core/engine/src/object/shape/shared_shape/mod.rs` |
| UniqueShape | `core/engine/src/object/shape/unique_shape.rs` |
| PropertyTable | `core/engine/src/object/shape/property_table.rs` |
| Slot / SlotAttributes | `core/engine/src/object/shape/slot.rs` |
| ForwardTransition cache | `core/engine/src/object/shape/shared_shape/forward_transition.rs` |
| ObjectTemplate | `core/engine/src/object/shape/shared_shape/template.rs` |
| InternalObjectMethods | `core/engine/src/object/internal_methods/mod.rs` |
| JsData / NativeObject | `core/engine/src/object/datatypes.rs` |
| Gc pointer | `core/gc/src/pointers/gc.rs` |
| GcRefCell | `core/gc/src/cell.rs` |
| GcBox (heap allocation) | `core/gc/src/internals/gc_box.rs` |
| GC collector | `core/gc/src/lib.rs` |
| Shapes documentation | `docs/shapes.md` |
