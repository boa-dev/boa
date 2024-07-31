//! This module contains types that help create custom module loaders from functions.
use boa_engine::module::{resolve_module_specifier, ModuleLoader, Referrer};
use boa_engine::{Context, JsError, JsNativeError, JsResult, JsString, Module, Source};
use std::io::Cursor;

/// Create a [`ModuleLoader`] from a function that takes a referrer and a path,
/// and returns a [Module] if it exists, or an error.
///
/// This function cannot be `async` and must be blocking. An `async` version of
/// this code will likely exist as a separate function in the future.
///
/// `F` cannot be a mutable closure as it could recursively call itself.
#[derive(Copy, Clone)]
pub struct FnModuleLoader<F>(F, &'static str)
where
    F: Fn(&Referrer, &JsString) -> JsResult<Module>;

impl<F> FnModuleLoader<F>
where
    F: Fn(&Referrer, &JsString) -> JsResult<Module>,
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
    F: Fn(&Referrer, &JsString) -> JsResult<Module>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FnModuleLoader").field(&self.1).finish()
    }
}

impl<F> ModuleLoader for FnModuleLoader<F>
where
    F: Fn(&Referrer, &JsString) -> JsResult<Module>,
{
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        finish_load(self.0(&referrer, &specifier), context);
    }
}

/// Create a module loader from a function that takes a resolved path
/// and optionally returns the source code. The path is resolved before
/// passing it. If the source cannot be found or would generate an
/// error, the function should return `None`.
///
/// This function cannot be `async` and must be blocking. An `async` version of
/// this code will likely exist as a separate function in the future.
///
/// `F` cannot be a mutable closure as it could recursively call itself.
pub struct SourceFnModuleLoader<F>(F, &'static str)
where
    F: Fn(&str) -> Option<String>;

impl<F> std::fmt::Debug for SourceFnModuleLoader<F>
where
    F: Fn(&str) -> Option<String>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SourceFnModuleLoader")
            .field(&self.1)
            .finish()
    }
}

impl<F> SourceFnModuleLoader<F>
where
    F: Fn(&str) -> Option<String>,
{
    /// Create a new [`SourceFnModuleLoader`] from a function.
    pub const fn new(f: F) -> Self {
        Self(f, "Unnamed")
    }

    /// Create a new [`SourceFnModuleLoader`] from a function, with a name.
    /// The name is used in error messages and debug strings.
    pub const fn named(f: F, name: &'static str) -> Self {
        Self(f, name)
    }
}

impl<F> ModuleLoader for SourceFnModuleLoader<F>
where
    F: Fn(&str) -> Option<String>,
{
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        match resolve_module_specifier(None, &specifier, referrer.path(), context) {
            Err(e) => finish_load(Err(e), context),
            Ok(p) => {
                let m = match self.0(&p.to_string_lossy()) {
                    Some(source) => Ok(Source::from_reader(
                        Cursor::new(source.into_bytes()),
                        Some(&p),
                    )),
                    None => Err(JsError::from_native(
                        JsNativeError::error().with_message("Module not found"),
                    )),
                };
                finish_load(m.and_then(|s| Module::parse(s, None, context)), context);
            }
        }
    }
}
