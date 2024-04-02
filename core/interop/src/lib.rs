//! Interop utilities between Boa and its host.

use std::cell::RefCell;

use boa_engine::module::SyntheticModuleInitializer;
use boa_engine::{Context, JsString, JsValue, Module, NativeFunction};
pub use boa_macros;

pub mod loaders;

/// A trait to convert a type into a JS module.
pub trait IntoJsModule {
    /// Converts the type into a JS module.
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

/// A trait to convert a type into a JS function.
/// This trait does not require the implementing type to be `Copy`, which
/// can lead to undefined behaviour if it contains Garbage Collected objects.
///
/// # Safety
/// For this trait to be implemented safely, the implementing type must not contain any
/// garbage collected objects (from [`boa_gc`]).
pub unsafe trait IntoJsFunctionUnsafe {
    /// Converts the type into a JS function.
    ///
    /// # Safety
    /// This function is unsafe to ensure the callee knows the risks of using this trait.
    /// The implementing type must not contain any garbage collected objects.
    unsafe fn into_js_function(self, context: &mut Context) -> NativeFunction;
}

unsafe impl<T: FnMut() + 'static> IntoJsFunctionUnsafe for T {
    unsafe fn into_js_function(self, _context: &mut Context) -> NativeFunction {
        let cell = RefCell::new(self);
        unsafe {
            NativeFunction::from_closure(move |_, _, _| {
                cell.borrow_mut()();
                Ok(JsValue::undefined())
            })
        }
    }
}

#[test]
#[allow(clippy::missing_panics_doc)]
pub fn into_js_module() {
    use boa_engine::builtins::promise::PromiseState;
    use boa_engine::{js_string, JsValue, Source};
    use std::rc::Rc;
    use std::sync::atomic::{AtomicU32, Ordering};

    let loader = Rc::new(loaders::HashMapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let foo_count = Rc::new(AtomicU32::new(0));
    let bar_count = Rc::new(AtomicU32::new(0));
    let module = unsafe {
        vec![
            (
                js_string!("foo"),
                IntoJsFunctionUnsafe::into_js_function(
                    {
                        let counter = foo_count.clone();
                        move || {
                            counter.fetch_add(1, Ordering::Relaxed);
                        }
                    },
                    &mut context,
                ),
            ),
            (
                js_string!("bar"),
                IntoJsFunctionUnsafe::into_js_function(
                    {
                        let counter = bar_count.clone();
                        move || {
                            counter.fetch_add(1, Ordering::Relaxed);
                        }
                    },
                    &mut context,
                ),
            ),
        ]
    }
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

    assert_eq!(foo_count.load(Ordering::Relaxed), 1);
    assert_eq!(bar_count.load(Ordering::Relaxed), 10);
    assert_eq!(v, JsValue::undefined());
}
