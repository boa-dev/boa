//! A [`ModuleLoader`] implementation that loads JSON file and expose the JSON
//! as JavaScript objects.
#![cfg(feature = "json")]

use std::collections::HashMap;
use std::path::PathBuf;

use boa_engine::module::{
    resolve_module_specifier, ModuleLoader, Referrer, SyntheticModuleInitializer,
};
use boa_engine::{js_string, Context, JsError, JsResult, JsString, JsValue, Module};

#[derive(Debug, Clone)]
pub struct JsonModuleLoader {
    root: PathBuf,
    cache: HashMap<PathBuf, Module>,
}

impl JsonModuleLoader {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            cache: Default::default(),
        }
    }
}

impl ModuleLoader for JsonModuleLoader {
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        let path = match resolve_module_specifier(
            Some(&self.root),
            &specifier,
            referrer.path(),
            context,
        ) {
            Ok(path) => path,
            Err(e) => {
                return finish_load(
                    Err(JsError::from_opaque(
                        js_string!(format!(
                            "Cannot resolve module specifier: {}. Reason: {}",
                            specifier.to_std_string_escaped(),
                            e.to_string()
                        ))
                        .into(),
                    )),
                    context,
                );
            }
        };

        if self.cache.contains_key(&path) {
            return finish_load(Ok(self.cache[&path].clone()), context);
        }

        if !path.exists() {
            return finish_load(
                Err(JsError::from_opaque(
                    js_string!(format!("Module not found: {}", path.display())).into(),
                )),
                context,
            );
        }

        let json_string = std::fs::read_to_string(&path).unwrap();
        let module = Module::synthetic(
            &[js_string!("default")],
            SyntheticModuleInitializer::from_copy_closure_with_captures(
                |module, json_string, context| {
                    let json = serde_json::from_str::<serde_json::Value>(json_string).unwrap();
                    let value = JsValue::from_json(&json, context)?;

                    module.set_export(&js_string!("default"), value)?;
                    Ok(())
                },
                json_string,
            ),
            Some(path),
            None,
            context,
        );

        finish_load(Ok(module), context);
    }
}
