use crate::{
    builtins::{
        array::Array, error::r#type::create_throw_type_error, iterable::IteratorPrototypes,
    },
    object::{JsObject, ObjectData},
    property::PropertyDescriptorBuilder,
    Context,
};

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
    pub fn objects(&self) -> &IntrinsicObjects {
        &self.objects
    }

    /// Return the cached standard constructors.
    #[inline]
    pub fn constructors(&self) -> &StandardConstructors {
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
    #[inline]
    pub fn async_generator_function(&self) -> &StandardConstructor {
        &self.async_generator_function
    }

    #[inline]
    pub fn async_generator(&self) -> &StandardConstructor {
        &self.async_generator
    }

    #[inline]
    pub fn object(&self) -> &StandardConstructor {
        &self.object
    }

    #[inline]
    pub fn proxy(&self) -> &StandardConstructor {
        &self.proxy
    }

    #[inline]
    pub fn date(&self) -> &StandardConstructor {
        &self.date
    }

    #[inline]
    pub fn function(&self) -> &StandardConstructor {
        &self.function
    }

    #[inline]
    pub fn async_function(&self) -> &StandardConstructor {
        &self.async_function
    }

    #[inline]
    pub fn generator(&self) -> &StandardConstructor {
        &self.generator
    }

    #[inline]
    pub fn generator_function(&self) -> &StandardConstructor {
        &self.generator_function
    }

    #[inline]
    pub fn array(&self) -> &StandardConstructor {
        &self.array
    }

    #[inline]
    pub fn bigint_object(&self) -> &StandardConstructor {
        &self.bigint
    }

    #[inline]
    pub fn number(&self) -> &StandardConstructor {
        &self.number
    }

    #[inline]
    pub fn boolean(&self) -> &StandardConstructor {
        &self.boolean
    }

    #[inline]
    pub fn string(&self) -> &StandardConstructor {
        &self.string
    }

    #[inline]
    pub fn regexp(&self) -> &StandardConstructor {
        &self.regexp
    }

    #[inline]
    pub fn symbol(&self) -> &StandardConstructor {
        &self.symbol
    }

    #[inline]
    pub fn error(&self) -> &StandardConstructor {
        &self.error
    }

    #[inline]
    pub fn reference_error(&self) -> &StandardConstructor {
        &self.reference_error
    }

    #[inline]
    pub fn type_error(&self) -> &StandardConstructor {
        &self.type_error
    }

    #[inline]
    pub fn range_error(&self) -> &StandardConstructor {
        &self.range_error
    }

    #[inline]
    pub fn syntax_error(&self) -> &StandardConstructor {
        &self.syntax_error
    }

    #[inline]
    pub fn eval_error(&self) -> &StandardConstructor {
        &self.eval_error
    }

    #[inline]
    pub fn uri_error(&self) -> &StandardConstructor {
        &self.uri_error
    }

    #[inline]
    pub fn aggregate_error(&self) -> &StandardConstructor {
        &self.aggregate_error
    }

    #[inline]
    pub fn map(&self) -> &StandardConstructor {
        &self.map
    }

    #[inline]
    pub fn set(&self) -> &StandardConstructor {
        &self.set
    }

    #[inline]
    pub fn typed_array(&self) -> &StandardConstructor {
        &self.typed_array
    }

    #[inline]
    pub fn typed_int8_array(&self) -> &StandardConstructor {
        &self.typed_int8_array
    }

    #[inline]
    pub fn typed_uint8_array(&self) -> &StandardConstructor {
        &self.typed_uint8_array
    }

    #[inline]
    pub fn typed_uint8clamped_array(&self) -> &StandardConstructor {
        &self.typed_uint8clamped_array
    }

    #[inline]
    pub fn typed_int16_array(&self) -> &StandardConstructor {
        &self.typed_int16_array
    }

    #[inline]
    pub fn typed_uint16_array(&self) -> &StandardConstructor {
        &self.typed_uint16_array
    }

    #[inline]
    pub fn typed_uint32_array(&self) -> &StandardConstructor {
        &self.typed_uint32_array
    }

    #[inline]
    pub fn typed_int32_array(&self) -> &StandardConstructor {
        &self.typed_int32_array
    }

    #[inline]
    pub fn typed_bigint64_array(&self) -> &StandardConstructor {
        &self.typed_bigint64_array
    }

    #[inline]
    pub fn typed_biguint64_array(&self) -> &StandardConstructor {
        &self.typed_biguint64_array
    }

    #[inline]
    pub fn typed_float32_array(&self) -> &StandardConstructor {
        &self.typed_float32_array
    }

    #[inline]
    pub fn typed_float64_array(&self) -> &StandardConstructor {
        &self.typed_float64_array
    }

    #[inline]
    pub fn array_buffer(&self) -> &StandardConstructor {
        &self.array_buffer
    }

    #[inline]
    pub fn data_view(&self) -> &StandardConstructor {
        &self.data_view
    }

    #[inline]
    pub fn date_time_format(&self) -> &StandardConstructor {
        &self.date_time_format
    }

    #[inline]
    pub fn promise(&self) -> &StandardConstructor {
        &self.promise
    }

    #[inline]
    pub fn weak_ref(&self) -> &StandardConstructor {
        &self.weak_ref
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
    pub fn iterator_prototypes(&self) -> &IteratorPrototypes {
        &self.iterator_prototypes
    }
}
