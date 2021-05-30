use boa::{object::ObjectInitializer, property::Attribute, Context, Result, Value};

/// The `$262.createRealm()` function.
///
/// Creates a new ECMAScript Realm, defines this API on the new realm's global object, and
/// returns the `$262` property of the new realm's global object.
#[allow(dead_code)]
fn create_realm(_this: &Value, _: &[Value], _context: &mut Context) -> Result<Value> {
    todo!()
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
#[allow(dead_code)]
fn eval_script(_this: &Value, _: &[Value], _context: &mut Context) -> Result<Value> {
    todo!()
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

/// The `$262.global()` function.
///
/// Returns a reference to the global object on which `$262` was initially defined.
fn global(_this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
    Ok(Value::from(context.global_object()))
}

/// Initializes the object in the context.
pub(super) fn init_262(context: &mut Context) {
    let object = ObjectInitializer::new(context)
        //.function(Self::create_realm, "createRealm", 0)
        //.function(Self::detach_array_buffer, "detachArrayBuffer", 0)
        //.function(Self::eval_script, "evalScript", 1)
        //.function(Self::gc, "gc", 0)
        .function(global, "global", 0)
        // .property(
        //     "IsHTMLDDA",
        //     is_html_dda,
        //     Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        // )
        // .property(
        //     "agent",
        //     agent,
        //     Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        // )
        .build();

    context.register_global_property(
        "$262",
        object,
        Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
    );
}
