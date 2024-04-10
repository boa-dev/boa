//! Functions to create [`Module`]s that expose JSON data directly
//! as the default export.
#![allow(clippy::module_name_repetitions)]

use boa_engine::{js_string, Context, JsValue, Module};

/// Create a module that exports a single JSON value as the default export, from its
/// JSON string. This required the `json` feature to be enabled.
///
/// # Errors
/// This will return an error if the JSON string is invalid or cannot be converted.
#[cfg(feature = "json")]
pub fn json_string_module(json: &str, context: &mut Context) -> boa_engine::JsResult<Module> {
    let json_value = serde_json::from_str::<serde_json::Value>(json).map_err(|e| {
        boa_engine::JsError::from_opaque(js_string!(format!("Failed to parse JSON: {}", e)).into())
    })?;
    let value: JsValue = JsValue::from_json(&json_value, context)?;
    Ok(Module::from_value_as_default(value, context))
}

#[cfg(feature = "json")]
#[test]
fn test_json_module_from_str() {
    use crate::loaders::HashMapModuleLoader;
    use boa_engine::builtins::promise::PromiseState;
    use boa_engine::Source;
    use std::rc::Rc;

    let loader = Rc::new(HashMapModuleLoader::new());
    let mut context = boa_engine::Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let json_string = r#"{"key":"value","other":123}"#;
    loader.register(
        "basic",
        json_string_module(json_string, &mut context).unwrap(),
    );

    let source = Source::from_bytes(
        b"
        import basic_json from 'basic';
        export let json = basic_json;
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs();

    match promise.state() {
        PromiseState::Pending => {}
        PromiseState::Fulfilled(v) => {
            assert!(v.is_undefined());
        }
        PromiseState::Rejected(e) => {
            panic!("Unexpected error: {:?}", e.to_string(&mut context).unwrap());
        }
    }

    let json = module
        .namespace(&mut context)
        .get(js_string!("json"), &mut context)
        .unwrap();

    assert_eq!(json.to_json(&mut context).unwrap().to_string(), json_string);
}
