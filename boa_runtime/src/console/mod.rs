//! Boa's implementation of JavaScript's `console` Web API object.
//!
//! The `console` object can be accessed from any global object.
//!
//! The specifics of how it works varies from browser to browser, but there is a de facto set of features that are typically provided.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [WHATWG `console` specification][spec]
//!
//! [spec]: https://console.spec.whatwg.org/
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Console

#[cfg(test)]
mod tests;

use boa_engine::{
    js_string,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject},
    value::JsValue,
    Context, JsResult, Source,
};
use boa_gc::{Finalize, Trace};
// use boa_profiler::Profiler;

/// This is the internal console object state.
#[derive(Debug, Default, Trace, Finalize)]
pub struct Console;
use boa_engine::property::Attribute;

fn print(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    print!("{}", args[0].to_string(context)?.to_std_string_escaped());
    Ok(JsValue::undefined())
}

impl Console {
    /// Name of the built-in `console` property.
    pub const NAME: &'static str = "console";

    /// Initializes the `console` built-in object.
    #[allow(clippy::too_many_lines)]
    pub fn init(context: &mut Context) -> JsObject {
        let func = FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(print))
            .name("__boa_print")
            .length(0)
            .build();

        context
            .register_global_property(js_string!("__boa_print"), func, Attribute::default())
            .expect("register_global_property __boa_print error");
        let core_str = include_str!("./deno-scripts/00_core.js");
        let console_str = include_str!("./deno-scripts/01_console.js");
        for s in [core_str, console_str] {
            // Execute code using anonymous arrow functions to avoid polluting the global scope.
            let bytes = format!(r#"(()=>{{{s}}})()"#).into_bytes();
            let source = Source::from_bytes(&bytes);
            context
                .eval(source)
                .expect("eval deno-console script error");
        }

        let binding = context
            .global_object()
            .get(js_string!("console"), context)
            .expect("get global console JsValue error");
        let console = binding
            .as_object()
            .expect("get global console JsObject error");
        console.to_owned()
    }
}
