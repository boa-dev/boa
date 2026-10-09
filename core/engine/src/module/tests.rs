use std::rc::Rc;

use boa_gc::{WeakGc, force_collect};

use super::{MapModuleLoader, Module};
use crate::{
    Context, JsValue, Source,
    builtins::promise::PromiseState,
    js_string,
    object::{JsObject, WeakJsObject},
};

/// Loads, links and evaluates `module`, and asserts that its evaluation succeeded.
#[track_caller]
fn evaluate(module: &Module, context: &mut Context) {
    let promise = module.load_link_evaluate(context);
    context.run_jobs().unwrap();
    assert_eq!(
        promise.state(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
}

/// Evaluates `module`, creates its namespace, drops both, and asserts that a collection frees them.
#[track_caller]
fn assert_collected_with_namespace(module: Module, context: &mut Context, description: &str) {
    evaluate(&module, context);

    let namespace = module.namespace(context);
    let module_ref = WeakGc::new(&module.inner);
    let namespace_ref = WeakJsObject::new(&namespace);
    drop((module, namespace));

    force_collect();

    assert!(
        !module_ref.is_upgradable(),
        "{description:?}: the module was not collected"
    );
    assert!(
        !namespace_ref.is_upgradable(),
        "{description:?}: the namespace was not collected"
    );
}

/// A module whose namespace was created is collected, together with the namespace, once nothing
/// references either of them, whatever the module exports.
#[test]
fn unreachable_module_with_namespace_is_collected() {
    for source in [
        "",
        "export const x = 1;",
        "export function f() {}",
        "export default 1;",
    ] {
        let context = &mut Context::default();
        let module = Module::parse(Source::from_bytes(source), None, context).unwrap();
        assert_collected_with_namespace(module, context, source);
    }

    let context = &mut Context::default();
    let json = js_string!(r#"{ "x": 1 }"#);
    let module = Module::parse_json(json.clone(), context).unwrap();
    assert_collected_with_namespace(module, context, &json.to_std_string_escaped());
}

/// Evaluates `source`, which re-exports from a module named `dependency`, and asserts that the
/// namespace of `source` keeps both modules alive and usable across a collection, and that a
/// collection frees them once the namespace is unreachable.
///
/// `dependency_exports` returns the object, reached from the namespace, that exposes the
/// dependency's `count` and `increment` exports.
#[track_caller]
fn assert_namespace_keeps_its_modules_alive(
    source: &str,
    dependency_exports: fn(&JsObject, &mut Context) -> JsObject,
) {
    let loader = Rc::new(MapModuleLoader::new());
    let context = &mut Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let dependency = Module::parse(
        Source::from_bytes("export let count = 0; export function increment() { return ++count; }"),
        None,
        context,
    )
    .unwrap();
    let module = Module::parse(Source::from_bytes(source), None, context).unwrap();
    loader.insert("dependency", dependency.clone());
    evaluate(&module, context);

    let namespace = module.namespace(context);
    let module_ref = WeakGc::new(&module.inner);
    let dependency_ref = WeakGc::new(&dependency.inner);
    let namespace_ref = WeakJsObject::new(&namespace);

    // Leave the namespace as the only path to both modules.
    loader.clear();
    drop((module, dependency));

    force_collect();

    assert!(
        module_ref.is_upgradable(),
        "{source:?}: the module was collected"
    );
    assert!(
        dependency_ref.is_upgradable(),
        "{source:?}: the dependency was collected"
    );

    let exports = dependency_exports(&namespace, context);
    let exports_ref = WeakJsObject::new(&exports);
    {
        let increment = exports.get(js_string!("increment"), context).unwrap();
        let result = increment
            .as_callable()
            .unwrap()
            .call(&JsValue::undefined(), &[], context)
            .unwrap();
        assert_eq!(result, JsValue::from(1));
    }
    assert_eq!(
        exports.get(js_string!("count"), context).unwrap(),
        JsValue::from(1)
    );

    drop((namespace, exports));

    force_collect();

    assert!(
        !module_ref.is_upgradable(),
        "{source:?}: the module was not collected"
    );
    assert!(
        !dependency_ref.is_upgradable(),
        "{source:?}: the dependency was not collected"
    );
    assert!(
        !namespace_ref.is_upgradable(),
        "{source:?}: the namespace was not collected"
    );
    assert!(
        !exports_ref.is_upgradable(),
        "{source:?}: the object exposing the dependency's exports was not collected"
    );
}

/// A module namespace keeps its module, and the modules its exports resolve to, alive and usable
/// across collections, and they are all collected once the namespace is unreachable.
#[test]
fn module_namespace_keeps_its_modules_alive_until_unreachable() {
    // Named re-exports resolve to single bindings of the dependency.
    assert_namespace_keeps_its_modules_alive(
        r#"export { count, increment } from "dependency";"#,
        |namespace, _| namespace.clone(),
    );

    // A namespace re-export resolves to the whole namespace of the dependency.
    assert_namespace_keeps_its_modules_alive(
        r#"export * as ns from "dependency";"#,
        |namespace, context| {
            namespace
                .get(js_string!("ns"), context)
                .unwrap()
                .as_object()
                .unwrap()
        },
    );
}
