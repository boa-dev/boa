//! Data structures that contain intrinsic objects and constructors.

use crate::{
    builtins::{
        array::Array, error::r#type::create_throw_type_error, iterable::IteratorPrototypes,
    },
    object::{JsObject, ObjectData},
    property::PropertyDescriptorBuilder,
    Context,
};

/// The intrinsic objects and constructors.
#[derive(Debug, Default)]
pub struct Intrinsics {
    /// Cached standard constructors
    pub(super) constructors: StandardConstructors,
    /// Cached intrinsic objects
    pub(super) objects: IntrinsicObjects,
}

impl Intrinsics {
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
}

/// Store a builtin constructor (such as `Object`) and its corresponding prototype.
#[derive(Debug, Clone)]
pub struct StandardConstructor {
    pub(crate) constructor: JsObject,
    pub(crate) prototype: JsObject,
}

impl Default for StandardConstructor {
    fn default() -> Self {
        Self {
            constructor: JsObject::empty(),
            prototype: JsObject::empty(),
        }
    }
}

impl StandardConstructor {
    /// Build a constructor with a defined prototype.
    fn with_prototype(prototype: JsObject) -> Self {
        Self {
            constructor: JsObject::empty(),
            prototype,
        }
    }

    /// Return the constructor object.
    ///
    /// This is the same as `Object`, `Array`, etc.
    #[inline]
    pub fn constructor(&self) -> JsObject {
        self.constructor.clone()
    }

    /// Return the prototype of the constructor object.
    ///
    /// This is the same as `Object.prototype`, `Array.prototype`, etc
    #[inline]
    pub fn prototype(&self) -> JsObject {
        self.prototype.clone()
    }
}

/// Cached core standard constructors.
#[derive(Debug, Clone)]
pub struct StandardConstructors {
    async_generator_function: StandardConstructor,
    async_generator: StandardConstructor,
    object: StandardConstructor,
    proxy: StandardConstructor,
    date: StandardConstructor,
    function: StandardConstructor,
    async_function: StandardConstructor,
    generator: StandardConstructor,
    generator_function: StandardConstructor,
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
    #[cfg(feature = "intl")]
    locale: StandardConstructor,
}

impl Default for StandardConstructors {
    fn default() -> Self {
        let result = Self {
            async_generator_function: StandardConstructor::default(),
            async_generator: StandardConstructor::default(),
            object: StandardConstructor::default(),
            proxy: StandardConstructor::default(),
            date: StandardConstructor::default(),
            function: StandardConstructor::default(),
            async_function: StandardConstructor::default(),
            generator: StandardConstructor::default(),
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
            #[cfg(feature = "intl")]
            locale: StandardConstructor::default(),
        };

        // The value of `Array.prototype` is the Array prototype object.
        result.array.prototype.insert(
            "length",
            PropertyDescriptorBuilder::new()
                .value(0)
                .writable(true)
                .enumerable(false)
                .configurable(false)
                .build(),
        );

        result
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

    /// Returns the `AsyncGenerator` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-objects
    #[inline]
    pub const fn async_generator(&self) -> &StandardConstructor {
        &self.async_generator
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

    /// Returns the `Generator` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generator-objects
    #[inline]
    pub const fn generator(&self) -> &StandardConstructor {
        &self.generator
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
    pub const fn bigint_object(&self) -> &StandardConstructor {
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
}

/// Cached intrinsic objects
#[derive(Debug, Default)]
pub struct IntrinsicObjects {
    /// %ThrowTypeError% intrinsic object
    throw_type_error: JsObject,

    /// %Array.prototype.values%
    array_prototype_values: JsObject,

    /// Cached iterator prototypes.
    iterator_prototypes: IteratorPrototypes,
}

impl IntrinsicObjects {
    /// Initialize the intrinsic objects
    pub fn init(context: &mut Context) -> Self {
        Self {
            throw_type_error: create_throw_type_error(context),
            array_prototype_values: Array::create_array_prototype_values(context).into(),
            iterator_prototypes: IteratorPrototypes::init(context),
        }
    }

    /// Get the `%ThrowTypeError%` intrinsic object
    #[inline]
    pub fn throw_type_error(&self) -> JsObject {
        self.throw_type_error.clone()
    }

    /// Get the `%Array.prototype.values%` intrinsic object.
    #[inline]
    pub fn array_prototype_values(&self) -> JsObject {
        self.array_prototype_values.clone()
    }

    /// Get the cached iterator prototypes.
    #[inline]
    pub const fn iterator_prototypes(&self) -> &IteratorPrototypes {
        &self.iterator_prototypes
    }
}
