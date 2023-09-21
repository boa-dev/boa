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

mod loader;
mod source;
mod synthetic;
pub use loader::*;
use source::SourceTextModule;
pub use synthetic::{SyntheticModule, SyntheticModuleInitializer};

use std::cell::{Cell, RefCell};
use std::hash::Hash;
use std::io::Read;
use std::rc::Rc;
use std::{collections::HashSet, hash::BuildHasherDefault};

use indexmap::IndexMap;
use rustc_hash::{FxHashSet, FxHasher};

use boa_ast::expression::Identifier;
use boa_gc::{Finalize, Gc, GcRefCell, Trace};
use boa_interner::Sym;
use boa_parser::{Parser, Source};
use boa_profiler::Profiler;

use crate::{
    builtins::promise::{PromiseCapability, PromiseState},
    environments::DeclarativeEnvironment,
    js_string,
    object::{FunctionObjectBuilder, JsObject, JsPromise, ObjectData},
    realm::Realm,
    Context, HostDefined, JsError, JsResult, JsString, JsValue, NativeFunction,
};

/// ECMAScript's [**Abstract module record**][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-abstract-module-records
#[derive(Clone, Trace, Finalize)]
pub struct Module {
    inner: Gc<ModuleRepr>,
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("realm", &self.inner.realm.addr())
            .field("environment", &self.inner.environment)
            .field("namespace", &self.inner.namespace)
            .field("kind", &self.inner.kind)
            .finish()
    }
}

#[derive(Trace, Finalize)]
struct ModuleRepr {
    realm: Realm,
    environment: GcRefCell<Option<Gc<DeclarativeEnvironment>>>,
    namespace: GcRefCell<Option<JsObject>>,
    kind: ModuleKind,
    host_defined: HostDefined,
}

/// The kind of a [`Module`].
#[derive(Debug, Trace, Finalize)]
pub(crate) enum ModuleKind {
    /// A [**Source Text Module Record**](https://tc39.es/ecma262/#sec-source-text-module-records)
    SourceText(SourceTextModule),
    /// A [**Synthetic Module Record**](https://tc39.es/proposal-json-modules/#sec-synthetic-module-records)
    Synthetic(SyntheticModule),
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
    pub fn parse<R: Read>(
        src: Source<'_, R>,
        realm: Option<Realm>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let _timer = Profiler::global().start_event("Module parsing", "Main");
        let mut parser = Parser::new(src);
        parser.set_identifier(context.next_parser_identifier());
        let module = parser.parse_module(context.interner_mut())?;

        let inner = Gc::new_cyclic(|weak| {
            let src = SourceTextModule::new(module, weak.clone());

            ModuleRepr {
                realm: realm.unwrap_or_else(|| context.realm().clone()),
                environment: GcRefCell::default(),
                namespace: GcRefCell::default(),
                kind: ModuleKind::SourceText(src),
                host_defined: HostDefined::default(),
            }
        });

        Ok(Self { inner })
    }

    /// Abstract operation [`CreateSyntheticModule ( exportNames, evaluationSteps, realm )`][spec].
    ///
    /// Creates a new Synthetic Module from its list of exported names, its evaluation steps and
    /// optionally a root realm.
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-createsyntheticmodule
    #[inline]
    pub fn synthetic(
        export_names: &[JsString],
        evaluation_steps: SyntheticModuleInitializer,
        realm: Option<Realm>,
        context: &mut Context<'_>,
    ) -> Self {
        let names: FxHashSet<Sym> = export_names
            .iter()
            .map(|string| context.interner_mut().get_or_intern(&**string))
            .collect();
        let realm = realm.unwrap_or_else(|| context.realm().clone());
        let inner = Gc::new_cyclic(|weak| {
            let synth = SyntheticModule::new(names, evaluation_steps, weak.clone());

            ModuleRepr {
                realm,
                environment: GcRefCell::default(),
                namespace: GcRefCell::default(),
                kind: ModuleKind::Synthetic(synth),
                host_defined: HostDefined::default(),
            }
        });

        Self { inner }
    }

    /// Gets the realm of this `Module`.
    #[inline]
    #[must_use]
    pub fn realm(&self) -> &Realm {
        &self.inner.realm
    }

    /// Returns the [`ECMAScript specification`][spec] defined [`\[\[HostDefined\]\]`][`HostDefined`] field of the [`Module`].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-abstract-module-records
    #[inline]
    #[must_use]
    pub fn host_defined(&self) -> &HostDefined {
        &self.inner.host_defined
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
            ModuleKind::Synthetic(_) => SyntheticModule::load(context),
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
            ModuleKind::Synthetic(synth) => synth.get_exported_names(),
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
            ModuleKind::Synthetic(synth) => synth.resolve_export(export_name),
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
            ModuleKind::Synthetic(synth) => {
                synth.link(context);
                Ok(())
            }
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
            // If module is not a Cyclic Module Record, then
            ModuleKind::Synthetic(synth) => {
                // a. Perform ? module.Link().
                synth.link(context);
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
    #[inline]
    pub fn evaluate(&self, context: &mut Context<'_>) -> JsPromise {
        match self.kind() {
            ModuleKind::SourceText(src) => src.evaluate(context),
            ModuleKind::Synthetic(synth) => synth.evaluate(context),
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
            ModuleKind::Synthetic(synth) => {
                // a. Let promise be ! module.Evaluate().
                let promise: JsPromise = synth.evaluate(context);
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
    /// let mut context = &mut Context::builder()
    ///     .module_loader(dyn_loader)
    ///     .build()
    ///     .unwrap();
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
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Fulfilled(JsValue::undefined())
    /// );
    /// ```
    #[allow(dropping_copy_types)]
    #[inline]
    pub fn load_link_evaluate(&self, context: &mut Context<'_>) -> JsResult<JsPromise> {
        let main_timer = Profiler::global().start_event("Module evaluation", "Main");

        let promise = self
            .load(context)
            .then(
                Some(
                    FunctionObjectBuilder::new(
                        context.realm(),
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
                        context.realm(),
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
