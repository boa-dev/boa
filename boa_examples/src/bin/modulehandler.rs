// This example implements a custom module handler which mimics
// the require/module.exports pattern

use boa_engine::{
    js_string, native_function::NativeFunction, prelude::JsObject, property::Attribute, Context,
    JsArgs, JsNativeError, JsResult, JsValue, Source,
};
use boa_runtime::Console;
use std::{error::Error, fs::read_to_string};

/// Adds the custom runtime to the context.
fn add_runtime(context: &mut Context<'_>) {
    // We first add the `console` object, to be able to call `console.log()`.
    let console = Console::init(context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");
}

fn main() -> Result<(), Box<dyn Error>> {
    let js_file_path = "./scripts/calctest.js";
    let buffer = read_to_string(js_file_path)?;

    // Creating the execution context
    let mut ctx = Context::default();

    // Adding the runtime intrinsics to the context
    add_runtime(&mut ctx);

    // Adding custom implementation that mimics 'require'
    ctx.register_global_callable(
        js_string!("require"),
        0,
        NativeFunction::from_fn_ptr(require),
    )?;

    // Adding custom object that mimics 'module.exports'
    let moduleobj = JsObject::default();
    moduleobj.set(
        js_string!("exports"),
        JsValue::from(js_string!(" ")),
        false,
        &mut ctx,
    )?;

    ctx.register_global_property(
        js_string!("module"),
        JsValue::from(moduleobj),
        Attribute::default(),
    )?;

    // Instantiating the engine with the execution context
    // Loading, parsing and executing the JS code from the source file
    ctx.eval(Source::from_bytes(&buffer))?;

    Ok(())
}

// Custom implementation that mimics the 'require' module loader
fn require(_: &JsValue, args: &[JsValue], ctx: &mut Context<'_>) -> JsResult<JsValue> {
    let arg = args.get_or_undefined(0);

    // BUG: Dev branch seems to be passing string arguments along with quotes
    let libfile = arg.to_string(ctx)?.to_std_string_escaped();

    // Read the module source file
    println!("Loading: {libfile}");
    let buffer =
        read_to_string(libfile).map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
    // Load and parse the module source
    ctx.eval(Source::from_bytes(&buffer))?;

    // Access module.exports and return as ResultValue
    let global_obj = ctx.global_object();
    let module = global_obj.get(js_string!("module"), ctx)?;
    module
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("`exports` property was not an object"))?
        .get(js_string!("exports"), ctx)
}
