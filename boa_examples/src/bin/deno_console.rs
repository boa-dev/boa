use boa_engine::property::Attribute;
use boa_engine::NativeFunction;
use boa_engine::{js_string, object::FunctionObjectBuilder, Context, JsResult, JsValue};
use boa_parser::Source;

fn print(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    print!("{}", args[0].to_string(context)?.to_std_string_escaped());
    Ok(JsValue::undefined())
}

fn main() -> JsResult<()> {
    let context = &mut Context::default();

    let func = FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(print))
        .name("__boa_print")
        .length(0)
        .build();

    context
        .register_global_property(js_string!("__boa_print"), func, Attribute::default())
        .unwrap();

    let core_str = include_str!("../../scripts/deno/00_core.js");
    let console_str = include_str!("../../scripts/deno/01_console.js");
    let code_str = include_str!("../../scripts/console.js");

    for s in [core_str, console_str, code_str] {
        // Execute code using anonymous arrow functions to avoid polluting the global scope.
        let bytes = format!(r#"(()=>{{{}}})()"#, s).into_bytes();
        let source = Source::from_bytes(&bytes);
        context.eval(source)?;
    }

    Ok(())
}
