//! Test for the class proc-macro.
#![allow(unused_crate_dependencies)]

use boa_engine::{js_str, js_string, Context, JsObject, JsString, JsValue, Source};
use boa_macros::{boa_class, Finalize, JsData, Trace};

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

    fn static_method() -> i32 {
        42
    }

    // Force this being a method (instead of a static function) by declaring it
    // as a method.
    #[boa(method)]
    fn method(context: &mut Context) -> JsValue {
        let obj = JsObject::with_null_proto();
        obj.set(js_string!("key"), 43, false, context).unwrap();
        obj.into()
    }

    #[boa(getter)]
    fn age(&self) -> i32 {
        self.age
    }

    #[boa(setter)]
    #[boa(name = "age")]
    fn set_age(&mut self, age: i32) {
        self.age = age;
    }

    fn speak(#[boa(error = "`this` was not an animal")] &self) -> JsString {
        match self.ty {
            AnimalType::Cat => js_string!("meow"),
            AnimalType::Dog => js_string!("woof"),
            AnimalType::Other => js_string!(r"¯\_(ツ)_/¯"),
        }
    }
}

#[test]
fn boa_class() {
    let mut context = Context::default();

    context.register_global_class::<Animal>().unwrap();

    let result = context
        .eval(Source::from_bytes(
            r#"
            let pet = new Animal("dog", 3);
            if (pet.age !== 3) {
                throw "age should be 3";
            }

            let v = Animal.staticMethod();
            if (v !== 42) {
                throw "Static method returned " + JSON.stringify(v);
            }
            v = Animal.method();
            if (v.key !== 43) {
                throw "Method returned " + JSON.stringify(v);
            }

            pet.age = 4;
            `My pet is ${pet.age} years old. Right, buddy? - ${pet.speak()}!`
     "#,
        ))
        .expect("Could not evaluate script");

    assert_eq!(
        result.as_string().unwrap(),
        &js_str!("My pet is 4 years old. Right, buddy? - woof!")
    );
}
