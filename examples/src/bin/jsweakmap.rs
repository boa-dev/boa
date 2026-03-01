//! Example demonstrating the `JsWeakMap` API wrapper.
use boa_engine::{Context, js_string, object::builtins::JsWeakMap};

fn main() {
    let mut context = Context::default();

    // Create a new WeakMap
    let weak_map = JsWeakMap::new(&mut context);

    // Create an object to use as a key
    let key = context
        .eval(boa_engine::Source::from_bytes("({})"))
        .unwrap();
    let key_obj = key.as_object().unwrap().clone();

    // Set a value
    weak_map
        .set(&key_obj, js_string!("hello").into(), &mut context)
        .unwrap();

    // Get the value
    let val = weak_map.get(&key_obj, &mut context).unwrap();
    println!("get: {}", val.display());

    // Has the key
    let has = weak_map.has(&key_obj, &mut context).unwrap();
    println!("has: {has}");

    // Delete the key
    let deleted = weak_map.delete(&key_obj, &mut context).unwrap();
    println!("deleted: {deleted}");

    // Has after delete
    let has_after = weak_map.has(&key_obj, &mut context).unwrap();
    println!("has after delete: {has_after}");
}
