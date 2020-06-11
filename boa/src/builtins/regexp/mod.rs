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

use regex::Regex;

use super::function::{make_builtin_fn, make_constructor_fn};
use crate::{
    builtins::{
        object::{InternalState, ObjectData},
        property::Property,
        value::{ResultValue, Value, ValueData},
    },
    exec::Interpreter,
    BoaProfiler,
};

#[cfg(test)]
mod tests;

/// The internal representation on a `RegExp` object.
#[derive(Debug)]
pub(crate) struct RegExp {
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

impl RegExp {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "RegExp";

    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: i32 = 2;

    /// Create a new `RegExp`
    pub(crate) fn make_regexp(
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        if args.is_empty() {
            return Err(Value::undefined());
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
                let obj = obj.borrow();
                let slots = obj.internal_slots();
                if slots.get("RegExpMatcher").is_some() {
                    // first argument is another `RegExp` object, so copy its pattern and flags
                    if let Some(body) = slots.get("OriginalSource") {
                        regex_body = ctx.to_string(body)?;
                    }
                    if let Some(flags) = slots.get("OriginalFlags") {
                        regex_flags = ctx.to_string(flags)?;
                    }
                }
            }
            _ => return Err(Value::undefined()),
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
        this.set_data(ObjectData::Ordinary);
        this.set_internal_slot("RegExpMatcher", Value::undefined());
        this.set_internal_slot("OriginalSource", Value::from(regex_body));
        this.set_internal_slot("OriginalFlags", Value::from(regex_flags));

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
        this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.dot_all)))
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
        this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.flags.clone())))
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
        this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.global)))
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
        this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.ignore_case)))
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
        this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.multiline)))
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
        this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.sticky)))
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
        this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.unicode)))
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
    pub(crate) fn test(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let arg_str = ctx.to_string(args.get(0).expect("could not get argument"))?;
        let mut last_index = usize::from(&this.get_field("lastIndex"));
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
            Ok(Value::boolean(result))
        });
        this.set_field("lastIndex", Value::from(last_index));
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
    pub(crate) fn exec(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let arg_str = ctx.to_string(args.get(0).expect("could not get argument"))?;
        let mut last_index = usize::from(&this.get_field("lastIndex"));
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
                        result.push(Value::from(
                            arg_str.get(start..end).expect("Could not get slice"),
                        ));
                    } else {
                        result.push(Value::undefined());
                    }
                }

                let result = Value::from(result);
                result.set_property("index", Property::default().value(Value::from(m.start())));
                result.set_property("input", Property::default().value(Value::from(arg_str)));
                result
            } else {
                if regex.use_last_index {
                    last_index = 0;
                }
                Value::null()
            };
            Ok(result)
        });
        this.set_field("lastIndex", Value::from(last_index));
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
    pub(crate) fn r#match(this: &mut Value, arg: String, ctx: &mut Interpreter) -> ResultValue {
        let (matcher, flags) = this
            .with_internal_state_ref(|regex: &RegExp| (regex.matcher.clone(), regex.flags.clone()));
        if flags.contains('g') {
            let mut matches = Vec::new();
            for mat in matcher.find_iter(&arg) {
                matches.push(Value::from(mat.as_str()));
            }
            if matches.is_empty() {
                return Ok(Value::null());
            }
            Ok(Value::from(matches))
        } else {
            Self::exec(this, &[Value::from(arg)], ctx)
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
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let body = ctx.to_string(&this.get_internal_slot("OriginalSource"))?;
        let flags = this.with_internal_state_ref(|regex: &RegExp| regex.flags.clone());
        Ok(Value::from(format!("/{}/{}", body, flags)))
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
    pub(crate) fn match_all(this: &mut Value, arg_str: String) -> ResultValue {
        let matches: Vec<Value> = this.with_internal_state_ref(|regex: &RegExp| {
            let mut matches = Vec::new();

            for m in regex.matcher.find_iter(&arg_str) {
                if let Some(caps) = regex.matcher.captures(&m.as_str()) {
                    let match_vec = caps
                        .iter()
                        .map(|group| match group {
                            Some(g) => Value::from(g.as_str()),
                            None => Value::undefined(),
                        })
                        .collect::<Vec<Value>>();

                    let match_val = Value::from(match_vec);

                    match_val
                        .set_property("index", Property::default().value(Value::from(m.start())));
                    match_val.set_property(
                        "input",
                        Property::default().value(Value::from(arg_str.clone())),
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
        let result = Value::from(matches);
        result.set_field("length", Value::from(length));
        result.set_data(ObjectData::Array);

        Ok(result)
    }

    /// Create a new `RegExp` object.
    pub(crate) fn create(global: &Value) -> Value {
        // Create prototype
        let prototype = Value::new_object(Some(global));
        prototype
            .as_object_mut()
            .unwrap()
            .insert_field("lastIndex", Value::from(0));

        make_builtin_fn(Self::test, "test", &prototype, 1);
        make_builtin_fn(Self::exec, "exec", &prototype, 1);
        make_builtin_fn(Self::to_string, "toString", &prototype, 0);
        make_builtin_fn(Self::get_dot_all, "dotAll", &prototype, 0);
        make_builtin_fn(Self::get_flags, "flags", &prototype, 0);
        make_builtin_fn(Self::get_global, "global", &prototype, 0);
        make_builtin_fn(Self::get_ignore_case, "ignoreCase", &prototype, 0);
        make_builtin_fn(Self::get_multiline, "multiline", &prototype, 0);
        make_builtin_fn(Self::get_source, "source", &prototype, 0);
        make_builtin_fn(Self::get_sticky, "sticky", &prototype, 0);
        make_builtin_fn(Self::get_unicode, "unicode", &prototype, 0);

        make_constructor_fn(
            Self::NAME,
            Self::LENGTH,
            Self::make_regexp,
            global,
            prototype,
            true,
        )
    }

    /// Initialise the `RegExp` object on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) -> (&str, Value) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        (Self::NAME, Self::create(global))
    }
}
