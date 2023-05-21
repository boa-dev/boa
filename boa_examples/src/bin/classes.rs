// NOTE: this example requires the `console` feature to run correctly.
use boa_engine::{
    class::{Class, ClassBuilder},
    error::JsNativeError,
    native_function::NativeFunction,
    property::Attribute,
    Context, JsArgs, JsResult, JsString, JsValue, Source,
};

use boa_gc::{Finalize, Trace};
use boa_runtime::Console;

// We create a new struct that is going to represent a person.
//
// We derive `Debug`, `Trace` and `Finalize`, it automatically implements `NativeObject`
// so we can pass it as an object in Javascript.
//
// The fields of the struct are not accessible by Javascript unless we create accessors for them.
/// Represents a `Person` object.
#[derive(Debug, Trace, Finalize)]
struct Person {
    /// The name of the person.
    name: JsString,
    /// The age of the person.
    age: u32,
}

// Here we implement a static method for Person that matches the `NativeFunction` signature.
//
// NOTE: The function does not have to be implemented inside Person, it can be a free function,
// or any function that matches the required signature.
impl Person {
    /// Says hello if `this` is a `Person`
    fn say_hello(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // We check if this is an object.
        if let Some(object) = this.as_object() {
            // If it is we downcast the type to type `Person`.
            if let Some(person) = object.downcast_ref::<Person>() {
                // and print a message to stdout.
                println!(
                    "Hello my name is {}, I'm {} years old",
                    person.name.to_std_string_escaped(),
                    person.age // Here we can access the native rust fields of the struct.
                );
                return Ok(JsValue::undefined());
            }
        }
        // If `this` was not an object or the type of `this` was not a native object `Person`,
        // we throw a `TypeError`.
        Err(JsNativeError::typ()
            .with_message("'this' is not a Person object")
            .into())
    }
}

impl Class for Person {
    // We set the binding name of this function to `"Person"`.
    // It does not have to be `"Person"`, it can be any string.
    const NAME: &'static str = "Person";
    // We set the length to `2` since we accept 2 arguments in the constructor.
    //
    // This is the same as `Object.length`.
    // NOTE: The default value of `LENGTH` is `0`.
    const LENGTH: usize = 2;

    // This is what is called when we construct a `Person` with the expression `new Person()`.
    fn constructor(_this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<Self> {
        // We get the first argument. If it is unavailable we default to `undefined`,
        // and then we call `to_string()`.
        //
        // This is equivalent to `String(arg)`.
        let name = args.get_or_undefined(0).to_string(context)?;
        // We get the second argument. If it is unavailable we default to `undefined`,
        // and then we call `to_u32`.
        //
        // This is equivalent to `arg | 0`.
        let age = args.get(1).cloned().unwrap_or_default().to_u32(context)?;

        // We construct a new native struct `Person`
        let person = Person { name, age };

        Ok(person) // and we return it.
    }

    /// Here is where the class is initialized.
    fn init(class: &mut ClassBuilder) -> JsResult<()> {
        // We add a inheritable method `sayHello` with `0` arguments of length.
        //
        // This function is added to the `Person` prototype.
        class.method("sayHello", 0, NativeFunction::from_fn_ptr(Self::say_hello));
        // We add a static method `is` using a closure, but it must be convertible
        // to a NativeFunction.
        // This means it must not contain state, or the code won't compile.
        //
        // This function is added to the `Person` class.
        class.static_method(
            "is",
            1,
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                if let Some(arg) = args.get(0) {
                    if let Some(object) = arg.as_object() {
                        // We check if the type of `args[0]` is `Person`
                        if object.is::<Person>() {
                            return Ok(true.into()); // and return `true` if it is.
                        }
                    }
                }
                Ok(false.into()) // Otherwise we return `false`.
            }),
        );

        // We add an `"inheritedProperty"` property to the prototype of `Person` with
        // a value of `10` and default attribute flags `READONLY`, `NON_ENUMERABLE` and `PERMANENT`.
        class.property("inheritedProperty", 10, Attribute::default());

        // Finally, we add a `"staticProperty"` property to `Person` with a value
        // of `"Im a static property"` and attribute flags `WRITABLE`, `ENUMERABLE` and `PERMANENT`.
        class.static_property(
            "staticProperty",
            "Im a static property",
            Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::PERMANENT,
        );

        Ok(())
    }
}

/// Adds the custom runtime to the context.
fn add_runtime(context: &mut Context<'_>) {
    // We first add the `console` object, to be able to call `console.log()`.
    let console = Console::init(context);
    context
        .register_global_property(Console::NAME, console, Attribute::all())
        .expect("the console builtin shouldn't exist");

    // Then we need to register the global class `Person` inside `context`.
    context
        .register_global_class::<Person>()
        .expect("the Person builtin shouldn't exist");
}

fn main() {
    // First we need to create a Javascript context.
    let mut context = Context::default();

    // Then, we add our custom runtime.
    add_runtime(&mut context);

    // Having done all of that, we can execute Javascript code with `eval`,
    // and access the `Person` class defined in Rust!
    context
        .eval(Source::from_bytes(
            r"
		let person = new Person('John', 19);
		person.sayHello();

		if (Person.is(person)) {
			console.log('person is a Person class instance.');
		}
		if (!Person.is('Hello')) {
			console.log('\'Hello\' string is not a Person class instance.');
		}

        console.log(Person.staticProperty);
        console.log(person.inheritedProperty);
	    console.log(Person.prototype.inheritedProperty === person.inheritedProperty);
    ",
        ))
        .unwrap();
}
