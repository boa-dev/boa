//! A `ModuleLoader` that loads modules from a `HashMap` based on the name.
use std::cell::RefCell;
use std::rc::Rc;

use rustc_hash::FxHashMap;

use boa_engine::module::{ModuleLoader, Referrer};
use boa_engine::{Context, JsNativeError, JsResult, JsString, Module};
use boa_gc::GcRefCell;

/// A `ModuleLoader` that loads modules from a `HashMap` based on the name.
/// After registering modules, this loader will look for the exact name
/// in its internal map to resolve.
#[derive(Debug, Clone)]
pub struct HashMapModuleLoader(GcRefCell<FxHashMap<JsString, Module>>);

impl Default for HashMapModuleLoader {
    fn default() -> Self {
        Self(GcRefCell::new(FxHashMap::default()))
    }
}

impl HashMapModuleLoader {
    /// Creates an empty `HashMapModuleLoader`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a module with a given name.
    pub fn register(&self, key: impl Into<JsString>, value: Module) {
        self.0.borrow_mut().insert(key.into(), value);
    }
}

impl FromIterator<(JsString, Module)> for HashMapModuleLoader {
    fn from_iter<T: IntoIterator<Item = (JsString, Module)>>(iter: T) -> Self {
        let map = iter.into_iter().collect();
        Self(GcRefCell::new(map))
    }
}

impl ModuleLoader for HashMapModuleLoader {
    fn load_imported_module(
        self: Rc<Self>,
        _referrer: Referrer,
        specifier: JsString,
        _context: &RefCell<&mut Context>,
    ) -> impl Future<Output = JsResult<Module>> {
        let result = self.0.borrow().get(&specifier).cloned().ok_or_else(|| {
            JsNativeError::typ()
                .with_message(format!(
                    "could not find module `{}`",
                    specifier.display_escaped()
                ))
                .into()
        });
        async { result }
    }
}
