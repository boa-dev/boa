use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;

use boa_gc::GcRefCell;
use boa_parser::Source;

use crate::script::Script;
use crate::{
    js_string, object::JsObject, realm::Realm, vm::ActiveRunnable, Context, JsError, JsNativeError,
    JsResult, JsString,
};

use super::Module;

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
///
/// # Note
///
/// This loader only works by using the type methods [`SimpleModuleLoader::insert`] and
/// [`SimpleModuleLoader::get`]. The utility methods on [`ModuleLoader`] don't work at the moment,
/// but we'll unify both APIs in the future.
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
}
