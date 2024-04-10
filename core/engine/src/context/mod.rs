//! The ECMAScript context.

use std::{cell::Cell, path::Path, rc::Rc};

use boa_ast::StatementList;
use boa_interner::Interner;
use boa_parser::source::ReadChar;
use boa_profiler::Profiler;
pub use hooks::{DefaultHooks, HostHooks};
#[cfg(feature = "intl")]
pub use icu::IcuError;
use intrinsics::Intrinsics;

use crate::vm::RuntimeLimits;
use crate::{
    builtins,
    class::{Class, ClassBuilder},
    job::{JobQueue, NativeJob, SimpleJobQueue},
    js_string,
    module::{IdleModuleLoader, ModuleLoader, SimpleModuleLoader},
    native_function::NativeFunction,
    object::{shape::RootShape, FunctionObjectBuilder, JsObject},
    optimizer::{Optimizer, OptimizerOptions, OptimizerStatistics},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    script::Script,
    vm::{ActiveRunnable, CallFrame, Vm},
    HostDefined, JsNativeError, JsResult, JsString, JsValue, NativeObject, Source,
};

use self::intrinsics::StandardConstructor;

mod hooks;
#[cfg(feature = "intl")]
pub(crate) mod icu;
pub mod intrinsics;

thread_local! {
    static CANNOT_BLOCK_COUNTER: Cell<u64> = const { Cell::new(0) };
}

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
///     js_string,
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
///     .property(js_string!("x"), 12, Attribute::READONLY)
///     .build();
/// context
///     .register_global_property(js_string!("arg"), arg, Attribute::all())
///     .expect("property shouldn't exist");
///
/// let value = context.eval(Source::from_bytes("test(arg)")).unwrap();
///
/// assert_eq!(value.as_number(), Some(12.0))
/// ```
pub struct Context {
    /// String interner in the context.
    interner: Interner,

    /// Execute in strict mode,
    strict: bool,

    /// Number of instructions remaining before a forced exit
    #[cfg(feature = "fuzz")]
    pub(crate) instructions_remaining: usize,

    pub(crate) vm: Vm,

    pub(crate) kept_alive: Vec<JsObject>,

    can_block: bool,

    /// Intl data provider.
    #[cfg(feature = "intl")]
    intl_provider: icu::IntlProvider,

    host_hooks: &'static dyn HostHooks,

    job_queue: Rc<dyn JobQueue>,

    module_loader: Rc<dyn ModuleLoader>,

    optimizer_options: OptimizerOptions,
    root_shape: RootShape,

    /// Unique identifier for each parser instance used during the context lifetime.
    parser_identifier: u32,

    data: HostDefined,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Context");

        debug
            .field("realm", &self.vm.realm)
            .field("interner", &self.interner)
            .field("vm", &self.vm)
            .field("strict", &self.strict)
            .field("promise_job_queue", &"JobQueue")
            .field("hooks", &"HostHooks")
            .field("module_loader", &"ModuleLoader")
            .field("optimizer_options", &self.optimizer_options);

        #[cfg(feature = "intl")]
        debug.field("intl_provider", &self.intl_provider);

        debug.finish_non_exhaustive()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.can_block {
            CANNOT_BLOCK_COUNTER.set(CANNOT_BLOCK_COUNTER.get() - 1);
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        ContextBuilder::default()
            .build()
            .expect("Building the default context should not fail")
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
    pub fn eval<R: ReadChar>(&mut self, src: Source<'_, R>) -> JsResult<JsValue> {
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
    ///     js_string,
    ///     object::ObjectInitializer,
    ///     property::{Attribute, PropertyDescriptor},
    ///     Context,
    /// };
    ///
    /// let mut context = Context::default();
    ///
    /// context
    ///     .register_global_property(
    ///         js_string!("myPrimitiveProperty"),
    ///         10,
    ///         Attribute::all(),
    ///     )
    ///     .expect("property shouldn't exist");
    ///
    /// let object = ObjectInitializer::new(&mut context)
    ///     .property(js_string!("x"), 0, Attribute::all())
    ///     .property(js_string!("y"), 1, Attribute::all())
    ///     .build();
    /// context
    ///     .register_global_property(
    ///         js_string!("myObjectProperty"),
    ///         object,
    ///         Attribute::all(),
    ///     )
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
        name: JsString,
        length: usize,
        body: NativeFunction,
    ) -> JsResult<()> {
        let function = FunctionObjectBuilder::new(self.realm(), body)
            .name(name.clone())
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
        name: JsString,
        length: usize,
        body: NativeFunction,
    ) -> JsResult<()> {
        let function = FunctionObjectBuilder::new(self.realm(), body)
            .name(name.clone())
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

    /// Registers a global class `C` in the currently active realm.
    ///
    /// Errors if the class has already been registered.
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
    /// context.register_global_class::<MyClass>()?;
    /// ```
    pub fn register_global_class<C: Class>(&mut self) -> JsResult<()> {
        if self.realm().has_class::<C>() {
            return Err(JsNativeError::typ()
                .with_message("cannot register a class twice")
                .into());
        }

        let mut class_builder = ClassBuilder::new::<C>(self);
        C::init(&mut class_builder)?;

        let class = class_builder.build();
        let property = PropertyDescriptor::builder()
            .value(class.constructor())
            .writable(C::ATTRIBUTES.writable())
            .enumerable(C::ATTRIBUTES.enumerable())
            .configurable(C::ATTRIBUTES.configurable());

        self.global_object()
            .define_property_or_throw(js_string!(C::NAME), property, self)?;
        self.realm().register_class::<C>(class);

        Ok(())
    }

    /// Removes the global class `C` from the currently active realm, returning the constructor
    /// and prototype of the class if `C` was registered.
    ///
    /// # Note
    ///
    /// This makes the constructor return an error on further calls, but note that this won't protect
    /// static properties from being accessed within variables that stored the constructor before being
    /// unregistered.  If you need that functionality, you can use a static accessor that first checks
    /// if the class is registered ([`Context::has_global_class`]) before returning the static value.
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
    /// context.register_global_class::<MyClass>()?;
    /// // ... code
    /// context.unregister_global_class::<MyClass>()?;
    /// ```
    pub fn unregister_global_class<C: Class>(&mut self) -> JsResult<Option<StandardConstructor>> {
        self.global_object()
            .delete_property_or_throw(js_string!(C::NAME), self)?;
        Ok(self.realm().unregister_class::<C>())
    }

    /// Checks if the currently active realm has the global class `C` registered.
    #[must_use]
    pub fn has_global_class<C: Class>(&self) -> bool {
        self.realm().has_class::<C>()
    }

    /// Gets the constructor and prototype of the global class `C` if the currently active realm has
    /// that class registered.
    #[must_use]
    pub fn get_global_class<C: Class>(&self) -> Option<StandardConstructor> {
        self.realm().get_class::<C>()
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
        self.vm.realm.global_object().clone()
    }

    /// Returns the currently active intrinsic constructors and objects.
    #[inline]
    #[must_use]
    pub fn intrinsics(&self) -> &Intrinsics {
        self.vm.realm.intrinsics()
    }

    /// Returns the currently active realm.
    #[inline]
    #[must_use]
    pub const fn realm(&self) -> &Realm {
        &self.vm.realm
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
    /// specific handling of each [`JobQueue`]. If you want to execute jobs concurrently, you must
    /// provide a custom implementor of `JobQueue` to the context.
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
        std::mem::replace(&mut self.vm.realm, realm)
    }

    /// Create a new Realm with the default global bindings.
    pub fn create_realm(&mut self) -> JsResult<Realm> {
        let realm = Realm::create(self.host_hooks, &self.root_shape)?;

        let old_realm = self.enter_realm(realm);

        builtins::set_default_global_bindings(self)?;

        Ok(self.enter_realm(old_realm))
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
    pub fn host_hooks(&self) -> &'static dyn HostHooks {
        self.host_hooks
    }

    /// Gets the job queue.
    #[inline]
    #[must_use]
    pub fn job_queue(&self) -> Rc<dyn JobQueue> {
        self.job_queue.clone()
    }

    /// Gets the module loader.
    #[must_use]
    pub fn module_loader(&self) -> Rc<dyn ModuleLoader> {
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

    /// Returns `true` if this context can be suspended by an `Atomics.wait` call.
    #[inline]
    #[must_use]
    pub fn can_block(&self) -> bool {
        self.can_block
    }

    /// Insert a type into the context-specific [`HostDefined`] field.
    #[inline]
    pub fn insert_data<T: NativeObject>(&mut self, value: T) -> Option<Box<T>> {
        self.data.insert(value)
    }

    /// Check if the context-specific [`HostDefined`] has type T.
    #[inline]
    #[must_use]
    pub fn has_data<T: NativeObject>(&self) -> bool {
        self.data.has::<T>()
    }

    /// Remove type T from the context-specific [`HostDefined`], if it exists.
    #[inline]
    pub fn remove_data<T: NativeObject>(&mut self) -> Option<Box<T>> {
        self.data.remove::<T>()
    }

    /// Get type T from the context-specific [`HostDefined`], if it exists.
    #[inline]
    #[must_use]
    pub fn get_data<T: NativeObject>(&self) -> Option<&T> {
        self.data.get::<T>()
    }
}

// ==== Private API ====

impl Context {
    /// Swaps the currently active realm with `realm`.
    pub(crate) fn swap_realm(&mut self, realm: &mut Realm) {
        std::mem::swap(&mut self.vm.realm, realm);
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
    pub(crate) fn can_declare_global_function(&mut self, name: &JsString) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object().clone();

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let name = name.clone().into();
        let existing_prop = global_object.__get_own_property__(&name, &mut self.into())?;

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
    pub(crate) fn can_declare_global_var(&mut self, name: &JsString) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object().clone();

        // 3. Let hasProperty be ? HasOwnProperty(globalObject, N).
        let has_property = global_object.has_own_property(name.clone(), self)?;

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
        name: JsString,
        configurable: bool,
    ) -> JsResult<()> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object().clone();

        // 3. Let hasProperty be ? HasOwnProperty(globalObject, N).
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
        name: JsString,
        function: JsObject,
        configurable: bool,
    ) -> JsResult<()> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object().clone();

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let existing_prop =
            global_object.__get_own_property__(&name.clone().into(), &mut self.into())?;

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
    pub(crate) fn has_restricted_global_property(&mut self, name: &JsString) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object().clone();

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let name = name.clone().into();
        let existing_prop = global_object.__get_own_property__(&name, &mut self.into())?;

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
            return frame.function(&self.vm);
        }

        None
    }
}

impl Context {
    /// Creates a `ContextCleanupGuard` that executes some cleanup after being dropped.
    pub(crate) fn guard<F>(&mut self, cleanup: F) -> ContextCleanupGuard<'_, F>
    where
        F: FnOnce(&mut Context) + 'static,
    {
        ContextCleanupGuard::new(self, cleanup)
    }

    /// Get the Intl data provider.
    #[cfg(feature = "intl")]
    pub(crate) const fn intl_provider(&self) -> &icu::IntlProvider {
        &self.intl_provider
    }
}

/// Builder for the [`Context`] type.
///
/// This builder allows custom initialization of the [`Interner`] within
/// the context.
#[derive(Default)]
pub struct ContextBuilder {
    interner: Option<Interner>,
    host_hooks: Option<&'static dyn HostHooks>,
    job_queue: Option<Rc<dyn JobQueue>>,
    module_loader: Option<Rc<dyn ModuleLoader>>,
    can_block: bool,
    #[cfg(feature = "intl")]
    icu: Option<icu::IntlProvider>,
    #[cfg(feature = "fuzz")]
    instructions_remaining: usize,
}

impl std::fmt::Debug for ContextBuilder {
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
            )
            .field("can_block", &self.can_block);

        #[cfg(feature = "intl")]
        out.field("icu", &self.icu);

        #[cfg(feature = "fuzz")]
        out.field("instructions_remaining", &self.instructions_remaining);

        out.finish()
    }
}

impl ContextBuilder {
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

    /// Provides a [`BufferProvider`] data provider to the [`Context`].
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
    /// [`BufferProvider`]: icu_provider::BufferProvider
    #[cfg(feature = "intl")]
    pub fn icu_buffer_provider<T: icu_provider::BufferProvider + 'static>(
        mut self,
        provider: T,
    ) -> Result<Self, IcuError> {
        self.icu = Some(icu::IntlProvider::try_new_with_buffer_provider(provider)?);
        Ok(self)
    }

    /// Provides an [`AnyProvider`] data provider to the [`Context`].
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
    /// [`AnyProvider`]: icu_provider::AnyProvider
    #[cfg(feature = "intl")]
    pub fn icu_any_provider<T: icu_provider::AnyProvider + 'static>(
        mut self,
        provider: T,
    ) -> Result<Self, IcuError> {
        self.icu = Some(icu::IntlProvider::try_new_with_any_provider(provider)?);
        Ok(self)
    }

    /// Initializes the [`HostHooks`] for the context.
    ///
    /// [`Host Hooks`]: https://tc39.es/ecma262/#sec-host-hooks-summary
    #[must_use]
    pub fn host_hooks<H: HostHooks + 'static>(mut self, host_hooks: &'static H) -> Self {
        self.host_hooks = Some(host_hooks);
        self
    }

    /// Initializes the [`JobQueue`] for the context.
    #[must_use]
    pub fn job_queue<Q: JobQueue + 'static>(mut self, job_queue: Rc<Q>) -> Self {
        self.job_queue = Some(job_queue);
        self
    }

    /// Initializes the [`ModuleLoader`] for the context.
    #[must_use]
    pub fn module_loader<M: ModuleLoader + 'static>(mut self, module_loader: Rc<M>) -> Self {
        self.module_loader = Some(module_loader);
        self
    }

    /// [`AgentCanSuspend ( )`][spec] aka `[[CanBlock]]`
    ///
    /// Defines if this context can be suspended by calls to the [`Atomics.wait`][wait] function.
    ///
    /// # Note
    ///
    /// By the specification, multiple agents cannot share the same thread if any of them has its
    /// `[[CanBlock]]` field set to true. The builder will verify at build time that all contexts on
    /// the current thread fulfill this requisite.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-agentcansuspend
    /// [wait]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/wait
    #[must_use]
    pub const fn can_block(mut self, can_block: bool) -> Self {
        self.can_block = can_block;
        self
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
    // TODO: try to use a custom error here, since most of the `JsError` APIs
    // require having a `Context` in the first place.
    pub fn build(self) -> JsResult<Context> {
        if self.can_block {
            if CANNOT_BLOCK_COUNTER.get() > 0 {
                return Err(JsNativeError::typ()
                    .with_message(
                        "a context that can block must be the only active context in its current thread",
                    )
                    .into());
            }
        } else {
            CANNOT_BLOCK_COUNTER.set(CANNOT_BLOCK_COUNTER.get() + 1);
        }

        let root_shape = RootShape::default();

        let host_hooks = self.host_hooks.unwrap_or(&DefaultHooks);
        let realm = Realm::create(host_hooks, &root_shape)?;
        let vm = Vm::new(realm);

        let module_loader: Rc<dyn ModuleLoader> = if let Some(loader) = self.module_loader {
            loader
        } else if let Ok(loader) = SimpleModuleLoader::new(Path::new(".")) {
            Rc::new(loader)
        } else {
            Rc::new(IdleModuleLoader)
        };

        let job_queue = self
            .job_queue
            .unwrap_or_else(|| Rc::new(SimpleJobQueue::new()));

        let mut context = Context {
            interner: self.interner.unwrap_or_default(),
            vm,
            strict: false,
            #[cfg(feature = "intl")]
            intl_provider: if let Some(icu) = self.icu {
                icu
            } else {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "intl_bundled")] {
                        icu::IntlProvider::try_new_with_buffer_provider(boa_icu_provider::buffer())
                            .expect("Failed to initialize default icu data.")
                    } else {
                        return Err(JsNativeError::typ()
                            .with_message("missing Intl provider for context")
                            .into()
                        );
                    }
                }
            },
            #[cfg(feature = "fuzz")]
            instructions_remaining: self.instructions_remaining,
            kept_alive: Vec::new(),
            host_hooks,
            job_queue,
            module_loader,
            optimizer_options: OptimizerOptions::OPTIMIZE_ALL,
            root_shape,
            parser_identifier: 0,
            can_block: self.can_block,
            data: HostDefined::default(),
        };

        builtins::set_default_global_bindings(&mut context)?;

        Ok(context)
    }
}

/// A cleanup guard for a [`Context`] that is executed when dropped.
#[derive(Debug)]
pub(crate) struct ContextCleanupGuard<'a, F>
where
    F: FnOnce(&mut Context) + 'static,
{
    context: &'a mut Context,
    cleanup: Option<F>,
}

impl<'a, F> ContextCleanupGuard<'a, F>
where
    F: FnOnce(&mut Context) + 'static,
{
    /// Creates a new `ContextCleanupGuard` from the current context and its cleanup operation.
    pub(crate) fn new(context: &'a mut Context, cleanup: F) -> Self {
        Self {
            context,
            cleanup: Some(cleanup),
        }
    }
}

impl<F> std::ops::Deref for ContextCleanupGuard<'_, F>
where
    F: FnOnce(&mut Context) + 'static,
{
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.context
    }
}

impl<F> std::ops::DerefMut for ContextCleanupGuard<'_, F>
where
    F: FnOnce(&mut Context) + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context
    }
}

impl<F> Drop for ContextCleanupGuard<'_, F>
where
    F: FnOnce(&mut Context) + 'static,
{
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup(self.context);
        }
    }
}
