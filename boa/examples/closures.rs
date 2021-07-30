use boa::{Context, JsString, Value};

fn main() -> Result<(), Value> {
    let mut context = Context::new();

    let variable = JsString::new("I am a captured variable");

    // We register a global closure function that has the name 'closure' with length 0.
    context.register_global_closure("closure", 0, move |_, _, _| {
        // This value is captured from main function.
        Ok(variable.clone().into())
    })?;

    assert_eq!(
        context.eval("closure()")?,
        "I am a captured variable".into()
    );

    Ok(())
}
