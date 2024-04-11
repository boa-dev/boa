#![allow(unused_crate_dependencies, missing_docs)]

use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::{ModuleLoader, Referrer};
use boa_engine::{js_string, Context, JsResult, JsString, Module, Source};
use std::rc::Rc;

#[test]
fn test_json_module_from_str() {
    struct TestModuleLoader(&'static str);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            &self,
            _referrer: Referrer,
            specifier: JsString,
            finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
            context: &mut Context,
        ) {
            assert_eq!(specifier.to_std_string_escaped(), "basic");

            finish_load(
                Ok(Module::from_json_as_default(self.0, context).unwrap()),
                context,
            );
        }
    }

    let json_string = r#"{"key":"value","other":123}"#;
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_string)))
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
    context.run_jobs();

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

    assert_eq!(json.to_json(&mut context).unwrap().to_string(), json_string);
}
