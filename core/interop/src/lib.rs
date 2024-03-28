use std::cell::RefCell;

use boa_engine::{Context, JsString, JsValue, Module, NativeFunction, Source};
use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::SyntheticModuleInitializer;

pub mod loaders;

pub trait IntoJsModule {
    fn into_js_module(self, context: &mut Context) -> Module;
}

impl<T: IntoIterator<Item = (JsString, NativeFunction)> + Clone> IntoJsModule for T {
    fn into_js_module(self, context: &mut Context) -> Module {
        let (names, fns): (Vec<_>, Vec<_>) = self.into_iter().unzip();
        let exports = names.clone();

        Module::synthetic(
            exports.as_slice(),
            unsafe {
                SyntheticModuleInitializer::from_closure(move |module, context| {
                    for (name, f) in names.iter().zip(fns.iter()) {
                        module
                            .set_export(name, f.clone().to_js_function(context.realm()).into())?;
                    }
                    Ok(())
                })
            },
            None,
            context,
        )
    }
}

pub trait IntoJsFunction {
    fn into_js_function(self, context: &mut Context) -> NativeFunction;
}

impl<T: FnMut() -> () + 'static> IntoJsFunction for T {
    fn into_js_function(self, _context: &mut Context) -> NativeFunction {
        let s = RefCell::new(self);

        unsafe {
            NativeFunction::from_closure(move |_, _, _| {
                s.borrow_mut()();
                Ok(JsValue::undefined())
            })
        }
    }
}

#[test]
pub fn into_js_module() {
    use boa_engine::{js_string, JsValue};
    use std::rc::Rc;

    let loader = Rc::new(loaders::HashMapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let foo_count = Rc::new(RefCell::new(0));
    let bar_count = Rc::new(RefCell::new(0));
    let module = vec![
        (
            js_string!("foo"),
            IntoJsFunction::into_js_function(
                {
                    let foo_count = foo_count.clone();
                    move || {
                        *foo_count.borrow_mut() += 1;
                    }
                },
                &mut context,
            ),
        ),
        (
            js_string!("bar"),
            IntoJsFunction::into_js_function(
                {
                    let bar_count = bar_count.clone();
                    move || {
                        *bar_count.borrow_mut() += 1;
                    }
                },
                &mut context,
            ),
        ),
    ]
    .into_js_module(&mut context);

    loader.register(js_string!("test"), module);

    let source = Source::from_bytes(
        r"
            import * as test from 'test';
            let result = test.foo();
            for (let i = 0; i < 10; i++) {
                test.bar();
            }

            result
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs();

    // Checking if the final promise didn't return an error.
    let PromiseState::Fulfilled(v) = promise_result.state() else {
        panic!("module didn't execute successfully!")
    };

    assert_eq!(*foo_count.borrow(), 1);
    assert_eq!(*bar_count.borrow(), 10);
    assert_eq!(v, JsValue::undefined());
}
