use boa_engine::{
    Context, JsArgs, JsObject, JsResult, JsValue, NativeFunction, js_string,
    object::{FunctionObjectBuilder, ObjectInitializer},
    property::Attribute,
};

fn get_loop(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let max = context.runtime_limits().loop_iteration_limit();
    Ok(JsValue::from(max))
}

fn set_loop(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_length(context)?;
    context
        .runtime_limits_mut()
        .set_loop_iteration_limit(value as u64);
    Ok(JsValue::undefined())
}

fn get_stack(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let max = context.runtime_limits().stack_size_limit();
    Ok(JsValue::from(max))
}

fn set_stack(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_length(context)?;
    context.runtime_limits_mut().set_stack_size_limit(value);
    Ok(JsValue::undefined())
}

fn get_recursion(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let max = context.runtime_limits().recursion_limit();
    Ok(JsValue::from(max))
}

fn set_recursion(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_length(context)?;
    context.runtime_limits_mut().set_recursion_limit(value);
    Ok(JsValue::undefined())
}

fn get_backtrace(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let max = context.runtime_limits().backtrace_limit();
    Ok(JsValue::from(max))
}

fn set_backtrace(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_length(context)?;
    context.runtime_limits_mut().set_backtrace_limit(value);
    Ok(JsValue::undefined())
}

pub(super) fn create_object(context: &mut Context) -> JsObject {
    let get_loop =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(get_loop))
            .name(js_string!("get loop"))
            .length(0)
            .build();
    let set_loop =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(set_loop))
            .name(js_string!("set loop"))
            .length(1)
            .build();

    let get_stack =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(get_stack))
            .name(js_string!("get stack"))
            .length(0)
            .build();
    let set_stack =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(set_stack))
            .name(js_string!("set stack"))
            .length(1)
            .build();

    let get_recursion =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(get_recursion))
            .name(js_string!("get recursion"))
            .length(0)
            .build();
    let set_recursion =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(set_recursion))
            .name(js_string!("set recursion"))
            .length(1)
            .build();
    let get_backtrace =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(get_backtrace))
            .name(js_string!("get backtrace"))
            .length(0)
            .build();
    let set_backtrace =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(set_backtrace))
            .name(js_string!("set backtrace"))
            .length(1)
            .build();

    ObjectInitializer::new(context)
        .accessor(
            js_string!("loop"),
            Some(get_loop),
            Some(set_loop),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .accessor(
            js_string!("stack"),
            Some(get_stack),
            Some(set_stack),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .accessor(
            js_string!("recursion"),
            Some(get_recursion),
            Some(set_recursion),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .accessor(
            js_string!("backtrace"),
            Some(get_backtrace),
            Some(set_backtrace),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .build()
}
