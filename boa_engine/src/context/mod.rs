//! Javascript context.

pub mod intrinsics;

use intrinsics::{IntrinsicObjects, Intrinsics};

use crate::{
    builtins::{self, function::NativeFunctionSignature},
    bytecompiler::ByteCompiler,
    class::{Class, ClassBuilder},
    object::{FunctionBuilder, GlobalPropertyMap, JsObject, ObjectData},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    syntax::{ast::node::StatementList, parser::ParseError, Parser},
    vm::{CallFrame, CodeBlock, FinallyReturn, GeneratorResumeKind, Vm},
    JsResult, JsValue,
};
use boa_gc::Gc;
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;

#[cfg(feature = "console")]
use crate::builtins::console::Console;

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
/// use boa_engine::{
///     Context,
///     object::ObjectInitializer,
///     property::{Attribute, PropertyDescriptor}
/// };
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

    /// Intrinsic objects
    intrinsics: Intrinsics,

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
            intrinsics: Intrinsics::default(),
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
        context.intrinsics.objects = IntrinsicObjects::init(&mut context);
        context.create_intrinsics();
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
        let _timer = Profiler::global().start_event("create_intrinsics", "interpreter");
        // Create intrinsics, add global objects here
        builtins::init(self);
    }

    /// Constructs an object with the `%Object.prototype%` prototype.
    #[inline]
    pub fn construct_object(&self) -> JsObject {
        JsObject::from_proto_and_data(
            self.intrinsics().constructors().object().prototype(),
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
            &self
                .intrinsics()
                .constructors()
                .error()
                .constructor()
                .into(),
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
                .intrinsics()
                .constructors()
                .range_error()
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
                .intrinsics()
                .constructors()
                .type_error()
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
                .intrinsics()
                .constructors()
                .reference_error()
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
                .intrinsics()
                .constructors()
                .syntax_error()
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
                .intrinsics()
                .constructors()
                .eval_error()
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
                .intrinsics()
                .constructors()
                .uri_error()
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
    /// to the global object, you can create the function object with
    /// [`FunctionBuilder`](crate::object::FunctionBuilder::native). And bind it to the global
    /// object with [`Context::register_global_property`](Context::register_global_property)
    /// method.
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
    /// to the global object, you can create the function object with
    /// [`FunctionBuilder`](crate::object::FunctionBuilder::closure). And bind it to the global
    /// object with [`Context::register_global_property`](Context::register_global_property)
    /// method.
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
    /// use boa_engine::{
    ///     Context,
    ///     property::{Attribute, PropertyDescriptor},
    ///     object::ObjectInitializer
    /// };
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
    ///# use boa_engine::Context;
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
        let main_timer = Profiler::global().start_event("Evaluation", "Main");

        let parsing_result = Parser::new(src.as_ref(), false)
            .parse_all(&mut self.interner)
            .map_err(|e| e.to_string());

        let statement_list = match parsing_result {
            Ok(statement_list) => statement_list,
            Err(e) => return self.throw_syntax_error(e),
        };

        let code_block = self.compile(&statement_list)?;
        let result = self.execute(code_block);

        // The main_timer needs to be dropped before the Profiler is.
        drop(main_timer);
        Profiler::global().drop();

        result
    }

    /// Compile the AST into a `CodeBlock` ready to be executed by the VM.
    #[inline]
    pub fn compile(&mut self, statement_list: &StatementList) -> JsResult<Gc<CodeBlock>> {
        let _timer = Profiler::global().start_event("Compilation", "Main");
        let mut compiler = ByteCompiler::new(Sym::MAIN, statement_list.strict(), self);
        compiler.create_declarations(statement_list.items())?;
        compiler.compile_statement_list(statement_list.items(), true)?;
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
        let _timer = Profiler::global().start_event("Execution", "Main");
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
            generator_resume_kind: GeneratorResumeKind::Normal,
        });

        self.realm.set_global_binding_number();
        let result = self.run();
        self.vm.pop_frame();
        let (result, _) = result?;
        Ok(result)
    }

    /// Return the intrinsic constructors and objects.
    #[inline]
    pub fn intrinsics(&self) -> &Intrinsics {
        &self.intrinsics
    }

    /// Set the value of trace on the context
    pub fn set_trace(&mut self, trace: bool) {
        self.vm.trace = trace;
    }
}
