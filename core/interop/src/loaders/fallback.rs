//! A module loader that tries to load modules from multiple loaders.
use std::cell::RefCell;
use std::rc::Rc;

use boa_engine::module::{ModuleLoader, Referrer};
use boa_engine::{Context, JsResult, JsString, Module};

/// A [`ModuleLoader`] that tries to load a module from one loader, and if that fails,
/// falls back to another loader.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct FallbackModuleLoader<L, R>(Rc<L>, Rc<R>);

impl<L, R> FallbackModuleLoader<L, R> {
    /// Create a new [`FallbackModuleLoader`] from two loaders.
    pub fn new(loader: L, fallback: R) -> Self {
        Self(Rc::new(loader), Rc::new(fallback))
    }
}

impl<L, R> ModuleLoader for FallbackModuleLoader<L, R>
where
    L: ModuleLoader,
    R: ModuleLoader,
{
    async fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        specifier: JsString,
        context: &RefCell<&mut Context>,
    ) -> JsResult<Module> {
        if let Ok(module) = self
            .0
            .clone()
            .load_imported_module(referrer.clone(), specifier.clone(), context)
            .await
        {
            return Ok(module);
        }

        self.1
            .clone()
            .load_imported_module(referrer, specifier, context)
            .await
    }
}
