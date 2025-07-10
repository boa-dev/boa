#![allow(unused_crate_dependencies)]
//! A test that mimics the `boa_engine`'s GCD test with a typed callback.

use boa_engine::object::builtins::JsFunction;
use boa_engine::{Context, Module, Source, js_string};
use boa_gc::Gc;
use boa_interop::{ContextData, IntoJsFunctionCopied};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

fn callback_from_js(ContextData(r): ContextData<Gc<AtomicUsize>>, result: usize) {
    r.store(result, Ordering::Relaxed);
}

#[test]
fn gcd_callback() {
    let assets_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/assets");

    // Create the engine.
    let context = &mut Context::default();
    let result = Gc::new(AtomicUsize::new(0));
    context.insert_data(result.clone());

    // Load the JavaScript code.
    let gcd_path = assets_dir.join("gcd_callback.js");
    let source = Source::from_filepath(&gcd_path).unwrap();
    let module = Module::parse(source, None, context).unwrap();
    module
        .load_link_evaluate(context)
        .await_blocking(context)
        .unwrap();

    let js_gcd = module
        .get_typed_fn::<(i32, i32, JsFunction), ()>(js_string!("gcd_callback"), context)
        .unwrap();

    let function = callback_from_js
        .into_js_function_copied(context)
        .to_js_function(context.realm());

    result.store(0, Ordering::Relaxed);
    assert_eq!(js_gcd.call(context, (6, 9, function.clone())), Ok(()));
    assert_eq!(result.load(Ordering::Relaxed), 3);

    result.store(0, Ordering::Relaxed);
    assert_eq!(js_gcd.call(context, (9, 6, function)), Ok(()));
    assert_eq!(result.load(Ordering::Relaxed), 3);
}
