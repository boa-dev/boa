//! A module loader that caches modules once they're resolved.
use boa_engine::module::{ModuleLoader, Referrer, resolve_module_specifier};
use boa_engine::{Context, JsError, JsNativeError, JsResult, JsString, Module};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

/// A module loader that caches modules once they're resolved.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct CachedModuleLoader<B>
where
    B: ModuleLoader + Clone + 'static,
{
    inner: B,
    // TODO: Use a specifier instead of a PathBuf.
    cache: Rc<RefCell<HashMap<PathBuf, Module>>>,
}

impl<B> CachedModuleLoader<B>
where
    B: ModuleLoader + Clone + 'static,
{
    /// Create a new [`CachedModuleLoader`] from an inner module loader and
    /// an empty cache.
    pub fn new(inner: B) -> Self {
        Self {
            inner,
            cache: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl<B> ModuleLoader for CachedModuleLoader<B>
where
    B: ModuleLoader + Clone + 'static,
{
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        let path = match resolve_module_specifier(None, &specifier, referrer.path(), context) {
            Ok(path) => path,
            Err(err) => {
                finish_load(
                    Err(JsError::from_native(
                        JsNativeError::typ()
                            .with_message("could not resolve module specifier")
                            .with_cause(err),
                    )),
                    context,
                );
                return;
            }
        };

        if let Some(module) = self.cache.borrow().get(&path).cloned() {
            finish_load(Ok(module), context);
        } else {
            self.inner.load_imported_module(
                referrer,
                specifier,
                {
                    let cache = self.cache.clone();
                    Box::new(move |result: JsResult<Module>, context| {
                        if let Ok(module) = &result {
                            cache.borrow_mut().insert(path, module.clone());
                        }
                        finish_load(result, context);
                    })
                },
                context,
            );
        }
    }
}
