//! Javascript context.

use crate::{
    builtins::{
        self,
        function::{Function, NativeFunctionSignature, ThisMode},
        intrinsics::IntrinsicObjects,
        iterable::IteratorPrototypes,
        typed_array::TypedArray,
    },
    class::{Class, ClassBuilder},
    exec::Interpreter,
    object::PROTOTYPE,
    object::{FunctionBuilder, JsObject, ObjectData},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    syntax::{
        ast::{
            node::{statement_list::RcStatementList, FormalParameter, StatementList},
            Node,
        },
        Parser,
    },
    BoaProfiler, Executable, JsResult, JsString, JsValue,
};

#[cfg(feature = "console")]
use crate::builtins::console::Console;

#[cfg(feature = "vm")]
use crate::vm::Vm;

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

/// Internal representation of the strict mode types.
#[derive(Debug, Copy, Clone)]
pub(crate) enum StrictType {
    Off,
    Global,
    Function,
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
/// let mut context = Context::new();
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

    /// The current executor.
    executor: Interpreter,

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

    /// Whether or not strict mode is active.
    strict: StrictType,

    #[cfg(feature = "vm")]
    pub(crate) vm: Vm,
}

impl Default for Context {
    fn default() -> Self {
        let realm = Realm::create();
        let executor = Interpreter::new();
        let mut context = Self {
            realm,
            executor,
            #[cfg(feature = "console")]
            console: Console::default(),
            iterator_prototypes: IteratorPrototypes::default(),
            typed_array_constructor: StandardConstructor::default(),
            standard_objects: Default::default(),
            intrinsic_objects: IntrinsicObjects::default(),
            strict: StrictType::Off,
            #[cfg(feature = "vm")]
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
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn executor(&mut self) -> &mut Interpreter {
        &mut self.executor
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
        matches!(self.strict, StrictType::Global | StrictType::Function)
    }

    /// Returns the strict mode type.
    #[inline]
    pub(crate) fn strict_type(&self) -> StrictType {
        self.strict
    }

    /// Set strict type.
    #[inline]
    pub(crate) fn set_strict(&mut self, strict: StrictType) {
        self.strict = strict;
    }

    /// Disable the strict mode.
    #[inline]
    pub fn set_strict_mode_off(&mut self) {
        self.strict = StrictType::Off;
    }

    /// Enable the global strict mode.
    #[inline]
    pub fn set_strict_mode_global(&mut self) {
        self.strict = StrictType::Global;
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
    pub fn global_object(&self) -> JsObject {
        self.realm.global_object.clone()
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

    /// Utility to create a function Value for Function Declarations, Arrow Functions or Function Expressions
    pub(crate) fn create_function<N, P>(
        &mut self,
        name: N,
        params: P,
        mut body: StatementList,
        constructor: bool,
        this_mode: ThisMode,
    ) -> JsResult<JsValue>
    where
        N: Into<JsString>,
        P: Into<Box<[FormalParameter]>>,
    {
        let name = name.into();
        let function_prototype = self.standard_objects().function_object().prototype();

        // Every new function has a prototype property pre-made
        let prototype = self.construct_object();

        // If a function is defined within a strict context, it is strict.
        if self.strict() {
            body.set_strict(true);
        }

        let params = params.into();
        let params_len = params.len();
        let func = Function::Ordinary {
            constructor,
            this_mode,
            body: RcStatementList::from(body),
            params,
            environment: self.get_current_environment().clone(),
        };

        let function =
            JsObject::from_proto_and_data(function_prototype, ObjectData::function(func));

        // Set constructor field to the newly created Value (function object)
        let constructor = PropertyDescriptor::builder()
            .value(function.clone())
            .writable(true)
            .enumerable(false)
            .configurable(true);
        prototype.define_property_or_throw("constructor", constructor, self)?;

        let prototype = PropertyDescriptor::builder()
            .value(prototype)
            .writable(true)
            .enumerable(false)
            .configurable(false);
        function.define_property_or_throw(PROTOTYPE, prototype, self)?;

        let length = PropertyDescriptor::builder()
            .value(params_len)
            .writable(false)
            .enumerable(false)
            .configurable(true);
        function.define_property_or_throw("length", length, self)?;

        let name = PropertyDescriptor::builder()
            .value(name)
            .writable(false)
            .enumerable(false)
            .configurable(true);
        function.define_property_or_throw("name", name, self)?;

        Ok(function.into())
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
    ) -> JsResult<()> {
        let function = FunctionBuilder::native(self, body)
            .name(name)
            .length(length)
            .constructor(true)
            .build();

        self.global_object().insert_property(
            name,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        Ok(())
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
        F: Fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue> + Copy + 'static,
    {
        let function = FunctionBuilder::closure(self, body)
            .name(name)
            .length(length)
            .constructor(true)
            .build();

        self.global_object().insert_property(
            name,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
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

    #[inline]
    pub(crate) fn set_value(&mut self, node: &Node, value: JsValue) -> JsResult<JsValue> {
        match node {
            Node::Identifier(ref name) => {
                self.set_mutable_binding(name.as_ref(), value.clone(), true)?;
                Ok(value)
            }
            Node::GetConstField(ref get_const_field_node) => Ok(get_const_field_node
                .obj()
                .run(self)?
                .set_field(get_const_field_node.field(), value, false, self)?),
            Node::GetField(ref get_field) => {
                let field = get_field.field().run(self)?;
                let key = field.to_property_key(self)?;
                Ok(get_field
                    .obj()
                    .run(self)?
                    .set_field(key, value, false, self)?)
            }
            _ => self.throw_type_error(format!("invalid assignment to {}", node)),
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
            .configurable(T::ATTRIBUTES.configurable());
        self.global_object().insert(T::NAME, property);
        Ok(())
    }

    /// Register a global property.
    ///
    /// # Example
    /// ```
    /// use boa::{Context, property::{Attribute, PropertyDescriptor}, object::ObjectInitializer};
    ///
    /// let mut context = Context::new();
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
        self.global_object().insert(
            key,
            PropertyDescriptor::builder()
                .value(value)
                .writable(attribute.writable())
                .enumerable(attribute.enumerable())
                .configurable(attribute.configurable()),
        );
    }

    /// Evaluates the given code.
    ///
    /// # Examples
    /// ```
    ///# use boa::Context;
    /// let mut context = Context::new();
    ///
    /// let value = context.eval("1 + 3").unwrap();
    ///
    /// assert!(value.is_number());
    /// assert_eq!(value.as_number().unwrap(), 4.0);
    /// ```
    #[cfg(not(feature = "vm"))]
    #[allow(clippy::unit_arg, clippy::drop_copy)]
    #[inline]
    pub fn eval<T: AsRef<[u8]>>(&mut self, src: T) -> JsResult<JsValue> {
        let main_timer = BoaProfiler::global().start_event("Main", "Main");
        let src_bytes: &[u8] = src.as_ref();

        let parsing_result = Parser::new(src_bytes, false)
            .parse_all()
            .map_err(|e| e.to_string());

        let execution_result = match parsing_result {
            Ok(statement_list) => {
                if statement_list.strict() {
                    self.set_strict_mode_global();
                }
                statement_list.run(self)
            }
            Err(e) => self.throw_syntax_error(e),
        };

        // The main_timer needs to be dropped before the BoaProfiler is.
        drop(main_timer);
        BoaProfiler::global().drop();

        execution_result
    }

    /// Evaluates the given code by compiling down to bytecode, then interpreting the bytecode into a value
    ///
    /// # Examples
    /// ```
    ///# use boa::Context;
    /// let mut context = Context::new();
    ///
    /// let value = context.eval("1 + 3").unwrap();
    ///
    /// assert!(value.is_number());
    /// assert_eq!(value.as_number().unwrap(), 4.0);
    /// ```
    #[cfg(feature = "vm")]
    #[allow(clippy::unit_arg, clippy::drop_copy)]
    pub fn eval<T: AsRef<[u8]>>(&mut self, src: T) -> JsResult<JsValue> {
        use gc::Gc;

        use crate::vm::CallFrame;

        let main_timer = BoaProfiler::global().start_event("Main", "Main");
        let src_bytes: &[u8] = src.as_ref();

        let parsing_result = Parser::new(src_bytes, false)
            .parse_all()
            .map_err(|e| e.to_string());

        let statement_list = match parsing_result {
            Ok(statement_list) => statement_list,
            Err(e) => return self.throw_syntax_error(e),
        };

        let mut compiler = crate::bytecompiler::ByteCompiler::new(
            JsString::new("<main>"),
            statement_list.strict(),
        );
        compiler.compile_statement_list(&statement_list, true);
        let code_block = compiler.finish();

        let environment = self.get_current_environment().clone();
        let fp = self.vm.stack.len();
        let global_object = self.global_object().into();

        self.vm.push_frame(CallFrame {
            prev: None,
            code: Gc::new(code_block),
            this: global_object,
            pc: 0,
            fp,
            environment,
            catch: None,
            has_thrown: false,
            finally_jump: None,
            pop_env_on_return: 0,
            param_count: 0,
            arg_count: 0,
        });
        let result = self.run();

        // The main_timer needs to be dropped before the BoaProfiler is.
        drop(main_timer);
        BoaProfiler::global().drop();

        result
    }

    /// Return the cached iterator prototypes.
    #[inline]
    pub fn iterator_prototypes(&self) -> &IteratorPrototypes {
        &self.iterator_prototypes
    }

    /// Return the cached TypedArray constructor.
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
    #[cfg(feature = "vm")]
    pub fn set_trace(&mut self, trace: bool) {
        self.vm.trace = trace;
    }
}
