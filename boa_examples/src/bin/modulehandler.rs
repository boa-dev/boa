// This example implements a custom module handler which mimics
// the require/module.exports pattern

use boa_engine::{
    native_function::NativeFunction, prelude::JsObject, property::Attribute, Context, JsResult,
    JsValue, Source,
};
use std::fs::read_to_string;

fn main() {
    let js_file_path = "./scripts/calctest.js";
    let buffer = read_to_string(js_file_path);

    if buffer.is_err() {
        println!("Error: {}", buffer.unwrap_err());
        return;
    }

    // Creating the execution context
    let mut ctx = Context::default();

    // Adding custom implementation that mimics 'require'
    ctx.register_global_callable("require", 0, NativeFunction::from_fn_ptr(require))
        .unwrap();

    // Adding custom object that mimics 'module.exports'
    let moduleobj = JsObject::default();
    moduleobj
        .set("exports", JsValue::from(" "), false, &mut ctx)
        .unwrap();
    ctx.register_global_property("module", JsValue::from(moduleobj), Attribute::default())
        .unwrap();

    // Instantiating the engine with the execution context
    // Loading, parsing and executing the JS code from the source file
    ctx.eval(Source::from_bytes(&buffer.unwrap())).unwrap();
}

// Custom implementation that mimics the 'require' module loader
fn require(_: &JsValue, args: &[JsValue], ctx: &mut Context<'_>) -> JsResult<JsValue> {
    let arg = args.get(0).unwrap();

    // BUG: Dev branch seems to be passing string arguments along with quotes
    let libfile = arg
        .to_string(ctx)
        .expect("Failed to convert to string")
        .to_std_string_escaped();

    // Read the module source file
    println!("Loading: {libfile}");
    let buffer = read_to_string(libfile);
    if let Err(..) = buffer {
        println!("Error: {}", buffer.unwrap_err());
        Ok(JsValue::Rational(-1.0))
    } else {
        // Load and parse the module source
        ctx.eval(Source::from_bytes(&buffer.unwrap())).unwrap();

        // Access module.exports and return as ResultValue
        let global_obj = ctx.global_object();
        let module = global_obj.get("module", ctx).unwrap();
        module.as_object().unwrap().get("exports", ctx)
    }
}
