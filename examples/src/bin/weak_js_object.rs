//! Example demonstrating the `WeakJsObject` API: an embedder-held weak reference to a `JsObject`
//! that does not keep the referenced object alive across garbage collections.
use boa_engine::{Context, Source, js_string, object::WeakJsObject};
use boa_gc::force_collect;
use std::collections::HashMap;

fn main() {
    let mut context = Context::default();

    // Create two independent JS objects. For now we hold strong handles to both.
    let first = context
        .eval(Source::from_bytes("({ name: 'first' })"))
        .unwrap()
        .as_object()
        .unwrap()
        .clone();
    let second = context
        .eval(Source::from_bytes("({ name: 'second' })"))
        .unwrap()
        .as_object()
        .unwrap()
        .clone();

    // A Rust-side registry that remembers objects by an embedder-chosen id *without* keeping them
    // alive. This is the shape of use case `WeakJsObject` exists for: for example, mapping host DOM
    // nodes to their JS wrappers so that the same node keeps yielding the same object, while still
    // letting the engine collect a wrapper once nothing else references it.
    let mut registry: HashMap<u32, WeakJsObject> = HashMap::new();
    registry.insert(1, WeakJsObject::new(&first));
    registry.insert(2, WeakJsObject::new(&second));

    // While the strong handles live, the registry can hand the objects back.
    println!("id 1 upgradable? {}", registry[&1].is_upgradable());
    println!("id 2 upgradable? {}", registry[&2].is_upgradable());
    println!("debug of id 1: {:?}", registry[&1]);

    // Drop the strong handle to the first object and run a collection. Its registry entry can no
    // longer be upgraded, and the weak entry never kept the object alive in the first place.
    drop(first);
    force_collect();

    println!("after dropping `first` and collecting:");
    println!("  id 1 upgradable? {}", registry[&1].is_upgradable());
    println!("  id 2 upgradable? {}", registry[&2].is_upgradable());

    // The surviving object still round-trips through its weak reference. Upgrading yields a strong
    // handle, which keeps the object alive for as long as it is held.
    let recovered = registry[&2].upgrade().expect("`second` is still alive");
    let name = recovered.get(js_string!("name"), &mut context).unwrap();
    println!("  id 2 name = {}", name.display());

    // Drop the last strong handles (the original and the upgraded one); now nothing is upgradable.
    drop(second);
    drop(recovered);
    force_collect();
    println!(
        "after dropping everything: id 2 upgradable? {}",
        registry[&2].is_upgradable()
    );
}
