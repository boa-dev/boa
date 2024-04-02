#![allow(unused_crate_dependencies, missing_docs)]

use std::path::PathBuf;
use std::rc::Rc;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::SimpleModuleLoader;
use boa_engine::{js_string, Context, JsValue, Source};

/// Test that relative imports work with the simple module loader.
#[test]
fn subdirectories() {
    let assets_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/assets");

    let loader = Rc::new(SimpleModuleLoader::new(assets_dir).unwrap());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let source = Source::from_bytes(b"import { file1 } from './file1.js'; file1()");
    let module = boa_engine::Module::parse(source, None, &mut context).unwrap();
    let result = module.load_link_evaluate(&mut context);

    context.run_jobs();
    match result.state() {
        PromiseState::Pending => {}
        PromiseState::Fulfilled(v) => {
            assert_eq!(v, JsValue::String(js_string!("file1..file1_1.file1_2")));
        }
        PromiseState::Rejected(reason) => {
            panic!("Module failed to load: {}", reason.display());
        }
    }
}
