//! The ECMAScript context.

mod hooks;
#[cfg(feature = "intl")]
pub(crate) mod icu;
pub mod intrinsics;
mod maybe_shared;

pub use hooks::{DefaultHooks, HostHooks};
#[cfg(feature = "intl")]
pub use icu::{BoaProvider, IcuError};
use intrinsics::Intrinsics;
pub use maybe_shared::MaybeShared;

#[cfg(not(feature = "intl"))]
pub use std::marker::PhantomData;
use std::{io::Read, path::Path, rc::Rc};

use crate::{
    builtins,
    class::{Class, ClassBuilder},
    job::{JobQueue, NativeJob, SimpleJobQueue},
    module::{IdleModuleLoader, ModuleLoader, SimpleModuleLoader},
    native_function::NativeFunction,
    object::{shape::RootShape, FunctionObjectBuilder, JsObject},
    optimizer::{Optimizer, OptimizerOptions, OptimizerStatistics},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    script::Script,
    vm::{ActiveRunnable, CallFrame, Vm},
    JsResult, JsValue, Source,
};
use boa_ast::{expression::Identifier, StatementList};
use boa_interner::Interner;
use boa_profiler::Profiler;

use crate::vm::RuntimeLimits;

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
///     Context, Source,
/// };
///
/// let script = r#"
///     function test(arg1) {
///         if(arg1 != null) {
///             return arg1.x;
///         }
///         return 112233;
///     }
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
    realm: Realm,

    /// String interner in the context.
    interner: Interner,

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

    host_hooks: MaybeShared<'host, dyn HostHooks>,

    job_queue: MaybeShared<'host, dyn JobQueue>,

    module_loader: MaybeShared<'host, dyn ModuleLoader>,

    optimizer_options: OptimizerOptions,
    root_shape: RootShape,

    /// Unique identifier for each parser instance used during the context lifetime.
    parser_identifier: u32,
}

impl std::fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Context");

        debug
            .field("realm", &self.realm)
            .field("interner", &self.interner)
            .field("vm", &self.vm)
            .field("strict", &self.strict)
            .field("promise_job_queue", &"JobQueue")
            .field("hooks", &"HostHooks")
            .field("module_loader", &"ModuleLoader")
            .field("optimizer_options", &self.optimizer_options);

        #[cfg(feature = "intl")]
        debug.field("icu", &self.icu);

        debug.finish()
    }
}

impl Default for Context<'_> {
    fn default() -> Self {
        ContextBuilder::default()
            .build()
            .expect("Building the default context should not fail")
    }
}

// ==== Public API ====
impl<'host> Context<'host> {
    /// Create a new [`ContextBuilder`] to specify the [`Interner`] and/or
    /// the icu data provider.
    #[must_use]
    pub fn builder() -> ContextBuilder<'static, 'static, 'static, 'static> {
        ContextBuilder::default()
    }

    /// Evaluates the given source by compiling down to bytecode, then interpreting the
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
    #[allow(clippy::unit_arg, dropping_copy_types)]
    pub fn eval<R: Read>(&mut self, src: Source<'_, R>) -> JsResult<JsValue> {
        let main_timer = Profiler::global().start_event("Script evaluation", "Main");

        let result = Script::parse(src, None, self)?.evaluate(self);

        // The main_timer needs to be dropped before the Profiler is.
        drop(main_timer);
        Profiler::global().drop();

        result
    }

    /// Applies optimizations to the [`StatementList`] inplace.
    pub fn optimize_statement_list(
        &mut self,
        statement_list: &mut StatementList,
    ) -> OptimizerStatistics {
        let mut optimizer = Optimizer::new(self);
        optimizer.apply(statement_list)
    }

    /// Register a global property.
    ///
    /// It will return an error if the property is already defined.
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
    /// context
    ///     .register_global_property("myPrimitiveProperty", 10, Attribute::all())
    ///     .expect("property shouldn't exist");
    ///
    /// let object = ObjectInitializer::new(&mut context)
    ///     .property("x", 0, Attribute::all())
    ///     .property("y", 1, Attribute::all())
    ///     .build();
    /// context
    ///     .register_global_property("myObjectProperty", object, Attribute::all())
    ///     .expect("property shouldn't exist");
    /// ```
    pub fn register_global_property<K, V>(
        &mut self,
        key: K,
        value: V,
        attribute: Attribute,
    ) -> JsResult<()>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        self.global_object().define_property_or_throw(
            key,
            PropertyDescriptor::builder()
                .value(value)
                .writable(attribute.writable())
                .enumerable(attribute.enumerable())
                .configurable(attribute.configurable()),
            self,
        )?;
        Ok(())
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
    pub fn register_global_callable(
        &mut self,
        name: &str,
        length: usize,
        body: NativeFunction,
    ) -> JsResult<()> {
        let function = FunctionObjectBuilder::new(&self.realm, body)
            .name(name)
            .length(length)
            .constructor(true)
            .build();

        self.global_object().define_property_or_throw(
            name,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
            self,
        )?;
        Ok(())
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
    ) -> JsResult<()> {
        let function = FunctionObjectBuilder::new(&self.realm, body)
            .name(name)
            .length(length)
            .constructor(false)
            .build();

        self.global_object().define_property_or_throw(
            name,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
            self,
        )?;
        Ok(())
    }

    /// Register a global class of type `T`, where `T` implements `Class`.
    ///
    /// It will return an error if the global property is already defined.
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
            .configurable(T::ATTRIBUTES.configurable());

        self.global_object()
            .define_property_or_throw(T::NAME, property, self)?;

        Ok(())
    }

    /// Gets the string interner.
    #[inline]
    #[must_use]
    pub const fn interner(&self) -> &Interner {
        &self.interner
    }

    /// Gets a mutable reference to the string interner.
    #[inline]
    pub fn interner_mut(&mut self) -> &mut Interner {
        &mut self.interner
    }

    /// Returns the global object.
    #[inline]
    #[must_use]
    pub fn global_object(&self) -> JsObject {
        self.realm.global_object().clone()
    }

    /// Returns the currently active intrinsic constructors and objects.
    #[inline]
    #[must_use]
    pub fn intrinsics(&self) -> &Intrinsics {
        self.realm.intrinsics()
    }

    /// Returns the currently active realm.
    #[inline]
    #[must_use]
    pub const fn realm(&self) -> &Realm {
        &self.realm
    }

    /// Set the value of trace on the context
    #[cfg(feature = "trace")]
    #[inline]
    pub fn set_trace(&mut self, trace: bool) {
        self.vm.trace = trace;
    }

    /// Get optimizer options.
    #[inline]
    #[must_use]
    pub const fn optimizer_options(&self) -> OptimizerOptions {
        self.optimizer_options
    }
    /// Enable or disable optimizations
    #[inline]
    pub fn set_optimizer_options(&mut self, optimizer_options: OptimizerOptions) {
        self.optimizer_options = optimizer_options;
    }

    /// Changes the strictness mode of the context.
    #[inline]
    pub fn strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    /// Enqueues a [`NativeJob`] on the [`JobQueue`].
    #[inline]
    pub fn enqueue_job(&mut self, job: NativeJob) {
        self.job_queue().enqueue_promise_job(job, self);
    }

    /// Runs all the jobs in the job queue.
    #[inline]
    pub fn run_jobs(&mut self) {
        self.job_queue().run_jobs(self);
        self.clear_kept_objects();
    }

    /// Asynchronously runs all the jobs in the job queue.
    ///
    /// # Note
    ///
    /// Concurrent job execution cannot be guaranteed by the engine, since this depends on the
    /// specific handling of each [`JobQueue`]. If you need to ensure that jobs are executed
    /// concurrently, you can provide a custom implementor of `JobQueue` to the context.
    #[allow(clippy::future_not_send)]
    pub async fn run_jobs_async(&mut self) {
        self.job_queue().run_jobs_async(self).await;
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
    #[inline]
    pub fn clear_kept_objects(&mut self) {
        self.kept_alive.clear();
    }

    /// Retrieves the current stack trace of the context.
    #[inline]
    pub fn stack_trace(&self) -> impl Iterator<Item = &CallFrame> {
        self.vm.frames.iter().rev()
    }

    /// Replaces the currently active realm with `realm`, and returns the old realm.
    #[inline]
    pub fn enter_realm(&mut self, realm: Realm) -> Realm {
        self.vm
            .environments
            .replace_global(realm.environment().clone());
        std::mem::replace(&mut self.realm, realm)
    }

    /// Get the [`RootShape`].
    #[inline]
    #[must_use]
    pub const fn root_shape(&self) -> &RootShape {
        &self.root_shape
    }

    /// Gets the host hooks.
    #[inline]
    #[must_use]
    pub fn host_hooks(&self) -> MaybeShared<'host, dyn HostHooks> {
        self.host_hooks.clone()
    }

    /// Gets the job queue.
    #[inline]
    #[must_use]
    pub fn job_queue(&self) -> MaybeShared<'host, dyn JobQueue> {
        self.job_queue.clone()
    }

    /// Gets the module loader.
    #[must_use]
    pub fn module_loader(&self) -> MaybeShared<'host, dyn ModuleLoader> {
        self.module_loader.clone()
    }

    /// Get the [`RuntimeLimits`].
    #[inline]
    #[must_use]
    pub const fn runtime_limits(&self) -> RuntimeLimits {
        self.vm.runtime_limits
    }

    /// Set the [`RuntimeLimits`].
    #[inline]
    pub fn set_runtime_limits(&mut self, runtime_limits: RuntimeLimits) {
        self.vm.runtime_limits = runtime_limits;
    }

    /// Get a mutable reference to the [`RuntimeLimits`].
    #[inline]
    pub fn runtime_limits_mut(&mut self) -> &mut RuntimeLimits {
        &mut self.vm.runtime_limits
    }
}

// ==== Private API ====

impl Context<'_> {
    /// Swaps the currently active realm with `realm`.
    pub(crate) fn swap_realm(&mut self, realm: &mut Realm) {
        std::mem::swap(&mut self.realm, realm);
    }

    /// Increment and get the parser identifier.
    pub(crate) fn next_parser_identifier(&mut self) -> u32 {
        self.parser_identifier += 1;
        self.parser_identifier
    }

    /// `CanDeclareGlobalFunction ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalfunction
    pub(crate) fn can_declare_global_function(&mut self, name: Identifier) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object().clone();

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let name = self.interner().resolve_expect(name.sym()).utf16().into();
        let existing_prop = global_object.__get_own_property__(&name, self)?;

        // 4. If existingProp is undefined, return ? IsExtensible(globalObject).
        let Some(existing_prop) = existing_prop else {
            return global_object.is_extensible(self);
        };

        // 5. If existingProp.[[Configurable]] is true, return true.
        if existing_prop.configurable() == Some(true) {
            return Ok(true);
        }

        // 6. If IsDataDescriptor(existingProp) is true and existingProp has attribute values { [[Writable]]: true, [[Enumerable]]: true }, return true.
        if existing_prop.is_data_descriptor()
            && existing_prop.writable() == Some(true)
            && existing_prop.enumerable() == Some(true)
        {
            return Ok(true);
        }

        // 7. Return false.
        Ok(false)
    }

    /// `CanDeclareGlobalVar ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalvar
    pub(crate) fn can_declare_global_var(&mut self, name: Identifier) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object().clone();

        // 3. Let hasProperty be ? HasOwnProperty(globalObject, N).
        let name = PropertyKey::from(self.interner().resolve_expect(name.sym()).utf16());
        let has_property = global_object.has_own_property(name, self)?;

        // 4. If hasProperty is true, return true.
        if has_property {
            return Ok(true);
        }

        // 5. Return ? IsExtensible(globalObject).
        global_object.is_extensible(self)
    }

    /// `CreateGlobalVarBinding ( N, D )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createglobalvarbinding
    pub(crate) fn create_global_var_binding(
        &mut self,
        name: Identifier,
        configurable: bool,
    ) -> JsResult<()> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object().clone();

        // 3. Let hasProperty be ? HasOwnProperty(globalObject, N).
        let name = PropertyKey::from(self.interner().resolve_expect(name.sym()).utf16());
        let has_property = global_object.has_own_property(name.clone(), self)?;

        // 4. Let extensible be ? IsExtensible(globalObject).
        let extensible = global_object.is_extensible(self)?;

        // 5. If hasProperty is false and extensible is true, then
        if !has_property && extensible {
            // a. Perform ? ObjRec.CreateMutableBinding(N, D).
            // b. Perform ? ObjRec.InitializeBinding(N, undefined).
            global_object.define_property_or_throw(
                name,
                PropertyDescriptor::builder()
                    .value(JsValue::undefined())
                    .writable(true)
                    .enumerable(true)
                    .configurable(configurable)
                    .build(),
                self,
            )?;
        }

        // 6. If envRec.[[VarNames]] does not contain N, then
        //     a. Append N to envRec.[[VarNames]].
        // 7. Return unused.
        Ok(())
    }

    /// `CreateGlobalFunctionBinding ( N, V, D )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createglobalfunctionbinding
    pub(crate) fn create_global_function_binding(
        &mut self,
        name: Identifier,
        function: JsObject,
        configurable: bool,
    ) -> JsResult<()> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object().clone();

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let name = PropertyKey::from(self.interner().resolve_expect(name.sym()).utf16());
        let existing_prop = global_object.__get_own_property__(&name, self)?;

        // 4. If existingProp is undefined or existingProp.[[Configurable]] is true, then
        let desc = if existing_prop.is_none()
            || existing_prop.and_then(|p| p.configurable()) == Some(true)
        {
            // a. Let desc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: D }.
            PropertyDescriptor::builder()
                .value(function.clone())
                .writable(true)
                .enumerable(true)
                .configurable(configurable)
                .build()
        }
        // 5. Else,
        else {
            // a. Let desc be the PropertyDescriptor { [[Value]]: V }.
            PropertyDescriptor::builder()
                .value(function.clone())
                .build()
        };

        // 6. Perform ? DefinePropertyOrThrow(globalObject, N, desc).
        global_object.define_property_or_throw(name.clone(), desc, self)?;

        // 7. Perform ? Set(globalObject, N, V, false).
        global_object.set(name, function, false, self)?;

        // 8. If envRec.[[VarNames]] does not contain N, then
        //     a. Append N to envRec.[[VarNames]].
        // 9. Return unused.
        Ok(())
    }

    /// `HasRestrictedGlobalProperty ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasrestrictedglobalproperty
    pub(crate) fn has_restricted_global_property(&mut self, name: Identifier) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object().clone();

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let name = PropertyKey::from(self.interner().resolve_expect(name.sym()).utf16());
        let existing_prop = global_object.__get_own_property__(&name, self)?;

        // 4. If existingProp is undefined, return false.
        let Some(existing_prop) = existing_prop else {
            return Ok(false);
        };

        // 5. If existingProp.[[Configurable]] is true, return false.
        if existing_prop.configurable() == Some(true) {
            return Ok(false);
        }

        // 6. Return true.
        Ok(true)
    }

    /// Returns `true` if this context is in strict mode.
    pub(crate) const fn is_strict(&self) -> bool {
        self.strict
    }

    /// `9.4.1 GetActiveScriptOrModule ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getactivescriptormodule
    pub(crate) fn get_active_script_or_module(&self) -> Option<ActiveRunnable> {
        // 1. If the execution context stack is empty, return null.
        // 2. Let ec be the topmost execution context on the execution context stack whose ScriptOrModule component is not null.
        // 3. If no such execution context exists, return null. Otherwise, return ec's ScriptOrModule.
        self.vm
            .frames
            .iter()
            .rev()
            .find_map(|frame| frame.active_runnable.clone())
    }

    /// Get `active function object`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#active-function-object
    pub(crate) fn active_function_object(&self) -> Option<JsObject> {
        if self.vm.native_active_function.is_some() {
            return self.vm.native_active_function.clone();
        }

        if let Some(frame) = self.vm.frames.last() {
            return frame.active_function.clone();
        }

        None
    }
}

impl<'host> Context<'host> {
    /// Creates a `ContextCleanupGuard` that executes some cleanup after being dropped.
    pub(crate) fn guard<F>(&mut self, cleanup: F) -> ContextCleanupGuard<'_, 'host, F>
    where
        F: FnOnce(&mut Context<'_>) + 'static,
    {
        ContextCleanupGuard::new(self, cleanup)
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
pub struct ContextBuilder<'icu, 'hooks, 'queue, 'module> {
    interner: Option<Interner>,
    host_hooks: Option<MaybeShared<'hooks, dyn HostHooks>>,
    job_queue: Option<MaybeShared<'queue, dyn JobQueue>>,
    module_loader: Option<MaybeShared<'module, dyn ModuleLoader>>,
    #[cfg(feature = "intl")]
    icu: Option<icu::Icu<'icu>>,
    #[cfg(not(feature = "intl"))]
    icu: PhantomData<&'icu ()>,
    #[cfg(feature = "fuzz")]
    instructions_remaining: usize,
}

impl std::fmt::Debug for ContextBuilder<'_, '_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Clone, Copy, Debug)]
        struct JobQueue;
        #[derive(Clone, Copy, Debug)]
        struct HostHooks;
        #[derive(Clone, Copy, Debug)]
        struct ModuleLoader;

        let mut out = f.debug_struct("ContextBuilder");

        out.field("interner", &self.interner)
            .field("host_hooks", &self.host_hooks.as_ref().map(|_| HostHooks))
            .field("job_queue", &self.job_queue.as_ref().map(|_| JobQueue))
            .field(
                "module_loader",
                &self.module_loader.as_ref().map(|_| ModuleLoader),
            );

        #[cfg(feature = "intl")]
        out.field("icu", &self.icu);

        #[cfg(feature = "fuzz")]
        out.field("instructions_remaining", &self.instructions_remaining);

        out.finish()
    }
}

impl<'icu, 'hooks, 'queue, 'module> ContextBuilder<'icu, 'hooks, 'queue, 'module> {
    /// Creates a new [`ContextBuilder`] with a default empty [`Interner`]
    /// and a default `BoaProvider` if the `intl` feature is enabled.
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
    ) -> Result<ContextBuilder<'_, 'hooks, 'queue, 'module>, IcuError> {
        Ok(ContextBuilder {
            icu: Some(icu::Icu::new(provider)?),
            ..self
        })
    }

    /// Initializes the [`HostHooks`] for the context.
    ///
    /// [`Host Hooks`]: https://tc39.es/ecma262/#sec-host-hooks-summary
    #[must_use]
    pub fn host_hooks<'new_hooks, H>(
        self,
        host_hooks: H,
    ) -> ContextBuilder<'icu, 'new_hooks, 'queue, 'module>
    where
        H: Into<MaybeShared<'new_hooks, dyn HostHooks>>,
    {
        ContextBuilder {
            host_hooks: Some(host_hooks.into()),
            ..self
        }
    }

    /// Initializes the [`JobQueue`] for the context.
    #[must_use]
    pub fn job_queue<'new_queue, Q>(
        self,
        job_queue: Q,
    ) -> ContextBuilder<'icu, 'hooks, 'new_queue, 'module>
    where
        Q: Into<MaybeShared<'new_queue, dyn JobQueue>>,
    {
        ContextBuilder {
            job_queue: Some(job_queue.into()),
            ..self
        }
    }

    /// Initializes the [`ModuleLoader`] for the context.
    #[must_use]
    pub fn module_loader<'new_module, M>(
        self,
        module_loader: M,
    ) -> ContextBuilder<'icu, 'hooks, 'queue, 'new_module>
    where
        M: Into<MaybeShared<'new_module, dyn ModuleLoader>>,
    {
        ContextBuilder {
            module_loader: Some(module_loader.into()),
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
    pub fn build<'host>(self) -> JsResult<Context<'host>>
    where
        'icu: 'host,
        'hooks: 'host,
        'queue: 'host,
        'module: 'host,
    {
        let root_shape = RootShape::default();

        let host_hooks = self.host_hooks.unwrap_or_else(|| {
            let hooks: &dyn HostHooks = &DefaultHooks;
            hooks.into()
        });
        let realm = Realm::create(&*host_hooks, &root_shape);
        let vm = Vm::new(realm.environment().clone());

        let module_loader = if let Some(loader) = self.module_loader {
            loader
        } else {
            SimpleModuleLoader::new(Path::new(".")).map_or_else(
                |_| {
                    let loader: &dyn ModuleLoader = &IdleModuleLoader;
                    loader.into()
                },
                |loader| {
                    let loader: Rc<dyn ModuleLoader> = Rc::new(loader);
                    loader.into()
                },
            )
        };

        let mut context = Context {
            realm,
            interner: self.interner.unwrap_or_default(),
            vm,
            strict: false,
            #[cfg(feature = "intl")]
            icu: self.icu.unwrap_or_else(|| {
                let buffer: &dyn icu_provider::BufferProvider = boa_icu_provider::buffer();
                let provider = BoaProvider::Buffer(buffer);
                icu::Icu::new(provider).expect("Failed to initialize default icu data.")
            }),
            #[cfg(feature = "fuzz")]
            instructions_remaining: self.instructions_remaining,
            kept_alive: Vec::new(),
            host_hooks,
            job_queue: self.job_queue.unwrap_or_else(|| {
                let queue: Rc<dyn JobQueue> = Rc::new(SimpleJobQueue::new());
                queue.into()
            }),
            module_loader,
            optimizer_options: OptimizerOptions::OPTIMIZE_ALL,
            root_shape,
            parser_identifier: 0,
        };

        builtins::set_default_global_bindings(&mut context)?;

        Ok(context)
    }
}

/// A cleanup guard for a [`Context`] that is executed when dropped.
#[derive(Debug)]
pub(crate) struct ContextCleanupGuard<'a, 'host, F>
where
    F: FnOnce(&mut Context<'_>) + 'static,
{
    context: &'a mut Context<'host>,
    cleanup: Option<F>,
}

impl<'a, 'host, F> ContextCleanupGuard<'a, 'host, F>
where
    F: FnOnce(&mut Context<'_>) + 'static,
{
    /// Creates a new `ContextCleanupGuard` from the current context and its cleanup operation.
    pub(crate) fn new(context: &'a mut Context<'host>, cleanup: F) -> Self {
        Self {
            context,
            cleanup: Some(cleanup),
        }
    }
}

impl<'host, F> std::ops::Deref for ContextCleanupGuard<'_, 'host, F>
where
    F: FnOnce(&mut Context<'_>) + 'static,
{
    type Target = Context<'host>;

    fn deref(&self) -> &Self::Target {
        self.context
    }
}

impl<F> std::ops::DerefMut for ContextCleanupGuard<'_, '_, F>
where
    F: FnOnce(&mut Context<'_>) + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context
    }
}

impl<F> Drop for ContextCleanupGuard<'_, '_, F>
where
    F: FnOnce(&mut Context<'_>) + 'static,
{
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup(self.context);
        }
    }
}
