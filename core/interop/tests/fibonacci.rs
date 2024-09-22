#![allow(unused_crate_dependencies)]
//! A test that goes back and forth between JavaScript and Rust.

// You can execute this example with `cargo run --example gcd`

use boa_engine::object::builtins::{JsFunction, TypedJsFunction};
use boa_engine::{js_error, js_str, Context, JsResult, Module, Source};
use boa_interop::IntoJsFunctionCopied;
use std::path::PathBuf;

#[allow(clippy::needless_pass_by_value)]
fn fibonacci(
    a: usize,
    cb_a: TypedJsFunction<(usize, JsFunction, JsFunction), usize>,
    cb_b: TypedJsFunction<(usize, JsFunction, JsFunction), usize>,
    context: &mut Context,
) -> JsResult<usize> {
    if a <= 1 {
        Ok(a)
    } else {
        Ok(
            cb_a.call(context, (a - 1, cb_b.clone().into(), cb_a.clone().into()))?
                + cb_b.call(context, (a - 2, cb_b.clone().into(), cb_a.clone().into()))?,
        )
    }
}

fn fibonacci_throw(
    a: usize,
    cb_a: TypedJsFunction<(usize, JsFunction, JsFunction), usize>,
    cb_b: TypedJsFunction<(usize, JsFunction, JsFunction), usize>,
    context: &mut Context,
) -> JsResult<usize> {
    if a < 5 {
        Err(js_error!("a is too small"))
    } else {
        fibonacci(a, cb_a, cb_b, context)
    }
}

#[test]
fn fibonacci_test() {
    let assets_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/assets");

    // Create the engine.
    let context = &mut Context::default();

    // Load the JavaScript code.
    let gcd_path = assets_dir.join("fibonacci.js");
    let source = Source::from_filepath(&gcd_path).unwrap();
    let module = Module::parse(source, None, context).unwrap();
    module
        .load_link_evaluate(context)
        .await_blocking(context)
        .unwrap();

    let fibonacci_js = module
        .get_typed_fn::<(usize, JsFunction, JsFunction), usize>(js_str!("fibonacci"), context)
        .unwrap();

    let fibonacci_rust = fibonacci
        .into_js_function_copied(context)
        .to_js_function(context.realm());

    assert_eq!(
        fibonacci_js
            .call(
                context,
                (
                    10,
                    fibonacci_rust.clone(),
                    fibonacci_js.as_js_function().clone()
                )
            )
            .unwrap(),
        55
    );

    let fibonacci_throw = fibonacci_throw
        .into_js_function_copied(context)
        .to_js_function(context.realm());
    assert_eq!(
        fibonacci_js
            .call(
                context,
                (
                    10,
                    fibonacci_throw.clone(),
                    fibonacci_js.as_js_function().clone()
                )
            )
            .unwrap_err()
            .to_string(),
        "\"a is too small\""
    );
}
