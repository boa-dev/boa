//! Test for the class proc-macro.
#![allow(unused_crate_dependencies)]

use boa_engine::{
    Context, Finalize, JsData, JsObject, JsString, Source, Trace, boa_class, js_string,
};

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

    #[boa(static)]
    fn marked_static_method() -> i32 {
        123
    }

    fn static_method() -> i32 {
        42
    }

    // Force this being a method (instead of a static function) by declaring it
    // as a method.
    #[boa(method)]
    #[boa(length = 11)]
    fn method(context: &mut Context) -> JsObject {
        let obj = JsObject::with_null_proto();
        obj.set(js_string!("key"), 43, false, context).unwrap();
        obj
    }

    #[boa(getter)]
    fn age(&self) -> i32 {
        self.age
    }

    #[boa(setter)]
    #[boa(method)]
    #[boa(rename = "age")]
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

const ASSERT_DECL: &str = r"
    function assertEq(lhs, rhs, message) {
      if (lhs !== rhs) {
        throw `AssertionError: ${message ? message + ',' : ''} expected ${JSON.stringify(rhs)}, actual ${JSON.stringify(lhs)}`;
      }
    }
";

#[test]
fn boa_class() {
    let mut context = Context::default();

    context.register_global_class::<Animal>().unwrap();

    context
        .eval(Source::from_bytes(ASSERT_DECL))
        .expect("Unreachable.");

    context
        .eval(Source::from_bytes(
            r#"
            let pet = new Animal("dog", 3);
            assertEq(pet.age, 3, "Age should be the age passed to constructor");

            assertEq(Animal.staticMethod(), 42, "Static method");
            assertEq(Animal.markedStaticMethod(), 123, "Marked static method");

            v = pet.method();
            assertEq(v.key, 43, "Method returned");

            pet.age = 4;
            assertEq(pet.age, 4, "Pet setter");

            pet.setAge(5);
            assertEq(pet.age, 5, "Pet.setAge");

            assertEq(Animal.prototype.method.length, 11, "Method.length");
            assertEq(Animal.prototype.speak.length, 0, "speak.length");
            assertEq(Animal.prototype.setAge.length, 1, "setAge.length");
     "#,
        ))
        .expect("Could not evaluate script");
}
