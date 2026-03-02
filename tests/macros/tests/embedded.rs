//! Tests for the embedded module loader.

#![allow(unused_crate_dependencies)]

use std::rc::Rc;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::embed_module;
use boa_engine::module::embedded::EmbeddedModuleLoader;
use boa_engine::{Context, JsString, JsValue, Module, Source, js_string};

fn load_module_and_test(module_loader: &Rc<EmbeddedModuleLoader>) {
    let context = Context::builder()
        .module_loader(module_loader.clone())
        .build()
        .unwrap();

    // Resolving modules that exist but haven't been cached yet should return None.
    assert_eq!(module_loader.get_module(&JsString::from("/file1.js")), None);
    assert_eq!(
        module_loader.get_module(&JsString::from("/non-existent.js")),
        None
    );

    let module = Module::parse(
        Source::from_bytes(b"export { bar } from '/file1.js';"),
        None,
        &context,
    )
    .expect("failed to parse module");
    let promise = module.load_link_evaluate(&context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Fulfilled(value) => {
            assert!(
                value.is_undefined(),
                "Expected undefined, got {}",
                value.display()
            );

            let bar = module
                .namespace(&context)
                .get(js_string!("bar"), &context)
                .unwrap()
                .as_callable()
                .unwrap();
            let value = bar.call(&JsValue::undefined(), &[], &context).unwrap();
            assert_eq!(
                value.as_number(),
                Some(6.),
                "Expected 6, got {}",
                value.display()
            );
        }
        PromiseState::Rejected(err) => {
            panic!("promise was not fulfilled: {:?}", err.to_string(&context))
        }
        PromiseState::Pending => panic!("Promise was not settled"),
    }
}

#[test]
fn simple() {
    #[cfg(target_family = "unix")]
    let module_loader = Rc::new(embed_module!("tests/embedded/", compress = "none"));
    #[cfg(target_family = "windows")]
    let module_loader = Rc::new(embed_module!("tests\\embedded\\"));

    load_module_and_test(&module_loader);
}

#[test]
fn compressed_lz4() {
    #[cfg(target_family = "unix")]
    let module_loader = Rc::new(embed_module!("tests/embedded/", compress = "lz4"));
    #[cfg(target_family = "windows")]
    let module_loader = Rc::new(embed_module!("tests\\embedded\\", compress = "lz4"));

    load_module_and_test(&module_loader);
}
