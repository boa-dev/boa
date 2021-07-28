use boa::{Context, JsString};

fn main() {
    let mut context = Context::new();

    let variable = JsString::new("I am a captured variable");

    context
        .register_global_closure("closure", 0, move |_, _, _| {
            // This value is captured from main function.
            Ok(variable.clone().into())
        })
        .unwrap();

    assert_eq!(
        context.eval("closure()"),
        Ok("I am an captured variable".into())
    );
}
