//! A [`ModuleLoader`] that "merges" two module loaders into one. It will
//! try to resolve using the first loader, and if it fails, it will try the
//! second.
//!
//! Any errors from the first loader will be ignored, and the second loader
//! will be used.

use boa_engine::{Context, JsResult, JsString, Module};
use boa_engine::module::{ModuleLoader, Referrer};

pub struct MergeModuleLoader<First, Second> {
    first: First,
    second: Second,
}

impl<First: ModuleLoader, Second: ModuleLoader> From<(First, Second)>
    for MergeModuleLoader<First, Second>
{
    fn from((first, second): (First, Second)) -> Self {
        Self::new(first, second)
    }
}

impl<First, Second> MergeModuleLoader<First, Second> {
    /// Create a new `MergeModuleLoader` from two module loaders.
    pub fn new(first: First, second: Second) -> Self {
        Self { first, second }
    }
}

impl<First, Second> ModuleLoader for MergeModuleLoader<First, Second>
where
    First: ModuleLoader,
    Second: ModuleLoader,
{
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        self.first.load_imported_module(
            referrer.clone(),
            specifier.clone(),
            Box::new(move |result, context| {
                if result.is_err() {
                    self.second
                        .load_imported_module(referrer, specifier, finish_load, context);
                } else {
                    finish_load(result, context);
                }
            }),
            context,
        );
    }
}
