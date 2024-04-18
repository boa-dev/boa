//! Embedded module loader. Creates a `ModuleLoader` instance that contains
//! files embedded in the binary at build time.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use boa_engine::module::{ModuleLoader, Referrer};
use boa_engine::{Context, JsNativeError, JsResult, JsString, Module, Source};

/// Create a module loader that embeds files from the filesystem at build
/// time. This is useful for bundling assets with the binary.
///
/// By default, will error if the total file size exceeds 1MB. This can be
/// changed by specifying the `max_size` parameter.
///
/// The embedded module will only contain files that have the `.js`, `.mjs`,
/// or `.cjs` extension.
#[macro_export]
macro_rules! embed_module {
    ($path: literal, max_size = $max_size: literal) => {
        $crate::loaders::embedded::EmbeddedModuleLoader::from_iter(
            $crate::boa_macros::embed_module_inner!($path, $max_size),
        )
    };
    ($path: literal) => {
        embed_module!($path, max_size = 1_048_576)
    };
}

#[derive(Debug, Clone)]
enum EmbeddedModuleEntry {
    Source(JsString, &'static [u8]),
    Module(Module),
}

impl EmbeddedModuleEntry {
    fn from_source(path: JsString, source: &'static [u8]) -> Self {
        Self::Source(path, source)
    }

    fn cache(&mut self, context: &mut Context) -> JsResult<&Module> {
        if let Self::Source(path, source) = self {
            let mut bytes: &[u8] = source;
            let path = path.to_std_string_escaped();
            let source = Source::from_reader(&mut bytes, Some(Path::new(&path)));
            match Module::parse(source, None, context) {
                Ok(module) => {
                    *self = Self::Module(module);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        };

        match self {
            Self::Module(module) => Ok(module),
            EmbeddedModuleEntry::Source(_, _) => unreachable!(),
        }
    }

    fn as_module(&self) -> Option<&Module> {
        match self {
            Self::Module(module) => Some(module),
            Self::Source(_, _) => None,
        }
    }
}

/// The resulting type of creating an embedded module loader.
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct EmbeddedModuleLoader {
    map: HashMap<JsString, RefCell<EmbeddedModuleEntry>>,
}

impl FromIterator<(&'static str, &'static [u8])> for EmbeddedModuleLoader {
    fn from_iter<T: IntoIterator<Item = (&'static str, &'static [u8])>>(iter: T) -> Self {
        Self {
            map: iter
                .into_iter()
                .map(|(path, source)| {
                    let p = JsString::from(path);
                    (
                        p.clone(),
                        RefCell::new(EmbeddedModuleEntry::from_source(p, source)),
                    )
                })
                .collect(),
        }
    }
}

impl ModuleLoader for EmbeddedModuleLoader {
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        let Ok(specifier_path) = boa_engine::module::resolve_module_specifier(
            None,
            &specifier,
            referrer.path(),
            context,
        ) else {
            let err = JsNativeError::typ().with_message(format!(
                "could not resolve module specifier `{}`",
                specifier.to_std_string_escaped()
            ));
            finish_load(Err(err.into()), context);
            return;
        };

        if let Some(module) = self
            .map
            .get(&JsString::from(specifier_path.to_string_lossy().as_ref()))
        {
            let mut embedded = module.borrow_mut();
            let module = embedded.cache(context);

            finish_load(module.cloned(), context);
        } else {
            let err = JsNativeError::typ().with_message(format!(
                "could not find module `{}`",
                specifier.to_std_string_escaped()
            ));
            finish_load(Err(err.into()), context);
        }
    }

    fn get_module(&self, specifier: JsString) -> Option<Module> {
        self.map
            .get(&specifier)
            .and_then(|module| module.borrow().as_module().cloned())
    }
}
