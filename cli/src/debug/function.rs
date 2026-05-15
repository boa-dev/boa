use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
    builtins::function::OrdinaryFunction,
    js_string,
    object::ObjectInitializer,
    vm::flowgraph::{Direction, Graph},
};
use cow_utils::CowUtils;

use crate::FlowgraphFormat;

fn flowgraph_parse_format_option(value: &JsValue) -> JsResult<FlowgraphFormat> {
    if value.is_undefined() {
        return Ok(FlowgraphFormat::Mermaid);
    }
    if let Some(string) = value.as_string() {
        return match string.to_std_string_escaped().cow_to_lowercase().as_ref() {
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
        return match string.to_std_string_escaped().cow_to_lowercase().as_ref() {
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

/// Returns the instruction flowgraph of a function in the specified format.
///
/// Supports multiple output formats: `"mermaid"` (default) and `"graphviz"`.
/// The direction can be configured via the options object with `"LeftRight"` (default),
/// `"RightLeft"`, `"TopBottom"`, or `"BottomTop"`.
///
/// # Errors
///
/// Returns a `TypeError` if the first argument is not a function, or if the format
/// or direction string is invalid.
///
/// # Examples
///
/// ```ignore
/// $boa.function.flowgraph(myFunc);
/// $boa.function.flowgraph(myFunc, "graphviz");
/// $boa.function.flowgraph(myFunc, { format: "mermaid", direction: "TopBottom" });
/// ```
fn flowgraph(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let Some(value) = args.first() else {
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
            format = flowgraph_parse_format_option(&arguments.get(js_string!("format"), context)?)?;
            direction = flowgraph_parse_direction_option(
                &arguments.get(js_string!("direction"), context)?,
            )?;
        } else {
            format = flowgraph_parse_format_option(arguments)?;
        }
    }
    let Some(function) = object.downcast_ref::<OrdinaryFunction>() else {
        return Err(JsNativeError::typ()
            .with_message("expected an ordinary function object")
            .into());
    };
    let code = function.codeblock();
    let mut graph = Graph::new(direction);
    code.to_graph(graph.subgraph(String::default()));
    let result = match format {
        FlowgraphFormat::Graphviz => graph.to_graphviz_format(),
        FlowgraphFormat::Mermaid => graph.to_mermaid_format(),
    };
    Ok(JsValue::new(js_string!(result)))
}

/// Prints the compiled bytecode of a function to stdout as a formatted dump.
///
/// The output shows opcodes, operands, constants, bindings, and exception
/// handler tables for the given function's compiled body. Returns `undefined`.
///
/// # Errors
///
/// Returns a `TypeError` if the first argument is not an ordinary function object.
///
/// # Examples
///
/// ```ignore
/// function add(x, y) { return x + y; }
/// $boa.function.bytecode(add);
/// // Prints:
/// // ------------------------Compiled Output: 'add'------------------------
/// // Location  Count    Handler    Opcode                     Operands
/// // 000000    0000      none      CreateMappedArgumentsObject
/// // 000001    0001      none      PutLexicalValue                           2: 0
/// // ...
/// ```
fn bytecode(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let Some(value) = args.first() else {
        return Err(JsNativeError::typ()
            .with_message("expected function argument")
            .into());
    };

    let Some(object) = value.as_object() else {
        return Err(JsNativeError::typ()
            .with_message(format!("expected object, got {}", value.type_of()))
            .into());
    };
    let Some(function) = object.downcast_ref::<OrdinaryFunction>() else {
        return Err(JsNativeError::typ()
            .with_message("expected an ordinary function object")
            .into());
    };
    let code = function.codeblock();

    println!("{code}");

    Ok(JsValue::undefined())
}

fn set_trace_flag_in_function_object(object: &JsObject, value: bool) -> JsResult<()> {
    let Some(function) = object.downcast_ref::<OrdinaryFunction>() else {
        return Err(JsNativeError::typ()
            .with_message("expected an ordinary function object")
            .into());
    };
    let code = function.codeblock();
    code.set_traceable(value);
    Ok(())
}

/// Executes a function with instruction-level tracing enabled.
///
/// Traces every bytecode instruction executed by the given function, logging
/// opcode, operands, accumulator values, and timing information to stdout.
/// Tracing is enabled only for the duration of this single call — the function
/// is not permanently marked as traceable.
///
/// The `this` value and additional arguments are forwarded to the traced function.
///
/// # Errors
///
/// Returns a `TypeError` if the first argument is not callable.
/// Propagates any error thrown by the traced function.
///
/// # Examples
///
/// ```ignore
/// const add = (a, b) => a + b;
/// let result = $boa.function.trace(add, undefined, 1, 2);
/// // Traces:
/// // 5μs  DefInitArg    0000: 'a'     2
/// // 4μs  DefInitArg    0001: 'b'     <empty>
/// // 3μs  GetName       0000: 'a'     1
/// // 1μs  GetName       0001: 'b'     2
/// // 2μs  Add                        3
/// // 1μs  Return                     3
/// // result === 3
/// ```
fn trace(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0);
    let this = args.get_or_undefined(1);

    let Some(callable) = value.as_callable() else {
        return Err(JsNativeError::typ()
            .with_message("expected callable object")
            .into());
    };

    let arguments = args.get(2..).unwrap_or(&[]);

    set_trace_flag_in_function_object(&callable, true)?;
    let result = callable.call(this, arguments, context);
    set_trace_flag_in_function_object(&callable, false)?;

    result
}

/// Marks a function as traceable across all future invocations.
///
/// Unlike `$boa.function.trace()` which runs a single traced call, `traceable`
/// permanently sets or clears the trace flag on the function's compiled code.
/// This is essential for tracing functions that suspend execution (async functions,
/// generators, async generators) where a single call may span multiple discrete
/// execution phases.
///
/// # Errors
///
/// Returns a `TypeError` if the first argument is not an ordinary function object.
///
/// # Examples
///
/// ```ignore
/// function* g() {
///     yield 1;
///     yield 2;
/// }
/// $boa.function.traceable(g, true);
/// let iter = g();
/// iter.next();  // traced
/// iter.next();  // traced
/// ```
fn traceable(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0);
    let traceable = args.get_or_undefined(1).to_boolean();

    let Some(callable) = value.as_callable() else {
        return Err(JsNativeError::typ()
            .with_message("expected callable object")
            .into());
    };

    set_trace_flag_in_function_object(&callable, traceable)?;

    Ok(value.clone())
}

pub(super) fn create_object(context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(flowgraph),
            js_string!("flowgraph"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(bytecode),
            js_string!("bytecode"),
            1,
        )
        .function(NativeFunction::from_fn_ptr(trace), js_string!("trace"), 1)
        .function(
            NativeFunction::from_fn_ptr(traceable),
            js_string!("traceable"),
            2,
        )
        .build()
}
