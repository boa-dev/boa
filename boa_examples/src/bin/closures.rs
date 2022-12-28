// This example goes into the details on how to pass closures as functions inside Rust and call them
// from Javascript.

use std::cell::{Cell, RefCell};

use boa_engine::{
    native_function::NativeFunction,
    js_string,
    object::{builtins::JsArray, FunctionObjectBuilder, JsObject},
    property::{Attribute, PropertyDescriptor},
    string::utf16,
    Context, JsError, JsNativeError, JsString, JsValue,
};
use boa_gc::{Finalize, GcCell, Trace};

fn main() -> Result<(), JsError> {
    // We create a new `Context` to create a new Javascript executor.
    let mut context = Context::default();

    // We make some operations in Rust that return a `Copy` value that we want to pass to a Javascript
    // function.
    let variable = 128 + 64 + 32 + 16 + 8 + 4 + 2 + 1;

    // We register a global closure function that has the name 'closure' with length 0.
    context.register_global_callable(
        "closure",
        0,
        NativeFunction::from_copy_closure(move |_, _, _| {
            println!("Called `closure`");
            // `variable` is captured from the main function.
            println!("variable = {variable}");
            println!();

            // We return the moved variable as a `JsValue`.
            Ok(JsValue::new(variable))
        }),
    );

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
    let object = JsObject::with_object_proto(&mut context);
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
        greeting: JsString::from("Hello!"),
        object,
    };

    // We can use `FunctionBuilder` to define a closure with additional captures and custom property
    // attributes.
    let js_function = FunctionObjectBuilder::new(
        &mut context,
        NativeFunction::from_copy_closure_with_captures(
            |_, _, captures, context| {
                let mut captures = captures.borrow_mut();
                let BigStruct { greeting, object } = &mut *captures;
                println!("Called `createMessage`");
                // We obtain the `name` property of `captures.object`
                let name = object.get("name", context)?;

                // We create a new message from our captured variable.
                let message = js_string!(
                    utf16!("message from `"),
                    &name.to_string(context)?,
                    utf16!("`: "),
                    greeting
                );

                // We can also mutate the moved data inside the closure.
                captures.greeting = js_string!(greeting, utf16!(" Hello!"));

                println!("{}", message.to_std_string_escaped());
                println!();

                // We convert `message` into `JsValue` to be able to return it.
                Ok(message.into())
            },
            // Here is where we move `clone_variable` into the closure.
            GcCell::new(clone_variable),
        ),
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
        Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
    );

    assert_eq!(
        context.eval("createMessage()")?,
        "message from `Boa dev`: Hello!".into()
    );

    // The data mutates between calls
    assert_eq!(
        context.eval("createMessage(); createMessage();")?,
        "message from `Boa dev`: Hello! Hello! Hello!".into()
    );

    // We have moved `Clone` variables into a closure and executed that closure
    // inside Javascript!

    // ADVANCED

    // If we can ensure the captured variables are not traceable by the garbage collector,
    // we can pass any static closure easily.

    let index = Cell::new(0i32);
    let numbers = RefCell::new(Vec::new());

    // We register a global closure that is not `Copy`.
    context.register_global_callable(
        "enumerate",
        0,
        // Note that it is required to use `unsafe` code, since the compiler cannot verify that the
        // types captured by the closure are not traceable.
        unsafe {
            NativeFunction::from_closure(move |_, _, context| {
                println!("Called `enumerate`");
                // `index` is captured from the main function.
                println!("index = {}", index.get());
                println!();

                numbers.borrow_mut().push(index.get());
                index.set(index.get() + 1);

                // We return the moved variable as a `JsValue`.
                Ok(
                    JsArray::from_iter(
                        numbers.borrow().iter().cloned().map(JsValue::from),
                        context,
                    )
                    .into(),
                )
            })
        },
    );

    // First call should return the array `[0]`.
    let result = context.eval("enumerate()")?;
    let object = result
        .as_object()
        .cloned()
        .ok_or_else(|| JsNativeError::typ().with_message("not an array!"))?;
    let array = JsArray::from_object(object)?;

    assert_eq!(array.get(0, &mut context)?, JsValue::from(0i32));
    assert_eq!(array.get(1, &mut context)?, JsValue::undefined());

    // First call should return the array `[0, 1]`.
    let result = context.eval("enumerate()")?;
    let object = result
        .as_object()
        .cloned()
        .ok_or_else(|| JsNativeError::typ().with_message("not an array!"))?;
    let array = JsArray::from_object(object)?;

    assert_eq!(array.get(0, &mut context)?, JsValue::from(0i32));
    assert_eq!(array.get(1, &mut context)?, JsValue::from(1i32));
    assert_eq!(array.get(2, &mut context)?, JsValue::undefined());

    // We have moved non-traceable variables into a closure and executed that closure inside Javascript!
    Ok(())
}
