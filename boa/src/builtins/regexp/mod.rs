//! This module implements the global `RegExp` object.
//!
//! `The `RegExp` object is used for matching text with a pattern.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-regexp-constructor
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp

use std::ops::Deref;

use gc::Gc;
use regex::Regex;

use crate::{
    builtins::{
        object::{InternalState, Object, ObjectInternalMethods, ObjectKind, PROTOTYPE},
        property::Property,
        value::{from_value, to_value, undefined, FromValue, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};

#[cfg(test)]
mod tests;

/// The internal representation on a `RegExp` object.
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

/// Helper function for getting an argument.
fn get_argument<T: FromValue>(args: &[Value], idx: usize) -> Result<T, Value> {
    match args.get(idx) {
        Some(arg) => from_value(arg.clone()).map_err(to_value),
        None => Err(to_value(format!("expected argument at index {}", idx))),
    }
}

/// Create a new `RegExp`
pub fn make_regexp(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        return Err(undefined());
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
                    regex_body =
                        from_value(body.clone()).expect("Could not convert value to String");
                }
                if let Some(flags) = slots.get("OriginalFlags") {
                    regex_flags =
                        from_value(flags.clone()).expect("Could not convert value to String");
                }
            }
        }
        _ => return Err(undefined()),
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
    this.set_internal_slot("RegExpMatcher", undefined());
    this.set_internal_slot("OriginalSource", to_value(regex_body));
    this.set_internal_slot("OriginalFlags", to_value(regex_flags));

    this.set_internal_state(regexp);
    Ok(this.clone())
}

/// `RegExp.prototype.dotAll`
///
/// The `dotAll` property indicates whether or not the "`s`" flag is used with the regular expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.dotAll
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/dotAll
fn get_dot_all(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.dot_all)))
}

/// `RegExp.prototype.flags`
///
/// The `flags` property returns a string consisting of the [`flags`][flags] of the current regular expression object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.flags
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/flags
/// [flags]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_Expressions#Advanced_searching_with_flags_2
fn get_flags(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.flags.clone())))
}

/// `RegExp.prototype.global`
///
/// The `global` property indicates whether or not the "`g`" flag is used with the regular expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.global
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/global
fn get_global(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.global)))
}

/// `RegExp.prototype.ignoreCase`
///
/// The `ignoreCase` property indicates whether or not the "`i`" flag is used with the regular expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.ignorecase
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/ignoreCase
fn get_ignore_case(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.ignore_case)))
}

/// `RegExp.prototype.multiline`
///
/// The multiline property indicates whether or not the "m" flag is used with the regular expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.multiline
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/multiline
fn get_multiline(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.multiline)))
}

/// `RegExp.prototype.source`
///
/// The `source` property returns a `String` containing the source text of the regexp object,
/// and it doesn't contain the two forward slashes on both sides and any flags.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.source
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/source
fn get_source(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(this.get_internal_slot("OriginalSource"))
}

/// `RegExp.prototype.sticky`
///
/// The `flags` property returns a string consisting of the [`flags`][flags] of the current regular expression object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.sticky
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/sticky
fn get_sticky(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.sticky)))
}

/// `RegExp.prototype.unicode`
///
/// The unicode property indicates whether or not the "`u`" flag is used with a regular expression.
/// unicode is a read-only property of an individual regular expression instance.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.unicode
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/unicode
fn get_unicode(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|regex: &RegExp| Ok(to_value(regex.unicode)))
}

/// `RegExp.prototype.test( string )`
///
/// The `test()` method executes a search for a match between a regular expression and a specified string.
///
/// Returns `true` or `false`.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-regexp.prototype.test
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/test
pub fn test(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let arg_str = get_argument::<String>(args, 0)?;
    let mut last_index =
        from_value::<usize>(this.get_field_slice("lastIndex")).map_err(to_value)?;
    let result = this.with_internal_state_ref(|regex: &RegExp| {
        let result = if let Some(m) = regex.matcher.find_at(arg_str.as_str(), last_index) {
            if regex.use_last_index {
                last_index = m.end();
            }
            true
        } else {
            if regex.use_last_index {
                last_index = 0;
            }
            false
        };
        Ok(Gc::new(ValueData::Boolean(result)))
    });
    this.set_field_slice("lastIndex", to_value(last_index));
    result
}

/// `RegExp.prototype.exec( string )`
///
/// The exec() method executes a search for a match in a specified string.
///
/// Returns a result array, or `null`.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-regexp.prototype.exec
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/exec
pub fn exec(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let arg_str = get_argument::<String>(args, 0)?;
    let mut last_index =
        from_value::<usize>(this.get_field_slice("lastIndex")).map_err(to_value)?;
    let result = this.with_internal_state_ref(|regex: &RegExp| {
        let mut locations = regex.matcher.capture_locations();
        let result = if let Some(m) =
            regex
                .matcher
                .captures_read_at(&mut locations, arg_str.as_str(), last_index)
        {
            if regex.use_last_index {
                last_index = m.end();
            }
            let mut result = Vec::with_capacity(locations.len());
            for i in 0..locations.len() {
                if let Some((start, end)) = locations.get(i) {
                    result.push(to_value(
                        arg_str.get(start..end).expect("Could not get slice"),
                    ));
                } else {
                    result.push(undefined());
                }
            }
            let result = to_value(result);
            result.set_prop_slice("index", Property::default().value(to_value(m.start())));
            result.set_prop_slice("input", Property::default().value(to_value(arg_str)));
            result
        } else {
            if regex.use_last_index {
                last_index = 0;
            }
            Gc::new(ValueData::Null)
        };
        Ok(result)
    });
    this.set_field_slice("lastIndex", to_value(last_index));
    result
}

/// `RegExp.prototype[ @@match ]( string )`
///
/// This method retrieves the matches when matching a string against a regular expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-regexp.prototype-@@match
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/@@match
pub fn r#match(this: &mut Value, arg: String, ctx: &mut Interpreter) -> ResultValue {
    let (matcher, flags) =
        this.with_internal_state_ref(|regex: &RegExp| (regex.matcher.clone(), regex.flags.clone()));
    if flags.contains('g') {
        let mut matches = Vec::new();
        for mat in matcher.find_iter(&arg) {
            matches.push(to_value(mat.as_str()));
        }
        if matches.is_empty() {
            return Ok(Gc::new(ValueData::Null));
        }
        Ok(to_value(matches))
    } else {
        exec(this, &[to_value(arg)], ctx)
    }
}

/// `RegExp.prototype.toString()`
///
/// Return a string representing the regular expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-regexp.prototype.tostring
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/toString
pub fn to_string(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let body = from_value::<String>(this.get_internal_slot("OriginalSource")).map_err(to_value)?;
    let flags = this.with_internal_state_ref(|regex: &RegExp| regex.flags.clone());
    Ok(to_value(format!("/{}/{}", body, flags)))
}

/// `RegExp.prototype[ @@matchAll ]( string )`
///
/// The `[@@matchAll]` method returns all matches of the regular expression against a string.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-regexp-prototype-matchall
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/@@matchAll
// TODO: it's returning an array, it should return an iterator
pub fn match_all(this: &mut Value, arg_str: String) -> ResultValue {
    let matches: Vec<Value> = this.with_internal_state_ref(|regex: &RegExp| {
        let mut matches = Vec::new();

        for m in regex.matcher.find_iter(&arg_str) {
            if let Some(caps) = regex.matcher.captures(&m.as_str()) {
                let match_vec = caps
                    .iter()
                    .map(|group| match group {
                        Some(g) => to_value(g.as_str()),
                        None => undefined(),
                    })
                    .collect::<Vec<Value>>();

                let match_val = to_value(match_vec);

                match_val.set_prop_slice("index", Property::default().value(to_value(m.start())));
                match_val.set_prop_slice(
                    "input",
                    Property::default().value(to_value(arg_str.clone())),
                );
                matches.push(match_val);

                if !regex.flags.contains('g') {
                    break;
                }
            }
        }

        matches
    });

    let length = matches.len();
    let result = to_value(matches);
    result.set_field_slice("length", to_value(length));
    result.set_kind(ObjectKind::Array);

    Ok(result)
}

/// Create a new `RegExp` object.
pub fn create(global: &Value) -> Value {
    // Create prototype
    let prototype = ValueData::new_obj(Some(global));
    prototype.set_field_slice("lastIndex", to_value(0));

    make_builtin_fn!(test, named "test", with length 1, of prototype);
    make_builtin_fn!(exec, named "exec", with length 1, of prototype);
    make_builtin_fn!(to_string, named "toString", of prototype);
    make_builtin_fn!(get_dot_all, named "dotAll", of prototype);
    make_builtin_fn!(get_flags, named "flags", of prototype);
    make_builtin_fn!(get_global, named "global", of prototype);
    make_builtin_fn!(get_ignore_case, named "ignoreCase", of prototype);
    make_builtin_fn!(get_multiline, named "multiline", of prototype);
    make_builtin_fn!(get_source, named "source", of prototype);
    make_builtin_fn!(get_sticky, named "sticky", of prototype);
    make_builtin_fn!(get_unicode, named "unicode", of prototype);

    make_constructor_fn!(make_regexp, make_regexp, global, prototype)
}

/// Initialise the `RegExp` object on the global object.
#[inline]
pub fn init(global: &Value) {
    global.set_field_slice("RegExp", create(global));
}
