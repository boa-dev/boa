//! Example demonstrating the `JsWeakSet` API wrapper.
use boa_engine::{Context, Source, object::builtins::JsWeakSet};

fn main() {
    let mut context = Context::default();

    // Create a new WeakSet
    let weak_set = JsWeakSet::new(&mut context);

    // Create an object to use as a value
    let val = context.eval(Source::from_bytes("({})")).unwrap();
    let val_obj = val.as_object().unwrap().clone();

    // Add the object
    weak_set.add(&val_obj, &mut context).unwrap();

    // Has the object
    let has = weak_set.has(&val_obj, &mut context).unwrap();
    println!("has: {has}");

    // Delete the object
    let deleted = weak_set.delete(&val_obj, &mut context).unwrap();
    println!("deleted: {deleted}");

    // Has after delete
    let has_after = weak_set.has(&val_obj, &mut context).unwrap();
    println!("has after delete: {has_after}");
}
