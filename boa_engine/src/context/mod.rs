//! The ECMAScript context.

mod hooks;
pub mod intrinsics;
use hooks::DefaultHooks;
pub use hooks::HostHooks;

#[cfg(feature = "intl")]
pub(crate) mod icu;

#[cfg(feature = "intl")]
pub use icu::BoaProvider;

use intrinsics::{IntrinsicObjects, Intrinsics};

use std::io::Read;
#[cfg(not(feature = "intl"))]
pub use std::marker::PhantomData;

#[cfg(feature = "console")]
use crate::builtins::console::Console;
use crate::{
    builtins,
    bytecompiler::ByteCompiler,
    class::{Class, ClassBuilder},
    job::{IdleJobQueue, JobQueue, NativeJob},
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, GlobalPropertyMap, JsObject},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    vm::{CallFrame, CodeBlock, Vm},
    JsResult, JsValue, Source,
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
///     Source
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
/// context.eval(Source::from_bytes(script)).unwrap();
///
/// // Create an object that can be used in eval calls.
/// let arg = ObjectInitializer::new(&mut context)
///     .property("x", 12, Attribute::READONLY)
///     .build();
/// context.register_global_property("arg", arg, Attribute::all());
///
/// let value = context.eval(Source::from_bytes("test(arg)")).unwrap();
///
/// assert_eq!(value.as_number(), Some(12.0))
/// ```
pub struct Context<'host> {
    /// realm holds both the global object and the environment
    pub(crate) realm: Realm,

    /// String interner in the context.
    interner: Interner,

    /// console object state.
    #[cfg(feature = "console")]
    console: Console,

    /// Intrinsic objects
    intrinsics: Intrinsics,

    /// Execute in strict mode,
    strict: bool,

    /// Number of instructions remaining before a forced exit
    #[cfg(feature = "fuzz")]
    pub(crate) instructions_remaining: usize,

    pub(crate) vm: Vm,

    pub(crate) kept_alive: Vec<JsObject>,

    /// ICU related utilities
    #[cfg(feature = "intl")]
    icu: icu::Icu<'host>,

    host_hooks: &'host dyn HostHooks,

    job_queue: &'host dyn JobQueue,
}

impl std::fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Context");

        debug
            .field("realm", &self.realm)
            .field("interner", &self.interner)
            .field("intrinsics", &self.intrinsics)
            .field("vm", &self.vm)
            .field("strict", &self.strict)
            .field("promise_job_queue", &"JobQueue")
            .field("hooks", &"HostHooks");

        #[cfg(feature = "console")]
        debug.field("console", &self.console);

        #[cfg(feature = "intl")]
        debug.field("icu", &self.icu);

        debug.finish()
    }
}

impl Default for Context<'_> {
    fn default() -> Self {
        ContextBuilder::default().build()
    }
}

// ==== Public API ====

impl Context<'_> {
    /// Create a new [`ContextBuilder`] to specify the [`Interner`] and/or
    /// the icu data provider.
    #[must_use]
    pub fn builder() -> ContextBuilder<'static, 'static, 'static> {
        ContextBuilder::default()
    }

    /// Evaluates the given script `Source` by compiling down to bytecode, then interpreting the
    /// bytecode into a value.
    ///
    /// # Examples
    /// ```
    /// # use boa_engine::{Context, Source};
    /// let mut context = Context::default();
    ///
    /// let source = Source::from_bytes("1 + 3");
    /// let value = context.eval(source).unwrap();
    ///
    /// assert!(value.is_number());
    /// assert_eq!(value.as_number().unwrap(), 4.0);
    /// ```
    ///
    /// Note that this won't run any scheduled promise jobs; you need to call [`Context::run_jobs`]
    /// on the context or [`JobQueue::run_jobs`] on the provided queue to run them.
    #[allow(clippy::unit_arg, clippy::drop_copy)]
    pub fn eval<R: Read>(&mut self, src: Source<'_, R>) -> JsResult<JsValue> {
        let main_timer = Profiler::global().start_event("Evaluation", "Main");

        let script = self.parse(src)?;
        let code_block = self.compile(&script)?;
        let result = self.execute(code_block);

        // The main_timer needs to be dropped before the Profiler is.
        drop(main_timer);
        Profiler::global().drop();

        result
    }

    /// Parse the given source script.
    pub fn parse<R: Read>(&mut self, src: Source<'_, R>) -> Result<StatementList, ParseError> {
        let _timer = Profiler::global().start_event("Parsing", "Main");
        let mut parser = Parser::new(src);
        if self.strict {
            parser.set_strict();
        }
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
    ///
    /// Note that this won't run any scheduled promise jobs; you need to call [`Context::run_jobs`]
    /// on the context or [`JobQueue::run_jobs`] on the provided queue to run them.
    pub fn execute(&mut self, code_block: Gc<CodeBlock>) -> JsResult<JsValue> {
        let _timer = Profiler::global().start_event("Execution", "Main");

        self.vm.push_frame(CallFrame::new(code_block));

        self.realm.set_global_binding_number();
        let result = self.run();
        self.vm.pop_frame();
        self.clear_kept_objects();

        result.map(|r| r.0)
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

    /// Register a global native callable.
    ///
    /// The function will be both `constructable` (call with `new <name>()`) and `callable` (call
    /// with `<name>()`).
    ///
    /// The function will be bound to the global object with `writable`, `non-enumerable`
    /// and `configurable` attributes. The same as when you create a function in JavaScript.
    ///
    /// # Note
    ///
    /// If you wish to only create the function object without binding it to the global object, you
    /// can use the [`FunctionObjectBuilder`] API.
    pub fn register_global_callable(&mut self, name: &str, length: usize, body: NativeFunction) {
        let function = FunctionObjectBuilder::new(self, body)
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
    /// The function will be bound to the global object with `writable`, `non-enumerable`
    /// and `configurable` attributes. The same as when you create a function in JavaScript.
    ///
    /// # Note
    ///
    /// The difference to [`Context::register_global_callable`] is, that the function will not be
    /// `constructable`. Usage of the function as a constructor will produce a `TypeError`.
    pub fn register_global_builtin_callable(
        &mut self,
        name: &str,
        length: usize,
        body: NativeFunction,
    ) {
        let function = FunctionObjectBuilder::new(self, body)
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

    /// Executes all code in strict mode.
    pub fn strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    /// Enqueues a [`NativeJob`] on the [`JobQueue`].
    pub fn enqueue_job(&mut self, job: NativeJob) {
        self.job_queue.enqueue_promise_job(job, self);
    }

    /// Runs all the jobs in the job queue.
    pub fn run_jobs(&mut self) {
        self.job_queue.run_jobs(self);
        self.clear_kept_objects();
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

impl Context<'_> {
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

    /// Sets up the default global objects within Global
    fn create_intrinsics(&mut self) {
        let _timer = Profiler::global().start_event("create_intrinsics", "interpreter");
        // Create intrinsics, add global objects here
        builtins::init(self);
    }
}

impl<'host> Context<'host> {
    /// Get the host hooks.
    pub(crate) fn host_hooks(&self) -> &'host dyn HostHooks {
        self.host_hooks
    }

    /// Get the job queue.
    pub(crate) fn job_queue(&mut self) -> &'host dyn JobQueue {
        self.job_queue
    }

    /// Get the ICU related utilities
    #[cfg(feature = "intl")]
    pub(crate) const fn icu(&self) -> &icu::Icu<'host> {
        &self.icu
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
#[derive(Default)]
pub struct ContextBuilder<'icu, 'hooks, 'queue> {
    interner: Option<Interner>,
    host_hooks: Option<&'hooks dyn HostHooks>,
    job_queue: Option<&'queue dyn JobQueue>,
    #[cfg(feature = "intl")]
    icu: Option<icu::Icu<'icu>>,
    #[cfg(not(feature = "intl"))]
    icu: PhantomData<&'icu ()>,
    #[cfg(feature = "fuzz")]
    instructions_remaining: usize,
}

impl std::fmt::Debug for ContextBuilder<'_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = f.debug_struct("ContextBuilder");

        out.field("interner", &self.interner)
            .field("host_hooks", &"HostHooks");

        #[cfg(feature = "intl")]
        out.field("icu", &self.icu);

        #[cfg(feature = "fuzz")]
        out.field("instructions_remaining", &self.instructions_remaining);

        out.finish()
    }
}

impl<'icu, 'hooks, 'queue> ContextBuilder<'icu, 'hooks, 'queue> {
    /// Creates a new [`ContextBuilder`] with a default empty [`Interner`]
    /// and a default [`BoaProvider`] if the `intl` feature is enabled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

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
    ///
    /// # Errors
    ///
    /// This returns `Err` if the provided provider doesn't have the required locale information
    /// to construct both a [`LocaleCanonicalizer`] and a [`LocaleExpander`]. Note that this doesn't
    /// mean that the provider will successfully construct all `Intl` services; that check is made
    /// until the creation of an instance of a service.
    ///
    /// [`LocaleCanonicalizer`]: icu_locid_transform::LocaleCanonicalizer
    /// [`LocaleExpander`]: icu_locid_transform::LocaleExpander
    #[cfg(feature = "intl")]
    pub fn icu_provider(
        self,
        provider: BoaProvider<'_>,
    ) -> Result<ContextBuilder<'_, 'hooks, 'queue>, icu_locid_transform::LocaleTransformError> {
        Ok(ContextBuilder {
            icu: Some(icu::Icu::new(provider)?),
            ..self
        })
    }

    /// Initializes the [`HostHooks`] for the context.
    ///
    /// [`Host Hooks`]: https://tc39.es/ecma262/#sec-host-hooks-summary
    #[must_use]
    pub fn host_hooks(self, host_hooks: &dyn HostHooks) -> ContextBuilder<'icu, '_, 'queue> {
        ContextBuilder {
            host_hooks: Some(host_hooks),
            ..self
        }
    }

    /// Initializes the [`JobQueue`] for the context.
    #[must_use]
    pub fn job_queue(self, job_queue: &dyn JobQueue) -> ContextBuilder<'icu, 'hooks, '_> {
        ContextBuilder {
            job_queue: Some(job_queue),
            ..self
        }
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

    /// Builds a new [`Context`] with the provided parameters, and defaults
    /// all missing parameters to their default values.
    #[must_use]
    pub fn build<'host>(self) -> Context<'host>
    where
        'icu: 'host,
        'hooks: 'host,
        'queue: 'host,
    {
        let intrinsics = Intrinsics::default();
        let mut context = Context {
            realm: Realm::create(intrinsics.constructors().object().prototype().into()),
            interner: self.interner.unwrap_or_default(),
            #[cfg(feature = "console")]
            console: Console::default(),
            intrinsics,
            vm: Vm::default(),
            strict: false,
            #[cfg(feature = "intl")]
            icu: self.icu.unwrap_or_else(|| {
                let provider = BoaProvider::Buffer(boa_icu_provider::buffer());
                icu::Icu::new(provider).expect("Failed to initialize default icu data.")
            }),
            #[cfg(feature = "fuzz")]
            instructions_remaining: self.instructions_remaining,
            kept_alive: Vec::new(),
            host_hooks: self.host_hooks.unwrap_or(&DefaultHooks),
            job_queue: self.job_queue.unwrap_or(&IdleJobQueue),
        };

        // Add new builtIns to Context Realm
        // At a later date this can be removed from here and called explicitly,
        // but for now we almost always want these default builtins
        context.intrinsics.objects = IntrinsicObjects::init(&mut context);
        context.create_intrinsics();
        context
    }
}
