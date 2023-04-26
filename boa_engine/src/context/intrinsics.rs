//! Data structures that contain intrinsic objects and constructors.

use boa_gc::{Finalize, Trace};

use crate::{
    builtins::{iterable::IteratorPrototypes, uri::UriFunctions},
    object::{
        shape::shared_shape::{template::ObjectTemplate, SharedShape},
        JsFunction, JsObject, ObjectData, CONSTRUCTOR, PROTOTYPE,
    },
    property::{Attribute, PropertyKey},
    JsSymbol,
};

/// The intrinsic objects and constructors.
///
/// `Intrinsics` is internally stored using a `Gc`, which makes it cheapily clonable
/// for multiple references to the same set of intrinsic objects.
#[derive(Debug, Trace, Finalize)]
pub struct Intrinsics {
    /// Cached standard constructors
    pub(super) constructors: StandardConstructors,
    /// Cached intrinsic objects
    pub(super) objects: IntrinsicObjects,
    /// Cached object templates.
    pub(super) templates: ObjectTemplates,
}

impl Intrinsics {
    pub(crate) fn new(root_shape: &SharedShape) -> Self {
        let constructors = StandardConstructors::default();
        let templates = ObjectTemplates::new(root_shape, &constructors);

        Self {
            constructors,
            objects: IntrinsicObjects::default(),
            templates,
        }
    }

    /// Return the cached intrinsic objects.
    #[inline]
    pub const fn objects(&self) -> &IntrinsicObjects {
        &self.objects
    }

    /// Return the cached standard constructors.
    #[inline]
    pub const fn constructors(&self) -> &StandardConstructors {
        &self.constructors
    }

    pub(crate) const fn templates(&self) -> &ObjectTemplates {
        &self.templates
    }
}

/// Store a builtin constructor (such as `Object`) and its corresponding prototype.
#[derive(Debug, Trace, Finalize)]
pub struct StandardConstructor {
    constructor: JsFunction,
    prototype: JsObject,
}

impl Default for StandardConstructor {
    fn default() -> Self {
        Self {
            constructor: JsFunction::empty_intrinsic_function(true),
            prototype: JsObject::with_null_proto(),
        }
    }
}

impl StandardConstructor {
    /// Build a constructor with a defined prototype.
    fn with_prototype(prototype: JsObject) -> Self {
        Self {
            constructor: JsFunction::empty_intrinsic_function(true),
            prototype,
        }
    }

    /// Return the prototype of the constructor object.
    ///
    /// This is the same as `Object.prototype`, `Array.prototype`, etc
    #[inline]
    pub fn prototype(&self) -> JsObject {
        self.prototype.clone()
    }

    /// Return the constructor object.
    ///
    /// This is the same as `Object`, `Array`, etc.
    #[inline]
    pub fn constructor(&self) -> JsObject {
        self.constructor.clone().into()
    }
}

/// Cached core standard constructors.
#[derive(Debug, Trace, Finalize)]
pub struct StandardConstructors {
    object: StandardConstructor,
    proxy: StandardConstructor,
    date: StandardConstructor,
    function: StandardConstructor,
    async_function: StandardConstructor,
    generator_function: StandardConstructor,
    async_generator_function: StandardConstructor,
    array: StandardConstructor,
    bigint: StandardConstructor,
    number: StandardConstructor,
    boolean: StandardConstructor,
    string: StandardConstructor,
    regexp: StandardConstructor,
    symbol: StandardConstructor,
    error: StandardConstructor,
    type_error: StandardConstructor,
    reference_error: StandardConstructor,
    range_error: StandardConstructor,
    syntax_error: StandardConstructor,
    eval_error: StandardConstructor,
    uri_error: StandardConstructor,
    aggregate_error: StandardConstructor,
    map: StandardConstructor,
    set: StandardConstructor,
    typed_array: StandardConstructor,
    typed_int8_array: StandardConstructor,
    typed_uint8_array: StandardConstructor,
    typed_uint8clamped_array: StandardConstructor,
    typed_int16_array: StandardConstructor,
    typed_uint16_array: StandardConstructor,
    typed_int32_array: StandardConstructor,
    typed_uint32_array: StandardConstructor,
    typed_bigint64_array: StandardConstructor,
    typed_biguint64_array: StandardConstructor,
    typed_float32_array: StandardConstructor,
    typed_float64_array: StandardConstructor,
    array_buffer: StandardConstructor,
    data_view: StandardConstructor,
    date_time_format: StandardConstructor,
    promise: StandardConstructor,
    weak_ref: StandardConstructor,
    weak_map: StandardConstructor,
    weak_set: StandardConstructor,
    #[cfg(feature = "intl")]
    collator: StandardConstructor,
    #[cfg(feature = "intl")]
    list_format: StandardConstructor,
    #[cfg(feature = "intl")]
    locale: StandardConstructor,
    #[cfg(feature = "intl")]
    segmenter: StandardConstructor,
}

impl Default for StandardConstructors {
    fn default() -> Self {
        Self {
            async_generator_function: StandardConstructor::default(),
            object: StandardConstructor::default(),
            proxy: StandardConstructor::default(),
            date: StandardConstructor::default(),
            function: StandardConstructor {
                constructor: JsFunction::empty_intrinsic_function(true),
                prototype: JsFunction::empty_intrinsic_function(false).into(),
            },
            async_function: StandardConstructor::default(),
            generator_function: StandardConstructor::default(),
            array: StandardConstructor::with_prototype(JsObject::from_proto_and_data(
                None,
                ObjectData::array(),
            )),
            bigint: StandardConstructor::default(),
            number: StandardConstructor::with_prototype(JsObject::from_proto_and_data(
                None,
                ObjectData::number(0.0),
            )),
            boolean: StandardConstructor::with_prototype(JsObject::from_proto_and_data(
                None,
                ObjectData::boolean(false),
            )),
            string: StandardConstructor::with_prototype(JsObject::from_proto_and_data(
                None,
                ObjectData::string("".into()),
            )),
            regexp: StandardConstructor::default(),
            symbol: StandardConstructor::default(),
            error: StandardConstructor::default(),
            type_error: StandardConstructor::default(),
            reference_error: StandardConstructor::default(),
            range_error: StandardConstructor::default(),
            syntax_error: StandardConstructor::default(),
            eval_error: StandardConstructor::default(),
            uri_error: StandardConstructor::default(),
            aggregate_error: StandardConstructor::default(),
            map: StandardConstructor::default(),
            set: StandardConstructor::default(),
            typed_array: StandardConstructor::default(),
            typed_int8_array: StandardConstructor::default(),
            typed_uint8_array: StandardConstructor::default(),
            typed_uint8clamped_array: StandardConstructor::default(),
            typed_int16_array: StandardConstructor::default(),
            typed_uint16_array: StandardConstructor::default(),
            typed_int32_array: StandardConstructor::default(),
            typed_uint32_array: StandardConstructor::default(),
            typed_bigint64_array: StandardConstructor::default(),
            typed_biguint64_array: StandardConstructor::default(),
            typed_float32_array: StandardConstructor::default(),
            typed_float64_array: StandardConstructor::default(),
            array_buffer: StandardConstructor::default(),
            data_view: StandardConstructor::default(),
            date_time_format: StandardConstructor::default(),
            promise: StandardConstructor::default(),
            weak_ref: StandardConstructor::default(),
            weak_map: StandardConstructor::default(),
            weak_set: StandardConstructor::default(),
            #[cfg(feature = "intl")]
            collator: StandardConstructor::default(),
            #[cfg(feature = "intl")]
            list_format: StandardConstructor::default(),
            #[cfg(feature = "intl")]
            locale: StandardConstructor::default(),
            #[cfg(feature = "intl")]
            segmenter: StandardConstructor::default(),
        }
    }
}

impl StandardConstructors {
    /// Returns the `AsyncGeneratorFunction` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorfunction-constructor
    #[inline]
    pub const fn async_generator_function(&self) -> &StandardConstructor {
        &self.async_generator_function
    }

    /// Returns the `Object` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-constructor
    #[inline]
    pub const fn object(&self) -> &StandardConstructor {
        &self.object
    }

    /// Returns the `Proxy` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-proxy-constructor
    #[inline]
    pub const fn proxy(&self) -> &StandardConstructor {
        &self.proxy
    }

    /// Returns the `Date` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date-constructor
    #[inline]
    pub const fn date(&self) -> &StandardConstructor {
        &self.date
    }

    /// Returns the `Function` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-constructor
    #[inline]
    pub const fn function(&self) -> &StandardConstructor {
        &self.function
    }

    /// Returns the `AsyncFunction` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-async-function-constructor
    #[inline]
    pub const fn async_function(&self) -> &StandardConstructor {
        &self.async_function
    }

    /// Returns the `GeneratorFunction` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorfunction-constructor
    #[inline]
    pub const fn generator_function(&self) -> &StandardConstructor {
        &self.generator_function
    }

    /// Returns the `Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array-constructor
    #[inline]
    pub const fn array(&self) -> &StandardConstructor {
        &self.array
    }

    /// Returns the `BigInt` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint-constructor
    #[inline]
    pub const fn bigint(&self) -> &StandardConstructor {
        &self.bigint
    }

    /// Returns the `Number` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-number-constructor
    #[inline]
    pub const fn number(&self) -> &StandardConstructor {
        &self.number
    }

    /// Returns the `Boolean` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-boolean-constructor
    #[inline]
    pub const fn boolean(&self) -> &StandardConstructor {
        &self.boolean
    }

    /// Returns the `String` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string-constructor
    #[inline]
    pub const fn string(&self) -> &StandardConstructor {
        &self.string
    }

    /// Returns the `RegExp` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexp-constructor
    #[inline]
    pub const fn regexp(&self) -> &StandardConstructor {
        &self.regexp
    }

    /// Returns the `Symbol` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-symbol-constructor
    #[inline]
    pub const fn symbol(&self) -> &StandardConstructor {
        &self.symbol
    }

    /// Returns the `Error` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-error-constructor
    #[inline]
    pub const fn error(&self) -> &StandardConstructor {
        &self.error
    }

    /// Returns the `ReferenceError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-referenceerror
    #[inline]
    pub const fn reference_error(&self) -> &StandardConstructor {
        &self.reference_error
    }

    /// Returns the `TypeError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror
    #[inline]
    pub const fn type_error(&self) -> &StandardConstructor {
        &self.type_error
    }

    /// Returns the `RangeError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-rangeerror
    #[inline]
    pub const fn range_error(&self) -> &StandardConstructor {
        &self.range_error
    }

    /// Returns the `SyntaxError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-syntaxerror
    #[inline]
    pub const fn syntax_error(&self) -> &StandardConstructor {
        &self.syntax_error
    }

    /// Returns the `EvalError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-evalerror
    #[inline]
    pub const fn eval_error(&self) -> &StandardConstructor {
        &self.eval_error
    }

    /// Returns the `URIError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-urierror
    #[inline]
    pub const fn uri_error(&self) -> &StandardConstructor {
        &self.uri_error
    }

    /// Returns the `AggregateError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-aggregate-error-constructor
    #[inline]
    pub const fn aggregate_error(&self) -> &StandardConstructor {
        &self.aggregate_error
    }

    /// Returns the `Map` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map-constructor
    #[inline]
    pub const fn map(&self) -> &StandardConstructor {
        &self.map
    }

    /// Returns the `Set` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set-constructor
    #[inline]
    pub const fn set(&self) -> &StandardConstructor {
        &self.set
    }

    /// Returns the `TypedArray` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_array(&self) -> &StandardConstructor {
        &self.typed_array
    }

    /// Returns the `Int8Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_int8_array(&self) -> &StandardConstructor {
        &self.typed_int8_array
    }

    /// Returns the `Uint8Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_uint8_array(&self) -> &StandardConstructor {
        &self.typed_uint8_array
    }

    /// Returns the `Uint8ClampedArray` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_uint8clamped_array(&self) -> &StandardConstructor {
        &self.typed_uint8clamped_array
    }

    /// Returns the `Int16Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_int16_array(&self) -> &StandardConstructor {
        &self.typed_int16_array
    }

    /// Returns the `Uint16Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_uint16_array(&self) -> &StandardConstructor {
        &self.typed_uint16_array
    }

    /// Returns the `Uint32Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_uint32_array(&self) -> &StandardConstructor {
        &self.typed_uint32_array
    }

    /// Returns the `Int32Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_int32_array(&self) -> &StandardConstructor {
        &self.typed_int32_array
    }

    /// Returns the `BigInt64Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_bigint64_array(&self) -> &StandardConstructor {
        &self.typed_bigint64_array
    }

    /// Returns the `BigUint64Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_biguint64_array(&self) -> &StandardConstructor {
        &self.typed_biguint64_array
    }

    /// Returns the `Float32Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_float32_array(&self) -> &StandardConstructor {
        &self.typed_float32_array
    }

    /// Returns the `Float64Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    pub const fn typed_float64_array(&self) -> &StandardConstructor {
        &self.typed_float64_array
    }

    /// Returns the `ArrayBuffer` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer-constructor
    #[inline]
    pub const fn array_buffer(&self) -> &StandardConstructor {
        &self.array_buffer
    }

    /// Returns the `DataView` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview-constructor
    #[inline]
    pub const fn data_view(&self) -> &StandardConstructor {
        &self.data_view
    }

    /// Returns the `Intl.DateTimeFormat` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl-datetimeformat-constructor
    #[inline]
    pub const fn date_time_format(&self) -> &StandardConstructor {
        &self.date_time_format
    }

    /// Returns the `Promise` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise-constructor
    #[inline]
    pub const fn promise(&self) -> &StandardConstructor {
        &self.promise
    }

    /// Returns the `WeakRef` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weak-ref-constructor
    #[inline]
    pub const fn weak_ref(&self) -> &StandardConstructor {
        &self.weak_ref
    }

    /// Returns the `WeakMap` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakmap-constructor
    #[inline]
    pub const fn weak_map(&self) -> &StandardConstructor {
        &self.weak_map
    }

    /// Returns the `WeakSet` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakset-constructor
    #[inline]
    pub const fn weak_set(&self) -> &StandardConstructor {
        &self.weak_set
    }

    /// Returns the `Intl.Collator` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.collator
    #[inline]
    #[cfg(feature = "intl")]
    pub const fn collator(&self) -> &StandardConstructor {
        &self.collator
    }

    /// Returns the `Intl.ListFormat` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.ListFormat
    #[inline]
    #[cfg(feature = "intl")]
    pub const fn list_format(&self) -> &StandardConstructor {
        &self.list_format
    }

    /// Returns the `Intl.Locale` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.Locale
    #[inline]
    #[cfg(feature = "intl")]
    pub const fn locale(&self) -> &StandardConstructor {
        &self.locale
    }

    /// Returns the `Intl.Segmenter` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.segmenter
    #[inline]
    #[cfg(feature = "intl")]
    pub const fn segmenter(&self) -> &StandardConstructor {
        &self.segmenter
    }
}

/// Cached intrinsic objects
#[derive(Debug, Trace, Finalize)]
pub struct IntrinsicObjects {
    /// [`%Reflect%`](https://tc39.es/ecma262/#sec-reflect)
    reflect: JsObject,

    /// [`%Math%`](https://tc39.es/ecma262/#sec-math)
    math: JsObject,

    /// [`%JSON%`](https://tc39.es/ecma262/#sec-json)
    json: JsObject,

    /// [`%ThrowTypeError%`](https://tc39.es/ecma262/#sec-%throwtypeerror%)
    throw_type_error: JsFunction,

    /// [`%Array.prototype.values%`](https://tc39.es/ecma262/#sec-array.prototype.values)
    array_prototype_values: JsFunction,

    /// Cached iterator prototypes.
    iterator_prototypes: IteratorPrototypes,

    /// [`%GeneratorFunction.prototype.prototype%`](https://tc39.es/ecma262/#sec-properties-of-generator-prototype)
    generator: JsObject,

    /// [`%AsyncGeneratorFunction.prototype.prototype%`](https://tc39.es/ecma262/#sec-properties-of-asyncgenerator-prototype)
    async_generator: JsObject,

    /// [`%eval%`](https://tc39.es/ecma262/#sec-eval-x)
    eval: JsFunction,

    /// URI related functions
    uri_functions: UriFunctions,

    /// [`%isFinite%`](https://tc39.es/ecma262/#sec-isfinite-number)
    is_finite: JsFunction,

    /// [`%isNaN%`](https://tc39.es/ecma262/#sec-isnan-number)
    is_nan: JsFunction,

    /// [`%parseFloat%`](https://tc39.es/ecma262/#sec-parsefloat-string)
    parse_float: JsFunction,

    /// [`%parseInt%`](https://tc39.es/ecma262/#sec-parseint-string-radix)
    parse_int: JsFunction,

    /// [`%escape%`](https://tc39.es/ecma262/#sec-escape-string)
    #[cfg(feature = "annex-b")]
    escape: JsFunction,

    /// [`%unescape%`](https://tc39.es/ecma262/#sec-unescape-string)
    #[cfg(feature = "annex-b")]
    unescape: JsFunction,

    /// [`%Intl%`](https://tc39.es/ecma402/#intl-object)
    #[cfg(feature = "intl")]
    intl: JsObject,

    /// [`%SegmentsPrototype%`](https://tc39.es/ecma402/#sec-%segmentsprototype%-object)
    #[cfg(feature = "intl")]
    segments_prototype: JsObject,
}

impl Default for IntrinsicObjects {
    fn default() -> Self {
        Self {
            reflect: JsObject::default(),
            math: JsObject::default(),
            json: JsObject::default(),
            throw_type_error: JsFunction::empty_intrinsic_function(false),
            array_prototype_values: JsFunction::empty_intrinsic_function(false),
            iterator_prototypes: IteratorPrototypes::default(),
            generator: JsObject::default(),
            async_generator: JsObject::default(),
            eval: JsFunction::empty_intrinsic_function(false),
            uri_functions: UriFunctions::default(),
            is_finite: JsFunction::empty_intrinsic_function(false),
            is_nan: JsFunction::empty_intrinsic_function(false),
            parse_float: JsFunction::empty_intrinsic_function(false),
            parse_int: JsFunction::empty_intrinsic_function(false),
            #[cfg(feature = "annex-b")]
            escape: JsFunction::empty_intrinsic_function(false),
            #[cfg(feature = "annex-b")]
            unescape: JsFunction::empty_intrinsic_function(false),
            #[cfg(feature = "intl")]
            intl: JsObject::default(),
            #[cfg(feature = "intl")]
            segments_prototype: JsObject::default(),
        }
    }
}

impl IntrinsicObjects {
    /// Gets the [`%ThrowTypeError%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%throwtypeerror%
    #[inline]
    pub fn throw_type_error(&self) -> JsFunction {
        self.throw_type_error.clone()
    }

    /// Gets the [`%Array.prototype.values%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.values
    #[inline]
    pub fn array_prototype_values(&self) -> JsFunction {
        self.array_prototype_values.clone()
    }

    /// Gets the cached iterator prototypes.
    #[inline]
    pub const fn iterator_prototypes(&self) -> &IteratorPrototypes {
        &self.iterator_prototypes
    }

    /// Gets the [`%GeneratorFunction.prototype.prototype%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generator-objects
    #[inline]
    pub fn generator(&self) -> JsObject {
        self.generator.clone()
    }

    /// Gets the [`%AsyncGeneratorFunction.prototype.prototype%`] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-objects
    #[inline]
    pub fn async_generator(&self) -> JsObject {
        self.async_generator.clone()
    }

    /// Gets the [`%eval%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-eval-x
    pub fn eval(&self) -> JsFunction {
        self.eval.clone()
    }

    /// Gets the URI intrinsic functions.
    pub const fn uri_functions(&self) -> &UriFunctions {
        &self.uri_functions
    }

    /// Gets the [`%Reflect%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect
    pub fn reflect(&self) -> JsObject {
        self.reflect.clone()
    }

    /// Gets the [`%Math%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math
    pub fn math(&self) -> JsObject {
        self.math.clone()
    }

    /// Gets the [`%JSON%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-json
    pub fn json(&self) -> JsObject {
        self.json.clone()
    }

    /// Gets the [`%isFinite%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isfinite-number
    pub fn is_finite(&self) -> JsFunction {
        self.is_finite.clone()
    }

    /// Gets the [`%isNaN%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isnan-number
    pub fn is_nan(&self) -> JsFunction {
        self.is_nan.clone()
    }

    /// Gets the [`%parseFloat%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parsefloat-string
    pub fn parse_float(&self) -> JsFunction {
        self.parse_float.clone()
    }

    /// Gets the [`%parseInt%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parseint-string-radix
    pub fn parse_int(&self) -> JsFunction {
        self.parse_int.clone()
    }

    /// Gets the [`%escape%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-escape-string
    #[cfg(feature = "annex-b")]
    pub fn escape(&self) -> JsFunction {
        self.escape.clone()
    }

    /// Gets the [`%unescape%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-unescape-string
    #[cfg(feature = "annex-b")]
    pub fn unescape(&self) -> JsFunction {
        self.unescape.clone()
    }

    /// Gets the [`%Intl%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma402/#intl-object
    #[cfg(feature = "intl")]
    pub fn intl(&self) -> JsObject {
        self.intl.clone()
    }

    /// Gets the [`%SegmentsPrototype%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-%segmentsprototype%-object
    #[cfg(feature = "intl")]
    pub fn segments_prototype(&self) -> JsObject {
        self.segments_prototype.clone()
    }
}

/// Contains commonly used [`ObjectTemplate`]s.
#[derive(Debug, Trace, Finalize)]
pub(crate) struct ObjectTemplates {
    iterator_result: ObjectTemplate,
    ordinary_object: ObjectTemplate,
    array: ObjectTemplate,
    number: ObjectTemplate,
    string: ObjectTemplate,
    symbol: ObjectTemplate,
    bigint: ObjectTemplate,
    boolean: ObjectTemplate,

    unmapped_arguments: ObjectTemplate,
    mapped_arguments: ObjectTemplate,

    function_with_prototype: ObjectTemplate,
    function_prototype: ObjectTemplate,

    function: ObjectTemplate,
    async_function: ObjectTemplate,

    function_without_proto: ObjectTemplate,
    function_with_prototype_without_proto: ObjectTemplate,
}

impl ObjectTemplates {
    pub(crate) fn new(root_shape: &SharedShape, constructors: &StandardConstructors) -> Self {
        // pre-initialize used shapes.
        let ordinary_object =
            ObjectTemplate::with_prototype(root_shape, constructors.object().prototype());
        let mut array = ObjectTemplate::new(root_shape);
        let length_property_key: PropertyKey = "length".into();
        array.property(
            length_property_key.clone(),
            Attribute::WRITABLE | Attribute::PERMANENT | Attribute::NON_ENUMERABLE,
        );
        array.set_prototype(constructors.array().prototype());

        let number = ObjectTemplate::with_prototype(root_shape, constructors.number().prototype());
        let symbol = ObjectTemplate::with_prototype(root_shape, constructors.symbol().prototype());
        let bigint = ObjectTemplate::with_prototype(root_shape, constructors.bigint().prototype());
        let boolean =
            ObjectTemplate::with_prototype(root_shape, constructors.boolean().prototype());
        let mut string = ObjectTemplate::new(root_shape);
        string.property(
            length_property_key.clone(),
            Attribute::READONLY | Attribute::PERMANENT | Attribute::NON_ENUMERABLE,
        );
        string.set_prototype(constructors.string().prototype());

        let name_property_key: PropertyKey = "name".into();
        let mut function = ObjectTemplate::new(root_shape);
        function.property(
            length_property_key.clone(),
            Attribute::READONLY | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        );
        function.property(
            name_property_key,
            Attribute::READONLY | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        );

        let function_without_proto = function.clone();
        let mut async_function = function.clone();
        let mut function_with_prototype = function.clone();

        function_with_prototype.property(
            PROTOTYPE.into(),
            Attribute::WRITABLE | Attribute::PERMANENT | Attribute::NON_ENUMERABLE,
        );

        let function_with_prototype_without_proto = function_with_prototype.clone();

        function.set_prototype(constructors.function().prototype());
        function_with_prototype.set_prototype(constructors.function().prototype());
        async_function.set_prototype(constructors.async_function().prototype());

        let mut function_prototype = ordinary_object.clone();
        function_prototype.property(
            CONSTRUCTOR.into(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        );

        let mut unmapped_arguments = ordinary_object.clone();

        // 4. Perform DefinePropertyOrThrow(obj, "length", PropertyDescriptor { [[Value]]: 𝔽(len),
        // [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }).
        unmapped_arguments.property(
            length_property_key,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        // 7. Perform ! DefinePropertyOrThrow(obj, @@iterator, PropertyDescriptor {
        // [[Value]]: %Array.prototype.values%, [[Writable]]: true, [[Enumerable]]: false,
        // [[Configurable]]: true }).
        unmapped_arguments.property(
            JsSymbol::iterator().into(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let mut mapped_arguments = unmapped_arguments.clone();

        // 8. Perform ! DefinePropertyOrThrow(obj, "callee", PropertyDescriptor {
        // [[Get]]: %ThrowTypeError%, [[Set]]: %ThrowTypeError%, [[Enumerable]]: false,
        // [[Configurable]]: false }).
        unmapped_arguments.accessor(
            "callee".into(),
            true,
            true,
            Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );

        // 21. Perform ! DefinePropertyOrThrow(obj, "callee", PropertyDescriptor {
        // [[Value]]: func, [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }).
        mapped_arguments.property(
            "callee".into(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let mut iterator_result = ordinary_object.clone();
        iterator_result.property(
            "value".into(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        );
        iterator_result.property(
            "done".into(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        );

        Self {
            iterator_result,
            ordinary_object,
            array,
            number,
            string,
            symbol,
            bigint,
            boolean,
            unmapped_arguments,
            mapped_arguments,
            function_with_prototype,
            function_prototype,
            function,
            async_function,
            function_without_proto,
            function_with_prototype_without_proto,
        }
    }

    /// Cached iterator result template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"done"`: (`WRITABLE`, `CONFIGURABLE`, `ENUMERABLE`)
    /// 3. `"value"`: (`WRITABLE`, `CONFIGURABLE`, `ENUMERABLE`)
    pub(crate) const fn iterator_result(&self) -> &ObjectTemplate {
        &self.iterator_result
    }

    /// Cached ordinary object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    pub(crate) const fn ordinary_object(&self) -> &ObjectTemplate {
        &self.ordinary_object
    }

    /// Cached array object template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`WRITABLE`, `PERMANENT`,`NON_ENUMERABLE`)
    /// 2. `__proto__`: `Array.prototype`
    pub(crate) const fn array(&self) -> &ObjectTemplate {
        &self.array
    }

    /// Cached number object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Number.prototype`
    pub(crate) const fn number(&self) -> &ObjectTemplate {
        &self.number
    }

    /// Cached string object template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `PERMANENT`,`NON_ENUMERABLE`)
    /// 2. `__proto__`: `String.prototype`
    pub(crate) const fn string(&self) -> &ObjectTemplate {
        &self.string
    }

    /// Cached symbol object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Symbol.prototype`
    pub(crate) const fn symbol(&self) -> &ObjectTemplate {
        &self.symbol
    }

    /// Cached bigint object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `BigInt.prototype`
    pub(crate) const fn bigint(&self) -> &ObjectTemplate {
        &self.bigint
    }

    /// Cached boolean object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Boolean.prototype`
    pub(crate) const fn boolean(&self) -> &ObjectTemplate {
        &self.boolean
    }

    /// Cached unmapped arguments object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"length"`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `@@iterator`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 4. `get/set` `"callee"`: (`NON_ENUMERABLE`, `PERMANENT`)
    pub(crate) const fn unmapped_arguments(&self) -> &ObjectTemplate {
        &self.unmapped_arguments
    }

    /// Cached mapped arguments object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"length"`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `@@iterator`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 4. `"callee"`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    pub(crate) const fn mapped_arguments(&self) -> &ObjectTemplate {
        &self.mapped_arguments
    }

    /// Cached function object with `"prototype"` property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"prototype"`: (`WRITABLE`, `PERMANENT`, `NON_ENUMERABLE`)
    /// 4. `__proto__`: `Function.prototype`
    pub(crate) const fn function_with_prototype(&self) -> &ObjectTemplate {
        &self.function_with_prototype
    }

    /// Cached constructor function object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"contructor"`: (`WRITABLE`, `CONFIGURABLE`, `NON_ENUMERABLE`)
    pub(crate) const fn function_prototype(&self) -> &ObjectTemplate {
        &self.function_prototype
    }

    /// Cached function object property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `__proto__`: `Function.prototype`
    pub(crate) const fn function(&self) -> &ObjectTemplate {
        &self.function
    }

    /// Cached function object property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `__proto__`: `AsyncFunction.prototype`
    pub(crate) const fn async_function(&self) -> &ObjectTemplate {
        &self.async_function
    }

    /// Cached function object without `__proto__` template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    pub(crate) const fn function_without_proto(&self) -> &ObjectTemplate {
        &self.function_without_proto
    }

    /// Cached function object with `"prototype"` and without `__proto__` template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"prototype"`: (`WRITABLE`, `PERMANENT`, `NON_ENUMERABLE`)
    pub(crate) const fn function_with_prototype_without_proto(&self) -> &ObjectTemplate {
        &self.function_with_prototype_without_proto
    }
}
