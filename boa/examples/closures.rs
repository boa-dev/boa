// This example goes into the details on how to pass closures as functions
// inside Rust and call them from Javascript.

use boa::{
    gc::{Finalize, Trace},
    object::{FunctionBuilder, JsObject},
    property::{Attribute, PropertyDescriptor},
    Context, JsString, JsValue,
};

fn main() -> Result<(), JsValue> {
    // We create a new `Context` to create a new Javascript executor.
    let mut context = Context::new();

    // We make some operations in Rust that return a `Copy` value that we want
    // to pass to a Javascript function.
    let variable = 128 + 64 + 32 + 16 + 8 + 4 + 2 + 1;

    // We register a global closure function that has the name 'closure' with length 0.
    context.register_global_closure("closure", 0, move |_, _, _| {
        println!("Called `closure`");
        // `variable` is captured from the main function.
        println!("variable = {}", variable);

        // We return the moved variable as a `JsValue`.
        Ok(JsValue::new(variable))
    })?;

    assert_eq!(context.eval("closure()")?, 255.into());

    // We have created a closure with moved variables and executed that closure
    // inside Javascript!

    // This struct is passed to a closure as a capture.
    #[derive(Debug, Clone, Trace, Finalize)]
    struct BigStruct {
        greeting: JsString,
        object: JsObject,
    }

    // We create a new `JsObject` with some data
    let object = context.construct_object();
    object.define_property_or_throw(
        "name",
        PropertyDescriptor::builder()
            .value("Boa dev")
            .writable(false)
            .enumerable(false)
            .configurable(false),
        &mut context,
    )?;

    // Now, we execute some operations that return a `Clone` type
    let clone_variable = BigStruct {
        greeting: JsString::from("Hello from Javascript!"),
        object,
    };

    // We can use `FunctionBuilder` to define a closure with additional
    // captures.
    let js_function = FunctionBuilder::closure_with_captures(
        &mut context,
        |_, _, captures, context| {
            println!("Called `createMessage`");
            // We obtain the `name` property of `captures.object`
            let name = captures.object.get("name", context)?;

            // We create a new message from our captured variable.
            let message = JsString::concat_array(&[
                "message from `",
                name.to_string(context)?.as_str(),
                "`: ",
                captures.greeting.as_str(),
            ]);

            println!("{}", message);

            // We convert `message` into `Jsvalue` to be able to return it.
            Ok(message.into())
        },
        // Here is where we move `clone_variable` into the closure.
        clone_variable,
    )
    // And here we assign `createMessage` to the `name` property of the closure.
    .name("createMessage")
    // By default all `FunctionBuilder`s set the `length` property to `0` and
    // the `constructable` property to `false`.
    .build();

    // We bind the newly constructed closure as a global property in Javascript.
    context.register_global_property(
        // We set the key to access the function the same as its name for
        // consistency, but it may be different if needed.
        "createMessage",
        // We pass `js_function` as a property value.
        js_function,
        // We assign to the "createMessage" property the desired attributes.
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
    );

    assert_eq!(
        context.eval("createMessage()")?,
        "message from `Boa dev`: Hello from Javascript!".into()
    );

    // We have moved `Clone` variables into a closure and executed that closure
    // inside Javascript!

    Ok(())
}
