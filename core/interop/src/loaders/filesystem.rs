//! Filesystem module loader. Loads modules from the filesystem.

use boa_engine::module::{ModuleLoader, Referrer, resolve_module_specifier};
use boa_engine::{Context, JsError, JsNativeError, JsResult, JsString, Module, Source, js_string};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// A module loader that loads modules from the filesystem.
#[derive(Clone, Debug)]
pub struct FsModuleLoader {
    root: PathBuf,
}

impl FsModuleLoader {
    /// Create a new [`FsModuleLoader`] from a root path.
    ///
    /// # Errors
    /// An error happens if the root path cannot be canonicalized (e.g. does
    /// not exists).
    pub fn new(root: impl AsRef<Path>) -> JsResult<Self> {
        let root = root.as_ref();
        let root = root.canonicalize().map_err(|e| {
            JsNativeError::typ()
                .with_message(format!("could not set module root `{}`", root.display()))
                .with_cause(JsError::from_opaque(js_string!(e.to_string()).into()))
        })?;

        Ok(Self { root })
    }
}

impl ModuleLoader for FsModuleLoader {
    fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        specifier: JsString,
        context: &RefCell<&mut Context>,
    ) -> impl Future<Output = JsResult<Module>> {
        let result = (|| {
            let short_path = specifier.to_std_string_escaped();
            let path = resolve_module_specifier(
                Some(&self.root),
                &specifier,
                referrer.path(),
                &mut context.borrow_mut(),
            )?;

            let source = Source::from_filepath(&path).map_err(|err| {
                JsNativeError::typ()
                    .with_message(format!("could not open file `{short_path}`"))
                    .with_cause(JsError::from_opaque(js_string!(err.to_string()).into()))
            })?;
            let module = Module::parse(source, None, &mut context.borrow_mut()).map_err(|err| {
                JsNativeError::syntax()
                    .with_message(format!("could not parse module `{short_path}`"))
                    .with_cause(err)
            })?;
            Ok(module)
        })();
        async { result }
    }
}
