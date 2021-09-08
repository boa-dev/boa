use boa::{Context, JsValue};

fn main() -> Result<(), JsValue> {
    let mut context = Context::new();

    let variable = "I am a captured variable";

    // We register a global closure function that has the name 'closure' with length 0.
    context.register_global_closure("closure", 0, move |_, _, _| {
        // This value is captured from main function.
        println!("variable = {}", variable);
        Ok(JsValue::new(variable))
    })?;

    assert_eq!(
        context.eval("closure()")?,
        "I am a captured variable".into()
    );

    Ok(())
}
