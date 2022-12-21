//! The ECMAScript context.

pub mod intrinsics;

#[cfg(feature = "intl")]
pub(crate) mod icu;

#[cfg(feature = "intl")]
pub use icu::BoaProvider;

use intrinsics::{IntrinsicObjects, Intrinsics};
use std::collections::VecDeque;

#[cfg(feature = "console")]
use crate::builtins::console::Console;
use crate::{
    builtins::{self, function::NativeFunctionSignature},
    bytecompiler::ByteCompiler,
    class::{Class, ClassBuilder},
    job::JobCallback,
    object::{FunctionBuilder, GlobalPropertyMap, JsObject},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    vm::{CallFrame, CodeBlock, FinallyReturn, GeneratorResumeKind, Vm},
    JsResult, JsValue,
};

use boa_ast::StatementList;
use boa_gc::Gc;
use boa_interner::{Interner, Sym};
use boa_parser::{Error as ParseError, Parser};
use boa_profiler::Profiler;

/// ECMAScript context. It is the primary way to interact with the runtime.
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
///     object::ObjectInitializer,
///     property::{Attribute, PropertyDescriptor},
///     Context,
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
/// context.register_global_property("arg", arg, Attribute::all());
///
/// let value = context.eval("test(arg)").unwrap();
///
/// assert_eq!(value.as_number(), Some(12.0))
/// ```
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

    /// ICU related utilities
    #[cfg(feature = "intl")]
    icu: icu::Icu<BoaProvider>,

    /// Number of instructions remaining before a forced exit
    #[cfg(feature = "fuzz")]
    pub(crate) instructions_remaining: usize,

    pub(crate) vm: Vm,

    pub(crate) promise_job_queue: VecDeque<JobCallback>,

    pub(crate) kept_alive: Vec<JsObject>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Context");

        debug
            .field("realm", &self.realm)
            .field("interner", &self.interner);

        #[cfg(feature = "console")]
        debug.field("console", &self.console);

        debug
            .field("intrinsics", &self.intrinsics)
            .field("vm", &self.vm)
            .field("promise_job_queue", &self.promise_job_queue);

        #[cfg(feature = "intl")]
        debug.field("icu", &self.icu);

        debug.finish()
    }
}

impl Default for Context {
    fn default() -> Self {
        ContextBuilder::default().build()
    }
}

// ==== Public API ====

impl Context {
    /// Create a new [`ContextBuilder`] to specify the [`Interner`] and/or
    /// the icu data provider.
    #[must_use]
    pub fn builder() -> ContextBuilder {
        ContextBuilder::default()
    }

    /// Evaluates the given code by compiling down to bytecode, then interpreting the bytecode into a value
    ///
    /// # Examples
    /// ```
    /// # use boa_engine::Context;
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

        let statement_list = self.parse(src)?;
        let code_block = self.compile(&statement_list)?;
        let result = self.execute(code_block);

        // The main_timer needs to be dropped before the Profiler is.
        drop(main_timer);
        Profiler::global().drop();

        result
    }

    /// Parse the given source text.
    pub fn parse<S>(&mut self, src: S) -> Result<StatementList, ParseError>
    where
        S: AsRef<[u8]>,
    {
        let _timer = Profiler::global().start_event("Parsing", "Main");
        let mut parser = Parser::new(src.as_ref());
        parser.parse_all(&mut self.interner)
    }

    /// Compile the AST into a `CodeBlock` ready to be executed by the VM.
    pub fn compile(&mut self, statement_list: &StatementList) -> JsResult<Gc<CodeBlock>> {
        let _timer = Profiler::global().start_event("Compilation", "Main");
        let mut compiler = ByteCompiler::new(Sym::MAIN, statement_list.strict(), false, self);
        compiler.create_decls(statement_list, false);
        compiler.compile_statement_list(statement_list, true, false)?;
        Ok(Gc::new(compiler.finish()))
    }

    /// Call the VM with a `CodeBlock` and return the result.
    ///
    /// Since this function receives a `Gc<CodeBlock>`, cloning the code is very cheap, since it's
    /// just a pointer copy. Therefore, if you'd like to execute the same `CodeBlock` multiple
    /// times, there is no need to re-compile it, and you can just call `clone()` on the
    /// `Gc<CodeBlock>` returned by the [`Self::compile()`] function.
    pub fn execute(&mut self, code_block: Gc<CodeBlock>) -> JsResult<JsValue> {
        let _timer = Profiler::global().start_event("Execution", "Main");

        self.vm.push_frame(CallFrame {
            code: code_block,
            pc: 0,
            catch: Vec::new(),
            finally_return: FinallyReturn::None,
            finally_jump: Vec::new(),
            pop_on_return: 0,
            loop_env_stack: Vec::from([0]),
            try_env_stack: Vec::from([crate::vm::TryStackEntry {
                num_env: 0,
                num_loop_stack_entries: 0,
            }]),
            param_count: 0,
            arg_count: 0,
            generator_resume_kind: GeneratorResumeKind::Normal,
            thrown: false,
            async_generator: None,
        });

        self.realm.set_global_binding_number();
        let result = self.run();
        self.vm.pop_frame();
        self.clear_kept_objects();
        self.run_queued_jobs()?;
        let (result, _) = result?;
        Ok(result)
    }

    /// Register a global property.
    ///
    /// # Example
    /// ```
    /// use boa_engine::{
    ///     object::ObjectInitializer,
    ///     property::{Attribute, PropertyDescriptor},
    ///     Context,
    /// };
    ///
    /// let mut context = Context::default();
    ///
    /// context.register_global_property("myPrimitiveProperty", 10, Attribute::all());
    ///
    /// let object = ObjectInitializer::new(&mut context)
    ///     .property("x", 0, Attribute::all())
    ///     .property("y", 1, Attribute::all())
    ///     .build();
    /// context.register_global_property("myObjectProperty", object, Attribute::all());
    /// ```
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

    /// Register a global native function.
    ///
    /// This is more efficient that creating a closure function, since this does not allocate,
    /// it is just a function pointer.
    ///
    /// The function will be both `constructable` (call with `new <name>()`) and `callable` (call
    /// with `<name>()`).
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

    /// Gets the string interner.
    #[inline]
    pub const fn interner(&self) -> &Interner {
        &self.interner
    }

    /// Gets a mutable reference to the string interner.
    #[inline]
    pub fn interner_mut(&mut self) -> &mut Interner {
        &mut self.interner
    }

    /// Return the global object.
    #[inline]
    pub const fn global_object(&self) -> &JsObject {
        self.realm.global_object()
    }

    /// Return the intrinsic constructors and objects.
    #[inline]
    pub const fn intrinsics(&self) -> &Intrinsics {
        &self.intrinsics
    }

    /// Set the value of trace on the context
    pub fn set_trace(&mut self, trace: bool) {
        self.vm.trace = trace;
    }

    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hostenqueuepromisejob
    pub fn host_enqueue_promise_job(&mut self, job: JobCallback /* , realm: Realm */) {
        // If realm is not null ...
        // TODO
        // Let scriptOrModule be ...
        // TODO
        self.promise_job_queue.push_back(job);
    }

    /// Abstract operation [`ClearKeptObjects`][clear].
    ///
    /// Clears all objects maintained alive by calls to the [`AddToKeptObjects`][add] abstract
    /// operation, used within the [`WeakRef`][weak] constructor.
    ///
    /// [clear]: https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-clear-kept-objects
    /// [add]: https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-addtokeptobjects
    /// [weak]: https://tc39.es/ecma262/multipage/managing-memory.html#sec-weak-ref-objects
    pub fn clear_kept_objects(&mut self) {
        self.kept_alive.clear();
    }
}

// ==== Private API ====

impl Context {
    /// A helper function for getting an immutable reference to the `console` object.
    #[cfg(feature = "console")]
    pub(crate) const fn console(&self) -> &Console {
        &self.console
    }

    /// A helper function for getting a mutable reference to the `console` object.
    #[cfg(feature = "console")]
    pub(crate) fn console_mut(&mut self) -> &mut Console {
        &mut self.console
    }

    /// Return a mutable reference to the global object string bindings.
    pub(crate) fn global_bindings_mut(&mut self) -> &mut GlobalPropertyMap {
        self.realm.global_bindings_mut()
    }

    /// Compile the AST into a `CodeBlock` ready to be executed by the VM in a `JSON.parse` context.
    pub(crate) fn compile_json_parse(
        &mut self,
        statement_list: &StatementList,
    ) -> JsResult<Gc<CodeBlock>> {
        let _timer = Profiler::global().start_event("Compilation", "Main");
        let mut compiler = ByteCompiler::new(Sym::MAIN, statement_list.strict(), true, self);
        compiler.create_decls(statement_list, false);
        compiler.compile_statement_list(statement_list, true, false)?;
        Ok(Gc::new(compiler.finish()))
    }

    /// Compile the AST into a `CodeBlock` with an additional declarative environment.
    pub(crate) fn compile_with_new_declarative(
        &mut self,
        statement_list: &StatementList,
        strict: bool,
    ) -> JsResult<Gc<CodeBlock>> {
        let _timer = Profiler::global().start_event("Compilation", "Main");
        let mut compiler = ByteCompiler::new(Sym::MAIN, statement_list.strict(), false, self);
        compiler.compile_statement_list_with_new_declarative(statement_list, true, strict)?;
        Ok(Gc::new(compiler.finish()))
    }

    /// Get the ICU related utilities
    #[cfg(feature = "intl")]
    pub(crate) const fn icu(&self) -> &icu::Icu<BoaProvider> {
        &self.icu
    }

    /// Sets up the default global objects within Global
    fn create_intrinsics(&mut self) {
        let _timer = Profiler::global().start_event("create_intrinsics", "interpreter");
        // Create intrinsics, add global objects here
        builtins::init(self);
    }

    /// Runs all the jobs in the job queue.
    fn run_queued_jobs(&mut self) -> JsResult<()> {
        while let Some(job) = self.promise_job_queue.pop_front() {
            job.call_job_callback(&JsValue::Undefined, &[], self)?;
            self.clear_kept_objects();
        }
        Ok(())
    }
}
/// Builder for the [`Context`] type.
///
/// This builder allows custom initialization of the [`Interner`] within
/// the context.
/// Additionally, if the `intl` feature is enabled, [`ContextBuilder`] becomes
/// the only way to create a new [`Context`], since now it requires a
/// valid data provider for the `Intl` functionality.
#[cfg_attr(
    feature = "intl",
    doc = "The required data in a valid provider is specified in [`BoaProvider`]"
)]
#[derive(Default, Debug)]
pub struct ContextBuilder {
    interner: Option<Interner>,
    #[cfg(feature = "intl")]
    icu: Option<icu::Icu<BoaProvider>>,
    #[cfg(feature = "fuzz")]
    instructions_remaining: usize,
}

impl ContextBuilder {
    /// Initializes the context [`Interner`] to the provided interner.
    ///
    /// This is useful when you want to initialize an [`Interner`] with
    /// a collection of words before parsing.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn interner(mut self, interner: Interner) -> Self {
        self.interner = Some(interner);
        self
    }

    /// Provides an icu data provider to the [`Context`].
    ///
    /// This function is only available if the `intl` feature is enabled.
    #[cfg(feature = "intl")]
    pub fn icu_provider(
        mut self,
        provider: BoaProvider,
    ) -> Result<Self, icu_locid_transform::LocaleTransformError> {
        self.icu = Some(icu::Icu::new(provider)?);
        Ok(self)
    }

    /// Specifies the number of instructions remaining to the [`Context`].
    ///
    /// This function is only available if the `fuzz` feature is enabled.
    #[cfg(feature = "fuzz")]
    #[must_use]
    pub const fn instructions_remaining(mut self, instructions_remaining: usize) -> Self {
        self.instructions_remaining = instructions_remaining;
        self
    }

    /// Creates a new [`ContextBuilder`] with a default empty [`Interner`]
    /// and a default [`BoaProvider`] if the `intl` feature is enabled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a new [`Context`] with the provided parameters, and defaults
    /// all missing parameters to their default values.
    #[must_use]
    pub fn build(self) -> Context {
        let intrinsics = Intrinsics::default();
        let mut context = Context {
            realm: Realm::create(intrinsics.constructors().object().prototype().into()),
            interner: self.interner.unwrap_or_default(),
            #[cfg(feature = "console")]
            console: Console::default(),
            intrinsics,
            vm: Vm {
                frames: Vec::with_capacity(16),
                stack: Vec::with_capacity(1024),
                trace: false,
                stack_size_limit: 1024,
            },
            #[cfg(feature = "intl")]
            icu: self.icu.unwrap_or_else(|| {
                use icu_provider_adapters::fallback::LocaleFallbackProvider;
                let provider = BoaProvider::Buffer(Box::new(
                    LocaleFallbackProvider::try_new_with_buffer_provider(boa_icu_provider::blob())
                        .expect("boa_icu_provider should return a valid provider"),
                ));
                icu::Icu::new(provider).expect("Failed to initialize default icu data.")
            }),
            #[cfg(feature = "fuzz")]
            instructions_remaining: self.instructions_remaining,
            promise_job_queue: VecDeque::new(),
            kept_alive: Vec::new(),
        };

        // Add new builtIns to Context Realm
        // At a later date this can be removed from here and called explicitly,
        // but for now we almost always want these default builtins
        context.intrinsics.objects = IntrinsicObjects::init(&mut context);
        context.create_intrinsics();
        context
    }
}
