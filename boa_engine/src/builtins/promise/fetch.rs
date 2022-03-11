
use crate::{
    builtins::BuiltIn,
    object::FunctionBuilder,
    property::Attribute,
    value::JsValue,
    Context, JsResult,
};

use super::JsArgs;

use tap::{Conv, Pipe};

#[derive(Debug, Copy, Clone)]
pub struct Fetch;

impl BuiltIn for Fetch {
    const NAME: &'static str = "fetch";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);

    fn init(context: &mut Context) -> Option<JsValue> {
        FunctionBuilder::native(context, Self::fetch)
            .name("fetch")
            .length(1)
            .constructor(false)
            .build()
            .conv::<JsValue>()
            .pipe(Some)
    }
}

impl Fetch {
    pub(crate) fn fetch(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let first_arg = args.get_or_undefined(0);

        if first_arg.is_undefined() {
            return context.throw_type_error("Failed to execute 'fetch': 1 argument required, but only 0 present.")           
        }

        println!("This is a call from fetch!");
        
        Ok(JsValue::Undefined)
    }
}
