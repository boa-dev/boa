//! The ECMAScript context.

mod hooks;
#[cfg(feature = "intl")]
pub(crate) mod icu;
pub mod intrinsics;
mod maybe_shared;

pub use hooks::{DefaultHooks, HostHooks};
#[cfg(feature = "intl")]
pub use icu::{IcuError, IcuProvider};
use intrinsics::Intrinsics;
pub use maybe_shared::MaybeShared;

#[cfg(not(feature = "intl"))]
pub use std::marker::PhantomData;
use std::{collections::VecDeque, io::Read, path::Path};

use crate::{
    builtins::{self},
    class::{Class, ClassBuilder},
    job::{JobQueue, NativeJob},
    js_string,
    module::{ModuleLoader, SimpleModuleLoader},
    native_function::NativeFunction,
    object::{shape::RootShape, FunctionObjectBuilder, JsObject},
    optimizer::{Optimizer, OptimizerOptions, OptimizerStatistics},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    script::Script,
    vm::{CallFrame, Vm},
    JsError, JsNativeError, JsResult, JsValue, Module, Source,
};
use boa_ast::{expression::Identifier, StatementList};
use boa_interner::Interner;
use boa_profiler::Profiler;

use crate::vm::RuntimeLimits;

///
pub trait Context<'icu>: JobQueue + ModuleLoader {
    ///
    fn as_raw_context(&self) -> &RawContext<'icu>;
    ///
    fn as_raw_context_mut(&mut self) -> &mut RawContext<'icu>;
}

impl std::fmt::Debug for dyn Context<'_> + '_ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Context")
            .field(self.as_raw_context())
            .finish()
    }
}

// ==== Public API ====

impl dyn Context<'_> + '_ {
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
    #[allow(clippy::unit_arg, clippy::drop_copy)]
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
        let function = FunctionObjectBuilder::new(self, body)
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
        let function = FunctionObjectBuilder::new(self, body)
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
    pub fn interner(&self) -> &Interner {
        self.as_raw_context().interner()
    }

    /// Gets a mutable reference to the string interner.
    #[inline]
    pub fn interner_mut(&mut self) -> &mut Interner {
        self.as_raw_context_mut().interner_mut()
    }

    /// Returns the global object.
    #[inline]
    pub fn global_object(&self) -> JsObject {
        self.as_raw_context().global_object()
    }

    /// Returns the currently active intrinsic constructors and objects.
    #[inline]
    pub fn intrinsics(&self) -> &Intrinsics {
        self.realm().intrinsics()
    }

    /// Returns the currently active realm.
    #[inline]
    pub fn realm(&self) -> &Realm {
        self.as_raw_context().realm()
    }

    /// Set the value of trace on the context
    #[cfg(feature = "trace")]
    #[inline]
    pub fn set_trace(&mut self, trace: bool) {
        self.as_raw_context_mut().set_trace(trace)
    }

    /// Get optimizer options.
    #[inline]
    pub fn optimizer_options(&self) -> OptimizerOptions {
        self.as_raw_context().optimizer_options()
    }
    /// Enable or disable optimizations
    #[inline]
    pub fn set_optimizer_options(&mut self, optimizer_options: OptimizerOptions) {
        self.as_raw_context_mut()
            .set_optimizer_options(optimizer_options);
    }

    /// Changes the strictness mode of the context.
    #[inline]
    pub fn strict(&mut self, strict: bool) {
        self.as_raw_context_mut().strict(strict);
    }

    /// Enqueues a [`NativeJob`] on the [`JobQueue`].
    #[inline]
    pub fn enqueue_job(&mut self, job: NativeJob) {
        self.enqueue_promise_job(job);
    }

    /// Runs all the jobs in the job queue and clears the kept objects.
    #[inline]
    pub fn run_jobs_and_cleanup(&mut self) {
        self.run_jobs();
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
        self.as_raw_context_mut().clear_kept_objects();
    }

    /// Replaces the currently active realm with `realm`, and returns the old realm.
    #[inline]
    pub fn enter_realm(&mut self, realm: Realm) -> Realm {
        self.as_raw_context_mut().enter_realm(realm)
    }

    /// Get the [`RootShape`].
    #[inline]
    pub fn root_shape(&self) -> &RootShape {
        self.as_raw_context().root_shape()
    }

    /// Gets the host hooks.
    #[inline]
    pub fn host_hooks(&self) -> &'static dyn HostHooks {
        self.as_raw_context().host_hooks()
    }

    /// Get the [`RuntimeLimits`].
    #[inline]
    pub fn runtime_limits(&self) -> RuntimeLimits {
        self.as_raw_context().runtime_limits()
    }

    /// Set the [`RuntimeLimits`].
    #[inline]
    pub fn set_runtime_limits(&mut self, runtime_limits: RuntimeLimits) {
        self.as_raw_context_mut().set_runtime_limits(runtime_limits);
    }

    /// Get a mutable reference to the [`RuntimeLimits`].
    #[inline]
    pub fn runtime_limits_mut(&mut self) -> &mut RuntimeLimits {
        self.as_raw_context_mut().runtime_limits_mut()
    }
}

// ==== Private API ====

impl<'host> dyn Context<'host> + '_ {
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

    /// Creates a `ContextCleanupGuard` that executes some cleanup after being dropped.
    pub(crate) fn guard<F>(&mut self, cleanup: F) -> ContextCleanupGuard<'_, 'host, F>
    where
        F: FnOnce(&mut dyn Context<'_>) + 'static,
    {
        ContextCleanupGuard::new(self, cleanup)
    }

    /// Gets the icu provider.
    #[cfg(feature = "intl")]
    pub(crate) fn icu_provider(&self) -> &IcuProvider<'host> {
        &self.as_raw_context().icu_provider()
    }
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
///     object::ObjectInitializer,
///     property::{Attribute, PropertyDescriptor},
///     Context,
///     Source
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
pub struct RawContext<'icu> {
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

    optimizer_options: OptimizerOptions,

    root_shape: RootShape,

    /// ICU related utilities
    #[cfg(feature = "intl")]
    icu_provider: IcuProvider<'icu>,

    pub(crate) host_hooks: &'static dyn HostHooks,

    /// Unique identifier for each parser instance used during the context lifetime.
    parser_identifier: u32,
}

impl std::fmt::Debug for RawContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("RawContext");

        debug
            .field("realm", &self.realm)
            .field("interner", &self.interner)
            .field("vm", &self.vm)
            .field("strict", &self.strict)
            .field("promise_job_queue", &"JobQueue")
            .field("hooks", &"HostHooks")
            .field("module_loader", &"ModuleLoader")
            .field("optimizer_options", &self.optimizer_options);

        debug.finish()
    }
}

impl Default for RawContext<'_> {
    fn default() -> Self {
        RawContextBuilder::default()
            .build()
            .expect("Building the default context should not fail")
    }
}

impl<'icu> RawContext<'icu> {
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

    /// Returns the global object.
    #[inline]
    pub fn global_object(&self) -> JsObject {
        self.realm.global_object().clone()
    }

    /// Returns the currently active intrinsic constructors and objects.
    #[inline]
    pub fn intrinsics(&self) -> &Intrinsics {
        self.realm.intrinsics()
    }

    /// Returns the currently active realm.
    #[inline]
    pub fn realm(&self) -> &Realm {
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
    pub fn optimizer_options(&self) -> OptimizerOptions {
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
            .replace_global(self.realm.environment().clone());
        std::mem::replace(&mut self.realm, realm)
    }

    /// Get the [`RootShape`].
    #[inline]
    pub fn root_shape(&self) -> &RootShape {
        &self.root_shape
    }

    /// Gets the host hooks.
    #[inline]
    pub fn host_hooks(&self) -> &'static dyn HostHooks {
        self.host_hooks
    }

    /// Get the [`RuntimeLimits`].
    #[inline]
    pub fn runtime_limits(&self) -> RuntimeLimits {
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

    /// Swaps the currently active realm with `realm`.
    pub(crate) fn swap_realm(&mut self, realm: &mut Realm) {
        let global_env = realm.environment().clone();
        std::mem::swap(&mut self.realm, realm);
        self.vm.environments.replace_global(global_env);
    }

    /// Increment and get the parser identifier.
    pub(crate) fn next_parser_identifier(&mut self) -> u32 {
        self.parser_identifier += 1;
        self.parser_identifier
    }

    /// Returns `true` if this context is in strict mode.
    pub(crate) fn is_strict(&self) -> bool {
        self.strict
    }

    /// Gets the icu provider.
    #[cfg(feature = "intl")]
    pub(crate) fn icu_provider(&self) -> &IcuProvider<'icu> {
        &self.icu_provider
    }
}

impl JobQueue for RawContext<'_> {
    fn enqueue_promise_job(&mut self, _job: NativeJob) {}

    fn run_jobs(&mut self) {}

    fn enqueue_future_job(&mut self, _future: crate::job::FutureJob) {}
}

impl ModuleLoader for RawContext<'_> {
    fn load_imported_module(
        &mut self,
        _referrer: crate::module::Referrer,
        _specifier: crate::JsString,
        _finish_load: Box<dyn FnOnce(JsResult<Module>, &mut dyn Context<'_>)>,
    ) {
    }
}

impl<'icu> Context<'icu> for RawContext<'icu> {
    #[inline]
    fn as_raw_context(&self) -> &RawContext<'icu> {
        self
    }

    #[inline]
    fn as_raw_context_mut(&mut self) -> &mut RawContext<'icu> {
        self
    }
}

/// Builder for the [`Context`] type.
///
/// This builder allows custom initialization of the [`Interner`] within
/// the context.
/// Additionally, if the `intl` feature is enabled, [`RawContextBuilder`] becomes
/// the only way to create a new [`Context`], since now it requires a
/// valid data provider for the `Intl` functionality.
#[cfg_attr(
    feature = "intl",
    doc = "The required data in a valid provider is specified in [`IcuProvider`]"
)]
pub struct RawContextBuilder<'icu> {
    interner: Option<Interner>,
    host_hooks: &'static dyn HostHooks,
    #[cfg(feature = "intl")]
    icu: Option<IcuProvider<'icu>>,
    #[cfg(not(feature = "intl"))]
    icu: PhantomData<&'icu ()>,
    #[cfg(feature = "fuzz")]
    instructions_remaining: usize,
}

impl Default for RawContextBuilder<'_> {
    fn default() -> Self {
        Self {
            interner: Default::default(),
            host_hooks: &DefaultHooks,
            icu: Default::default(),
            #[cfg(feature = "fuzz")]
            instructions_remaining: Default::default(),
        }
    }
}

impl std::fmt::Debug for RawContextBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Clone, Copy, Debug)]
        struct HostHooks;

        let mut out = f.debug_struct("ContextBuilder");

        out.field("host_hooks", &HostHooks)
            .field("interner", &self.interner);

        #[cfg(feature = "intl")]
        out.field("icu", &self.icu);

        #[cfg(feature = "fuzz")]
        out.field("instructions_remaining", &self.instructions_remaining);

        out.finish()
    }
}

impl<'icu> RawContextBuilder<'icu> {
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
        provider: IcuProvider<'_>,
    ) -> Result<RawContextBuilder<'_>, IcuError> {
        Ok(RawContextBuilder {
            icu: Some(provider),
            ..self
        })
    }

    /// Initializes the [`HostHooks`] for the context.
    ///
    /// [`Host Hooks`]: https://tc39.es/ecma262/#sec-host-hooks-summary
    #[must_use]
    pub fn host_hooks(self, host_hooks: &'static dyn HostHooks) -> RawContextBuilder<'icu> {
        RawContextBuilder { host_hooks, ..self }
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
    pub fn build(self) -> JsResult<RawContext<'icu>> {
        let root_shape = RootShape::default();

        let realm = Realm::create(self.host_hooks, &root_shape);
        let vm = Vm::new(realm.environment().clone());

        // let module_loader = if let Some(loader) = self.module_loader {
        //     loader
        // } else {
        //     SimpleModuleLoader::new(Path::new(".")).map_or_else(
        //         |_| {
        //             let loader: &dyn ModuleLoader = &IdleModuleLoader;
        //             loader.into()
        //         },
        //         |loader| {
        //             let loader: Rc<dyn ModuleLoader> = Rc::new(loader);
        //             loader.into()
        //         },
        //     )
        // };

        let mut context = RawContext {
            realm,
            interner: self.interner.unwrap_or_default(),
            vm,
            strict: false,
            #[cfg(feature = "intl")]
            icu_provider: self.icu.unwrap_or_else(|| {
                IcuProvider::from_buffer_provider(boa_icu_provider::buffer())
                    .expect("Failed to initialize default icu data.")
            }),
            #[cfg(feature = "fuzz")]
            instructions_remaining: self.instructions_remaining,
            kept_alive: Vec::new(),
            host_hooks: self.host_hooks,
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
pub(crate) struct ContextCleanupGuard<'a, 'icu, F>
where
    F: FnOnce(&mut dyn Context<'_>) + 'static,
{
    context: &'a mut (dyn Context<'icu> + 'a),
    cleanup: Option<F>,
}

impl<'a, 'icu, F> ContextCleanupGuard<'a, 'icu, F>
where
    F: FnOnce(&mut dyn Context<'_>) + 'static,
{
    /// Creates a new `ContextCleanupGuard` from the current context and its cleanup operation.
    pub(crate) fn new(context: &'a mut dyn Context<'icu>, cleanup: F) -> Self {
        Self {
            context,
            cleanup: Some(cleanup),
        }
    }
}

impl<'a, 'icu, F> std::ops::Deref for ContextCleanupGuard<'a, 'icu, F>
where
    F: FnOnce(&mut dyn Context<'_>) + 'static,
{
    type Target = dyn Context<'icu> + 'a;

    fn deref(&self) -> &Self::Target {
        self.context
    }
}

impl<F> std::ops::DerefMut for ContextCleanupGuard<'_, '_, F>
where
    F: FnOnce(&mut dyn Context<'_>) + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context
    }
}

impl<F> Drop for ContextCleanupGuard<'_, '_, F>
where
    F: FnOnce(&mut dyn Context<'_>) + 'static,
{
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup(self.context);
        }
    }
}

///
#[derive(Debug)]
pub struct DefaultContext<'icu> {
    raw: RawContext<'icu>,
    module_loader: SimpleModuleLoader,
    job_queue: VecDeque<NativeJob>,
}

impl DefaultContext<'_> {
    ///
    pub fn with_root<P>(root: P) -> JsResult<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            raw: RawContext::default(),
            module_loader: SimpleModuleLoader::new(root)?,
            job_queue: VecDeque::default(),
        })
    }

    ///
    #[inline]
    pub fn module_loader(&mut self) -> &mut SimpleModuleLoader {
        &mut self.module_loader
    }
}

impl Default for DefaultContext<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            raw: RawContext::default(),
            module_loader: SimpleModuleLoader::new(".").expect(""),
            job_queue: VecDeque::default(),
        }
    }
}

impl<'icu> Context<'icu> for DefaultContext<'icu> {
    #[inline]
    fn as_raw_context(&self) -> &RawContext<'icu> {
        &self.raw
    }

    #[inline]
    fn as_raw_context_mut(&mut self) -> &mut RawContext<'icu> {
        &mut self.raw
    }
}

impl JobQueue for DefaultContext<'_> {
    #[inline]
    fn enqueue_promise_job(&mut self, job: NativeJob) {
        self.job_queue.push_back(job);
    }

    #[inline]
    fn run_jobs(&mut self) {
        while let Some(job) = self.job_queue.pop_front() {
            if job.call(self).is_err() {
                self.job_queue.clear();
                return;
            }
        }
    }

    #[inline]
    fn enqueue_future_job(&mut self, future: crate::job::FutureJob) {
        let job = pollster::block_on(future);
        self.enqueue_promise_job(job);
    }
}

impl ModuleLoader for DefaultContext<'_> {
    fn load_imported_module(
        &mut self,
        _referrer: crate::module::Referrer,
        specifier: crate::JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut dyn Context<'_>)>,
    ) {
        let result = (|| {
            let path = specifier
                .to_std_string()
                .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
            let short_path = Path::new(&path);
            let path = self.module_loader.root().join(short_path);
            let path = path.canonicalize().map_err(|err| {
                JsNativeError::typ()
                    .with_message(format!(
                        "could not canonicalize path `{}`",
                        short_path.display()
                    ))
                    .with_cause(JsError::from_opaque(js_string!(err.to_string()).into()))
            })?;
            if let Some(module) = self.module_loader.get(&path) {
                return Ok(module);
            }
            let source = Source::from_filepath(&path).map_err(|err| {
                JsNativeError::typ()
                    .with_message(format!("could not open file `{}`", short_path.display()))
                    .with_cause(JsError::from_opaque(js_string!(err.to_string()).into()))
            })?;
            let module = Module::parse(source, None, self).map_err(|err| {
                JsNativeError::syntax()
                    .with_message(format!("could not parse module `{}`", short_path.display()))
                    .with_cause(err)
            })?;
            self.module_loader.insert(path, module.clone());
            Ok(module)
        })();

        finish_load(result, self);
    }
}
