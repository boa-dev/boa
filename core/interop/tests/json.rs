#![allow(unused_crate_dependencies, missing_docs)]
#![cfg(feature = "json")]
use boa_engine::builtins::promise::PromiseState;
use boa_engine::{js_string, Module, Source};
use boa_interop::loaders::json::JsonModuleLoader;
use std::rc::Rc;

#[test]
fn works() {
    let loader = Rc::new(JsonModuleLoader::new(
        std::path::PathBuf::from("tests/assets/json")
            .canonicalize()
            .unwrap(),
    ));

    let mut context = boa_engine::Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        import basic_json from 'basic.json';

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

    assert_eq!(
        json.to_json(&mut context).unwrap().to_string(),
        r#"{"number":123,"test":"boa"}"#
    );
}
