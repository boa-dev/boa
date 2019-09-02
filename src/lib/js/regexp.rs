use std::ops::Deref;

use gc::Gc;
use regex::Regex;

use crate::{
    exec::Interpreter,
    js::{
        function::NativeFunctionData,
        object::{InternalState, ObjectKind, PROTOTYPE},
        value::{from_value, to_value, FromValue, ResultValue, Value, ValueData},
    },
};

#[derive(Debug)]
struct RegExp {
    matcher: Regex,
    use_last_index: bool,
}

impl InternalState for RegExp {}

fn get_argument<T: FromValue>(args: &[Value], idx: usize) -> Result<T, Value> {
    match args.get(idx) {
        Some(arg) => from_value(arg.clone()).map_err(to_value),
        None => Err(to_value(format!("expected argument at index {}", idx))),
    }
}

/// Create a new `RegExp`
pub fn make_regexp(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        return Err(Gc::new(ValueData::Undefined));
    }
    let mut regex_body = String::new();
    let mut regex_flags = String::new();
    #[allow(clippy::indexing_slicing)] // length has been checked
    match args[0].deref() {
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
            if let ValueData::String(flags) = flags.deref() {
                regex_flags = flags.into();
            }
        }
    }

    let matcher = Regex::new(regex_body.as_str()).expect("failed to create matcher");
    // use last index if the global or sticky flag are used
    let use_last_index = regex_flags.contains(|c| c == 'g' || c == 'y');
    let regexp = RegExp {
        matcher,
        use_last_index,
    };

    // This value is used by console.log and other routines to match Object type
    // to its Javascript Identifier (global constructor method name)
    this.set_kind(ObjectKind::Ordinary);
    this.set_internal_slot("RegExpMatcher", Gc::new(ValueData::Undefined));
    this.set_internal_slot("OriginalSource", to_value(regex_body));
    this.set_internal_slot("OriginalFlags", to_value(regex_flags));

    this.set_internal_state(regexp);
    Ok(this.clone())
}

/// Search for a match between this regex and a specified string
pub fn test(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let arg_str = get_argument::<String>(args, 0)?;
    let mut last_index = from_value::<usize>(this.get_field("lastIndex")).map_err(to_value)?;
    let result = this.with_internal_state_ref(|regex: &RegExp| {
        let result = match regex.matcher.find_at(arg_str.as_str(), last_index) {
            Some(m) => {
                if regex.use_last_index {
                    last_index = m.end();
                }
                true
            }
            None => {
                if regex.use_last_index {
                    last_index = 0;
                }
                false
            }
        };
        Ok(Gc::new(ValueData::Boolean(result)))
    });
    this.set_field_slice("lastIndex", to_value(last_index));
    result
}

/// Create a new `RegExp` object
pub fn _create(global: &Value) -> Value {
    let regexp = to_value(make_regexp as NativeFunctionData);
    let proto = ValueData::new_obj(Some(global));
    proto.set_field_slice("test", to_value(test as NativeFunctionData));
    proto.set_field_slice("lastIndex", to_value(0));
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
    fn test_constructors() {
        let mut engine = Executor::new();
        let init = r#"
        let constructed = new RegExp("[0-9]+(\\.[0-9]+)?");
        let literal = /[0-9]+(\.[0-9]+)?/;
        let ctor_literal = new RegExp(/[0-9]+(\.[0-9]+)?/);
        "#;

        forward(&mut engine, init);
        assert_eq!(forward(&mut engine, "constructed.test('1.0')"), "true");
        assert_eq!(forward(&mut engine, "literal.test('1.0')"), "true");
        assert_eq!(forward(&mut engine, "ctor_literal.test('1.0')"), "true");
    }

    #[test]
    fn test_last_index() {
        let mut engine = Executor::new();
        let init = r#"
        let regex = /[0-9]+(\.[0-9]+)?/g;
        "#;

        forward(&mut engine, init);
        assert_eq!(forward(&mut engine, "regex.lastIndex"), "0");
        assert_eq!(forward(&mut engine, "regex.test('1.0foo')"), "true");
        assert_eq!(forward(&mut engine, "regex.lastIndex"), "3");
        assert_eq!(forward(&mut engine, "regex.test('1.0foo')"), "false");
        assert_eq!(forward(&mut engine, "regex.lastIndex"), "0");
    }
}
