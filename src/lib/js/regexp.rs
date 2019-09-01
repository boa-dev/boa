use crate::{
    exec::Interpreter,
    js::{
        function::NativeFunctionData,
        object::{ObjectKind, PROTOTYPE},
        value::{from_value, to_value, ResultValue, Value, ValueData},
    },
};
use gc::Gc;

/// Create a new `RegExp`
pub fn make_regexp(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        return Err(Gc::new(ValueData::Undefined));
    }
    let mut regex_body = String::new();
    let mut regex_flags = String::new();
    #[allow(clippy::indexing_slicing)] // length has been checked
    let body = (&args[0]).clone();
    match *body {
        ValueData::String(ref body) => {
            // first argument is a string -> use it as regex pattern
            regex_body = body.into();
        }
        ValueData::Object(ref obj) => {
            let slots = &*obj.borrow().internal_slots;
            if slots.get("RegExpMatcher").is_some() {
                // first argument is another `RegExp` object, so copy its pattern and flags
                if let Some(body) = slots.get("OriginalSource") {
                    regex_body = from_value(body.clone()).unwrap();
                }
                if let Some(flags) = slots.get("OriginalFlags") {
                    regex_flags = from_value(flags.clone()).unwrap();
                }
            }
        }
        _ => return Err(Gc::new(ValueData::Undefined)),
    }
    // if a second argument is given and it's a string, use it as flags
    match args.get(1) {
        None => {}
        Some(flags) => {
            if let ValueData::String(ref flags) = *flags.clone() {
                regex_flags = flags.into();
            }
        }
    }

    // TODO: we probably should parse the pattern here and store the parsed matcher in the `RegExp` somehow

    // This value is used by console.log and other routines to match Object type
    // to its Javascript Identifier (global constructor method name)
    this.set_kind(ObjectKind::Ordinary);
    this.set_internal_slot("RegExpMatcher", Gc::new(ValueData::Undefined));
    this.set_internal_slot("OriginalSource", to_value(regex_body));
    this.set_internal_slot("OriginalFlags", to_value(regex_flags));
    Ok(this.clone())
}

/// Search for a match between this regex and a specified string
pub fn test(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    // TODO: execute regex
    unimplemented!()
}

/// Create a new `RegExp` object
pub fn _create(global: &Value) -> Value {
    let regexp = to_value(make_regexp as NativeFunctionData);
    let proto = ValueData::new_obj(Some(global));
    proto.set_field_slice("test", to_value(test as NativeFunctionData));
    regexp.set_field_slice(PROTOTYPE, proto);
    regexp
}

/// Initialise the `RegExp` object on the global object
pub fn init(global: &Value) {
    global.set_field_slice("RegExp", _create(global));
}

#[cfg(test)]
mod tests {
    use crate::exec::Executor;
    use crate::forward;

    #[test]
    fn test() {
        let mut engine = Executor::new();
        let init = r#"
        let constructed = new RegExp("[0-9]+(\\.[0-9]+)?");
        let literal = /[0-9]+(\.[0-9]+)?/;
        let ctor_literal = new RegExp(/[0-9]+(\.[0-9]+)?/);
        "#;

        forward(&mut engine, init);
        let a = forward(&mut engine, "constructed.test('1.0')");
        assert_eq!(a, "true");
    }
}
