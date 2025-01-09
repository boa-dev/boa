//! Tests for the embedded module loader.

#![allow(unused_crate_dependencies)]

use std::rc::Rc;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::ModuleLoader;
use boa_engine::{js_string, Context, JsString, JsValue, Module, Source};
use boa_interop::embed_module;

#[test]
fn simple() {
    #[cfg(target_family = "unix")]
    let module_loader = Rc::new(embed_module!("tests/embedded/"));
    #[cfg(target_family = "windows")]
    let module_loader = Rc::new(embed_module!("tests\\embedded\\"));

    let mut context = Context::builder()
        .module_loader(module_loader.clone())
        .build()
        .unwrap();

    // Resolving modules that exist but haven't been cached yet should return None.
    assert_eq!(module_loader.get_module(JsString::from("/file1.js")), None);
    assert_eq!(
        module_loader.get_module(JsString::from("/non-existent.js")),
        None
    );

    let module = Module::parse(
        Source::from_bytes(b"export { bar } from '/file1.js';"),
        None,
        &mut context,
    )
    .expect("failed to parse module");
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs();

    match promise.state() {
        PromiseState::Fulfilled(value) => {
            assert!(
                value.is_undefined(),
                "Expected undefined, got {}",
                value.display()
            );

            let bar = module
                .namespace(&mut context)
                .get(js_string!("bar"), &mut context)
                .unwrap()
                .as_callable()
                .cloned()
                .unwrap();
            let value = bar.call(&JsValue::undefined(), &[], &mut context).unwrap();
            assert_eq!(
                value.as_number(),
                Some(6.),
                "Expected 6, got {}",
                value.display()
            );
        }
        PromiseState::Rejected(err) => panic!(
            "promise was not fulfilled: {:?}",
            err.to_string(&mut context)
        ),
        PromiseState::Pending => panic!("Promise was not settled"),
    }
}
