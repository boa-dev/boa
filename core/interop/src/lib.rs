//! Interop utilities between Boa and its host.

pub use boa_engine;
pub use boa_macros;

pub mod loaders;
pub mod macros;

// Re-export in case some people depend on boa_interop.
#[deprecated(note = "Please use these exports from boa_engine::interop instead.")]
pub use boa_engine::interop::{ContextData, Ignore, JsClass, JsRest};

#[deprecated(note = "Please use these exports from boa_engine instead.")]
pub use boa_engine::{IntoJsFunctionCopied, IntoJsModule, UnsafeIntoJsFunction};

#[test]
#[allow(clippy::missing_panics_doc)]
fn into_js_module() {
    use boa_engine::{Context, JsValue, Module, Source, js_string};
    use boa_gc::{Gc, GcRefCell};
    use std::cell::RefCell;
    use std::rc::Rc;

    type ResultType = Gc<GcRefCell<JsValue>>;

    let loader = Rc::new(loaders::HashMapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let foo_count = Rc::new(RefCell::new(0));
    let bar_count = Rc::new(RefCell::new(0));
    let dad_count = Rc::new(RefCell::new(0));

    context.insert_data(Gc::new(GcRefCell::new(JsValue::undefined())));

    let module = unsafe {
        vec![
            (
                js_string!("foo"),
                {
                    let counter = foo_count.clone();
                    move || {
                        *counter.borrow_mut() += 1;
                        let result = *counter.borrow();
                        result
                    }
                }
                .into_js_function_unsafe(&mut context),
            ),
            (
                js_string!("bar"),
                UnsafeIntoJsFunction::into_js_function_unsafe(
                    {
                        let counter = bar_count.clone();
                        move |i: i32| {
                            *counter.borrow_mut() += i;
                        }
                    },
                    &mut context,
                ),
            ),
            (
                js_string!("dad"),
                UnsafeIntoJsFunction::into_js_function_unsafe(
                    {
                        let counter = dad_count.clone();
                        move |args: JsRest<'_>, context: &mut Context| {
                            *counter.borrow_mut() += args
                                .into_iter()
                                .map(|i| i.try_js_into::<i32>(context).unwrap())
                                .sum::<i32>();
                        }
                    },
                    &mut context,
                ),
            ),
            (
                js_string!("send"),
                (move |value: JsValue, ContextData(result): ContextData<ResultType>| {
                    *result.borrow_mut() = value;
                })
                .into_js_function_copied(&mut context),
            ),
        ]
    }
    .into_js_module(&mut context);

    loader.register(js_string!("test"), module);

    let source = Source::from_bytes(
        r"
            import * as test from 'test';
            let result = test.foo();
            test.foo();
            for (let i = 1; i <= 5; i++) {
                test.bar(i);
            }
            for (let i = 1; i < 5; i++) {
                test.dad(1, 2, 3);
            }

            test.send(result);
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    // Checking if the final promise didn't return an error.
    assert!(
        promise_result.state().as_fulfilled().is_some(),
        "module didn't execute successfully! Promise: {:?}",
        promise_result.state()
    );

    let result = context.get_data::<ResultType>().unwrap().borrow().clone();

    assert_eq!(*foo_count.borrow(), 2);
    assert_eq!(*bar_count.borrow(), 15);
    assert_eq!(*dad_count.borrow(), 24);
    assert_eq!(result.try_js_into(&mut context), Ok(1u32));
}

#[test]
fn can_throw_exception() {
    use boa_engine::{Context, JsError, JsResult, JsValue, Module, Source, js_string};
    use std::rc::Rc;

    let loader = Rc::new(loaders::HashMapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let module = vec![(
        js_string!("doTheThrow"),
        IntoJsFunctionCopied::into_js_function_copied(
            |message: JsValue| -> JsResult<()> { Err(JsError::from_opaque(message)) },
            &mut context,
        ),
    )]
    .into_js_module(&mut context);

    loader.register(js_string!("test"), module);

    let source = Source::from_bytes(
        r"
            import * as test from 'test';
            try {
                test.doTheThrow('javascript');
            } catch(e) {
                throw 'from ' + e;
            }
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    // Checking if the final promise didn't return an error.
    assert_eq!(
        promise_result.state().as_rejected(),
        Some(&js_string!("from javascript").into())
    );
}

#[test]
fn class() {
    use boa_engine::class::{Class, ClassBuilder};
    use boa_engine::property::Attribute;
    use boa_engine::{Context, JsResult, JsValue, Module, Source, js_string};
    use boa_macros::{Finalize, JsData, Trace};
    use std::rc::Rc;

    #[derive(Debug, Trace, Finalize, JsData)]
    struct Test {
        value: i32,
    }

    impl Test {
        #[allow(clippy::needless_pass_by_value)]
        fn get_value(this: JsClass<Test>) -> i32 {
            this.borrow().value
        }

        #[allow(clippy::needless_pass_by_value)]
        fn set_value(this: JsClass<Test>, new_value: i32) {
            (*this.borrow_mut()).value = new_value;
        }
    }

    impl Class for Test {
        const NAME: &'static str = "Test";

        fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
            let get_value = Self::get_value.into_js_function_copied(class.context());
            class.method(js_string!("getValue"), 0, get_value);
            let set_value = Self::set_value.into_js_function_copied(class.context());
            class.method(js_string!("setValue"), 1, set_value);

            let get_value_getter = Self::get_value
                .into_js_function_copied(class.context())
                .to_js_function(class.context().realm());
            let set_value_setter = Self::set_value
                .into_js_function_copied(class.context())
                .to_js_function(class.context().realm());
            class.accessor(
                js_string!("value_get"),
                Some(get_value_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            );
            class.accessor(
                js_string!("value_set"),
                None,
                Some(set_value_setter),
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            );

            Ok(())
        }

        fn data_constructor(
            _new_target: &JsValue,
            _args: &[JsValue],
            _context: &mut Context,
        ) -> JsResult<Self> {
            Ok(Self { value: 123 })
        }
    }

    let loader = Rc::new(loaders::HashMapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    context.register_global_class::<Test>().unwrap();

    let source = Source::from_bytes(
        r"
            let t = new Test();
            if (t.getValue() != 123) {
                throw 'invalid value';
            }
            t.setValue(456);
            if (t.getValue() != 456) {
                throw 'invalid value 456';
            }
            if (t.value_get != 456) {
                throw 'invalid value 456';
            }
            t.value_set = 789;
            if (t.getValue() != 789) {
                throw 'invalid value 789';
            }
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    // Checking if the final promise didn't return an error.
    assert!(
        promise_result.state().as_fulfilled().is_some(),
        "module didn't execute successfully! Promise: {:?}",
        promise_result.state()
    );
}
