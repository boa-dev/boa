use std::ops::Deref;

use gc::Gc;
use regex::Regex;

use crate::{
    exec::Interpreter,
    js::{
        function::NativeFunctionData,
        object::{InternalState, ObjectKind, PROTOTYPE},
        property::Property,
        value::{from_value, to_value, FromValue, ResultValue, Value, ValueData},
    },
};

#[derive(Debug)]
struct RegExp {
    /// Regex matcher.
    matcher: Regex,
    /// Update last_index, set if global or sticky flags are set.
    use_last_index: bool,
    /// String of parsed flags.
    flags: String,
    /// Flag 's' - dot matches newline characters.
    dot_all: bool,
    /// Flag 'g'
    global: bool,
    /// Flag 'i' - ignore case.
    ignore_case: bool,
    /// Flag 'm' - '^' and '$' match beginning/end of line.
    multiline: bool,
    /// Flag 'y'
    sticky: bool,
    /// Flag 'u' - Unicode.
    unicode: bool,
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

    // parse flags
    let mut sorted_flags = String::new();
    let mut pattern = String::new();
    let mut dot_all = false;
    let mut global = false;
    let mut ignore_case = false;
    let mut multiline = false;
    let mut sticky = false;
    let mut unicode = false;
    if regex_flags.contains('g') {
        global = true;
        sorted_flags.push('g');
    }
    if regex_flags.contains('i') {
        ignore_case = true;
        sorted_flags.push('i');
        pattern.push('i');
    }
    if regex_flags.contains('m') {
        multiline = true;
        sorted_flags.push('m');
        pattern.push('m');
    }
    if regex_flags.contains('s') {
        dot_all = true;
        sorted_flags.push('s');
        pattern.push('s');
    }
    if regex_flags.contains('u') {
        unicode = true;
        sorted_flags.push('u');
        //pattern.push('s'); // rust uses utf-8 anyway
    }
    if regex_flags.contains('y') {
        sticky = true;
        sorted_flags.push('y');
    }
    // the `regex` crate uses '(?{flags})` inside the pattern to enable flags
    if !pattern.is_empty() {
        pattern = format!("(?{})", pattern);
    }
    pattern.push_str(regex_body.as_str());

    let matcher = Regex::new(pattern.as_str()).expect("failed to create matcher");
    let regexp = RegExp {
        matcher,
        use_last_index: global || sticky,
        flags: sorted_flags,
        dot_all,
        global,
        ignore_case,
        multiline,
        sticky,
        unicode,
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

fn get_dot_all(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.dot_all)))
}

fn get_flags(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.flags.clone())))
}

fn get_global(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.global)))
}

fn get_ignore_case(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.ignore_case)))
}

fn get_multiline(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.multiline)))
}

fn get_source(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(this.get_internal_slot("OriginalSource"))
}

fn get_sticky(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.sticky)))
}

fn get_unicode(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.unicode)))
}

fn _make_prop(getter: NativeFunctionData) -> Property {
    Property::default()
        .get(to_value(getter))
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

/// Search for a match between this regex and a specified string
pub fn exec(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let arg_str = get_argument::<String>(args, 0)?;
    let mut last_index = from_value::<usize>(this.get_field("lastIndex")).map_err(to_value)?;
    let result = this.with_internal_state_ref(|regex: &RegExp| {
        let mut locations = regex.matcher.capture_locations();
        let result =
            match regex
                .matcher
                .captures_read_at(&mut locations, arg_str.as_str(), last_index)
            {
                Some(m) => {
                    if regex.use_last_index {
                        last_index = m.end();
                    }
                    let mut result = Vec::with_capacity(locations.len());
                    for i in 0..locations.len() {
                        if let Some((start, end)) = locations.get(i) {
                            result.push(to_value(&arg_str[start..end]));
                        } else {
                            result.push(Gc::new(ValueData::Undefined));
                        }
                    }
                    let result = to_value(result);
                    result.set_prop_slice("index", Property::default().value(to_value(m.start())));
                    result.set_prop_slice("input", Property::default().value(to_value(arg_str)));
                    result
                }
                None => {
                    if regex.use_last_index {
                        last_index = 0;
                    }
                    Gc::new(ValueData::Null)
                }
            };
        Ok(result)
    });
    this.set_field_slice("lastIndex", to_value(last_index));
    result
}

/// Return a string representing the regular expression
pub fn to_string(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let body = from_value::<String>(this.get_internal_slot("OriginalSource")).map_err(to_value)?;
    let flags = this.with_internal_state_ref(|regex: &RegExp| regex.flags.clone());
    Ok(to_value(format!("/{}/{}", body, flags)))
}

/// Create a new `RegExp` object
pub fn _create(global: &Value) -> Value {
    let regexp = to_value(make_regexp as NativeFunctionData);
    let proto = ValueData::new_obj(Some(global));
    proto.set_field_slice("test", to_value(test as NativeFunctionData));
    proto.set_field_slice("exec", to_value(exec as NativeFunctionData));
    proto.set_field_slice("toString", to_value(to_string as NativeFunctionData));
    proto.set_field_slice("lastIndex", to_value(0));
    proto.set_prop_slice("dotAll", _make_prop(get_dot_all));
    proto.set_prop_slice("flags", _make_prop(get_flags));
    proto.set_prop_slice("global", _make_prop(get_global));
    proto.set_prop_slice("ignoreCase", _make_prop(get_ignore_case));
    proto.set_prop_slice("multiline", _make_prop(get_multiline));
    proto.set_prop_slice("source", _make_prop(get_source));
    proto.set_prop_slice("sticky", _make_prop(get_sticky));
    proto.set_prop_slice("unicode", _make_prop(get_unicode));
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

    // TODO: uncomment this test when property getters are supported

    //    #[test]
    //    fn test_flags() {
    //        let mut engine = Executor::new();
    //        let init = r#"
    //                var re_gi = /test/gi;
    //                var re_sm = /test/sm;
    //                "#;
    //
    //        forward(&mut engine, init);
    //        assert_eq!(forward(&mut engine, "re_gi.global"), "true");
    //        assert_eq!(forward(&mut engine, "re_gi.ignoreCase"), "true");
    //        assert_eq!(forward(&mut engine, "re_gi.multiline"), "false");
    //        assert_eq!(forward(&mut engine, "re_gi.dotAll"), "false");
    //        assert_eq!(forward(&mut engine, "re_gi.unicode"), "false");
    //        assert_eq!(forward(&mut engine, "re_gi.sticky"), "false");
    //        assert_eq!(forward(&mut engine, "re_gi.flags"), "gi");
    //
    //        assert_eq!(forward(&mut engine, "re_sm.global"), "false");
    //        assert_eq!(forward(&mut engine, "re_sm.ignoreCase"), "false");
    //        assert_eq!(forward(&mut engine, "re_sm.multiline"), "true");
    //        assert_eq!(forward(&mut engine, "re_sm.dotAll"), "true");
    //        assert_eq!(forward(&mut engine, "re_sm.unicode"), "false");
    //        assert_eq!(forward(&mut engine, "re_sm.sticky"), "false");
    //        assert_eq!(forward(&mut engine, "re_sm.flags"), "ms");
    //    }

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

    #[test]
    fn test_exec() {
        let mut engine = Executor::new();
        let init = r#"
        var re = /quick\s(brown).+?(jumps)/ig;
        var result = re.exec('The Quick Brown Fox Jumps Over The Lazy Dog');
        "#;

        forward(&mut engine, init);
        assert_eq!(forward(&mut engine, "result[0]"), "Quick Brown Fox Jumps");
        assert_eq!(forward(&mut engine, "result[1]"), "Brown");
        assert_eq!(forward(&mut engine, "result[2]"), "Jumps");
        assert_eq!(forward(&mut engine, "result.index"), "4");
        assert_eq!(
            forward(&mut engine, "result.input"),
            "The Quick Brown Fox Jumps Over The Lazy Dog"
        );
    }

    #[test]
    fn test_to_string() {
        let mut engine = Executor::new();

        assert_eq!(
            forward(&mut engine, "(new RegExp('a+b+c')).toString()"),
            "/a+b+c/"
        );
        assert_eq!(
            forward(&mut engine, "(new RegExp('bar', 'g')).toString()"),
            "/bar/g"
        );
        assert_eq!(
            forward(&mut engine, "(new RegExp('\\\\n', 'g')).toString()"),
            "/\\n/g"
        );
        assert_eq!(forward(&mut engine, "/\\n/g.toString()"), "/\\n/g");
    }
}
