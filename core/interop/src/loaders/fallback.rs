//! A module loader that tries to load modules from multiple loaders.
use boa_engine::module::{ModuleLoader, Referrer};
use boa_engine::{Context, JsResult, JsString, Module};

/// A [`ModuleLoader`] that tries to load a module from one loader, and if that fails,
/// falls back to another loader.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct FallbackModuleLoader<L, R>(L, R);

impl<L, R> FallbackModuleLoader<L, R> {
    /// Create a new [`FallbackModuleLoader`] from two loaders.
    pub fn new(loader: L, fallback: R) -> Self {
        Self(loader, fallback)
    }
}

impl<L, R> ModuleLoader for FallbackModuleLoader<L, R>
where
    L: ModuleLoader,
    R: ModuleLoader + Clone + 'static,
{
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        self.0.load_imported_module(
            referrer.clone(),
            specifier.clone(),
            {
                let fallback = self.1.clone();
                Box::new(move |result, context| {
                    if result.is_ok() {
                        finish_load(result, context);
                    } else {
                        fallback.load_imported_module(referrer, specifier, finish_load, context);
                    }
                })
            },
            context,
        );
    }
}
