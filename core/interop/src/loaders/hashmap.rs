use std::collections::HashMap;

use boa_engine::{Context, JsNativeError, JsResult, JsString, Module};
use boa_engine::module::{ModuleLoader, Referrer};
use boa_gc::GcRefCell;

/// A ModuleLoader that loads modules from a HashMap based on the name.
/// After registering modules, this loader will look for the exact name
/// in its internal map to resolve.
#[derive(Debug, Clone)]
pub struct HashMapModuleLoader(GcRefCell<HashMap<JsString, Module>>);

impl HashMapModuleLoader {
    pub fn new() -> Self {
        Self(GcRefCell::new(HashMap::new()))
    }

    pub fn register(&self, key: impl Into<JsString>, value: Module) {
        self.0.borrow_mut().insert(key.into(), value);
    }
}

impl ModuleLoader for HashMapModuleLoader {
    fn load_imported_module(
        &self,
        _referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        // First, try to resolve from our internal cached.
        if let Some(module) = self.0.borrow().get(&specifier) {
            finish_load(Ok(module.clone()), context);
        } else {
            let err = JsNativeError::typ().with_message(format!(
                "could not find module `{}`",
                specifier.to_std_string_escaped()
            ));
            finish_load(Err(err.into()), context);
        }
    }

    fn get_module(&self, specifier: JsString) -> Option<Module> {
        self.0.borrow().get(&specifier).cloned()
    }
}
