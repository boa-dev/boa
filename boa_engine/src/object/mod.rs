//! Boa's representation of a JavaScript object and builtin object wrappers
//!
//! For the builtin object wrappers, please see [`object::builtins`][builtins] for implementors.

pub use jsobject::{RecursionLimiter, Ref, RefMut};
pub use operations::IntegrityLevel;
pub use property_map::*;
use thin_vec::ThinVec;

use self::{
    internal_methods::{
        arguments::ARGUMENTS_EXOTIC_INTERNAL_METHODS,
        array::ARRAY_EXOTIC_INTERNAL_METHODS,
        bound_function::{
            BOUND_CONSTRUCTOR_EXOTIC_INTERNAL_METHODS, BOUND_FUNCTION_EXOTIC_INTERNAL_METHODS,
        },
        function::{CONSTRUCTOR_INTERNAL_METHODS, FUNCTION_INTERNAL_METHODS},
        immutable_prototype::IMMUTABLE_PROTOTYPE_EXOTIC_INTERNAL_METHODS,
        integer_indexed::INTEGER_INDEXED_EXOTIC_INTERNAL_METHODS,
        module_namespace::MODULE_NAMESPACE_EXOTIC_INTERNAL_METHODS,
        proxy::{
            PROXY_EXOTIC_INTERNAL_METHODS_ALL, PROXY_EXOTIC_INTERNAL_METHODS_BASIC,
            PROXY_EXOTIC_INTERNAL_METHODS_WITH_CALL,
        },
        string::STRING_EXOTIC_INTERNAL_METHODS,
        InternalObjectMethods, ORDINARY_INTERNAL_METHODS,
    },
    shape::Shape,
};
#[cfg(feature = "intl")]
use crate::builtins::intl::{
    collator::Collator,
    date_time_format::DateTimeFormat,
    list_format::ListFormat,
    plural_rules::PluralRules,
    segmenter::{SegmentIterator, Segmenter, Segments},
};
use crate::{
    builtins::{
        array::ArrayIterator,
        array_buffer::ArrayBuffer,
        async_generator::AsyncGenerator,
        error::ErrorKind,
        function::{arguments::Arguments, FunctionKind},
        function::{arguments::ParameterMap, BoundFunction, ConstructorKind, Function},
        generator::Generator,
        iterable::AsyncFromSyncIterator,
        map::ordered_map::OrderedMap,
        map::MapIterator,
        object::for_in_iterator::ForInIterator,
        proxy::Proxy,
        regexp::RegExpStringIterator,
        set::ordered_set::OrderedSet,
        set::SetIterator,
        string::StringIterator,
        typed_array::{integer_indexed_object::IntegerIndexed, TypedArrayKind},
        DataView, Date, Promise, RegExp,
    },
    js_string,
    module::ModuleNamespace,
    native_function::NativeFunction,
    property::{Attribute, PropertyDescriptor, PropertyKey},
    string::utf16,
    Context, JsBigInt, JsString, JsSymbol, JsValue,
};

use boa_gc::{custom_trace, Finalize, Trace, WeakGc};
use std::{
    any::Any,
    fmt::{self, Debug},
    ops::{Deref, DerefMut},
};

#[cfg(test)]
mod tests;

pub(crate) mod internal_methods;

pub mod builtins;
mod jsobject;
mod operations;
mod property_map;
pub mod shape;

pub(crate) use builtins::*;

pub use jsobject::*;

pub(crate) trait JsObjectType:
    Into<JsValue> + Into<JsObject> + Deref<Target = JsObject>
{
}

/// Const `constructor`, usually set on prototypes as a key to point to their respective constructor object.
pub const CONSTRUCTOR: &[u16] = utf16!("constructor");

/// Const `prototype`, usually set on constructors as a key to point to their respective prototype object.
pub const PROTOTYPE: &[u16] = utf16!("prototype");

/// Common field names.

/// A type alias for an object prototype.
///
/// A `None` values means that the prototype is the `null` value.
pub type JsPrototype = Option<JsObject>;

/// The internal storage of an object's property values.
///
/// The [`shape::Shape`] contains the property names and attributes.
pub(crate) type ObjectStorage = Vec<JsValue>;

/// This trait allows Rust types to be passed around as objects.
///
/// This is automatically implemented when a type implements `Any` and `Trace`.
pub trait NativeObject: Any + Trace {
    /// Convert the Rust type which implements `NativeObject` to a `&dyn Any`.
    fn as_any(&self) -> &dyn Any;

    /// Convert the Rust type which implements `NativeObject` to a `&mut dyn Any`.
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<T: Any + Trace> NativeObject for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// The internal representation of a JavaScript object.
#[derive(Debug, Finalize)]
pub struct Object {
    /// The type of the object.
    kind: ObjectKind,
    /// The collection of properties contained in the object
    properties: PropertyMap,
    /// Whether it can have new properties added to it.
    pub(crate) extensible: bool,
    /// The `[[PrivateElements]]` internal slot.
    private_elements: ThinVec<(PrivateName, PrivateElement)>,
}

impl Default for Object {
    fn default() -> Self {
        Self {
            kind: ObjectKind::Ordinary,
            properties: PropertyMap::default(),
            extensible: true,
            private_elements: ThinVec::new(),
        }
    }
}

unsafe impl Trace for Object {
    boa_gc::custom_trace!(this, {
        mark(&this.kind);
        mark(&this.properties);
        for (_, element) in &this.private_elements {
            mark(element);
        }
    });
}

/// A Private Name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateName {
    /// The `[[Description]]` internal slot of the private name.
    description: JsString,

    /// The unique identifier of the private name.
    id: usize,
}

impl PrivateName {
    /// Create a new private name.
    pub(crate) const fn new(description: JsString, id: usize) -> Self {
        Self { description, id }
    }
}

/// The representation of private object elements.
#[derive(Clone, Debug, Trace, Finalize)]
pub enum PrivateElement {
    /// A private field.
    Field(JsValue),

    /// A private method.
    Method(JsObject),

    /// A private element accessor.
    Accessor {
        /// A getter function.
        getter: Option<JsObject>,

        /// A setter function.
        setter: Option<JsObject>,
    },
}

/// Defines the kind of an object and its internal methods
pub struct ObjectData {
    pub(crate) kind: ObjectKind,
    pub(crate) internal_methods: &'static InternalObjectMethods,
}

impl Debug for ObjectData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr: *const _ = self.internal_methods;
        f.debug_struct("ObjectData")
            .field("kind", &self.kind)
            .field("internal_methods", &ptr)
            .finish()
    }
}

/// Defines the different types of objects.
#[derive(Finalize)]
pub enum ObjectKind {
    /// The `AsyncFromSyncIterator` object kind.
    AsyncFromSyncIterator(AsyncFromSyncIterator),

    /// The `AsyncGenerator` object kind.
    AsyncGenerator(AsyncGenerator),

    /// The `AsyncGeneratorFunction` object kind.
    AsyncGeneratorFunction(Function),

    /// The `Array` object kind.
    Array,

    /// The `ArrayIterator` object kind.
    ArrayIterator(ArrayIterator),

    /// The `ArrayBuffer` object kind.
    ArrayBuffer(ArrayBuffer),

    /// The `Map` object kind.
    Map(OrderedMap<JsValue>),

    /// The `MapIterator` object kind.
    MapIterator(MapIterator),

    /// The `RegExp` object kind.
    RegExp(Box<RegExp>),

    /// The `RegExpStringIterator` object kind.
    RegExpStringIterator(RegExpStringIterator),

    /// The `BigInt` object kind.
    BigInt(JsBigInt),

    /// The `Boolean` object kind.
    Boolean(bool),

    /// The `DataView` object kind.
    DataView(DataView),

    /// The `ForInIterator` object kind.
    ForInIterator(ForInIterator),

    /// The `Function` object kind.
    Function(Function),

    /// The `BoundFunction` object kind.
    BoundFunction(BoundFunction),

    /// The `Generator` object kind.
    Generator(Generator),

    /// The `GeneratorFunction` object kind.
    GeneratorFunction(Function),

    /// The `Set` object kind.
    Set(OrderedSet),

    /// The `SetIterator` object kind.
    SetIterator(SetIterator),

    /// The `String` object kind.
    String(JsString),

    /// The `StringIterator` object kind.
    StringIterator(StringIterator),

    /// The `Number` object kind.
    Number(f64),

    /// The `Symbol` object kind.
    Symbol(JsSymbol),

    /// The `Error` object kind.
    Error(ErrorKind),

    /// The ordinary object kind.
    Ordinary,

    /// The `Proxy` object kind.
    Proxy(Proxy),

    /// The `Date` object kind.
    Date(Date),

    /// The `Global` object kind.
    Global,

    /// The arguments exotic object kind.
    Arguments(Arguments),

    /// The rust native object kind.
    NativeObject(Box<dyn NativeObject>),

    /// The integer-indexed exotic object kind.
    IntegerIndexed(IntegerIndexed),

    /// The `Promise` object kind.
    Promise(Promise),

    /// The `WeakRef` object kind.
    WeakRef(WeakGc<VTableObject>),

    /// The `WeakMap` object kind.
    WeakMap(boa_gc::WeakMap<VTableObject, JsValue>),

    /// The `WeakSet` object kind.
    WeakSet(boa_gc::WeakMap<VTableObject, ()>),

    /// The `ModuleNamespace` object kind.
    ModuleNamespace(ModuleNamespace),

    /// The `Intl.Collator` object kind.
    #[cfg(feature = "intl")]
    Collator(Box<Collator>),

    /// The `Intl.DateTimeFormat` object kind.
    #[cfg(feature = "intl")]
    DateTimeFormat(Box<DateTimeFormat>),

    /// The `Intl.ListFormat` object kind.
    #[cfg(feature = "intl")]
    ListFormat(Box<ListFormat>),

    /// The `Intl.Locale` object kind.
    #[cfg(feature = "intl")]
    Locale(Box<icu_locid::Locale>),

    /// The `Intl.Segmenter` object kind.
    #[cfg(feature = "intl")]
    Segmenter(Segmenter),

    /// The `Segments` object kind.
    #[cfg(feature = "intl")]
    Segments(Segments),

    /// The `Segment Iterator` object kind.
    #[cfg(feature = "intl")]
    SegmentIterator(SegmentIterator),

    /// The `PluralRules` object kind.
    #[cfg(feature = "intl")]
    PluralRules(PluralRules),
}

unsafe impl Trace for ObjectKind {
    custom_trace! {this, {
        match this {
            Self::AsyncFromSyncIterator(a) => mark(a),
            Self::ArrayIterator(i) => mark(i),
            Self::ArrayBuffer(b) => mark(b),
            Self::Map(m) => mark(m),
            Self::MapIterator(i) => mark(i),
            Self::RegExpStringIterator(i) => mark(i),
            Self::DataView(v) => mark(v),
            Self::ForInIterator(i) => mark(i),
            Self::Function(f) | Self::GeneratorFunction(f) | Self::AsyncGeneratorFunction(f) => mark(f),
            Self::BoundFunction(f) => mark(f),
            Self::Generator(g) => mark(g),
            Self::Set(s) => mark(s),
            Self::SetIterator(i) => mark(i),
            Self::StringIterator(i) => mark(i),
            Self::Proxy(p) => mark(p),
            Self::Arguments(a) => mark(a),
            Self::NativeObject(o) => mark(o),
            Self::IntegerIndexed(i) => mark(i),
            Self::Promise(p) => mark(p),
            Self::AsyncGenerator(g) => mark(g),
            Self::WeakRef(wr) => mark(wr),
            Self::WeakMap(wm) => mark(wm),
            Self::WeakSet(ws) => mark(ws),
            Self::ModuleNamespace(m) => mark(m),
            #[cfg(feature = "intl")]
            Self::DateTimeFormat(f) => mark(f),
            #[cfg(feature = "intl")]
            Self::Collator(co) => mark(co),
            #[cfg(feature = "intl")]
            Self::Segments(seg) => mark(seg),
            #[cfg(feature = "intl")]
            Self::SegmentIterator(it) => mark(it),
            #[cfg(feature = "intl")]
            Self::ListFormat(_)
            | Self::Locale(_)
            | Self::Segmenter(_)
            | Self::PluralRules(_) => {}
            Self::RegExp(_)
            | Self::BigInt(_)
            | Self::Boolean(_)
            | Self::String(_)
            | Self::Date(_)
            | Self::Array
            | Self::Error(_)
            | Self::Ordinary
            | Self::Global
            | Self::Number(_)
            | Self::Symbol(_) => {}
        }
    }}
}

impl ObjectData {
    /// Create the immutable `%Object.prototype%` object data
    pub(crate) fn object_prototype() -> Self {
        Self {
            kind: ObjectKind::Ordinary,
            internal_methods: &IMMUTABLE_PROTOTYPE_EXOTIC_INTERNAL_METHODS,
        }
    }

    /// Create the `AsyncFromSyncIterator` object data
    #[must_use]
    pub fn async_from_sync_iterator(async_from_sync_iterator: AsyncFromSyncIterator) -> Self {
        Self {
            kind: ObjectKind::AsyncFromSyncIterator(async_from_sync_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `AsyncGenerator` object data
    #[must_use]
    pub fn async_generator(async_generator: AsyncGenerator) -> Self {
        Self {
            kind: ObjectKind::AsyncGenerator(async_generator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `AsyncGeneratorFunction` object data
    #[must_use]
    pub fn async_generator_function(function: Function) -> Self {
        Self {
            internal_methods: &FUNCTION_INTERNAL_METHODS,
            kind: ObjectKind::GeneratorFunction(function),
        }
    }

    /// Create the `Array` object data and reference its exclusive internal methods
    #[must_use]
    pub fn array() -> Self {
        Self {
            kind: ObjectKind::Array,
            internal_methods: &ARRAY_EXOTIC_INTERNAL_METHODS,
        }
    }

    /// Create the `ArrayIterator` object data
    #[must_use]
    pub fn array_iterator(array_iterator: ArrayIterator) -> Self {
        Self {
            kind: ObjectKind::ArrayIterator(array_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `ArrayBuffer` object data
    #[must_use]
    pub fn array_buffer(array_buffer: ArrayBuffer) -> Self {
        Self {
            kind: ObjectKind::ArrayBuffer(array_buffer),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Map` object data
    #[must_use]
    pub fn map(map: OrderedMap<JsValue>) -> Self {
        Self {
            kind: ObjectKind::Map(map),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `MapIterator` object data
    #[must_use]
    pub fn map_iterator(map_iterator: MapIterator) -> Self {
        Self {
            kind: ObjectKind::MapIterator(map_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `RegExp` object data
    #[must_use]
    pub fn reg_exp(reg_exp: Box<RegExp>) -> Self {
        Self {
            kind: ObjectKind::RegExp(reg_exp),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `RegExpStringIterator` object data
    #[must_use]
    pub fn reg_exp_string_iterator(reg_exp_string_iterator: RegExpStringIterator) -> Self {
        Self {
            kind: ObjectKind::RegExpStringIterator(reg_exp_string_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `BigInt` object data
    #[must_use]
    pub fn big_int(big_int: JsBigInt) -> Self {
        Self {
            kind: ObjectKind::BigInt(big_int),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Boolean` object data
    #[must_use]
    pub fn boolean(boolean: bool) -> Self {
        Self {
            kind: ObjectKind::Boolean(boolean),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `DataView` object data
    #[must_use]
    pub fn data_view(data_view: DataView) -> Self {
        Self {
            kind: ObjectKind::DataView(data_view),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Promise` object data
    #[must_use]
    pub fn promise(promise: Promise) -> Self {
        Self {
            kind: ObjectKind::Promise(promise),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `ForInIterator` object data
    #[must_use]
    pub fn for_in_iterator(for_in_iterator: ForInIterator) -> Self {
        Self {
            kind: ObjectKind::ForInIterator(for_in_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Function` object data
    #[must_use]
    pub fn function(function: Function, constructor: bool) -> Self {
        Self {
            internal_methods: if constructor {
                &CONSTRUCTOR_INTERNAL_METHODS
            } else {
                &FUNCTION_INTERNAL_METHODS
            },
            kind: ObjectKind::Function(function),
        }
    }

    /// Create the `BoundFunction` object data
    #[must_use]
    pub fn bound_function(bound_function: BoundFunction, constructor: bool) -> Self {
        Self {
            kind: ObjectKind::BoundFunction(bound_function),
            internal_methods: if constructor {
                &BOUND_CONSTRUCTOR_EXOTIC_INTERNAL_METHODS
            } else {
                &BOUND_FUNCTION_EXOTIC_INTERNAL_METHODS
            },
        }
    }

    /// Create the `Generator` object data
    #[must_use]
    pub fn generator(generator: Generator) -> Self {
        Self {
            kind: ObjectKind::Generator(generator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `GeneratorFunction` object data
    #[must_use]
    pub fn generator_function(function: Function) -> Self {
        Self {
            internal_methods: &FUNCTION_INTERNAL_METHODS,
            kind: ObjectKind::GeneratorFunction(function),
        }
    }

    /// Create the `Set` object data
    #[must_use]
    pub fn set(set: OrderedSet) -> Self {
        Self {
            kind: ObjectKind::Set(set),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `SetIterator` object data
    #[must_use]
    pub fn set_iterator(set_iterator: SetIterator) -> Self {
        Self {
            kind: ObjectKind::SetIterator(set_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `String` object data and reference its exclusive internal methods
    #[must_use]
    pub fn string(string: JsString) -> Self {
        Self {
            kind: ObjectKind::String(string),
            internal_methods: &STRING_EXOTIC_INTERNAL_METHODS,
        }
    }

    /// Create the `StringIterator` object data
    #[must_use]
    pub fn string_iterator(string_iterator: StringIterator) -> Self {
        Self {
            kind: ObjectKind::StringIterator(string_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Number` object data
    #[must_use]
    pub fn number(number: f64) -> Self {
        Self {
            kind: ObjectKind::Number(number),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Symbol` object data
    #[must_use]
    pub fn symbol(symbol: JsSymbol) -> Self {
        Self {
            kind: ObjectKind::Symbol(symbol),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Error` object data
    pub(crate) fn error(error: ErrorKind) -> Self {
        Self {
            kind: ObjectKind::Error(error),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Ordinary` object data
    #[must_use]
    pub fn ordinary() -> Self {
        Self {
            kind: ObjectKind::Ordinary,
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Proxy` object data
    #[must_use]
    pub fn proxy(proxy: Proxy, call: bool, construct: bool) -> Self {
        Self {
            kind: ObjectKind::Proxy(proxy),
            internal_methods: if call && construct {
                &PROXY_EXOTIC_INTERNAL_METHODS_ALL
            } else if call {
                &PROXY_EXOTIC_INTERNAL_METHODS_WITH_CALL
            } else {
                &PROXY_EXOTIC_INTERNAL_METHODS_BASIC
            },
        }
    }

    /// Create the `Date` object data
    #[must_use]
    pub fn date(date: Date) -> Self {
        Self {
            kind: ObjectKind::Date(date),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Arguments` object data
    #[must_use]
    pub fn arguments(arguments: Arguments) -> Self {
        Self {
            internal_methods: if matches!(arguments, Arguments::Unmapped) {
                &ORDINARY_INTERNAL_METHODS
            } else {
                &ARGUMENTS_EXOTIC_INTERNAL_METHODS
            },
            kind: ObjectKind::Arguments(arguments),
        }
    }

    /// Creates the `WeakRef` object data
    #[must_use]
    pub fn weak_ref(weak_ref: WeakGc<VTableObject>) -> Self {
        Self {
            kind: ObjectKind::WeakRef(weak_ref),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `WeakMap` object data
    #[must_use]
    pub fn weak_map(weak_map: boa_gc::WeakMap<VTableObject, JsValue>) -> Self {
        Self {
            kind: ObjectKind::WeakMap(weak_map),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `WeakSet` object data
    #[must_use]
    pub fn weak_set(weak_set: boa_gc::WeakMap<VTableObject, ()>) -> Self {
        Self {
            kind: ObjectKind::WeakSet(weak_set),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `NativeObject` object data
    #[must_use]
    pub fn native_object<T: NativeObject>(native_object: T) -> Self {
        Self {
            kind: ObjectKind::NativeObject(Box::new(native_object)),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Creates the `IntegerIndexed` object data
    #[must_use]
    pub fn integer_indexed(integer_indexed: IntegerIndexed) -> Self {
        Self {
            kind: ObjectKind::IntegerIndexed(integer_indexed),
            internal_methods: &INTEGER_INDEXED_EXOTIC_INTERNAL_METHODS,
        }
    }

    /// Creates the `ModuleNamespace` object data
    #[must_use]
    pub fn module_namespace(namespace: ModuleNamespace) -> Self {
        Self {
            kind: ObjectKind::ModuleNamespace(namespace),
            internal_methods: &MODULE_NAMESPACE_EXOTIC_INTERNAL_METHODS,
        }
    }

    /// Create the `Collator` object data
    #[cfg(feature = "intl")]
    #[must_use]
    pub fn collator(date_time_fmt: Collator) -> Self {
        Self {
            kind: ObjectKind::Collator(Box::new(date_time_fmt)),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `DateTimeFormat` object data
    #[cfg(feature = "intl")]
    #[must_use]
    pub fn date_time_format(date_time_fmt: Box<DateTimeFormat>) -> Self {
        Self {
            kind: ObjectKind::DateTimeFormat(date_time_fmt),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `ListFormat` object data
    #[cfg(feature = "intl")]
    #[must_use]
    pub fn list_format(list_format: ListFormat) -> Self {
        Self {
            kind: ObjectKind::ListFormat(Box::new(list_format)),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Locale` object data
    #[cfg(feature = "intl")]
    #[must_use]
    pub fn locale(locale: icu_locid::Locale) -> Self {
        Self {
            kind: ObjectKind::Locale(Box::new(locale)),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Segmenter` object data
    #[cfg(feature = "intl")]
    #[must_use]
    pub fn segmenter(segmenter: Segmenter) -> Self {
        Self {
            kind: ObjectKind::Segmenter(segmenter),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Segments` object data
    #[cfg(feature = "intl")]
    #[must_use]
    pub fn segments(segments: Segments) -> Self {
        Self {
            kind: ObjectKind::Segments(segments),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `SegmentIterator` object data
    #[cfg(feature = "intl")]
    #[must_use]
    pub fn segment_iterator(segment_iterator: SegmentIterator) -> Self {
        Self {
            kind: ObjectKind::SegmentIterator(segment_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `PluralRules` object data
    #[cfg(feature = "intl")]
    #[must_use]
    pub fn plural_rules(plural_rules: PluralRules) -> Self {
        Self {
            kind: ObjectKind::PluralRules(plural_rules),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }
}

impl Debug for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AsyncFromSyncIterator(_) => "AsyncFromSyncIterator",
            Self::AsyncGenerator(_) => "AsyncGenerator",
            Self::AsyncGeneratorFunction(_) => "AsyncGeneratorFunction",
            Self::Array => "Array",
            Self::ArrayIterator(_) => "ArrayIterator",
            Self::ArrayBuffer(_) => "ArrayBuffer",
            Self::ForInIterator(_) => "ForInIterator",
            Self::Function(_) => "Function",
            Self::BoundFunction(_) => "BoundFunction",
            Self::Generator(_) => "Generator",
            Self::GeneratorFunction(_) => "GeneratorFunction",
            Self::RegExp(_) => "RegExp",
            Self::RegExpStringIterator(_) => "RegExpStringIterator",
            Self::Map(_) => "Map",
            Self::MapIterator(_) => "MapIterator",
            Self::Set(_) => "Set",
            Self::SetIterator(_) => "SetIterator",
            Self::String(_) => "String",
            Self::StringIterator(_) => "StringIterator",
            Self::Symbol(_) => "Symbol",
            Self::Error(_) => "Error",
            Self::Ordinary => "Ordinary",
            Self::Proxy(_) => "Proxy",
            Self::Boolean(_) => "Boolean",
            Self::Number(_) => "Number",
            Self::BigInt(_) => "BigInt",
            Self::Date(_) => "Date",
            Self::Global => "Global",
            Self::Arguments(_) => "Arguments",
            Self::NativeObject(_) => "NativeObject",
            Self::IntegerIndexed(_) => "TypedArray",
            Self::DataView(_) => "DataView",
            Self::Promise(_) => "Promise",
            Self::WeakRef(_) => "WeakRef",
            Self::WeakMap(_) => "WeakMap",
            Self::WeakSet(_) => "WeakSet",
            Self::ModuleNamespace(_) => "ModuleNamespace",
            #[cfg(feature = "intl")]
            Self::Collator(_) => "Collator",
            #[cfg(feature = "intl")]
            Self::DateTimeFormat(_) => "DateTimeFormat",
            #[cfg(feature = "intl")]
            Self::ListFormat(_) => "ListFormat",
            #[cfg(feature = "intl")]
            Self::Locale(_) => "Locale",
            #[cfg(feature = "intl")]
            Self::Segmenter(_) => "Segmenter",
            #[cfg(feature = "intl")]
            Self::Segments(_) => "Segments",
            #[cfg(feature = "intl")]
            Self::SegmentIterator(_) => "SegmentIterator",
            #[cfg(feature = "intl")]
            Self::PluralRules(_) => "PluralRules",
        })
    }
}

impl Object {
    /// Returns a mutable reference to the kind of an object.
    pub(crate) fn kind_mut(&mut self) -> &mut ObjectKind {
        &mut self.kind
    }

    /// Returns the shape of the object.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.properties.shape
    }

    /// Returns the kind of the object.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> &ObjectKind {
        &self.kind
    }

    /// Checks if it's an `AsyncFromSyncIterator` object.
    #[inline]
    #[must_use]
    pub const fn is_async_from_sync_iterator(&self) -> bool {
        matches!(self.kind, ObjectKind::AsyncFromSyncIterator(_))
    }

    /// Returns a reference to the `AsyncFromSyncIterator` data on the object.
    #[inline]
    #[must_use]
    pub const fn as_async_from_sync_iterator(&self) -> Option<&AsyncFromSyncIterator> {
        match self.kind {
            ObjectKind::AsyncFromSyncIterator(ref async_from_sync_iterator) => {
                Some(async_from_sync_iterator)
            }
            _ => None,
        }
    }

    /// Checks if it's an `AsyncGenerator` object.
    #[inline]
    #[must_use]
    pub const fn is_async_generator(&self) -> bool {
        matches!(self.kind, ObjectKind::AsyncGenerator(_))
    }

    /// Returns a reference to the async generator data on the object.
    #[inline]
    #[must_use]
    pub const fn as_async_generator(&self) -> Option<&AsyncGenerator> {
        match self.kind {
            ObjectKind::AsyncGenerator(ref async_generator) => Some(async_generator),
            _ => None,
        }
    }

    /// Returns a mutable reference to the async generator data on the object.
    #[inline]
    pub fn as_async_generator_mut(&mut self) -> Option<&mut AsyncGenerator> {
        match self.kind {
            ObjectKind::AsyncGenerator(ref mut async_generator) => Some(async_generator),
            _ => None,
        }
    }

    /// Checks if the object is a `Array` object.
    #[inline]
    #[must_use]
    pub const fn is_array(&self) -> bool {
        matches!(self.kind, ObjectKind::Array)
    }

    #[inline]
    #[must_use]
    pub(crate) const fn has_viewed_array_buffer(&self) -> bool {
        self.is_typed_array() || self.is_data_view()
    }

    /// Checks if the object is a `DataView` object.
    #[inline]
    #[must_use]
    pub const fn is_data_view(&self) -> bool {
        matches!(self.kind, ObjectKind::DataView(_))
    }

    /// Checks if the object is a `ArrayBuffer` object.
    #[inline]
    #[must_use]
    pub const fn is_array_buffer(&self) -> bool {
        matches!(self.kind, ObjectKind::ArrayBuffer(_))
    }

    /// Gets the array buffer data if the object is a `ArrayBuffer`.
    #[inline]
    #[must_use]
    pub const fn as_array_buffer(&self) -> Option<&ArrayBuffer> {
        match &self.kind {
            ObjectKind::ArrayBuffer(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Gets the mutable array buffer data if the object is a `ArrayBuffer`.
    #[inline]
    pub fn as_array_buffer_mut(&mut self) -> Option<&mut ArrayBuffer> {
        match &mut self.kind {
            ObjectKind::ArrayBuffer(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Checks if the object is a `ArrayIterator` object.
    #[inline]
    #[must_use]
    pub const fn is_array_iterator(&self) -> bool {
        matches!(self.kind, ObjectKind::ArrayIterator(_))
    }

    /// Gets the array-iterator data if the object is a `ArrayIterator`.
    #[inline]
    #[must_use]
    pub const fn as_array_iterator(&self) -> Option<&ArrayIterator> {
        match self.kind {
            ObjectKind::ArrayIterator(ref iter) => Some(iter),
            _ => None,
        }
    }

    /// Gets the mutable array-iterator data if the object is a `ArrayIterator`.
    #[inline]
    pub fn as_array_iterator_mut(&mut self) -> Option<&mut ArrayIterator> {
        match &mut self.kind {
            ObjectKind::ArrayIterator(iter) => Some(iter),
            _ => None,
        }
    }

    /// Gets the mutable string-iterator data if the object is a `StringIterator`.
    #[inline]
    pub fn as_string_iterator_mut(&mut self) -> Option<&mut StringIterator> {
        match &mut self.kind {
            ObjectKind::StringIterator(iter) => Some(iter),
            _ => None,
        }
    }

    /// Gets the mutable regexp-string-iterator data if the object is a `RegExpStringIterator`.
    #[inline]
    pub fn as_regexp_string_iterator_mut(&mut self) -> Option<&mut RegExpStringIterator> {
        match &mut self.kind {
            ObjectKind::RegExpStringIterator(iter) => Some(iter),
            _ => None,
        }
    }

    /// Gets the for-in-iterator data if the object is a `ForInIterator`.
    #[inline]
    #[must_use]
    pub const fn as_for_in_iterator(&self) -> Option<&ForInIterator> {
        match &self.kind {
            ObjectKind::ForInIterator(iter) => Some(iter),
            _ => None,
        }
    }

    /// Gets the mutable for-in-iterator data if the object is a `ForInIterator`.
    #[inline]
    pub fn as_for_in_iterator_mut(&mut self) -> Option<&mut ForInIterator> {
        match &mut self.kind {
            ObjectKind::ForInIterator(iter) => Some(iter),
            _ => None,
        }
    }

    /// Checks if the object is a `Map` object.
    #[inline]
    #[must_use]
    pub const fn is_map(&self) -> bool {
        matches!(self.kind, ObjectKind::Map(_))
    }

    /// Gets the map data if the object is a `Map`.
    #[inline]
    #[must_use]
    pub const fn as_map(&self) -> Option<&OrderedMap<JsValue>> {
        match self.kind {
            ObjectKind::Map(ref map) => Some(map),
            _ => None,
        }
    }

    /// Gets the mutable map data if the object is a `Map`.
    #[inline]
    pub fn as_map_mut(&mut self) -> Option<&mut OrderedMap<JsValue>> {
        match &mut self.kind {
            ObjectKind::Map(map) => Some(map),
            _ => None,
        }
    }

    /// Checks if the object is a `MapIterator` object.
    #[inline]
    #[must_use]
    pub const fn is_map_iterator(&self) -> bool {
        matches!(self.kind, ObjectKind::MapIterator(_))
    }

    /// Gets the map iterator data if the object is a `MapIterator`.
    #[inline]
    #[must_use]
    pub const fn as_map_iterator_ref(&self) -> Option<&MapIterator> {
        match &self.kind {
            ObjectKind::MapIterator(iter) => Some(iter),
            _ => None,
        }
    }

    /// Gets the mutable map iterator data if the object is a `MapIterator`.
    #[inline]
    pub fn as_map_iterator_mut(&mut self) -> Option<&mut MapIterator> {
        match &mut self.kind {
            ObjectKind::MapIterator(iter) => Some(iter),
            _ => None,
        }
    }

    /// Checks if the object is a `Set` object.
    #[inline]
    #[must_use]
    pub const fn is_set(&self) -> bool {
        matches!(self.kind, ObjectKind::Set(_))
    }

    /// Gets the set data if the object is a `Set`.
    #[inline]
    #[must_use]
    pub const fn as_set(&self) -> Option<&OrderedSet> {
        match self.kind {
            ObjectKind::Set(ref set) => Some(set),
            _ => None,
        }
    }

    /// Gets the mutable set data if the object is a `Set`.
    #[inline]
    pub fn as_set_mut(&mut self) -> Option<&mut OrderedSet> {
        match &mut self.kind {
            ObjectKind::Set(set) => Some(set),
            _ => None,
        }
    }

    /// Checks if the object is a `SetIterator` object.
    #[inline]
    #[must_use]
    pub const fn is_set_iterator(&self) -> bool {
        matches!(self.kind, ObjectKind::SetIterator(_))
    }

    /// Gets the mutable set iterator data if the object is a `SetIterator`.
    #[inline]
    pub fn as_set_iterator_mut(&mut self) -> Option<&mut SetIterator> {
        match &mut self.kind {
            ObjectKind::SetIterator(iter) => Some(iter),
            _ => None,
        }
    }

    /// Checks if the object is a `String` object.
    #[inline]
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self.kind, ObjectKind::String(_))
    }

    /// Gets the string data if the object is a `String`.
    #[inline]
    #[must_use]
    pub fn as_string(&self) -> Option<JsString> {
        match self.kind {
            ObjectKind::String(ref string) => Some(string.clone()),
            _ => None,
        }
    }

    /// Checks if the object is a `Function` object.
    #[inline]
    #[must_use]
    pub const fn is_function(&self) -> bool {
        matches!(self.kind, ObjectKind::Function(_))
    }

    /// Gets the function data if the object is a `Function`.
    #[inline]
    #[must_use]
    pub const fn as_function(&self) -> Option<&Function> {
        match self.kind {
            ObjectKind::Function(ref function) | ObjectKind::GeneratorFunction(ref function) => {
                Some(function)
            }
            _ => None,
        }
    }

    /// Gets the mutable function data if the object is a `Function`.
    #[inline]
    pub fn as_function_mut(&mut self) -> Option<&mut Function> {
        match self.kind {
            ObjectKind::Function(ref mut function)
            | ObjectKind::GeneratorFunction(ref mut function) => Some(function),
            _ => None,
        }
    }

    /// Gets the bound function data if the object is a `BoundFunction`.
    #[inline]
    #[must_use]
    pub const fn as_bound_function(&self) -> Option<&BoundFunction> {
        match self.kind {
            ObjectKind::BoundFunction(ref bound_function) => Some(bound_function),
            _ => None,
        }
    }

    /// Checks if the object is a `Generator` object.
    #[inline]
    #[must_use]
    pub const fn is_generator(&self) -> bool {
        matches!(self.kind, ObjectKind::Generator(_))
    }

    /// Gets the generator data if the object is a `Generator`.
    #[inline]
    #[must_use]
    pub const fn as_generator(&self) -> Option<&Generator> {
        match self.kind {
            ObjectKind::Generator(ref generator) => Some(generator),
            _ => None,
        }
    }

    /// Gets the mutable generator data if the object is a `Generator`.
    #[inline]
    pub fn as_generator_mut(&mut self) -> Option<&mut Generator> {
        match self.kind {
            ObjectKind::Generator(ref mut generator) => Some(generator),
            _ => None,
        }
    }

    /// Checks if the object is a `Symbol` object.
    #[inline]
    #[must_use]
    pub const fn is_symbol(&self) -> bool {
        matches!(self.kind, ObjectKind::Symbol(_))
    }

    /// Gets the error data if the object is a `Symbol`.
    #[inline]
    #[must_use]
    pub fn as_symbol(&self) -> Option<JsSymbol> {
        match self.kind {
            ObjectKind::Symbol(ref symbol) => Some(symbol.clone()),
            _ => None,
        }
    }

    /// Checks if the object is a `Error` object.
    #[inline]
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.kind, ObjectKind::Error(_))
    }

    /// Gets the error data if the object is a `Error`.
    #[inline]
    #[must_use]
    pub const fn as_error(&self) -> Option<ErrorKind> {
        match self.kind {
            ObjectKind::Error(e) => Some(e),
            _ => None,
        }
    }

    /// Checks if the object is a `Boolean` object.
    #[inline]
    #[must_use]
    pub const fn is_boolean(&self) -> bool {
        matches!(self.kind, ObjectKind::Boolean(_))
    }

    /// Gets the boolean data if the object is a `Boolean`.
    #[inline]
    #[must_use]
    pub const fn as_boolean(&self) -> Option<bool> {
        match self.kind {
            ObjectKind::Boolean(boolean) => Some(boolean),
            _ => None,
        }
    }

    /// Checks if the object is a `Number` object.
    #[inline]
    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(self.kind, ObjectKind::Number(_))
    }

    /// Gets the number data if the object is a `Number`.
    #[inline]
    #[must_use]
    pub const fn as_number(&self) -> Option<f64> {
        match self.kind {
            ObjectKind::Number(number) => Some(number),
            _ => None,
        }
    }

    /// Checks if the object is a `BigInt` object.
    #[inline]
    #[must_use]
    pub const fn is_bigint(&self) -> bool {
        matches!(self.kind, ObjectKind::BigInt(_))
    }

    /// Gets the bigint data if the object is a `BigInt`.
    #[inline]
    #[must_use]
    pub const fn as_bigint(&self) -> Option<&JsBigInt> {
        match self.kind {
            ObjectKind::BigInt(ref bigint) => Some(bigint),
            _ => None,
        }
    }

    /// Checks if the object is a `Date` object.
    #[inline]
    #[must_use]
    pub const fn is_date(&self) -> bool {
        matches!(self.kind, ObjectKind::Date(_))
    }

    /// Gets the date data if the object is a `Date`.
    #[inline]
    #[must_use]
    pub const fn as_date(&self) -> Option<&Date> {
        match self.kind {
            ObjectKind::Date(ref date) => Some(date),
            _ => None,
        }
    }

    /// Gets the mutable date data if the object is a `Date`.
    #[inline]
    pub fn as_date_mut(&mut self) -> Option<&mut Date> {
        match self.kind {
            ObjectKind::Date(ref mut date) => Some(date),
            _ => None,
        }
    }

    /// Checks if it a `RegExp` object.
    #[inline]
    #[must_use]
    pub const fn is_regexp(&self) -> bool {
        matches!(self.kind, ObjectKind::RegExp(_))
    }

    /// Gets the regexp data if the object is a regexp.
    #[inline]
    #[must_use]
    pub const fn as_regexp(&self) -> Option<&RegExp> {
        match self.kind {
            ObjectKind::RegExp(ref regexp) => Some(regexp),
            _ => None,
        }
    }

    /// Checks if it a `TypedArray` object.
    #[inline]
    #[must_use]
    pub const fn is_typed_array(&self) -> bool {
        matches!(self.kind, ObjectKind::IntegerIndexed(_))
    }

    /// Checks if it a `Uint8Array` object.
    #[inline]
    #[must_use]
    pub const fn is_typed_uint8_array(&self) -> bool {
        if let ObjectKind::IntegerIndexed(ref int) = self.kind {
            matches!(int.typed_array_name(), TypedArrayKind::Uint8)
        } else {
            false
        }
    }

    /// Checks if it a `Int8Array` object.
    #[inline]
    #[must_use]
    pub const fn is_typed_int8_array(&self) -> bool {
        if let ObjectKind::IntegerIndexed(ref int) = self.kind {
            matches!(int.typed_array_name(), TypedArrayKind::Int8)
        } else {
            false
        }
    }

    /// Checks if it a `Uint16Array` object.
    #[inline]
    #[must_use]
    pub const fn is_typed_uint16_array(&self) -> bool {
        if let ObjectKind::IntegerIndexed(ref int) = self.kind {
            matches!(int.typed_array_name(), TypedArrayKind::Uint16)
        } else {
            false
        }
    }

    /// Checks if it a `Int16Array` object.
    #[inline]
    #[must_use]
    pub const fn is_typed_int16_array(&self) -> bool {
        if let ObjectKind::IntegerIndexed(ref int) = self.kind {
            matches!(int.typed_array_name(), TypedArrayKind::Int16)
        } else {
            false
        }
    }

    /// Checks if it a `Uint32Array` object.
    #[inline]
    #[must_use]
    pub const fn is_typed_uint32_array(&self) -> bool {
        if let ObjectKind::IntegerIndexed(ref int) = self.kind {
            matches!(int.typed_array_name(), TypedArrayKind::Uint32)
        } else {
            false
        }
    }

    /// Checks if it a `Int32Array` object.
    #[inline]
    #[must_use]
    pub const fn is_typed_int32_array(&self) -> bool {
        if let ObjectKind::IntegerIndexed(ref int) = self.kind {
            matches!(int.typed_array_name(), TypedArrayKind::Int32)
        } else {
            false
        }
    }

    /// Checks if it a `Float32Array` object.
    #[inline]
    #[must_use]
    pub const fn is_typed_float32_array(&self) -> bool {
        if let ObjectKind::IntegerIndexed(ref int) = self.kind {
            matches!(int.typed_array_name(), TypedArrayKind::Float32)
        } else {
            false
        }
    }

    /// Checks if it a `Float64Array` object.
    #[inline]
    #[must_use]
    pub const fn is_typed_float64_array(&self) -> bool {
        if let ObjectKind::IntegerIndexed(ref int) = self.kind {
            matches!(int.typed_array_name(), TypedArrayKind::Float64)
        } else {
            false
        }
    }

    /// Gets the data view data if the object is a `DataView`.
    #[inline]
    #[must_use]
    pub const fn as_data_view(&self) -> Option<&DataView> {
        match &self.kind {
            ObjectKind::DataView(data_view) => Some(data_view),
            _ => None,
        }
    }

    /// Gets the mutable data view data if the object is a `DataView`.
    #[inline]
    pub fn as_data_view_mut(&mut self) -> Option<&mut DataView> {
        match &mut self.kind {
            ObjectKind::DataView(data_view) => Some(data_view),
            _ => None,
        }
    }

    /// Checks if it is an `Arguments` object.
    #[inline]
    #[must_use]
    pub const fn is_arguments(&self) -> bool {
        matches!(self.kind, ObjectKind::Arguments(_))
    }

    /// Gets the mapped arguments data if this is a mapped arguments object.
    #[inline]
    #[must_use]
    pub const fn as_mapped_arguments(&self) -> Option<&ParameterMap> {
        match self.kind {
            ObjectKind::Arguments(Arguments::Mapped(ref args)) => Some(args),
            _ => None,
        }
    }

    /// Gets the mutable mapped arguments data if this is a mapped arguments object.
    #[inline]
    pub fn as_mapped_arguments_mut(&mut self) -> Option<&mut ParameterMap> {
        match self.kind {
            ObjectKind::Arguments(Arguments::Mapped(ref mut args)) => Some(args),
            _ => None,
        }
    }

    /// Gets the typed array data (integer indexed object) if this is a typed array.
    #[inline]
    #[must_use]
    pub const fn as_typed_array(&self) -> Option<&IntegerIndexed> {
        match self.kind {
            ObjectKind::IntegerIndexed(ref integer_indexed_object) => Some(integer_indexed_object),
            _ => None,
        }
    }

    /// Gets the typed array data (integer indexed object) if this is a typed array.
    #[inline]
    pub fn as_typed_array_mut(&mut self) -> Option<&mut IntegerIndexed> {
        match self.kind {
            ObjectKind::IntegerIndexed(ref mut integer_indexed_object) => {
                Some(integer_indexed_object)
            }
            _ => None,
        }
    }

    /// Checks if it an ordinary object.
    #[inline]
    #[must_use]
    pub const fn is_ordinary(&self) -> bool {
        matches!(self.kind, ObjectKind::Ordinary)
    }

    /// Checks if it's an proxy object.
    #[inline]
    #[must_use]
    pub const fn is_proxy(&self) -> bool {
        matches!(self.kind, ObjectKind::Proxy(_))
    }

    /// Gets the proxy data if the object is a `Proxy`.
    #[inline]
    #[must_use]
    pub const fn as_proxy(&self) -> Option<&Proxy> {
        match self.kind {
            ObjectKind::Proxy(ref proxy) => Some(proxy),
            _ => None,
        }
    }

    /// Gets the mutable proxy data if the object is a `Proxy`.
    #[inline]
    pub fn as_proxy_mut(&mut self) -> Option<&mut Proxy> {
        match self.kind {
            ObjectKind::Proxy(ref mut proxy) => Some(proxy),
            _ => None,
        }
    }

    /// Gets the weak map data if the object is a `WeakMap`.
    #[inline]
    #[must_use]
    pub const fn as_weak_map(&self) -> Option<&boa_gc::WeakMap<VTableObject, JsValue>> {
        match self.kind {
            ObjectKind::WeakMap(ref weak_map) => Some(weak_map),
            _ => None,
        }
    }

    /// Gets the mutable weak map data if the object is a `WeakMap`.
    #[inline]
    pub fn as_weak_map_mut(&mut self) -> Option<&mut boa_gc::WeakMap<VTableObject, JsValue>> {
        match self.kind {
            ObjectKind::WeakMap(ref mut weak_map) => Some(weak_map),
            _ => None,
        }
    }

    /// Gets the weak set data if the object is a `WeakSet`.
    #[inline]
    #[must_use]
    pub const fn as_weak_set(&self) -> Option<&boa_gc::WeakMap<VTableObject, ()>> {
        match self.kind {
            ObjectKind::WeakSet(ref weak_set) => Some(weak_set),
            _ => None,
        }
    }

    /// Gets the mutable weak set data if the object is a `WeakSet`.
    #[inline]
    pub fn as_weak_set_mut(&mut self) -> Option<&mut boa_gc::WeakMap<VTableObject, ()>> {
        match self.kind {
            ObjectKind::WeakSet(ref mut weak_set) => Some(weak_set),
            _ => None,
        }
    }

    /// Gets the prototype instance of this object.
    #[inline]
    #[must_use]
    pub fn prototype(&self) -> JsPrototype {
        self.properties.shape.prototype()
    }

    /// Sets the prototype instance of the object.
    ///
    /// [More information][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-invariants-of-the-essential-internal-methods
    #[track_caller]
    pub fn set_prototype<O: Into<JsPrototype>>(&mut self, prototype: O) -> bool {
        let prototype = prototype.into();
        if self.extensible {
            self.properties.shape = self.properties.shape.change_prototype_transition(prototype);
            true
        } else {
            // If target is non-extensible, [[SetPrototypeOf]] must return false
            // unless V is the SameValue as the target's observed [[GetPrototypeOf]] value.
            self.prototype() == prototype
        }
    }

    /// Returns `true` if it holds an Rust type that implements `NativeObject`.
    #[inline]
    #[must_use]
    pub const fn is_native_object(&self) -> bool {
        matches!(self.kind, ObjectKind::NativeObject(_))
    }

    /// Gets the native object data if the object is a `NativeObject`.
    #[inline]
    #[must_use]
    pub fn as_native_object(&self) -> Option<&dyn NativeObject> {
        match self.kind {
            ObjectKind::NativeObject(ref object) => Some(object.as_ref()),
            _ => None,
        }
    }

    /// Checks if it is a `Promise` object.
    #[inline]
    #[must_use]
    pub const fn is_promise(&self) -> bool {
        matches!(self.kind, ObjectKind::Promise(_))
    }

    /// Gets the promise data if the object is a `Promise`.
    #[inline]
    #[must_use]
    pub const fn as_promise(&self) -> Option<&Promise> {
        match self.kind {
            ObjectKind::Promise(ref promise) => Some(promise),
            _ => None,
        }
    }

    /// Gets the mutable promise data if the object is a `Promise`.
    #[inline]
    pub fn as_promise_mut(&mut self) -> Option<&mut Promise> {
        match self.kind {
            ObjectKind::Promise(ref mut promise) => Some(promise),
            _ => None,
        }
    }

    /// Gets the `WeakRef` data if the object is a `WeakRef`.
    #[inline]
    #[must_use]
    pub const fn as_weak_ref(&self) -> Option<&WeakGc<VTableObject>> {
        match self.kind {
            ObjectKind::WeakRef(ref weak_ref) => Some(weak_ref),
            _ => None,
        }
    }

    /// Gets a reference to the module namespace if the object is a `ModuleNamespace`.
    #[inline]
    #[must_use]
    pub const fn as_module_namespace(&self) -> Option<&ModuleNamespace> {
        match &self.kind {
            ObjectKind::ModuleNamespace(ns) => Some(ns),
            _ => None,
        }
    }

    /// Gets a mutable reference module namespace if the object is a `ModuleNamespace`.
    #[inline]
    pub fn as_module_namespace_mut(&mut self) -> Option<&mut ModuleNamespace> {
        match &mut self.kind {
            ObjectKind::ModuleNamespace(ns) => Some(ns),
            _ => None,
        }
    }

    /// Gets the `Collator` data if the object is a `Collator`.
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn as_collator(&self) -> Option<&Collator> {
        match self.kind {
            ObjectKind::Collator(ref collator) => Some(collator),
            _ => None,
        }
    }

    /// Gets a mutable reference to the `Collator` data if the object is a `Collator`.
    #[inline]
    #[cfg(feature = "intl")]
    pub fn as_collator_mut(&mut self) -> Option<&mut Collator> {
        match self.kind {
            ObjectKind::Collator(ref mut collator) => Some(collator),
            _ => None,
        }
    }

    /// Checks if it is a `Locale` object.
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn is_locale(&self) -> bool {
        matches!(self.kind, ObjectKind::Locale(_))
    }

    /// Gets the `Locale` data if the object is a `Locale`.
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn as_locale(&self) -> Option<&icu_locid::Locale> {
        match self.kind {
            ObjectKind::Locale(ref locale) => Some(locale),
            _ => None,
        }
    }

    /// Gets the `ListFormat` data if the object is a `ListFormat`.
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn as_list_format(&self) -> Option<&ListFormat> {
        match self.kind {
            ObjectKind::ListFormat(ref lf) => Some(lf),
            _ => None,
        }
    }

    /// Checks if it is a `Segmenter` object.
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn is_segmenter(&self) -> bool {
        matches!(self.kind, ObjectKind::Segmenter(_))
    }

    /// Gets the `Segmenter` data if the object is a `Segmenter`.
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn as_segmenter(&self) -> Option<&Segmenter> {
        match self.kind {
            ObjectKind::Segmenter(ref seg) => Some(seg),
            _ => None,
        }
    }

    /// Gets the `Segments` data if the object is a `Segments`.
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn as_segments(&self) -> Option<&Segments> {
        match self.kind {
            ObjectKind::Segments(ref seg) => Some(seg),
            _ => None,
        }
    }

    /// Gets the `SegmentIterator` data if the object is a `SegmentIterator`.
    #[inline]
    #[cfg(feature = "intl")]
    pub fn as_segment_iterator_mut(&mut self) -> Option<&mut SegmentIterator> {
        match &mut self.kind {
            ObjectKind::SegmentIterator(it) => Some(it),
            _ => None,
        }
    }

    /// Gets the `PluralRules` data if the object is a `PluralRules`.
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn as_plural_rules(&self) -> Option<&PluralRules> {
        match &self.kind {
            ObjectKind::PluralRules(it) => Some(it),
            _ => None,
        }
    }

    /// Gets a mutable reference to the `PluralRules` data if the object is a `PluralRules`.
    #[inline]
    #[cfg(feature = "intl")]
    pub fn as_plural_rules_mut(&mut self) -> Option<&mut PluralRules> {
        match &mut self.kind {
            ObjectKind::PluralRules(plural_rules) => Some(plural_rules),
            _ => None,
        }
    }

    /// Return `true` if it is a native object and the native type is `T`.
    #[must_use]
    pub fn is<T>(&self) -> bool
    where
        T: NativeObject,
    {
        match self.kind {
            ObjectKind::NativeObject(ref object) => object.deref().as_any().is::<T>(),
            _ => false,
        }
    }

    /// Downcast a reference to the object,
    /// if the object is type native object type `T`.
    #[must_use]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: NativeObject,
    {
        match self.kind {
            ObjectKind::NativeObject(ref object) => object.deref().as_any().downcast_ref::<T>(),
            _ => None,
        }
    }

    /// Downcast a mutable reference to the object,
    /// if the object is type native object type `T`.
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: NativeObject,
    {
        match self.kind {
            ObjectKind::NativeObject(ref mut object) => {
                object.deref_mut().as_mut_any().downcast_mut::<T>()
            }
            _ => None,
        }
    }

    /// Returns the properties of the object.
    #[inline]
    #[must_use]
    pub const fn properties(&self) -> &PropertyMap {
        &self.properties
    }

    #[inline]
    pub(crate) fn properties_mut(&mut self) -> &mut PropertyMap {
        &mut self.properties
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name, then `true` is returned
    /// otherwise, `false` is returned.
    pub(crate) fn insert<K, P>(&mut self, key: K, property: P) -> bool
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.properties.insert(&key.into(), property.into())
    }

    /// Helper function for property removal without checking if it's configurable.
    ///
    /// Returns `true` if the property was removed, `false` otherwise.
    #[inline]
    pub(crate) fn remove(&mut self, key: &PropertyKey) -> bool {
        self.properties.remove(key)
    }

    /// Append a private element to an object.
    pub(crate) fn append_private_element(&mut self, name: PrivateName, element: PrivateElement) {
        if let PrivateElement::Accessor { getter, setter } = &element {
            for (key, value) in &mut self.private_elements {
                if name == *key {
                    if let PrivateElement::Accessor {
                        getter: existing_getter,
                        setter: existing_setter,
                    } = value
                    {
                        if existing_getter.is_none() {
                            *existing_getter = getter.clone();
                        }
                        if existing_setter.is_none() {
                            *existing_setter = setter.clone();
                        }
                        return;
                    }
                }
            }
        }

        self.private_elements.push((name, element));
    }
}

/// The functions binding.
///
/// Specifies what is the name of the function object (`name` property),
/// and the binding name of the function object which can be different
/// from the function name.
///
/// The only way to construct this is with the `From` trait.
///
/// There are two implementations:
///  - From a single type `T` which implements `Into<FunctionBinding>` which sets the binding
/// name and the function name to the same value
///  - From a tuple `(B: Into<PropertyKey>, N: AsRef<str>)` the `B` is the binding name
/// and the `N` is the function name.
#[derive(Debug, Clone)]
pub struct FunctionBinding {
    pub(crate) binding: PropertyKey,
    pub(crate) name: JsString,
}

impl From<&str> for FunctionBinding {
    #[inline]
    fn from(name: &str) -> Self {
        let name: JsString = name.into();

        Self {
            binding: name.clone().into(),
            name,
        }
    }
}

impl From<String> for FunctionBinding {
    #[inline]
    fn from(name: String) -> Self {
        let name: JsString = name.into();

        Self {
            binding: name.clone().into(),
            name,
        }
    }
}

impl From<JsString> for FunctionBinding {
    #[inline]
    fn from(name: JsString) -> Self {
        Self {
            binding: name.clone().into(),
            name,
        }
    }
}

impl<B, N> From<(B, N)> for FunctionBinding
where
    B: Into<PropertyKey>,
    N: Into<JsString>,
{
    fn from((binding, name): (B, N)) -> Self {
        Self {
            binding: binding.into(),
            name: name.into(),
        }
    }
}

/// Builder for creating native function objects
#[derive(Debug)]
pub struct FunctionObjectBuilder<'ctx, 'host> {
    context: &'ctx mut Context<'host>,
    function: NativeFunction,
    constructor: Option<ConstructorKind>,
    name: JsString,
    length: usize,
}

impl<'ctx, 'host> FunctionObjectBuilder<'ctx, 'host> {
    /// Create a new `FunctionBuilder` for creating a native function.
    #[inline]
    pub fn new(context: &'ctx mut Context<'host>, function: NativeFunction) -> Self {
        Self {
            context,
            function,
            constructor: None,
            name: js_string!(),
            length: 0,
        }
    }

    /// Specify the name property of object function object.
    ///
    /// The default is `""` (empty string).
    #[must_use]
    pub fn name<N>(mut self, name: N) -> Self
    where
        N: Into<JsString>,
    {
        self.name = name.into();
        self
    }

    /// Specify the length property of object function object.
    ///
    /// How many arguments this function takes.
    ///
    /// The default is `0`.
    #[inline]
    #[must_use]
    pub const fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    /// Specify whether the object function object can be called with `new` keyword.
    ///
    /// The default is `false`.
    #[must_use]
    pub fn constructor(mut self, yes: bool) -> Self {
        self.constructor = yes.then_some(ConstructorKind::Base);
        self
    }

    /// Build the function object.
    #[must_use]
    pub fn build(self) -> JsFunction {
        let function = Function::new(
            FunctionKind::Native {
                function: self.function,
                constructor: self.constructor,
            },
            self.context.realm().clone(),
        );
        let object = self.context.intrinsics().templates().function().create(
            ObjectData::function(function, self.constructor.is_some()),
            vec![self.length.into(), self.name.into()],
        );

        JsFunction::from_object_unchecked(object)
    }
}

/// Builder for creating objects with properties.
///
/// # Examples
///
/// ```
/// # use boa_engine::{
/// #     Context,
/// #     JsValue,
/// #     NativeFunction,
/// #     object::ObjectInitializer,
/// #     property::Attribute
/// # };
/// let mut context = Context::default();
/// let object = ObjectInitializer::new(&mut context)
///     .property("hello", "world", Attribute::all())
///     .property(1, 1, Attribute::all())
///     .function(NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::undefined())), "func", 0)
///     .build();
/// ```
///
/// The equivalent in JavaScript would be:
/// ```text
/// let object = {
///     hello: "world",
///     "1": 1,
///     func: function() {}
/// }
/// ```
#[derive(Debug)]
pub struct ObjectInitializer<'ctx, 'host> {
    context: &'ctx mut Context<'host>,
    object: JsObject,
}

impl<'ctx, 'host> ObjectInitializer<'ctx, 'host> {
    /// Create a new `ObjectBuilder`.
    #[inline]
    pub fn new(context: &'ctx mut Context<'host>) -> Self {
        let object = JsObject::with_object_proto(context.intrinsics());
        Self { context, object }
    }

    /// Create a new `ObjectBuilder` with custom [`NativeObject`] data.
    pub fn with_native<T: NativeObject>(data: T, context: &'ctx mut Context<'host>) -> Self {
        let object = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            ObjectData::native_object(data),
        );
        Self { context, object }
    }

    /// Add a function to the object.
    pub fn function<B>(&mut self, function: NativeFunction, binding: B, length: usize) -> &mut Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = FunctionObjectBuilder::new(self.context, function)
            .name(binding.name)
            .length(length)
            .constructor(false)
            .build();

        self.object.borrow_mut().insert(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Add a property to the object.
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.object.borrow_mut().insert(key, property);
        self
    }

    /// Add new accessor property to the object.
    ///
    /// # Panics
    ///
    /// If both getter or setter are [`None`].
    pub fn accessor<K>(
        &mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        // Accessors should have at least one function.
        assert!(set.is_some() || get.is_some());

        let property = PropertyDescriptor::builder()
            .maybe_get(get)
            .maybe_set(set)
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.object.borrow_mut().insert(key, property);
        self
    }

    /// Build the object.
    #[inline]
    pub fn build(&mut self) -> JsObject {
        self.object.clone()
    }

    /// Gets the context used to create the object.
    #[inline]
    pub fn context(&mut self) -> &mut Context<'host> {
        self.context
    }
}

/// Builder for creating constructors objects, like `Array`.
#[derive(Debug)]
pub struct ConstructorBuilder<'ctx, 'host> {
    context: &'ctx mut Context<'host>,
    function: NativeFunction,
    constructor_object: Object,
    has_prototype_property: bool,
    prototype: Object,
    name: JsString,
    length: usize,
    callable: bool,
    kind: Option<ConstructorKind>,
    inherit: Option<JsPrototype>,
    custom_prototype: Option<JsPrototype>,
}

impl<'ctx, 'host> ConstructorBuilder<'ctx, 'host> {
    /// Create a new `ConstructorBuilder`.
    #[inline]
    pub fn new(
        context: &'ctx mut Context<'host>,
        function: NativeFunction,
    ) -> ConstructorBuilder<'ctx, 'host> {
        Self {
            context,
            function,
            constructor_object: Object {
                kind: ObjectKind::Ordinary,
                properties: PropertyMap::default(),
                extensible: true,
                private_elements: ThinVec::new(),
            },
            prototype: Object {
                kind: ObjectKind::Ordinary,
                properties: PropertyMap::default(),
                extensible: true,
                private_elements: ThinVec::new(),
            },
            length: 0,
            name: js_string!(),
            callable: true,
            kind: Some(ConstructorKind::Base),
            inherit: None,
            custom_prototype: None,
            has_prototype_property: true,
        }
    }

    /// Add new method to the constructors prototype.
    pub fn method<B>(&mut self, function: NativeFunction, binding: B, length: usize) -> &mut Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = FunctionObjectBuilder::new(self.context, function)
            .name(binding.name)
            .length(length)
            .constructor(false)
            .build();

        self.prototype.insert(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Add new static method to the constructors object itself.
    pub fn static_method<B>(
        &mut self,
        function: NativeFunction,
        binding: B,
        length: usize,
    ) -> &mut Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = FunctionObjectBuilder::new(self.context, function)
            .name(binding.name)
            .length(length)
            .constructor(false)
            .build();

        self.constructor_object.insert(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Add new data property to the constructor's prototype.
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.prototype.insert(key, property);
        self
    }

    /// Add new static data property to the constructor object itself.
    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.constructor_object.insert(key, property);
        self
    }

    /// Add new accessor property to the constructor's prototype.
    pub fn accessor<K>(
        &mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        let property = PropertyDescriptor::builder()
            .maybe_get(get)
            .maybe_set(set)
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.prototype.insert(key, property);
        self
    }

    /// Add new static accessor property to the constructor object itself.
    pub fn static_accessor<K>(
        &mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        let property = PropertyDescriptor::builder()
            .maybe_get(get)
            .maybe_set(set)
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.constructor_object.insert(key, property);
        self
    }

    /// Add new property to the constructor's prototype.
    pub fn property_descriptor<K, P>(&mut self, key: K, property: P) -> &mut Self
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let property = property.into();
        self.prototype.insert(key, property);
        self
    }

    /// Add new static property to the constructor object itself.
    pub fn static_property_descriptor<K, P>(&mut self, key: K, property: P) -> &mut Self
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let property = property.into();
        self.constructor_object.insert(key, property);
        self
    }

    /// Specify how many arguments the constructor function takes.
    ///
    /// Default is `0`.
    #[inline]
    pub fn length(&mut self, length: usize) -> &mut Self {
        self.length = length;
        self
    }

    /// Specify the name of the constructor function.
    ///
    /// Default is `"[object]"`
    pub fn name<N>(&mut self, name: N) -> &mut Self
    where
        N: AsRef<str>,
    {
        self.name = name.as_ref().into();
        self
    }

    /// Specify whether the constructor function can be called.
    ///
    /// Default is `true`
    #[inline]
    pub fn callable(&mut self, callable: bool) -> &mut Self {
        self.callable = callable;
        self
    }

    /// Specify whether the constructor function can be called with `new` keyword.
    ///
    /// Default is `true`
    #[inline]
    pub fn constructor(&mut self, constructor: bool) -> &mut Self {
        self.kind = constructor.then_some(ConstructorKind::Base);
        self
    }

    /// Specify the parent prototype which objects created by this constructor
    /// inherit from.
    ///
    /// Default is `Object.prototype`
    pub fn inherit<O: Into<JsPrototype>>(&mut self, prototype: O) -> &mut Self {
        self.inherit = Some(prototype.into());
        self
    }

    /// Specify the `[[Prototype]]` internal field of this constructor.
    ///
    /// Default is `Function.prototype`
    pub fn custom_prototype<O: Into<JsPrototype>>(&mut self, prototype: O) -> &mut Self {
        self.custom_prototype = Some(prototype.into());
        self
    }

    /// Specify whether the constructor function has a 'prototype' property.
    ///
    /// Default is `true`
    #[inline]
    pub fn has_prototype_property(&mut self, has_prototype_property: bool) -> &mut Self {
        self.has_prototype_property = has_prototype_property;
        self
    }

    /// Return the current context.
    #[inline]
    pub fn context(&mut self) -> &mut Context<'host> {
        self.context
    }

    /// Build the constructor function object.
    #[must_use]
    pub fn build(mut self) -> JsFunction {
        // Create the native function
        let function = Function::new(
            FunctionKind::Native {
                function: self.function,
                constructor: self.kind,
            },
            self.context.realm().clone(),
        );

        let length = PropertyDescriptor::builder()
            .value(self.length)
            .writable(false)
            .enumerable(false)
            .configurable(true);
        let name = PropertyDescriptor::builder()
            .value(self.name.clone())
            .writable(false)
            .enumerable(false)
            .configurable(true);

        let prototype = {
            if let Some(proto) = self.inherit.take() {
                self.prototype.set_prototype(proto);
            } else {
                self.prototype.set_prototype(
                    self.context
                        .intrinsics()
                        .constructors()
                        .object()
                        .prototype(),
                );
            }

            JsObject::from_object_and_vtable(self.prototype, &ORDINARY_INTERNAL_METHODS)
        };

        let constructor = {
            let mut constructor = self.constructor_object;
            constructor.insert(utf16!("length"), length);
            constructor.insert(utf16!("name"), name);
            let data = ObjectData::function(function, self.kind.is_some());

            constructor.kind = data.kind;

            if let Some(proto) = self.custom_prototype.take() {
                constructor.set_prototype(proto);
            } else {
                constructor.set_prototype(
                    self.context
                        .intrinsics()
                        .constructors()
                        .function()
                        .prototype(),
                );
            }

            if self.has_prototype_property {
                constructor.insert(
                    PROTOTYPE,
                    PropertyDescriptor::builder()
                        .value(prototype.clone())
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                );
            }

            JsObject::from_object_and_vtable(constructor, data.internal_methods)
        };

        {
            let mut prototype = prototype.borrow_mut();
            prototype.insert(
                CONSTRUCTOR,
                PropertyDescriptor::builder()
                    .value(constructor.clone())
                    .writable(true)
                    .enumerable(false)
                    .configurable(true),
            );
        }

        JsFunction::from_object_unchecked(constructor)
    }
}
