//! Module related types.

mod source;
use boa_parser::Source;
use boa_profiler::Profiler;
pub use source::SourceTextModule;

use boa_ast::expression::Identifier;
use boa_interner::Sym;
use indexmap::IndexMap;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use std::cell::{Cell, RefCell};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{collections::HashSet, hash::BuildHasherDefault};

use boa_gc::{Finalize, Gc, GcRefCell, Trace};

use crate::object::FunctionObjectBuilder;
use crate::property::{PropertyDescriptor, PropertyKey};
use crate::{
    builtins::promise::{PromiseCapability, PromiseState},
    environments::DeclarativeEnvironment,
    object::{JsObject, JsPromise, ObjectData},
    realm::Realm,
    Context, JsError, JsResult, JsString, JsValue,
};
use crate::{js_string, JsNativeError, JsSymbol, NativeFunction};

///
#[derive(Debug)]
pub enum Referrer {
    ///
    Module(SourceTextModule),
    ///
    Realm(Realm), // TODO: script
}

///
pub trait ModuleLoader {
    /// Host hook [`HostLoadImportedModule ( referrer, specifier, hostDefined, payload )`][spec].
    ///
    /// This hook allows to customize the module loading functionality of the engine. Technically,
    /// this should return `()` instead of `JsResult`, leaving the responsibility of calling
    /// [`FinishLoadingImportedModule`][finish] to the host, but this simpler API just provides
    /// a closures that replaces [`FinishLoadingImportedModule`]. Specifically, the requirements of
    /// this hook per the spec are as follows:
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
        _import_meta: JsObject,
        _module: Module,
        _context: &mut Context<'_>,
    ) {
    }
}

/// A simple module loader that loads modules relative to a root path.
#[derive(Debug)]
pub struct SimpleModuleLoader {
    root: PathBuf,
    module_map: GcRefCell<FxHashMap<PathBuf, Module>>,
}

impl SimpleModuleLoader {
    /// Creates a new `SimpleModuleLoader`.
    pub fn new(root: &Path) -> JsResult<Self> {
        let absolute = root
            .canonicalize()
            .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
        Ok(Self {
            root: absolute,
            module_map: GcRefCell::default(),
        })
    }

    /// Inserts a new module onto the module map.
    pub fn insert(&self, path: PathBuf, module: Module) {
        self.module_map.borrow_mut().insert(path, module);
    }

    /// Gets a module from its original path.
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
            let path = Path::new(&path);
            let path = self.root.join(path);
            let path = path
                .canonicalize()
                .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
            if let Some(module) = self.get(&path) {
                return Ok(module);
            }
            let source = Source::from_filepath(&path)
                .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
            let module = context.parse_module(source, None)?;
            self.insert(path, module.clone());
            Ok(module)
        })();

        finish_load(result, context);
    }
}

/// ECMAScript's [**Abstract module record**][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-abstract-module-records
#[derive(Debug, Clone, Trace, Finalize)]
pub struct Module {
    inner: Gc<Inner>,
}

#[derive(Trace, Finalize)]
struct Inner {
    realm: Realm,
    environment: GcRefCell<Option<Gc<DeclarativeEnvironment>>>,
    namespace: GcRefCell<Option<JsObject>>,
    kind: ModuleKind,
    host_defined: (),
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner")
            .field("realm", &self.realm.addr())
            .field("environment", &self.environment)
            .field("namespace", &self.namespace)
            .field("kind", &self.kind)
            .field("host_defined", &self.host_defined)
            .finish()
    }
}

#[derive(Debug, Trace, Finalize)]
enum ModuleKind {
    Source(SourceTextModule),
    #[allow(unused)]
    Synthetic,
}

#[derive(Debug, Clone)]
struct GraphLoadingState {
    capability: PromiseCapability,
    loading: Cell<bool>,
    pending_modules: Cell<usize>,
    visited: RefCell<HashSet<SourceTextModule>>,
}

#[derive(Debug, Clone)]
pub(crate) struct ExportLocator {
    module: Module,
    binding_name: BindingName,
}

impl ExportLocator {
    pub(crate) const fn module(&self) -> &Module {
        &self.module
    }

    pub(crate) const fn binding_name(&self) -> BindingName {
        self.binding_name
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum BindingName {
    Name(Identifier),
    Namespace,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ResolveExportError {
    NotFound(Sym),
    Ambiguous(Sym),
}

impl Module {
    pub(crate) fn realm(&self) -> &Realm {
        &self.inner.realm
    }

    pub(crate) fn environment(&self) -> Option<Gc<DeclarativeEnvironment>> {
        self.inner.environment.borrow().clone()
    }

    fn kind(&self) -> &ModuleKind {
        &self.inner.kind
    }

    ///
    #[allow(clippy::missing_panics_doc)]
    pub fn load(&self, context: &mut Context<'_>) -> JsPromise {
        match self.kind() {
            ModuleKind::Source(_) => SourceTextModule::load(self, context),
            ModuleKind::Synthetic => todo!("synthetic.load()"),
        }
    }

    fn inner_load(&self, state: &Rc<GraphLoadingState>, context: &mut Context<'_>) {
        assert!(state.loading.get());

        if let ModuleKind::Source(_) = self.kind() {
            SourceTextModule::inner_load(self, state, context);
            if !state.loading.get() {
                return;
            }
        }

        assert!(state.pending_modules.get() >= 1);

        state.pending_modules.set(state.pending_modules.get() - 1);
        if state.pending_modules.get() == 0 {
            state.loading.set(false);
            state
                .capability
                .resolve()
                .call(&JsValue::undefined(), &[], context)
                .expect("marking a module as loaded should not fail");
        }
    }

    fn resume_load(
        state: &Rc<GraphLoadingState>,
        completion: JsResult<Module>,
        context: &mut Context<'_>,
    ) {
        if !state.loading.get() {
            return;
        }

        match completion {
            Ok(m) => {
                m.inner_load(state, context);
            }
            Err(err) => {
                state.loading.set(false);
                state
                    .capability
                    .reject()
                    .call(&JsValue::undefined(), &[err.to_opaque(context)], context)
                    .expect("cannot fail for the default reject function");
            }
        }
    }

    fn get_exported_names(&self, export_star_set: &mut Vec<SourceTextModule>) -> FxHashSet<Sym> {
        match self.kind() {
            ModuleKind::Source(src) => src.get_exported_names(export_star_set),
            ModuleKind::Synthetic => todo!("synthetic.get_exported_names()"),
        }
    }

    #[allow(clippy::mutable_key_type)]
    pub(crate) fn resolve_export(
        &self,
        export_name: Sym,
        resolve_set: &mut FxHashSet<(Module, Sym)>,
    ) -> Result<ExportLocator, ResolveExportError> {
        match self.kind() {
            ModuleKind::Source(_) => {
                SourceTextModule::resolve_export(self, export_name, resolve_set)
            }
            ModuleKind::Synthetic => todo!("synthetic.resolve_export()"),
        }
    }

    ///
    #[allow(clippy::missing_panics_doc)]
    pub fn link(&self, context: &mut Context<'_>) -> JsResult<()> {
        match self.kind() {
            ModuleKind::Source(_) => SourceTextModule::link(self, context),
            ModuleKind::Synthetic => todo!("synthetic.link()"),
        }
    }

    fn inner_link(
        &self,
        stack: &mut Vec<SourceTextModule>,
        index: usize,
        context: &mut Context<'_>,
    ) -> JsResult<usize> {
        match self.kind() {
            ModuleKind::Source(_) => SourceTextModule::inner_link(self, stack, index, context),
            #[allow(unreachable_code)]
            ModuleKind::Synthetic => {
                todo!("synthetic.load()");
                Ok(index)
            }
        }
    }

    pub(crate) fn get_namespace(&self, context: &mut Context<'_>) -> JsObject {
        if let Some(obj) = self.inner.namespace.borrow().clone() {
            return obj;
        }

        let exported_names = self.get_exported_names(&mut Vec::default());

        let unambiguous_names = exported_names
            .into_iter()
            .filter_map(|name| {
                self.resolve_export(name, &mut HashSet::default())
                    .ok()
                    .map(|_| name)
            })
            .collect();

        let namespace = ModuleNamespace::create(self.clone(), unambiguous_names, context);

        *self.inner.namespace.borrow_mut() = Some(namespace.clone());

        namespace
    }

    ///
    #[allow(clippy::missing_panics_doc)]
    pub fn evaluate(&self, context: &mut Context<'_>) -> JsPromise {
        match self.kind() {
            ModuleKind::Source(src) => src.evaluate(context),
            ModuleKind::Synthetic => todo!("synthetic.evaluate()"),
        }
    }

    fn inner_evaluate(
        &self,
        stack: &mut Vec<SourceTextModule>,
        index: usize,
        context: &mut Context<'_>,
    ) -> JsResult<usize> {
        match self.kind() {
            ModuleKind::Source(src) => src.inner_evaluate(stack, index, None, context),
            #[allow(unused, clippy::diverging_sub_expression)]
            ModuleKind::Synthetic => {
                let promise: JsPromise = todo!("module.Evaluate()");
                let state = promise.state()?;
                match state {
                    PromiseState::Pending => {
                        unreachable!("b. Assert: promise.[[PromiseState]] is not pending.")
                    }
                    PromiseState::Fulfilled(_) => Ok(index),
                    PromiseState::Rejected(err) => Err(JsError::from_opaque(err)),
                }
            }
        }
    }

    /// Loads, links and evaluates the given module `src` by compiling down to bytecode, then
    /// returning a promise that will resolve when the module executes.
    ///
    /// # Examples
    /// ```ignore
    /// # use boa_engine::{Context, Source};
    /// let loader: &ModuleLoader = &ModuleLoader::new(Path::new("."));
    /// let mut context = Context::builder().module_loader(loader).build().unwrap();
    ///
    /// let source = Source::from_bytes("1 + 3");
    ///
    /// let module = context.parse_module(source, None).unwrap();
    ///
    /// loader.insert(Path::new("./main.mjs").canonicalize().unwrap(), module.clone());
    ///
    /// let promise = module.load_link_evaluate(context).unwrap();
    ///
    /// context.run_jobs();
    ///
    /// assert!(matches!(promise.state(), PromiseState::Fulfilled(JsValue::undefined())));
    /// ```
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
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.inner.as_ref(), other.inner.as_ref())
    }
}

impl Eq for Module {}

impl Hash for Module {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.inner.as_ref(), state);
    }
}

/// An object containing the exports of a module.
#[derive(Debug, Trace, Finalize)]
pub struct ModuleNamespace {
    module: Module,
    #[unsafe_ignore_trace]
    exports: IndexMap<JsString, Sym, BuildHasherDefault<FxHasher>>,
}

impl ModuleNamespace {
    pub(crate) fn create(module: Module, names: Vec<Sym>, context: &mut Context<'_>) -> JsObject {
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

        let namespace = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            None,
            ObjectData::module_namespace(ModuleNamespace { module, exports }),
        );

        namespace.borrow_mut().properties_mut().insert(
            &PropertyKey::Symbol(JsSymbol::to_string_tag()),
            PropertyDescriptor::builder()
                .value(js_string!("Module"))
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
        );
        namespace
    }

    pub(crate) const fn exports(&self) -> &IndexMap<JsString, Sym, BuildHasherDefault<FxHasher>> {
        &self.exports
    }

    pub(crate) const fn module(&self) -> &Module {
        &self.module
    }
}
