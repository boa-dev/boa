#![allow(unused_crate_dependencies, missing_docs)]

use std::path::PathBuf;
use std::rc::Rc;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::SimpleModuleLoader;
use boa_engine::{Context, JsValue, Source, js_string};

/// Test that relative imports work with the simple module loader.
#[test]
fn subdirectories() {
    let assets_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/assets");

    let loader = Rc::new(SimpleModuleLoader::new(assets_dir).unwrap());
    let context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let source = Source::from_bytes(b"export { file1 } from 'file1.js';");
    let module = boa_engine::Module::parse(source, None, &context).unwrap();
    let result = module.load_link_evaluate(&context);

    context.run_jobs().unwrap();
    match result.state() {
        PromiseState::Pending => {}
        PromiseState::Fulfilled(v) => {
            assert!(v.is_undefined());

            let foo_value = module
                .namespace(&context)
                .get(js_string!("file1"), &context)
                .unwrap()
                .as_callable()
                .unwrap()
                .call(&JsValue::undefined(), &[], &context)
                .unwrap();

            assert_eq!(foo_value, js_string!("file1..file1_1.file1_2").into());
        }
        PromiseState::Rejected(reason) => {
            panic!("Module failed to load: {}", reason.display());
        }
    }
}
