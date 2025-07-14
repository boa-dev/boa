//! A module loader that caches modules once they're resolved.
use boa_engine::module::{ModuleLoader, Referrer, resolve_module_specifier};
use boa_engine::{Context, JsNativeError, JsResult, JsString, Module};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

/// A module loader that caches modules once they're resolved.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct CachedModuleLoader<B> {
    inner: Rc<B>,
    // TODO: Use a specifier instead of a PathBuf.
    cache: RefCell<HashMap<PathBuf, Module>>,
}

impl<B> CachedModuleLoader<B> {
    /// Create a new [`CachedModuleLoader`] from an inner module loader and
    /// an empty cache.
    pub fn new(inner: B) -> Self {
        Self {
            inner: Rc::new(inner),
            cache: RefCell::new(HashMap::new()),
        }
    }
}

impl<B> ModuleLoader for CachedModuleLoader<B>
where
    B: ModuleLoader,
{
    async fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        specifier: JsString,
        context: &RefCell<&mut Context>,
    ) -> JsResult<Module> {
        let path =
            resolve_module_specifier(None, &specifier, referrer.path(), &mut context.borrow_mut())
                .map_err(|err| {
                    JsNativeError::typ()
                        .with_message("could not resolve module specifier")
                        .with_cause(err)
                })?;

        if let Some(module) = self.cache.borrow().get(&path).cloned() {
            return Ok(module);
        }

        let module = self
            .inner
            .clone()
            .load_imported_module(referrer, specifier, context)
            .await?;

        self.cache.borrow_mut().insert(path, module.clone());

        Ok(module)
    }
}
