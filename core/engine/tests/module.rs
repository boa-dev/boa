#![allow(unused_crate_dependencies, missing_docs)]

use std::cell::RefCell;
use std::future;
use std::rc::Rc;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::{MapModuleLoader, ModuleLoader, Referrer};
use boa_engine::{
    Context, JsNativeError, JsNativeErrorKind, JsResult, JsString, JsValue, Module, Source,
    js_string,
};

#[test]
fn test_json_module_from_str() {
    struct TestModuleLoader(JsString);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: boa_engine::module::ModuleRequest,
            context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");
            let src = self.0.clone();

            future::ready(Ok(
                Module::parse_json(src, &mut context.borrow_mut()).unwrap()
            ))
        }
    }

    let json_string = js_string!(r#"{"key":"value","other":123}"#);
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_string.clone())))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        import basic_json from 'basic';
        export let json = basic_json;
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Pending => {}
        PromiseState::Fulfilled(v) => {
            assert!(v.is_undefined());
        }
        PromiseState::Rejected(e) => {
            panic!("Unexpected error: {:?}", e.to_string(&mut context).unwrap());
        }
    }

    let json = module
        .namespace(&mut context)
        .get(js_string!("json"), &mut context)
        .unwrap();

    assert_eq!(
        JsString::from(json.to_json(&mut context).unwrap().unwrap().to_string()),
        json_string
    );
}

#[test]
fn test_json_module_dynamic_import() {
    struct TestModuleLoader(JsString);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: boa_engine::module::ModuleRequest,
            context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            // Verify attributes were passed correctly
            let type_attr = request
                .get_attribute("type")
                .expect("should have type attribute");
            assert_eq!(type_attr.to_std_string_escaped(), "json");

            let src = self.0.clone();
            future::ready(Ok(
                Module::parse_json(src, &mut context.borrow_mut()).unwrap()
            ))
        }
    }

    let json_content = js_string!(r#"{"key":"value","other":123}"#);
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_content.clone())))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        export let p = import('basic', { with: { type: 'json' } });
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    // Get the exported promise 'p'
    let p = module
        .namespace(&mut context)
        .get(js_string!("p"), &mut context)
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state() {
        PromiseState::Fulfilled(module_ns) => {
            let default_export = module_ns
                .as_object()
                .unwrap()
                .get(js_string!("default"), &mut context)
                .unwrap();

            assert_eq!(
                JsString::from(
                    default_export
                        .to_json(&mut context)
                        .unwrap()
                        .unwrap()
                        .to_string()
                ),
                json_content
            );
        }
        PromiseState::Rejected(e) => {
            panic!(
                "Dynamic import failed: {:?}",
                e.to_string(&mut context).unwrap()
            );
        }
        PromiseState::Pending => panic!("Dynamic import is still pending"),
    }
}

#[test]
fn test_json_module_static_import_with_attributes() {
    struct TestModuleLoader(JsString);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: boa_engine::module::ModuleRequest,
            context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            let type_attr = request
                .get_attribute("type")
                .expect("should have type attribute");
            assert_eq!(type_attr.to_std_string_escaped(), "json");

            let src = self.0.clone();
            future::ready(Ok(
                Module::parse_json(src, &mut context.borrow_mut()).unwrap()
            ))
        }
    }

    let json_string = js_string!(r#"{"static":"import"}"#);
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_string.clone())))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        import json from 'basic' with { type: 'json' };
        export let value = json;
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    assert_eq!(
        promise.state(),
        PromiseState::Fulfilled(JsValue::undefined())
    );

    let value = module
        .namespace(&mut context)
        .get(js_string!("value"), &mut context)
        .unwrap();

    assert_eq!(
        JsString::from(value.to_json(&mut context).unwrap().unwrap().to_string()),
        json_string
    );
}

#[test]
fn test_json_module_reexport_with_attributes() {
    struct TestModuleLoader(JsString);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: boa_engine::module::ModuleRequest,
            context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            let type_attr = request
                .get_attribute("type")
                .expect("should have type attribute");
            assert_eq!(type_attr.to_std_string_escaped(), "json");

            let src = self.0.clone();
            future::ready(Ok(
                Module::parse_json(src, &mut context.borrow_mut()).unwrap()
            ))
        }
    }

    let json_string = js_string!(r#"{"re":"export"}"#);
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_string.clone())))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        export { default as json } from 'basic' with { type: 'json' };
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    assert_eq!(
        promise.state(),
        PromiseState::Fulfilled(JsValue::undefined())
    );

    let json = module
        .namespace(&mut context)
        .get(js_string!("json"), &mut context)
        .unwrap();

    assert_eq!(
        JsString::from(json.to_json(&mut context).unwrap().unwrap().to_string()),
        json_string
    );
}

#[test]
fn test_dynamic_import_invalid_options() {
    struct TestModuleLoader;
    impl ModuleLoader for TestModuleLoader {
        async fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            _request: boa_engine::module::ModuleRequest,
            _context: &RefCell<&mut Context>,
        ) -> JsResult<Module> {
            panic!("Module loading should not be triggered for invalid options");
        }
    }

    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        export let p = import('basic', 'invalid-option-string');
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    // Get the exported promise 'p'
    let p = module
        .namespace(&mut context)
        .get(js_string!("p"), &mut context)
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state() {
        PromiseState::Rejected(e) => {
            let error = e.as_object().unwrap();
            let name = error.get(js_string!("name"), &mut context).unwrap();
            assert_eq!(name.as_string().unwrap(), js_string!("TypeError"));
        }
        state => panic!("Dynamic import should be rejected with TypeError, got {state:?}"),
    }
}

#[test]
fn test_dynamic_import_non_string_attribute_value() {
    struct TestModuleLoader;
    impl ModuleLoader for TestModuleLoader {
        async fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            _request: boa_engine::module::ModuleRequest,
            _context: &RefCell<&mut Context>,
        ) -> JsResult<Module> {
            panic!("Module loading should not be triggered for invalid attribute values");
        }
    }

    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        export let p = import('basic', { with: { type: 123 } });
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    let p = module
        .namespace(&mut context)
        .get(js_string!("p"), &mut context)
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state() {
        PromiseState::Rejected(e) => {
            let error = e.as_object().unwrap();
            let name = error.get(js_string!("name"), &mut context).unwrap();
            assert_eq!(name.as_string().unwrap(), js_string!("TypeError"));
            let message = error.get(js_string!("message"), &mut context).unwrap();
            assert_eq!(
                message.as_string().unwrap(),
                js_string!("import attribute value must be a string")
            );
        }
        state => panic!("Dynamic import should be rejected with TypeError, got {state:?}"),
    }
}

#[test]
fn test_dynamic_import_symbol_key() {
    struct TestModuleLoader(JsString);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: boa_engine::module::ModuleRequest,
            context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            // Verify attributes were passed correctly (symbol key should be ignored)
            assert!(request.get_attribute("type").is_none());

            let src = self.0.clone();
            future::ready(Ok(
                Module::parse_json(src, &mut context.borrow_mut()).unwrap()
            ))
        }
    }

    let json_content = js_string!(r#"{"ignore":"symbol"}"#);
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_content.clone())))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        let sym = Symbol('type');
        export let p = import('basic', { with: { [sym]: 'json' } });
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    let p = module
        .namespace(&mut context)
        .get(js_string!("p"), &mut context)
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state() {
        PromiseState::Fulfilled(module_ns) => {
            let default_export = module_ns
                .as_object()
                .unwrap()
                .get(js_string!("default"), &mut context)
                .unwrap();

            assert_eq!(
                JsString::from(
                    default_export
                        .to_json(&mut context)
                        .unwrap()
                        .unwrap()
                        .to_string()
                ),
                json_content
            );
        }
        PromiseState::Rejected(e) => {
            panic!(
                "Dynamic import failed: {:?}",
                e.to_string(&mut context).unwrap()
            );
        }
        PromiseState::Pending => panic!("Dynamic import is still pending"),
    }
}

/// Linking a module initializes its `var` bindings to `undefined`, while its lexical bindings stay
/// uninitialized until its body runs.
#[test]
fn test_module_var_bindings_are_initialized_at_link_time() {
    let mut context = Context::default();
    let module = Module::parse(
        Source::from_bytes("export var x = 1; export let y = 2;"),
        None,
        &mut context,
    )
    .unwrap();

    let promise = module.load(&mut context);
    context.run_jobs().unwrap();
    assert!(promise.state().as_fulfilled().is_some());
    module.link(&mut context).unwrap();

    let namespace = module.namespace(&mut context);
    assert_eq!(
        namespace.get(js_string!("x"), &mut context).unwrap(),
        JsValue::undefined()
    );
    let error = namespace.get(js_string!("y"), &mut context).unwrap_err();
    assert_eq!(
        error.as_native().map(JsNativeError::kind),
        Some(&JsNativeErrorKind::Reference)
    );
}

/// Links and evaluates the module `a`, which imports the module `b`, which imports `a` back, and
/// returns `a`'s export named `result`.
///
/// `b`'s body runs before `a`'s, because `b`'s import of `a` finds `a` already evaluating.
fn evaluate_cycle(a: &str, b: &str) -> JsValue {
    let loader = Rc::new(MapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let a = Module::parse(Source::from_bytes(a), None, &mut context).unwrap();
    let b = Module::parse(Source::from_bytes(b), None, &mut context).unwrap();
    loader.insert("a", a.clone());
    loader.insert("b", b);

    let promise = a.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();
    if let PromiseState::Rejected(reason) = promise.state() {
        panic!("module evaluation failed: {}", reason.display());
    }
    assert_eq!(
        promise.state(),
        PromiseState::Fulfilled(JsValue::undefined())
    );

    a.namespace(&mut context)
        .get(js_string!("result"), &mut context)
        .unwrap()
}

/// A module's `var` bindings are initialized to `undefined` when the module is linked, so code that
/// runs before the module's body reads them as `undefined`.
#[test]
fn test_module_var_bindings_read_before_evaluation_are_undefined() {
    let result = evaluate_cycle(
        r#"
        import { observed } from "b";
        export var exported = 1;
        var local = 2;
        export function readLocal() { return local; }
        export const result = observed;
        "#,
        r#"
        import { exported, readLocal } from "a";
        import * as a from "a";
        export const observed = [exported, a.exported, readLocal()].map(String).join();
        "#,
    );

    assert_eq!(result, js_string!("undefined,undefined,undefined").into());
}

/// A module's body does not reinitialize its `var` bindings, so a value written to one before the
/// body runs is kept.
#[test]
fn test_module_var_binding_written_before_evaluation_keeps_its_value() {
    let result = evaluate_cycle(
        r#"
        import "b";
        export var value;
        export function setValue(v) { value = v; }
        export const result = value;
        "#,
        r#"
        import { setValue } from "a";
        setValue(5);
        "#,
    );

    assert_eq!(result, JsValue::from(5));
}
