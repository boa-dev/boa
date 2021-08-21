use boa::{
    class::{Class, ClassBuilder},
    gc::{Finalize, Trace},
    property::Attribute,
    Context, JsValue, JsResult,
};

// We create a new struct that is going to represent a person.
//
// We derive `Debug`, `Trace` and `Finalize`, It automatically implements `NativeObject`
// so we can pass it an object in JavaScript.
//
// The fields of the struct are not accessible by JavaScript unless accessors are created for them.
/// This  Represents a Person.
#[derive(Debug, Trace, Finalize)]
struct Person {
    /// The name of the person.
    name: String,
    /// The age of the preson.
    age: u32,
}

// Here we implement a static method for Person that matches the `NativeFunction` signiture.
//
// NOTE: The function does not have to be implemented of Person it can be a free function,
// or any function that matches that signature.
impl Person {
    /// This function says hello
    fn say_hello(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // We check if this is an object.
        if let Some(object) = this.as_object() {
            // If it is we downcast the type to type `Person`.
            if let Some(person) = object.downcast_ref::<Person>() {
                // we print the message to stdout.
                println!(
                    "Hello my name is {}, I'm {} years old",
                    person.name,
                    person.age // Here we can access the native rust fields of Person struct.
                );
                return Ok(JsValue::undefined());
            }
        }
        // If `this` was not an object or the type was not an native object `Person`,
        // we throw a `TypeError`.
        context.throw_type_error("'this' is not a Person object")
    }
}

impl Class for Person {
    // we set the binging name of this function to be `"Person"`.
    // It does not have to be `"Person"` it can be any string.
    const NAME: &'static str = "Person";
    // We set the length to `2` since we accept 2 arguments in the constructor.
    //
    // This is the same as `Object.length`.
    // NOTE: If this is not defiend that the default is `0`.
    const LENGTH: usize = 2;

    // This is what is called when we do `new Person()`
    fn constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
        // We get the first argument. If it is unavailable we get `undefined`. And, then call `to_string()`.
        //
        // This is equivalent to `String(arg)`.
        let name = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?;
        // We get the second argument. If it is unavailable we get `undefined`. And, then call `to_u32`.
        //
        // This is equivalent to `arg | 0`.
        let age = args.get(1).cloned().unwrap_or_default().to_u32(context)?;

        // we construct the native struct `Person`
        let person = Person {
            name: name.to_string(),
            age,
        };

        Ok(person) // and we return it.
    }

    /// This is where the object is intitialized.
    fn init(class: &mut ClassBuilder) -> JsResult<()> {
        // we add a inheritable method `sayHello` with length `0` the amount of args it takes.
        //
        // This function is added to `Person.prototype.sayHello()`
        class.method("sayHello", 0, Self::say_hello);
        // we add a static mathod `is`, and here we use a closure, but it must be convertible
        // to a NativeFunction. it must not contain state, if it does it will give a compilation error.
        //
        // This function is added to `Person.is()`
        class.static_method("is", 1, |_this, args, _ctx| {
            if let Some(arg) = args.get(0) {
                if let Some(object) = arg.as_object() {
                    if object.is::<Person>() {
                        // we check if the object type is `Person`
                        return Ok(true.into()); // return `true`.
                    }
                }
            }
            Ok(false.into()) // otherwise `false`.
        });

        // Add a inherited property with the value `10`, with default attribute.
        // (`READONLY, NON_ENUMERABLE, PERMANENT).
        class.property("inheritedProperty", 10, Attribute::default());

        // Add a static property with the value `"Im a static property"`, with default attribute.
        // (`WRITABLE, ENUMERABLE, PERMANENT`).
        class.static_property(
            "staticProperty",
            "Im a static property",
            Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::PERMANENT,
        );

        Ok(())
    }
}

fn main() {
    let mut context = Context::new();

    // we register the global class `Person`.
    context.register_global_class::<Person>().unwrap();

    context
        .eval(
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
        )
        .unwrap();
}
