use boa_engine::{
    builtins::function::OrdinaryFunction,
    js_string,
    object::{FunctionObjectBuilder, ObjectInitializer},
    optimizer::{
        control_flow_graph::{
            ControlFlowGraph, GraphEliminateUnreachableBasicBlocks, GraphSimplification,
        },
        OptimizerOptions,
    },
    property::Attribute,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};

fn get_constant_folding(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    Ok(context
        .optimizer_options()
        .contains(OptimizerOptions::CONSTANT_FOLDING)
        .into())
}

fn set_constant_folding(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_boolean();
    let mut options = context.optimizer_options();
    options.set(OptimizerOptions::CONSTANT_FOLDING, value);
    context.set_optimizer_options(options);
    Ok(JsValue::undefined())
}

fn get_statistics(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    Ok(context
        .optimizer_options()
        .contains(OptimizerOptions::STATISTICS)
        .into())
}

fn set_statistics(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_boolean();
    let mut options = context.optimizer_options();
    options.set(OptimizerOptions::STATISTICS, value);
    context.set_optimizer_options(options);
    Ok(JsValue::undefined())
}

fn graph(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
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
    let object = object.borrow();
    let Some(function) = object.downcast_ref::<OrdinaryFunction>() else {
        return Err(JsNativeError::typ()
            .with_message("expected function object")
            .into());
    };
    let code = function.codeblock();

    let cfg = ControlFlowGraph::generate(code.bytecode());
    // println!("{:#?}", cfg);

    let bytecode = cfg.finalize();
    assert_eq!(code.bytecode(), &bytecode);

    let mut cfg = ControlFlowGraph::generate(&bytecode);
    println!("Original\n{cfg:#?}\n");

    let changed = GraphSimplification::perform(&mut cfg);
    println!("Simplified({changed}) \n{cfg:#?}");

    let changed = GraphEliminateUnreachableBasicBlocks::perform(&mut cfg);
    println!("Eliminate Unreachble({changed}) \n{cfg:#?}");

    Ok(JsValue::undefined())
}

pub(super) fn create_object(context: &mut Context) -> JsObject {
    let get_constant_folding = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(get_constant_folding),
    )
    .name("get constantFolding")
    .length(0)
    .build();
    let set_constant_folding = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(set_constant_folding),
    )
    .name("set constantFolding")
    .length(1)
    .build();

    let get_statistics =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(get_statistics))
            .name("get statistics")
            .length(0)
            .build();
    let set_statistics =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(set_statistics))
            .name("set statistics")
            .length(1)
            .build();
    ObjectInitializer::new(context)
        .accessor(
            js_string!("constantFolding"),
            Some(get_constant_folding),
            Some(set_constant_folding),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .accessor(
            js_string!("statistics"),
            Some(get_statistics),
            Some(set_statistics),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .function(NativeFunction::from_fn_ptr(graph), js_string!("graph"), 1)
        .build()
}
