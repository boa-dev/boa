use boa_engine::{
    builtins::function::Function,
    object::ObjectInitializer,
    vm::flowgraph::{Direction, Graph},
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};

use crate::FlowgraphFormat;

fn flowgraph_parse_format_option(value: &JsValue) -> JsResult<FlowgraphFormat> {
    if value.is_undefined() {
        return Ok(FlowgraphFormat::Mermaid);
    }

    if let Some(string) = value.as_string() {
        return match string.to_std_string_escaped().to_lowercase().as_str() {
            "mermaid" => Ok(FlowgraphFormat::Mermaid),
            "graphviz" => Ok(FlowgraphFormat::Graphviz),
            format => Err(JsNativeError::typ()
                .with_message(format!("Unknown format type '{format}'"))
                .into()),
        };
    }

    Err(JsNativeError::typ()
        .with_message("format type must be a string")
        .into())
}

fn flowgraph_parse_direction_option(value: &JsValue) -> JsResult<Direction> {
    if value.is_undefined() {
        return Ok(Direction::LeftToRight);
    }

    if let Some(string) = value.as_string() {
        return match string.to_std_string_escaped().to_lowercase().as_str() {
            "leftright" | "lr" => Ok(Direction::LeftToRight),
            "rightleft" | "rl" => Ok(Direction::RightToLeft),
            "topbottom" | "tb" => Ok(Direction::TopToBottom),
            "bottomtop" | "bt " => Ok(Direction::BottomToTop),
            direction => Err(JsNativeError::typ()
                .with_message(format!("Unknown direction type '{direction}'"))
                .into()),
        };
    }

    Err(JsNativeError::typ()
        .with_message("direction type must be a string")
        .into())
}

/// Get functions instruction flowgraph
fn flowgraph(_this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    let Some(value) = args.get(0) else {
        return Err(JsNativeError::typ()
        .with_message("expected function argument")
        .into());
    };

    let Some(object) = value.as_object() else {
        return Err(JsNativeError::typ()
        .with_message(format!("expected object, got {}", value.type_of()))
        .into());
    };

    let mut format = FlowgraphFormat::Mermaid;
    let mut direction = Direction::LeftToRight;
    if let Some(arguments) = args.get(1) {
        if let Some(arguments) = arguments.as_object() {
            format = flowgraph_parse_format_option(&arguments.get("format", context)?)?;
            direction = flowgraph_parse_direction_option(&arguments.get("direction", context)?)?;
        } else if value.is_string() {
            format = flowgraph_parse_format_option(value)?;
        } else {
            return Err(JsNativeError::typ()
                .with_message("options argument must be a string or object")
                .into());
        }
    }

    let object = object.borrow();

    let Some(function) = object.as_function() else {
        return Err(JsNativeError::typ()
        .with_message("expected function object")
        .into());
    };

    let code = match function {
        Function::Ordinary { code, .. }
        | Function::Async { code, .. }
        | Function::Generator { code, .. }
        | Function::AsyncGenerator { code, .. } => code,
        Function::Native { .. } => {
            return Err(JsNativeError::typ()
                .with_message("native functions do not have bytecode")
                .into())
        }
    };

    let mut graph = Graph::new(direction);
    code.to_graph(context.interner(), graph.subgraph(String::default()));
    let result = match format {
        FlowgraphFormat::Graphviz => graph.to_graphviz_format(),
        FlowgraphFormat::Mermaid => graph.to_mermaid_format(),
    };

    Ok(JsValue::new(result))
}

/// Trace function.
fn trace(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0);
    let this = args.get_or_undefined(1);

    let Some(callable) = value.as_callable() else {
        return Err(JsNativeError::typ()
        .with_message("expected callable object")
        .into());
    };

    let arguments = args.get(2..).unwrap_or(&[]);

    context.set_trace(true);
    let result = callable.call(this, arguments, context);
    context.set_trace(false);
    result
}

pub(super) fn create_object(context: &mut Context<'_>) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(flowgraph), "flowgraph", 1)
        .function(NativeFunction::from_fn_ptr(trace), "trace", 1)
        .build()
}
