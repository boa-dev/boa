//! Filesystem module loader. Loads modules from the filesystem.

use boa_engine::module::{resolve_module_specifier, ModuleLoader, Referrer};
use boa_engine::{js_string, Context, JsError, JsNativeError, JsResult, JsString, Module, Source};
use std::path::PathBuf;

/// A module loader that loads modules from the filesystem.
#[derive(Clone, Debug)]
pub struct FsModuleLoader {
    root: PathBuf,
}

impl FsModuleLoader {
    /// Create a new [`FsModuleLoader`] from a root path.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl ModuleLoader for FsModuleLoader {
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        let result = (|| -> JsResult<Module> {
            let short_path = specifier.to_std_string_escaped();
            let path =
                resolve_module_specifier(Some(&self.root), &specifier, referrer.path(), context)?;

            let source = Source::from_filepath(&path).map_err(|err| {
                JsNativeError::typ()
                    .with_message(format!("could not open file `{short_path}`"))
                    .with_cause(JsError::from_opaque(js_string!(err.to_string()).into()))
            })?;
            let module = Module::parse(source, None, context).map_err(|err| {
                JsNativeError::syntax()
                    .with_message(format!("could not parse module `{short_path}`"))
                    .with_cause(err)
            })?;
            Ok(module)
        })();

        finish_load(result, context);
    }
}
