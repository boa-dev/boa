//! This module contains types that help create custom module loaders from functions.
use boa_engine::module::{ModuleLoader, Referrer, resolve_module_specifier};
use boa_engine::{Context, JsNativeError, JsResult, JsString, Module, Source};
use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

/// Create a [`ModuleLoader`] from a function that
/// takes a referrer and a path, and returns a [Module] if it exists, or an error.
///
/// This function cannot be `async` and must be blocking. An `async` version of
/// this code will likely exist as a separate function in the future.
///
/// `F` cannot be a mutable closure as it could recursively call itself.
#[derive(Copy, Clone)]
pub struct FnModuleLoader<F>
where
    F: Fn(&Referrer, &JsString) -> JsResult<Module>,
{
    factory: F,
    name: &'static str,
}

impl<F> FnModuleLoader<F>
where
    F: Fn(&Referrer, &JsString) -> JsResult<Module>,
{
    /// Create a new [`FnModuleLoader`] from a function that takes a path and returns
    /// a [Module] if it exists.
    pub const fn new(factory: F) -> Self {
        Self::named(factory, "Unnamed")
    }

    /// Create a new [`FnModuleLoader`] from a function that takes a path and returns
    /// a [Module] if it exists, with a name.
    pub const fn named(factory: F, name: &'static str) -> Self {
        Self { factory, name }
    }
}

impl<F> std::fmt::Debug for FnModuleLoader<F>
where
    F: Fn(&Referrer, &JsString) -> JsResult<Module>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FnModuleLoader").field(&self.name).finish()
    }
}

impl<F> ModuleLoader for FnModuleLoader<F>
where
    F: Fn(&Referrer, &JsString) -> JsResult<Module> + 'static,
{
    fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        specifier: JsString,
        _context: &RefCell<&mut Context>,
    ) -> impl Future<Output = JsResult<Module>> {
        let module = (self.factory)(&referrer, &specifier);
        async { module }
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
    F: Fn(&str) -> Option<String> + 'static,
{
    fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        specifier: JsString,
        context: &RefCell<&mut Context>,
    ) -> impl Future<Output = JsResult<Module>> {
        let result = (|| {
            let p = resolve_module_specifier(
                None,
                &specifier,
                referrer.path(),
                &mut context.borrow_mut(),
            )?;

            let m = self.0(&p.to_string_lossy())
                .ok_or_else(|| JsNativeError::error().with_message("Module not found"))?;
            let s = Source::from_reader(Cursor::new(m.as_bytes()), Some(&p));

            Module::parse(s, None, &mut context.borrow_mut())
        })();
        async { result }
    }
}
