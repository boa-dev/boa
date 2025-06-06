//! Test for the class proc-macro.
#![allow(unused_crate_dependencies)]

use boa_engine::{js_string, Context, JsObject, JsString, Module, Source};
use boa_macros::{boa_class, boa_module, Finalize, JsData, Trace};
use std::rc::Rc;

#[derive(Clone, Trace, Finalize, JsData)]
enum AnimalType {
    Cat,
    Dog,
    Other,
}

#[derive(Clone, Trace, Finalize, JsData)]
struct Animal {
    ty: AnimalType,
    age: i32,
}

#[boa_class]
#[boa(rename = "camelCase")]
impl Animal {
    #[boa(constructor)]
    #[allow(clippy::needless_pass_by_value)]
    fn new(name: String, age: i32) -> Self {
        let ty = match name.as_str() {
            "cat" => AnimalType::Cat,
            "dog" => AnimalType::Dog,
            _ => AnimalType::Other,
        };

        Self { ty, age }
    }

    #[boa(getter)]
    fn age(&self) -> i32 {
        self.age
    }

    fn speak(#[boa(error = "`this` was not an animal")] &self) -> JsString {
        match self.ty {
            AnimalType::Cat => js_string!("meow"),
            AnimalType::Dog => js_string!("woof"),
            AnimalType::Other => js_string!(r"¯\_(ツ)_/¯"),
        }
    }
}

#[boa_module]
#[boa(rename = "camelCase")]
#[boa(rename_class = "PascalCase")]
mod hello {
    use boa_engine::{js_string, JsString};

    fn world() -> JsString {
        js_string!("hello world")
    }

    type Animal = super::Animal;

    const SOME_LITERAL_NUMBER: i32 = 1234;
}

const ASSERT_DECL: &str = r"
    function assertEq(lhs, rhs, message) {
      if (lhs !== rhs) {
        throw `AssertionError: ${message ? message + ',' : ''} expected ${JSON.stringify(rhs)}, actual ${JSON.stringify(lhs)}`;
      }
    }
";

#[test]
fn boa_module() {
    let module_loader = Rc::new(boa_interop::loaders::HashMapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(module_loader.clone())
        .build()
        .expect("Could not create context.");

    module_loader.register("/hello.js", hello::boa_module(None, &mut context));

    context
        .eval(Source::from_bytes(ASSERT_DECL))
        .expect("Unreachable.");

    let module = Module::parse(
        Source::from_bytes(
            r#"
                import * as m from '/hello.js';

                assertEq(m.someLiteralNumber, 1234, "Const value");
                assertEq(m.world(), "hello world", "Method call");

                let pet = new m.Animal("dog", 8);
                assertEq(pet.age, 8, "Property of class");
                assertEq(pet.speak(), "woof", "Method class of class");
            "#,
        ),
        None,
        &mut context,
    )
    .expect("Could not load module");

    let result = module
        .load_link_evaluate(&mut context)
        .await_blocking(&mut context);

    if let Err(e) = result {
        panic!("error: {e:?}\n{e}");
    }
}
