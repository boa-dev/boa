//! Boa's implementation of the ECMAScript's module system.
//!
//! This module contains the [`Module`] type, which represents an [**Abstract Module Record**][module],
//! a [`ModuleLoader`] trait for custom module loader implementations, and [`SimpleModuleLoader`],
//! the default `ModuleLoader` for [`Context`] which can be used for most simple usecases.
//!
//! Every module roughly follows the same lifecycle:
//! - Parse using [`Module::parse`].
//! - Load all its dependencies using [`Module::load`].
//! - Link its dependencies together using [`Module::link`].
//! - Evaluate the module and its dependencies using [`Module::evaluate`].
//!
//! The [`ModuleLoader`] trait allows customizing the "load" step on the lifecycle
//! of a module, which allows doing things like fetching modules from urls, having multiple
//! "modpaths" from where to import modules, or using Rust futures to avoid blocking the main thread
//! on loads.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-modules
//! [module]: https://tc39.es/ecma262/#sec-abstract-module-records

mod source;
use source::SourceTextModule;

use std::cell::{Cell, RefCell};
use std::hash::Hash;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{collections::HashSet, hash::BuildHasherDefault};

use indexmap::IndexMap;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use boa_ast::expression::Identifier;
use boa_gc::{Finalize, Gc, GcRefCell, Trace};
use boa_interner::Sym;
use boa_parser::{Parser, Source};
use boa_profiler::Profiler;

use crate::object::FunctionObjectBuilder;
use crate::script::Script;
use crate::vm::ActiveRunnable;
use crate::{
    builtins::promise::{PromiseCapability, PromiseState},
    environments::DeclarativeEnvironment,
    object::{JsObject, JsPromise, ObjectData},
    realm::Realm,
    Context, JsError, JsResult, JsString, JsValue,
};
use crate::{js_string, JsNativeError, NativeFunction};

/// The referrer from which a load request of a module originates.
#[derive(Debug, Clone)]
pub enum Referrer {
    /// A [**Source Text Module Record**](https://tc39.es/ecma262/#sec-source-text-module-records).
    Module(Module),
    /// A [**Realm**](https://tc39.es/ecma262/#sec-code-realms).
    Realm(Realm),
    /// A [**Script Record**](https://tc39.es/ecma262/#sec-script-records)
    Script(Script),
}

impl From<ActiveRunnable> for Referrer {
    fn from(value: ActiveRunnable) -> Self {
        match value {
            ActiveRunnable::Script(script) => Self::Script(script),
            ActiveRunnable::Module(module) => Self::Module(module),
        }
    }
}

/// Module loading related host hooks.
///
/// This trait allows to customize the behaviour of the engine on module load requests and
/// `import.meta` requests.
pub trait ModuleLoader {
    /// Host hook [`HostLoadImportedModule ( referrer, specifier, hostDefined, payload )`][spec].
    ///
    /// This hook allows to customize the module loading functionality of the engine. Technically,
    /// this should call the [`FinishLoadingImportedModule`][finish] operation, but this simpler API just provides
    /// a closure that replaces `FinishLoadingImportedModule`.
    ///
    /// # Requirements
    ///
    /// - The host environment must perform `FinishLoadingImportedModule(referrer, specifier, payload, result)`,
    /// where result is either a normal completion containing the loaded Module Record or a throw
    /// completion, either synchronously or asynchronously. This is equivalent to calling the `finish_load`
    /// callback.
    /// - If this operation is called multiple times with the same `(referrer, specifier)` pair and
    /// it performs FinishLoadingImportedModule(referrer, specifier, payload, result) where result
    /// is a normal completion, then it must perform
    /// `FinishLoadingImportedModule(referrer, specifier, payload, result)` with the same result each
    /// time.
    /// - The operation must treat payload as an opaque value to be passed through to
    /// `FinishLoadingImportedModule`. (can be ignored)
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-HostLoadImportedModule
    /// [finish]: https://tc39.es/ecma262/#sec-FinishLoadingImportedModule
    #[allow(clippy::type_complexity)]
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context<'_>)>,
        context: &mut Context<'_>,
    );

    /// Registers a new module into the module loader.
    ///
    /// This is a convenience method for module loaders caching already parsed modules, since it
    /// allows registering a new module through the `&dyn ModuleLoader` provided by
    /// [`Context::module_loader`].
    ///
    /// Does nothing by default.
    fn register_module(&self, _specifier: JsString, _module: Module) {}

    /// Gets the module associated with the provided specifier.
    ///
    /// This is a convenience method for module loaders caching already parsed modules, since it
    /// allows getting a cached module through the `&dyn ModuleLoader` provided by
    /// [`Context::module_loader`].
    ///
    /// Returns `None` by default.
    fn get_module(&self, _specifier: JsString) -> Option<Module> {
        None
    }

    /// Host hooks [`HostGetImportMetaProperties ( moduleRecord )`][meta] and
    /// [`HostFinalizeImportMeta ( importMeta, moduleRecord )`][final].
    ///
    /// This unifies both APIs into a single hook that can be overriden on both cases.
    /// The most common usage is to add properties to `import_meta` and return, but this also
    /// allows modifying the import meta object in more exotic ways before exposing it to ECMAScript
    /// code.
    ///
    /// The default implementation of `HostGetImportMetaProperties` is to return a new empty List.
    ///
    /// [meta]: https://tc39.es/ecma262/#sec-hostgetimportmetaproperties
    /// [final]: https://tc39.es/ecma262/#sec-hostfinalizeimportmeta
    fn init_import_meta(
        &self,
        _import_meta: &JsObject,
        _module: &Module,
        _context: &mut Context<'_>,
    ) {
    }
}

/// A module loader that throws when trying to load any modules.
///
/// Useful to disable the module system on platforms that don't have a filesystem, for example.
#[derive(Debug, Clone, Copy)]
pub struct IdleModuleLoader;

impl ModuleLoader for IdleModuleLoader {
    fn load_imported_module(
        &self,
        _referrer: Referrer,
        _specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context<'_>)>,
        context: &mut Context<'_>,
    ) {
        finish_load(
            Err(JsNativeError::typ()
                .with_message("module resolution is disabled for this context")
                .into()),
            context,
        );
    }
}

/// A simple module loader that loads modules relative to a root path.
#[derive(Debug)]
pub struct SimpleModuleLoader {
    root: PathBuf,
    module_map: GcRefCell<FxHashMap<PathBuf, Module>>,
}

impl SimpleModuleLoader {
    /// Creates a new `SimpleModuleLoader` from a root module path.
    pub fn new<P: AsRef<Path>>(root: P) -> JsResult<Self> {
        if cfg!(target_family = "wasm") {
            return Err(JsNativeError::typ()
                .with_message("cannot resolve a relative path in WASM targets")
                .into());
        }
        let root = root.as_ref();
        let absolute = root.canonicalize().map_err(|e| {
            JsNativeError::typ()
                .with_message(format!("could not set module root `{}`", root.display()))
                .with_cause(JsError::from_opaque(js_string!(e.to_string()).into()))
        })?;
        Ok(Self {
            root: absolute,
            module_map: GcRefCell::default(),
        })
    }

    /// Inserts a new module onto the module map.
    #[inline]
    pub fn insert(&self, path: PathBuf, module: Module) {
        self.module_map.borrow_mut().insert(path, module);
    }

    /// Gets a module from its original path.
    #[inline]
    pub fn get(&self, path: &Path) -> Option<Module> {
        self.module_map.borrow().get(path).cloned()
    }
}

impl ModuleLoader for SimpleModuleLoader {
    fn load_imported_module(
        &self,
        _referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context<'_>)>,
        context: &mut Context<'_>,
    ) {
        let result = (|| {
            let path = specifier
                .to_std_string()
                .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
            let short_path = Path::new(&path);
            let path = self.root.join(short_path);
            let path = path.canonicalize().map_err(|err| {
                JsNativeError::typ()
                    .with_message(format!(
                        "could not canonicalize path `{}`",
                        short_path.display()
                    ))
                    .with_cause(JsError::from_opaque(js_string!(err.to_string()).into()))
            })?;
            if let Some(module) = self.get(&path) {
                return Ok(module);
            }
            let source = Source::from_filepath(&path).map_err(|err| {
                JsNativeError::typ()
                    .with_message(format!("could not open file `{}`", short_path.display()))
                    .with_cause(JsError::from_opaque(js_string!(err.to_string()).into()))
            })?;
            let module = Module::parse(source, None, context).map_err(|err| {
                JsNativeError::syntax()
                    .with_message(format!("could not parse module `{}`", short_path.display()))
                    .with_cause(err)
            })?;
            self.insert(path, module.clone());
            Ok(module)
        })();

        finish_load(result, context);
    }

    // TODO: Try to unify `ModuleLoader::register_module` with `SimpleModuleLoader::insert`.
    fn register_module(&self, _specifier: JsString, _module: Module) {
        panic!(
            "`SimpleModuleLoader` uses paths to cache the modules instead of specifiers.
        To register a module, you need to use the `SimpleModuleLoader::insert` method."
        )
    }

    // TODO: Try to unify `ModuleLoader::get_module` with `SimpleModuleLoader::get`.
    fn get_module(&self, _specifier: JsString) -> Option<Module> {
        panic!(
            "`SimpleModuleLoader` uses paths to cache the modules instead of specifiers.
        To get a module, you need to use the `SimpleModuleLoader::get` method."
        )
    }
}

/// ECMAScript's [**Abstract module record**][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-abstract-module-records
#[derive(Clone, Trace, Finalize)]
pub struct Module {
    inner: Gc<Inner>,
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("realm", &self.inner.realm.addr())
            .field("environment", &self.inner.environment)
            .field("namespace", &self.inner.namespace)
            .field("kind", &self.inner.kind)
            .field("host_defined", &self.inner.host_defined)
            .finish()
    }
}

#[derive(Trace, Finalize)]
struct Inner {
    realm: Realm,
    environment: GcRefCell<Option<Gc<DeclarativeEnvironment>>>,
    namespace: GcRefCell<Option<JsObject>>,
    kind: ModuleKind,
    host_defined: (),
}

/// The kind of a [`Module`].
#[derive(Debug, Trace, Finalize)]
pub(crate) enum ModuleKind {
    /// A [**Source Text Module Record**](https://tc39.es/ecma262/#sec-source-text-module-records)
    SourceText(SourceTextModule),
    /// A [**Synthetic Module Record**](https://tc39.es/proposal-json-modules/#sec-synthetic-module-records)
    #[allow(unused)]
    Synthetic,
}

/// Return value of the [`Module::resolve_export`] operation.
///
/// Indicates how to access a specific export in a module.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedBinding {
    module: Module,
    binding_name: BindingName,
}

/// The local name of the resolved binding within its containing module.
///
/// Note that a resolved binding can resolve to a single binding inside a module (`export var a = 1"`)
/// or to a whole module namespace (`export * as ns from "mod.js"`).
#[derive(Debug, Clone, Copy)]
pub(crate) enum BindingName {
    /// A local binding.
    Name(Identifier),
    /// The whole namespace of the containing module.
    Namespace,
}

impl ResolvedBinding {
    /// Gets the module from which the export resolved.
    pub(crate) const fn module(&self) -> &Module {
        &self.module
    }

    /// Gets the binding associated with the resolved export.
    pub(crate) const fn binding_name(&self) -> BindingName {
        self.binding_name
    }
}

#[derive(Debug, Clone)]
struct GraphLoadingState {
    capability: PromiseCapability,
    loading: Cell<bool>,
    pending_modules: Cell<usize>,
    visited: RefCell<HashSet<SourceTextModule>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ResolveExportError {
    NotFound,
    Ambiguous,
}

impl Module {
    /// Abstract operation [`ParseModule ( sourceText, realm, hostDefined )`][spec].
    ///
    /// Parses the provided `src` as an ECMAScript module, returning an error if parsing fails.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parsemodule
    #[inline]
    pub fn parse<R: Read>(
        src: Source<'_, R>,
        realm: Option<Realm>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let _timer = Profiler::global().start_event("Module parsing", "Main");
        let mut parser = Parser::new(src);
        parser.set_identifier(context.next_parser_identifier());
        let module = parser.parse_module(context.interner_mut())?;

        let src = SourceTextModule::new(module);

        let module = Self {
            inner: Gc::new(Inner {
                realm: realm.unwrap_or_else(|| context.realm().clone()),
                environment: GcRefCell::default(),
                namespace: GcRefCell::default(),
                kind: ModuleKind::SourceText(src.clone()),
                host_defined: (),
            }),
        };

        src.set_parent(module.clone());

        Ok(module)
    }

    /// Gets the realm of this `Module`.
    #[inline]
    pub fn realm(&self) -> &Realm {
        &self.inner.realm
    }

    /// Gets the kind of this `Module`.
    pub(crate) fn kind(&self) -> &ModuleKind {
        &self.inner.kind
    }

    /// Gets the environment of this `Module`.
    pub(crate) fn environment(&self) -> Option<Gc<DeclarativeEnvironment>> {
        self.inner.environment.borrow().clone()
    }

    /// Abstract method [`LoadRequestedModules ( [ hostDefined ] )`][spec].
    ///
    /// Prepares the module for linking by loading all its module dependencies. Returns a `JsPromise`
    /// that will resolve when the loading process either completes or fails.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn load(&self, context: &mut Context<'_>) -> JsPromise {
        match self.kind() {
            ModuleKind::SourceText(_) => {
                // Concrete method [`LoadRequestedModules ( [ hostDefined ] )`][spec].
                //
                // [spec]: https://tc39.es/ecma262/#sec-LoadRequestedModules
                // TODO: 1. If hostDefined is not present, let hostDefined be empty.

                // 2. Let pc be ! NewPromiseCapability(%Promise%).
                let pc = PromiseCapability::new(
                    &context.intrinsics().constructors().promise().constructor(),
                    context,
                )
                .expect(
                    "capability creation must always succeed when using the `%Promise%` intrinsic",
                );

                // 4. Perform InnerModuleLoading(state, module).
                self.inner_load(
                    // 3. Let state be the GraphLoadingState Record {
                    //     [[IsLoading]]: true, [[PendingModulesCount]]: 1, [[Visited]]: « »,
                    //     [[PromiseCapability]]: pc, [[HostDefined]]: hostDefined
                    // }.
                    &Rc::new(GraphLoadingState {
                        capability: pc.clone(),
                        loading: Cell::new(true),
                        pending_modules: Cell::new(1),
                        visited: RefCell::default(),
                    }),
                    context,
                );

                // 5. Return pc.[[Promise]].
                JsPromise::from_object(pc.promise().clone())
                    .expect("promise created from the %Promise% intrinsic is always native")
            }
            ModuleKind::Synthetic => todo!("synthetic.load()"),
        }
    }

    /// Abstract operation [`InnerModuleLoading`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-InnerModuleLoading
    fn inner_load(&self, state: &Rc<GraphLoadingState>, context: &mut Context<'_>) {
        // 1. Assert: state.[[IsLoading]] is true.
        assert!(state.loading.get());

        if let ModuleKind::SourceText(src) = self.kind() {
            // continues on `inner_load
            src.inner_load(state, context);
            if !state.loading.get() {
                return;
            }
        }

        // 3. Assert: state.[[PendingModulesCount]] ≥ 1.
        assert!(state.pending_modules.get() >= 1);

        // 4. Set state.[[PendingModulesCount]] to state.[[PendingModulesCount]] - 1.
        state.pending_modules.set(state.pending_modules.get() - 1);
        // 5. If state.[[PendingModulesCount]] = 0, then

        if state.pending_modules.get() == 0 {
            // a. Set state.[[IsLoading]] to false.
            state.loading.set(false);
            // b. For each Cyclic Module Record loaded of state.[[Visited]], do
            //    i. If loaded.[[Status]] is new, set loaded.[[Status]] to unlinked.
            // By default, all modules start on `unlinked`.

            // c. Perform ! Call(state.[[PromiseCapability]].[[Resolve]], undefined, « undefined »).
            state
                .capability
                .resolve()
                .call(&JsValue::undefined(), &[], context)
                .expect("marking a module as loaded should not fail");
        }
        // 6. Return unused.
    }

    /// Abstract method [`GetExportedNames([exportStarSet])`][spec].
    ///
    /// Returns a list of all the names exported from this module.
    ///
    /// # Note
    ///
    /// This must only be called if the [`JsPromise`] returned by [`Module::load`] has fulfilled.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    fn get_exported_names(&self, export_star_set: &mut Vec<SourceTextModule>) -> FxHashSet<Sym> {
        match self.kind() {
            ModuleKind::SourceText(src) => src.get_exported_names(export_star_set),
            ModuleKind::Synthetic => todo!("synthetic.get_exported_names()"),
        }
    }

    /// Abstract method [`ResolveExport(exportName [, resolveSet])`][spec].
    ///
    /// Returns the corresponding local binding of a binding exported by this module.
    /// The spec requires that this operation must be idempotent; calling this multiple times
    /// with the same `export_name` and `resolve_set` should always return the same result.
    ///
    /// # Note
    ///
    /// This must only be called if the [`JsPromise`] returned by [`Module::load`] has fulfilled.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    #[allow(clippy::mutable_key_type)]
    pub(crate) fn resolve_export(
        &self,
        export_name: Sym,
        resolve_set: &mut FxHashSet<(Self, Sym)>,
    ) -> Result<ResolvedBinding, ResolveExportError> {
        match self.kind() {
            ModuleKind::SourceText(src) => src.resolve_export(export_name, resolve_set),
            ModuleKind::Synthetic => todo!("synthetic.resolve_export()"),
        }
    }

    /// Abstract method [`Link() `][spec].
    ///
    /// Prepares this module for evaluation by resolving all its module dependencies and initializing
    /// its environment.
    ///
    /// # Note
    ///
    /// This must only be called if the [`JsPromise`] returned by [`Module::load`] has fulfilled.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn link(&self, context: &mut Context<'_>) -> JsResult<()> {
        match self.kind() {
            ModuleKind::SourceText(src) => src.link(context),
            ModuleKind::Synthetic => todo!("synthetic.link()"),
        }
    }

    /// Abstract operation [`InnerModuleLinking ( module, stack, index )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-InnerModuleLinking
    fn inner_link(
        &self,
        stack: &mut Vec<SourceTextModule>,
        index: usize,
        context: &mut Context<'_>,
    ) -> JsResult<usize> {
        match self.kind() {
            ModuleKind::SourceText(src) => src.inner_link(stack, index, context),
            #[allow(unreachable_code)]
            // If module is not a Cyclic Module Record, then
            ModuleKind::Synthetic => {
                // a. Perform ? module.Link().
                todo!("synthetic.link()");
                // b. Return index.
                Ok(index)
            }
        }
    }

    /// Abstract method [`Evaluate()`][spec].
    ///
    /// Evaluates this module, returning a promise for the result of the evaluation of this module
    /// and its dependencies.
    /// If the promise is rejected, hosts are expected to handle the promise rejection and rethrow
    /// the evaluation error.
    ///
    /// # Note
    ///
    /// This must only be called if the [`Module::link`] method finished successfully.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn evaluate(&self, context: &mut Context<'_>) -> JsPromise {
        match self.kind() {
            ModuleKind::SourceText(src) => src.evaluate(context),
            ModuleKind::Synthetic => todo!("synthetic.evaluate()"),
        }
    }

    /// Abstract operation [`InnerModuleLinking ( module, stack, index )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-InnerModuleLinking
    fn inner_evaluate(
        &self,
        stack: &mut Vec<SourceTextModule>,
        index: usize,
        context: &mut Context<'_>,
    ) -> JsResult<usize> {
        match self.kind() {
            ModuleKind::SourceText(src) => src.inner_evaluate(stack, index, None, context),
            // 1. If module is not a Cyclic Module Record, then
            #[allow(unused, clippy::diverging_sub_expression)]
            ModuleKind::Synthetic => {
                // a. Let promise be ! module.Evaluate().
                let promise: JsPromise = todo!("module.Evaluate()");
                let state = promise.state()?;
                match state {
                    PromiseState::Pending => {
                        unreachable!("b. Assert: promise.[[PromiseState]] is not pending.")
                    }
                    // d. Return index.
                    PromiseState::Fulfilled(_) => Ok(index),
                    // c. If promise.[[PromiseState]] is rejected, then
                    //    i. Return ThrowCompletion(promise.[[PromiseResult]]).
                    PromiseState::Rejected(err) => Err(JsError::from_opaque(err)),
                }
            }
        }
    }

    /// Loads, links and evaluates this module, returning a promise that will resolve after the module
    /// finishes its lifecycle.
    ///
    /// # Examples
    /// ```
    /// # use std::path::Path;
    /// # use boa_engine::{Context, Source, Module, JsValue};
    /// # use boa_engine::builtins::promise::PromiseState;
    /// # use boa_engine::module::{ModuleLoader, SimpleModuleLoader};
    /// let loader = &SimpleModuleLoader::new(Path::new(".")).unwrap();
    /// let dyn_loader: &dyn ModuleLoader = loader;
    /// let mut context = &mut Context::builder().module_loader(dyn_loader).build().unwrap();
    ///
    /// let source = Source::from_bytes("1 + 3");
    ///
    /// let module = Module::parse(source, None, context).unwrap();
    ///
    /// loader.insert(Path::new("main.mjs").to_path_buf(), module.clone());
    ///
    /// let promise = module.load_link_evaluate(context).unwrap();
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(promise.state().unwrap(), PromiseState::Fulfilled(JsValue::undefined()));
    /// ```
    #[allow(clippy::drop_copy)]
    #[inline]
    pub fn load_link_evaluate(&self, context: &mut Context<'_>) -> JsResult<JsPromise> {
        let main_timer = Profiler::global().start_event("Module evaluation", "Main");

        let promise = self
            .load(context)
            .then(
                Some(
                    FunctionObjectBuilder::new(
                        context,
                        NativeFunction::from_copy_closure_with_captures(
                            |_, _, module, context| {
                                module.link(context)?;
                                Ok(JsValue::undefined())
                            },
                            self.clone(),
                        ),
                    )
                    .build(),
                ),
                None,
                context,
            )?
            .then(
                Some(
                    FunctionObjectBuilder::new(
                        context,
                        NativeFunction::from_copy_closure_with_captures(
                            |_, _, module, context| Ok(module.evaluate(context).into()),
                            self.clone(),
                        ),
                    )
                    .build(),
                ),
                None,
                context,
            )?;

        // The main_timer needs to be dropped before the Profiler is.
        drop(main_timer);
        Profiler::global().drop();

        Ok(promise)
    }

    /// Abstract operation [`GetModuleNamespace ( module )`][spec].
    ///
    /// Gets the [**Module Namespace Object**][ns] that represents this module's exports.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getmodulenamespace
    /// [ns]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects
    pub fn namespace(&self, context: &mut Context<'_>) -> JsObject {
        // 1. Assert: If module is a Cyclic Module Record, then module.[[Status]] is not new or unlinked.
        // 2. Let namespace be module.[[Namespace]].
        // 3. If namespace is empty, then
        // 4. Return namespace.
        self.inner
            .namespace
            .borrow_mut()
            .get_or_insert_with(|| {
                // a. Let exportedNames be module.GetExportedNames().
                let exported_names = self.get_exported_names(&mut Vec::default());

                // b. Let unambiguousNames be a new empty List.
                let unambiguous_names = exported_names
                    .into_iter()
                    // c. For each element name of exportedNames, do
                    .filter_map(|name| {
                        // i. Let resolution be module.ResolveExport(name).
                        // ii. If resolution is a ResolvedBinding Record, append name to unambiguousNames.
                        self.resolve_export(name, &mut HashSet::default())
                            .ok()
                            .map(|_| name)
                    })
                    .collect();

                //     d. Set namespace to ModuleNamespaceCreate(module, unambiguousNames).
                ModuleNamespace::create(self.clone(), unambiguous_names, context)
            })
            .clone()
    }
}

impl PartialEq for Module {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.inner.as_ref(), other.inner.as_ref())
    }
}

impl Eq for Module {}

impl Hash for Module {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.inner.as_ref(), state);
    }
}

/// Module namespace exotic object.
///
/// Exposes the bindings exported by a [`Module`] to be accessed from ECMAScript code.
#[derive(Debug, Trace, Finalize)]
pub struct ModuleNamespace {
    module: Module,
    #[unsafe_ignore_trace]
    exports: IndexMap<JsString, Sym, BuildHasherDefault<FxHasher>>,
}

impl ModuleNamespace {
    /// Abstract operation [`ModuleNamespaceCreate ( module, exports )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-modulenamespacecreate
    pub(crate) fn create(module: Module, names: Vec<Sym>, context: &mut Context<'_>) -> JsObject {
        // 1. Assert: module.[[Namespace]] is empty.
        // ignored since this is ensured by `Module::namespace`.

        // 6. Let sortedExports be a List whose elements are the elements of exports ordered as if an Array of the same values had been sorted using %Array.prototype.sort% using undefined as comparefn.
        let mut exports = names
            .into_iter()
            .map(|sym| {
                (
                    context
                        .interner()
                        .resolve_expect(sym)
                        .into_common::<JsString>(false),
                    sym,
                )
            })
            .collect::<IndexMap<_, _, _>>();
        exports.sort_keys();

        // 2. Let internalSlotsList be the internal slots listed in Table 32.
        // 3. Let M be MakeBasicObject(internalSlotsList).
        // 4. Set M's essential internal methods to the definitions specified in 10.4.6.
        // 5. Set M.[[Module]] to module.
        // 7. Set M.[[Exports]] to sortedExports.
        // 8. Create own properties of M corresponding to the definitions in 28.3.
        let namespace = context.intrinsics().templates().namespace().create(
            ObjectData::module_namespace(Self { module, exports }),
            vec![js_string!("Module").into()],
        );

        // 9. Set module.[[Namespace]] to M.
        // Ignored because this is done by `Module::namespace`

        // 10. Return M.
        namespace
    }

    /// Gets the export names of the Module Namespace object.
    pub(crate) const fn exports(&self) -> &IndexMap<JsString, Sym, BuildHasherDefault<FxHasher>> {
        &self.exports
    }

    /// Gest the module associated with this Module Namespace object.
    pub(crate) const fn module(&self) -> &Module {
        &self.module
    }
}
