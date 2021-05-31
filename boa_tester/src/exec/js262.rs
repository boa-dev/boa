use boa::{
    exec::Executable,
    object::{GcObject, ObjectInitializer},
    property::Attribute,
    Context, Result, Value,
};

/// Initializes the object in the context.
pub(super) fn init(context: &mut Context, agent: GcObject) -> GcObject {
    let global_obj = context.global_object();

    let obj = ObjectInitializer::new(context)
        .function(create_realm, "createRealm", 0)
        .function(eval_script, "evalScript", 1)
        .property("global", global_obj, Attribute::default())
        .property("agent", agent, Attribute::default())
        .build();

    context.register_global_property("$262", obj.clone(), Attribute::default());

    obj
}

/// The `$262.createRealm()` function.
///
/// Creates a new ECMAScript Realm, defines this API on the new realm's global object, and
/// returns the `$262` property of the new realm's global object.
fn create_realm(_this: &Value, _: &[Value], _context: &mut Context) -> Result<Value> {
    // eprintln!("called $262.createRealm()");

    let mut context = Context::new();

    // add the $262 object.
    let agent = super::agent::init(&mut context);
    let js_262 = init(&mut context, agent);

    Ok(Value::from(js_262))
}

/// The `$262.detachArrayBuffer()` function.
///
/// Implements the `DetachArrayBuffer` abstract operation.
#[allow(dead_code)]
fn detach_array_buffer(_this: &Value, _: &[Value], _context: &mut Context) -> Result<Value> {
    todo!()
}

/// The `$262.evalScript()` function.
///
/// Accepts a string value as its first argument and executes it as an ECMAScript script.
fn eval_script(_this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
    // eprintln!("called $262.evalScript()");

    if let Some(source_text) = args.get(0).and_then(|val| val.as_string()) {
        match boa::parse(source_text.as_str(), false) {
            // TODO: check strict
            Err(e) => context.throw_type_error(format!("Uncaught Syntax Error: {}", e)),
            Ok(script) => script.run(context),
        }
    } else {
        Ok(Value::undefined())
    }
}

/// The `$262.gc()` function.
///
/// Wraps the host's garbage collection invocation mechanism, if such a capability exists.
/// Must throw an exception if no capability exists. This is necessary for testing the
/// semantics of any feature that relies on garbage collection, e.g. the `WeakRef` API.
#[allow(dead_code)]
fn gc(_this: &Value, _: &[Value], _context: &mut Context) -> Result<Value> {
    todo!()
}
