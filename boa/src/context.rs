//! Javascript context.

use crate::{
    builtins::{
        self, function::NativeFunctionSignature, intrinsics::IntrinsicObjects,
        iterable::IteratorPrototypes, typed_array::TypedArray,
    },
    bytecompiler::ByteCompiler,
    class::{Class, ClassBuilder},
    gc::Gc,
    object::{FunctionBuilder, GlobalPropertyMap, JsObject, ObjectData},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    syntax::{ast::node::StatementList, parser::ParseError, Parser},
    vm::{CallFrame, CodeBlock, FinallyReturn, Vm},
    BoaProfiler, Interner, JsResult, JsValue,
};
use boa_interner::Sym;

#[cfg(feature = "console")]
use crate::builtins::console::Console;

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

/// Cached core standard objects.
#[derive(Debug, Clone)]
pub struct StandardObjects {
    object: StandardConstructor,
    proxy: StandardConstructor,
    function: StandardConstructor,
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
}

impl Default for StandardObjects {
    fn default() -> Self {
        Self {
            object: StandardConstructor::default(),
            proxy: StandardConstructor::default(),
            function: StandardConstructor::default(),
            array: StandardConstructor::default(),
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
        }
    }
}

impl StandardObjects {
    #[inline]
    pub fn object_object(&self) -> &StandardConstructor {
        &self.object
    }

    #[inline]
    pub fn proxy_object(&self) -> &StandardConstructor {
        &self.proxy
    }

    #[inline]
    pub fn function_object(&self) -> &StandardConstructor {
        &self.function
    }

    #[inline]
    pub fn array_object(&self) -> &StandardConstructor {
        &self.array
    }

    #[inline]
    pub fn bigint_object(&self) -> &StandardConstructor {
        &self.bigint
    }

    #[inline]
    pub fn number_object(&self) -> &StandardConstructor {
        &self.number
    }

    #[inline]
    pub fn boolean_object(&self) -> &StandardConstructor {
        &self.boolean
    }

    #[inline]
    pub fn string_object(&self) -> &StandardConstructor {
        &self.string
    }

    #[inline]
    pub fn regexp_object(&self) -> &StandardConstructor {
        &self.regexp
    }

    #[inline]
    pub fn symbol_object(&self) -> &StandardConstructor {
        &self.symbol
    }

    #[inline]
    pub fn error_object(&self) -> &StandardConstructor {
        &self.error
    }

    #[inline]
    pub fn reference_error_object(&self) -> &StandardConstructor {
        &self.reference_error
    }

    #[inline]
    pub fn type_error_object(&self) -> &StandardConstructor {
        &self.type_error
    }

    #[inline]
    pub fn range_error_object(&self) -> &StandardConstructor {
        &self.range_error
    }

    #[inline]
    pub fn syntax_error_object(&self) -> &StandardConstructor {
        &self.syntax_error
    }

    #[inline]
    pub fn eval_error_object(&self) -> &StandardConstructor {
        &self.eval_error
    }

    #[inline]
    pub fn uri_error_object(&self) -> &StandardConstructor {
        &self.uri_error
    }

    #[inline]
    pub fn map_object(&self) -> &StandardConstructor {
        &self.map
    }

    #[inline]
    pub fn set_object(&self) -> &StandardConstructor {
        &self.set
    }

    #[inline]
    pub fn typed_array_object(&self) -> &StandardConstructor {
        &self.typed_array
    }

    #[inline]
    pub fn typed_int8_array_object(&self) -> &StandardConstructor {
        &self.typed_int8_array
    }

    #[inline]
    pub fn typed_uint8_array_object(&self) -> &StandardConstructor {
        &self.typed_uint8_array
    }

    #[inline]
    pub fn typed_uint8clamped_array_object(&self) -> &StandardConstructor {
        &self.typed_uint8clamped_array
    }

    #[inline]
    pub fn typed_int16_array_object(&self) -> &StandardConstructor {
        &self.typed_int16_array
    }

    #[inline]
    pub fn typed_uint16_array_object(&self) -> &StandardConstructor {
        &self.typed_uint16_array
    }

    #[inline]
    pub fn typed_uint32_array_object(&self) -> &StandardConstructor {
        &self.typed_uint32_array
    }

    #[inline]
    pub fn typed_int32_array_object(&self) -> &StandardConstructor {
        &self.typed_int32_array
    }

    #[inline]
    pub fn typed_bigint64_array_object(&self) -> &StandardConstructor {
        &self.typed_bigint64_array
    }

    #[inline]
    pub fn typed_biguint64_array_object(&self) -> &StandardConstructor {
        &self.typed_biguint64_array
    }

    #[inline]
    pub fn typed_float32_array_object(&self) -> &StandardConstructor {
        &self.typed_float32_array
    }

    #[inline]
    pub fn typed_float64_array_object(&self) -> &StandardConstructor {
        &self.typed_float64_array
    }

    #[inline]
    pub fn array_buffer_object(&self) -> &StandardConstructor {
        &self.array_buffer
    }

    #[inline]
    pub fn data_view_object(&self) -> &StandardConstructor {
        &self.data_view
    }
}

/// Javascript context. It is the primary way to interact with the runtime.
///
/// `Context`s constructed in a thread share the same runtime, therefore it
/// is possible to share objects from one context to another context, but they
/// have to be in the same thread.
///
/// # Examples
///
/// ## Execute Function of Script File
///
/// ```rust
/// use boa::{Context, object::ObjectInitializer, property::{Attribute, PropertyDescriptor}};
///
/// let script = r#"
/// function test(arg1) {
///     if(arg1 != null) {
///         return arg1.x;
///     }
///     return 112233;
/// }
/// "#;
///
/// let mut context = Context::default();
///
/// // Populate the script definition to the context.
/// context.eval(script).unwrap();
///
/// // Create an object that can be used in eval calls.
/// let arg = ObjectInitializer::new(&mut context)
///     .property("x", 12, Attribute::READONLY)
///     .build();
/// context.register_global_property(
///     "arg",
///     arg,
///     Attribute::all()
/// );
///
/// let value = context.eval("test(arg)").unwrap();
///
/// assert_eq!(value.as_number(), Some(12.0))
/// ```
#[derive(Debug)]
pub struct Context {
    /// realm holds both the global object and the environment
    pub(crate) realm: Realm,

    /// String interner in the context.
    interner: Interner,

    /// console object state.
    #[cfg(feature = "console")]
    console: Console,

    /// Cached iterator prototypes.
    iterator_prototypes: IteratorPrototypes,

    /// Cached TypedArray constructor.
    typed_array_constructor: StandardConstructor,

    /// Cached standard objects and their prototypes.
    standard_objects: StandardObjects,

    /// Cached intrinsic objects
    intrinsic_objects: IntrinsicObjects,

    /// Whether or not global strict mode is active.
    strict: bool,

    pub(crate) vm: Vm,
}

impl Default for Context {
    fn default() -> Self {
        let mut context = Self {
            realm: Realm::create(),
            interner: Interner::default(),
            #[cfg(feature = "console")]
            console: Console::default(),
            iterator_prototypes: IteratorPrototypes::default(),
            typed_array_constructor: StandardConstructor::default(),
            standard_objects: StandardObjects::default(),
            intrinsic_objects: IntrinsicObjects::default(),
            strict: false,
            vm: Vm {
                frame: None,
                stack: Vec::with_capacity(1024),
                trace: false,
                stack_size_limit: 1024,
            },
        };

        // Add new builtIns to Context Realm
        // At a later date this can be removed from here and called explicitly,
        // but for now we almost always want these default builtins
        let typed_array_constructor_constructor = TypedArray::init(&mut context);
        let typed_array_constructor_prototype = typed_array_constructor_constructor
            .get("prototype", &mut context)
            .expect("prototype must exist")
            .as_object()
            .expect("prototype must be object")
            .clone();
        context.typed_array_constructor.constructor = typed_array_constructor_constructor;
        context.typed_array_constructor.prototype = typed_array_constructor_prototype;
        context.create_intrinsics();
        context.iterator_prototypes = IteratorPrototypes::init(&mut context);
        context.intrinsic_objects = IntrinsicObjects::init(&mut context);
        context
    }
}

impl Context {
    /// Create a new `Context`.
    #[inline]
    pub fn new(interner: Interner) -> Self {
        Self {
            interner,
            ..Self::default()
        }
    }

    /// Gets the string interner.
    #[inline]
    pub fn interner(&self) -> &Interner {
        &self.interner
    }

    /// Gets a mutable reference to the string interner.
    #[inline]
    pub fn interner_mut(&mut self) -> &mut Interner {
        &mut self.interner
    }

    /// A helper function for getting an immutable reference to the `console` object.
    #[cfg(feature = "console")]
    pub(crate) fn console(&self) -> &Console {
        &self.console
    }

    /// A helper function for getting a mutable reference to the `console` object.
    #[cfg(feature = "console")]
    #[inline]
    pub(crate) fn console_mut(&mut self) -> &mut Console {
        &mut self.console
    }

    /// Returns if strict mode is currently active.
    #[inline]
    pub fn strict(&self) -> bool {
        self.strict
    }

    /// Set the global strict mode of the context.
    #[inline]
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict = strict;
    }

    /// Sets up the default global objects within Global
    #[inline]
    fn create_intrinsics(&mut self) {
        let _timer = BoaProfiler::global().start_event("create_intrinsics", "interpreter");
        // Create intrinsics, add global objects here
        builtins::init(self);
    }

    /// Constructs an object with the `%Object.prototype%` prototype.
    #[inline]
    pub fn construct_object(&self) -> JsObject {
        JsObject::from_proto_and_data(
            self.standard_objects().object_object().prototype(),
            ObjectData::ordinary(),
        )
    }

    pub fn parse<S>(&mut self, src: S) -> Result<StatementList, ParseError>
    where
        S: AsRef<[u8]>,
    {
        Parser::new(src.as_ref(), self.strict).parse_all(&mut self.interner)
    }

    /// <https://tc39.es/ecma262/#sec-call>
    #[inline]
    pub(crate) fn call(
        &mut self,
        f: &JsValue,
        this: &JsValue,
        args: &[JsValue],
    ) -> JsResult<JsValue> {
        f.as_callable()
            .ok_or_else(|| self.construct_type_error("Value is not callable"))
            .and_then(|obj| obj.call(this, args, self))
    }

    /// Return the global object.
    #[inline]
    pub fn global_object(&self) -> &JsObject {
        self.realm.global_object()
    }

    /// Return a reference to the global object string bindings.
    #[inline]
    pub(crate) fn global_bindings(&self) -> &GlobalPropertyMap {
        self.realm.global_bindings()
    }

    /// Return a mutable reference to the global object string bindings.
    #[inline]
    pub(crate) fn global_bindings_mut(&mut self) -> &mut GlobalPropertyMap {
        self.realm.global_bindings_mut()
    }

    /// Constructs a `Error` with the specified message.
    #[inline]
    pub fn construct_error<M>(&mut self, message: M) -> JsValue
    where
        M: Into<Box<str>>,
    {
        crate::builtins::error::Error::constructor(
            &self.standard_objects().error_object().constructor().into(),
            &[message.into().into()],
            self,
        )
        .expect("Into<String> used as message")
    }

    /// Throws a `Error` with the specified message.
    #[inline]
    pub fn throw_error<M, R>(&mut self, message: M) -> JsResult<R>
    where
        M: Into<Box<str>>,
    {
        Err(self.construct_error(message))
    }

    /// Constructs a `RangeError` with the specified message.
    #[inline]
    pub fn construct_range_error<M>(&mut self, message: M) -> JsValue
    where
        M: Into<Box<str>>,
    {
        crate::builtins::error::RangeError::constructor(
            &self
                .standard_objects()
                .range_error_object()
                .constructor()
                .into(),
            &[message.into().into()],
            self,
        )
        .expect("Into<String> used as message")
    }

    /// Throws a `RangeError` with the specified message.
    #[inline]
    pub fn throw_range_error<M, R>(&mut self, message: M) -> JsResult<R>
    where
        M: Into<Box<str>>,
    {
        Err(self.construct_range_error(message))
    }

    /// Constructs a `TypeError` with the specified message.
    #[inline]
    pub fn construct_type_error<M>(&mut self, message: M) -> JsValue
    where
        M: Into<Box<str>>,
    {
        crate::builtins::error::TypeError::constructor(
            &self
                .standard_objects()
                .type_error_object()
                .constructor()
                .into(),
            &[message.into().into()],
            self,
        )
        .expect("Into<String> used as message")
    }

    /// Throws a `TypeError` with the specified message.
    #[inline]
    pub fn throw_type_error<M, R>(&mut self, message: M) -> JsResult<R>
    where
        M: Into<Box<str>>,
    {
        Err(self.construct_type_error(message))
    }

    /// Constructs a `ReferenceError` with the specified message.
    #[inline]
    pub fn construct_reference_error<M>(&mut self, message: M) -> JsValue
    where
        M: Into<Box<str>>,
    {
        crate::builtins::error::ReferenceError::constructor(
            &self
                .standard_objects()
                .reference_error_object()
                .constructor()
                .into(),
            &[message.into().into()],
            self,
        )
        .expect("Into<String> used as message")
    }

    /// Throws a `ReferenceError` with the specified message.
    #[inline]
    pub fn throw_reference_error<M, R>(&mut self, message: M) -> JsResult<R>
    where
        M: Into<Box<str>>,
    {
        Err(self.construct_reference_error(message))
    }

    /// Constructs a `SyntaxError` with the specified message.
    #[inline]
    pub fn construct_syntax_error<M>(&mut self, message: M) -> JsValue
    where
        M: Into<Box<str>>,
    {
        crate::builtins::error::SyntaxError::constructor(
            &self
                .standard_objects()
                .syntax_error_object()
                .constructor()
                .into(),
            &[message.into().into()],
            self,
        )
        .expect("Into<String> used as message")
    }

    /// Throws a `SyntaxError` with the specified message.
    #[inline]
    pub fn throw_syntax_error<M, R>(&mut self, message: M) -> JsResult<R>
    where
        M: Into<Box<str>>,
    {
        Err(self.construct_syntax_error(message))
    }

    /// Constructs a `EvalError` with the specified message.
    pub fn construct_eval_error<M>(&mut self, message: M) -> JsValue
    where
        M: Into<Box<str>>,
    {
        crate::builtins::error::EvalError::constructor(
            &self
                .standard_objects()
                .eval_error_object()
                .constructor()
                .into(),
            &[message.into().into()],
            self,
        )
        .expect("Into<String> used as message")
    }

    /// Constructs a `URIError` with the specified message.
    pub fn construct_uri_error<M>(&mut self, message: M) -> JsValue
    where
        M: Into<Box<str>>,
    {
        crate::builtins::error::UriError::constructor(
            &self
                .standard_objects()
                .uri_error_object()
                .constructor()
                .into(),
            &[message.into().into()],
            self,
        )
        .expect("Into<String> used as message")
    }

    /// Throws a `EvalError` with the specified message.
    pub fn throw_eval_error<M, R>(&mut self, message: M) -> JsResult<R>
    where
        M: Into<Box<str>>,
    {
        Err(self.construct_eval_error(message))
    }

    /// Throws a `URIError` with the specified message.
    pub fn throw_uri_error<M>(&mut self, message: M) -> JsResult<JsValue>
    where
        M: Into<Box<str>>,
    {
        Err(self.construct_uri_error(message))
    }

    /// Register a global native function.
    ///
    /// This is more efficient that creating a closure function, since this does not allocate,
    /// it is just a function pointer.
    ///
    /// The function will be both `constructable` (call with `new`).
    ///
    /// The function will be bound to the global object with `writable`, `non-enumerable`
    /// and `configurable` attributes. The same as when you create a function in JavaScript.
    ///
    /// # Note
    ///
    /// If you want to make a function only `constructable`, or wish to bind it differently
    /// to the global object, you can create the function object with [`FunctionBuilder`](crate::object::FunctionBuilder::native).
    /// And bind it to the global object with [`Context::register_global_property`](Context::register_global_property) method.
    #[inline]
    pub fn register_global_function(
        &mut self,
        name: &str,
        length: usize,
        body: NativeFunctionSignature,
    ) {
        let function = FunctionBuilder::native(self, body)
            .name(name)
            .length(length)
            .constructor(true)
            .build();

        self.global_bindings_mut().insert(
            name.into(),
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
        );
    }

    /// Register a global native function that is not a constructor.
    ///
    /// This is more efficient that creating a closure function, since this does not allocate,
    /// it is just a function pointer.
    ///
    /// The function will be bound to the global object with `writable`, `non-enumerable`
    /// and `configurable` attributes. The same as when you create a function in JavaScript.
    ///
    /// # Note
    ///
    /// The difference to [`Context::register_global_function`](Context::register_global_function) is,
    /// that the function will not be `constructable`.
    /// Usage of the function as a constructor will produce a `TypeError`.
    #[inline]
    pub fn register_global_builtin_function(
        &mut self,
        name: &str,
        length: usize,
        body: NativeFunctionSignature,
    ) {
        let function = FunctionBuilder::native(self, body)
            .name(name)
            .length(length)
            .constructor(false)
            .build();

        self.global_bindings_mut().insert(
            name.into(),
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
        );
    }

    /// Register a global closure function.
    ///
    /// The function will be both `constructable` (call with `new`).
    ///
    /// The function will be bound to the global object with `writable`, `non-enumerable`
    /// and `configurable` attributes. The same as when you create a function in JavaScript.
    ///
    /// # Note #1
    ///
    /// If you want to make a function only `constructable`, or wish to bind it differently
    /// to the global object, you can create the function object with [`FunctionBuilder`](crate::object::FunctionBuilder::closure).
    /// And bind it to the global object with [`Context::register_global_property`](Context::register_global_property) method.
    ///
    /// # Note #2
    ///
    /// This function will only accept `Copy` closures, meaning you cannot
    /// move `Clone` types, just `Copy` types. If you need to move `Clone` types
    /// as captures, see [`FunctionBuilder::closure_with_captures`].
    ///
    /// See <https://github.com/boa-dev/boa/issues/1515> for an explanation on
    /// why we need to restrict the set of accepted closures.
    #[inline]
    pub fn register_global_closure<F>(&mut self, name: &str, length: usize, body: F) -> JsResult<()>
    where
        F: Fn(&JsValue, &[JsValue], &mut Self) -> JsResult<JsValue> + Copy + 'static,
    {
        let function = FunctionBuilder::closure(self, body)
            .name(name)
            .length(length)
            .constructor(true)
            .build();

        self.global_bindings_mut().insert(
            name.into(),
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
        );
        Ok(())
    }

    /// <https://tc39.es/ecma262/#sec-hasproperty>
    #[inline]
    pub(crate) fn has_property(&mut self, obj: &JsValue, key: &PropertyKey) -> JsResult<bool> {
        if let Some(obj) = obj.as_object() {
            obj.__has_property__(key, self)
        } else {
            Ok(false)
        }
    }

    /// Register a global class of type `T`, where `T` implements `Class`.
    ///
    /// # Example
    /// ```ignore
    /// #[derive(Debug, Trace, Finalize)]
    /// struct MyClass;
    ///
    /// impl Class for MyClass {
    ///    // ...
    /// }
    ///
    /// context.register_global_class::<MyClass>();
    /// ```
    #[inline]
    pub fn register_global_class<T>(&mut self) -> JsResult<()>
    where
        T: Class,
    {
        let mut class_builder = ClassBuilder::new::<T>(self);
        T::init(&mut class_builder)?;

        let class = class_builder.build();
        let property = PropertyDescriptor::builder()
            .value(class)
            .writable(T::ATTRIBUTES.writable())
            .enumerable(T::ATTRIBUTES.enumerable())
            .configurable(T::ATTRIBUTES.configurable())
            .build();

        self.global_bindings_mut().insert(T::NAME.into(), property);
        Ok(())
    }

    /// Register a global property.
    ///
    /// # Example
    /// ```
    /// use boa::{Context, property::{Attribute, PropertyDescriptor}, object::ObjectInitializer};
    ///
    /// let mut context = Context::default();
    ///
    /// context.register_global_property(
    ///     "myPrimitiveProperty",
    ///     10,
    ///     Attribute::all()
    /// );
    ///
    /// let object = ObjectInitializer::new(&mut context)
    ///    .property(
    ///         "x",
    ///         0,
    ///         Attribute::all()
    ///     )
    ///     .property(
    ///         "y",
    ///         1,
    ///         Attribute::all()
    ///     )
    ///    .build();
    /// context.register_global_property(
    ///     "myObjectProperty",
    ///     object,
    ///     Attribute::all()
    /// );
    /// ```
    #[inline]
    pub fn register_global_property<K, V>(&mut self, key: K, value: V, attribute: Attribute)
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        self.realm.global_property_map.insert(
            &key.into(),
            PropertyDescriptor::builder()
                .value(value)
                .writable(attribute.writable())
                .enumerable(attribute.enumerable())
                .configurable(attribute.configurable())
                .build(),
        );
    }

    /// Evaluates the given code by compiling down to bytecode, then interpreting the bytecode into a value
    ///
    /// # Examples
    /// ```
    ///# use boa::Context;
    /// let mut context = Context::default();
    ///
    /// let value = context.eval("1 + 3").unwrap();
    ///
    /// assert!(value.is_number());
    /// assert_eq!(value.as_number().unwrap(), 4.0);
    /// ```
    #[allow(clippy::unit_arg, clippy::drop_copy)]
    pub fn eval<S>(&mut self, src: S) -> JsResult<JsValue>
    where
        S: AsRef<[u8]>,
    {
        let main_timer = BoaProfiler::global().start_event("Evaluation", "Main");

        let parsing_result = Parser::new(src.as_ref(), false)
            .parse_all(&mut self.interner)
            .map_err(|e| e.to_string());

        let statement_list = match parsing_result {
            Ok(statement_list) => statement_list,
            Err(e) => return self.throw_syntax_error(e),
        };

        let code_block = self.compile(&statement_list)?;
        let result = self.execute(code_block);

        // The main_timer needs to be dropped before the BoaProfiler is.
        drop(main_timer);
        BoaProfiler::global().drop();

        result
    }

    /// Compile the AST into a `CodeBlock` ready to be executed by the VM.
    #[inline]
    pub fn compile(&mut self, statement_list: &StatementList) -> JsResult<Gc<CodeBlock>> {
        let _timer = BoaProfiler::global().start_event("Compilation", "Main");
        let mut compiler = ByteCompiler::new(Sym::MAIN, statement_list.strict(), self);
        for node in statement_list.items() {
            compiler.create_declarations(node)?;
        }
        compiler.compile_statement_list(statement_list, true)?;
        Ok(Gc::new(compiler.finish()))
    }

    /// Call the VM with a `CodeBlock` and return the result.
    ///
    /// Since this function receives a `Gc<CodeBlock>`, cloning the code is very cheap, since it's
    /// just a pointer copy. Therefore, if you'd like to execute the same `CodeBlock` multiple
    /// times, there is no need to re-compile it, and you can just call `clone()` on the
    /// `Gc<CodeBlock>` returned by the [`Self::compile()`] function.
    #[inline]
    pub fn execute(&mut self, code_block: Gc<CodeBlock>) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("Execution", "Main");
        let global_object = self.global_object().clone().into();

        self.vm.push_frame(CallFrame {
            prev: None,
            code: code_block,
            this: global_object,
            pc: 0,
            catch: Vec::new(),
            finally_return: FinallyReturn::None,
            finally_jump: Vec::new(),
            pop_on_return: 0,
            loop_env_stack: vec![0],
            try_env_stack: vec![crate::vm::TryStackEntry {
                num_env: 0,
                num_loop_stack_entries: 0,
            }],
            param_count: 0,
            arg_count: 0,
        });

        self.realm.set_global_binding_number();
        let result = self.run();
        self.vm.pop_frame();
        result
    }

    /// Return the cached iterator prototypes.
    #[inline]
    pub fn iterator_prototypes(&self) -> &IteratorPrototypes {
        &self.iterator_prototypes
    }

    /// Return the cached `TypedArray` constructor.
    #[inline]
    pub(crate) fn typed_array_constructor(&self) -> &StandardConstructor {
        &self.typed_array_constructor
    }

    /// Return the core standard objects.
    #[inline]
    pub fn standard_objects(&self) -> &StandardObjects {
        &self.standard_objects
    }

    /// Return the intrinsic objects.
    #[inline]
    pub fn intrinsics(&self) -> &IntrinsicObjects {
        &self.intrinsic_objects
    }

    /// Set the value of trace on the context
    pub fn set_trace(&mut self, trace: bool) {
        self.vm.trace = trace;
    }
}
