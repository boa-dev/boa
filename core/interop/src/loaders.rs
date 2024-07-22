//! A collection of JS [`boa_engine::module::ModuleLoader`]s utilities to help in
//! creating custom module loaders.

use boa_engine::module::{ModuleLoader, Referrer};
use boa_engine::{Context, JsError, JsNativeError, JsResult, JsString, Module};
pub use hashmap::HashMapModuleLoader;

pub mod embedded;
pub mod hashmap;

/// Create a [`ModuleLoader`] from a function that takes a referrer and a path,
/// and returns a [Module] if it exists. This is the simplest way to create a
/// custom [`ModuleLoader`].
///
/// This function cannot be `async` and must be blocking. An `async` version of
/// this code will likely exist as a separate function in the future.
///
/// `F` cannot be a mutable closure as it could recursively call itself.
pub struct FnModuleLoader<F>(F, &'static str)
where
    F: Fn(&JsString) -> Option<Module>;

impl<F> FnModuleLoader<F>
where
    F: Fn(&JsString) -> Option<Module>,
{
    /// Create a new [`FnModuleLoader`] from a function that takes a path and returns
    /// a [Module] if it exists.
    pub const fn new(f: F) -> Self {
        Self(f, "Unnamed")
    }

    /// Create a new [`FnModuleLoader`] from a function that takes a path and returns
    /// a [Module] if it exists, with a name.
    pub const fn named(f: F, name: &'static str) -> Self {
        Self(f, name)
    }
}

impl<F> std::fmt::Debug for FnModuleLoader<F>
where
    F: Fn(&JsString) -> Option<Module>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FnModuleLoader").field(&self.1).finish()
    }
}

impl<F> ModuleLoader for FnModuleLoader<F>
where
    F: Fn(&JsString) -> Option<Module>,
{
    fn load_imported_module(
        &self,
        _referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        let maybe_module = self.0(&specifier);
        finish_load(
            maybe_module.ok_or_else(|| {
                JsError::from_native(JsNativeError::error().with_message("Module not found"))
            }),
            context,
        );
    }
}
