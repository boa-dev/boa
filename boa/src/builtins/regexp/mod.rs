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

use crate::{
    builtins::BuiltIn,
    gc::{empty_trace, Finalize, Trace},
    object::{ConstructorBuilder, ObjectData},
    property::{Attribute, Property},
    value::{RcString, Value},
    BoaProfiler, Context, Result,
};
use regex::Regex;

#[cfg(test)]
mod tests;

/// The internal representation on a `RegExp` object.
#[derive(Debug, Clone, Finalize)]
pub struct RegExp {
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

    pub(crate) original_source: String,
    original_flags: String,
}

unsafe impl Trace for RegExp {
    empty_trace!();
}

impl BuiltIn for RegExp {
    const NAME: &'static str = "RegExp";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let regexp_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().regexp_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .property("lastIndex", 0, Attribute::all())
        .method(Self::test, "test", 1)
        .method(Self::exec, "exec", 1)
        .method(Self::to_string, "toString", 0)
        .build();

        // TODO: add them RegExp accessor properties

        (Self::NAME, regexp_object.into(), Self::attribute())
    }
}

impl RegExp {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "RegExp";

    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 2;

    /// Create a new `RegExp`
    pub(crate) fn constructor(this: &Value, args: &[Value], _: &mut Context) -> Result<Value> {
        let arg = args.get(0).ok_or_else(Value::undefined)?;
        let mut regex_body = String::new();
        let mut regex_flags = String::new();
        match arg {
            Value::String(ref body) => {
                // first argument is a string -> use it as regex pattern
                regex_body = body.to_string();
            }
            Value::Object(ref obj) => {
                let obj = obj.borrow();
                if let Some(regex) = obj.as_regexp() {
                    // first argument is another `RegExp` object, so copy its pattern and flags
                    regex_body = regex.original_source.clone();
                    regex_flags = regex.original_flags.clone();
                }
            }
            _ => return Err(Value::undefined()),
        }
        // if a second argument is given and it's a string, use it as flags
        if let Some(Value::String(flags)) = args.get(1) {
            regex_flags = flags.to_string();
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
            original_source: regex_body,
            original_flags: regex_flags,
        };

        this.set_data(ObjectData::RegExp(Box::new(regexp)));

        Ok(this.clone())
    }

    // /// `RegExp.prototype.dotAll`
    // ///
    // /// The `dotAll` property indicates whether or not the "`s`" flag is used with the regular expression.
    // ///
    // /// More information:
    // ///  - [ECMAScript reference][spec]
    // ///  - [MDN documentation][mdn]
    // ///
    // /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.dotAll
    // /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/dotAll
    // fn get_dot_all(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
    //     this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.dot_all)))
    // }

    // /// `RegExp.prototype.flags`
    // ///
    // /// The `flags` property returns a string consisting of the [`flags`][flags] of the current regular expression object.
    // ///
    // /// More information:
    // ///  - [ECMAScript reference][spec]
    // ///  - [MDN documentation][mdn]
    // ///
    // /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.flags
    // /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/flags
    // /// [flags]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_Expressions#Advanced_searching_with_flags_2
    // fn get_flags(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
    //     this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.flags.clone())))
    // }

    // /// `RegExp.prototype.global`
    // ///
    // /// The `global` property indicates whether or not the "`g`" flag is used with the regular expression.
    // ///
    // /// More information:
    // ///  - [ECMAScript reference][spec]
    // ///  - [MDN documentation][mdn]
    // ///
    // /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.global
    // /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/global
    // fn get_global(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
    //     this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.global)))
    // }

    // /// `RegExp.prototype.ignoreCase`
    // ///
    // /// The `ignoreCase` property indicates whether or not the "`i`" flag is used with the regular expression.
    // ///
    // /// More information:
    // ///  - [ECMAScript reference][spec]
    // ///  - [MDN documentation][mdn]
    // ///
    // /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.ignorecase
    // /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/ignoreCase
    // fn get_ignore_case(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
    //     this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.ignore_case)))
    // }

    // /// `RegExp.prototype.multiline`
    // ///
    // /// The multiline property indicates whether or not the "m" flag is used with the regular expression.
    // ///
    // /// More information:
    // ///  - [ECMAScript reference][spec]
    // ///  - [MDN documentation][mdn]
    // ///
    // /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.multiline
    // /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/multiline
    // fn get_multiline(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
    //     this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.multiline)))
    // }

    // /// `RegExp.prototype.source`
    // ///
    // /// The `source` property returns a `String` containing the source text of the regexp object,
    // /// and it doesn't contain the two forward slashes on both sides and any flags.
    // ///
    // /// More information:
    // ///  - [ECMAScript reference][spec]
    // ///  - [MDN documentation][mdn]
    // ///
    // /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.source
    // /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/source
    // fn get_source(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
    //     Ok(this.get_internal_slot("OriginalSource"))
    // }

    // /// `RegExp.prototype.sticky`
    // ///
    // /// The `flags` property returns a string consisting of the [`flags`][flags] of the current regular expression object.
    // ///
    // /// More information:
    // ///  - [ECMAScript reference][spec]
    // ///  - [MDN documentation][mdn]
    // ///
    // /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.sticky
    // /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/sticky
    // fn get_sticky(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
    //     this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.sticky)))
    // }

    // /// `RegExp.prototype.unicode`
    // ///
    // /// The unicode property indicates whether or not the "`u`" flag is used with a regular expression.
    // /// unicode is a read-only property of an individual regular expression instance.
    // ///
    // /// More information:
    // ///  - [ECMAScript reference][spec]
    // ///  - [MDN documentation][mdn]
    // ///
    // /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.unicode
    // /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/unicode
    // fn get_unicode(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
    //     this.with_internal_state_ref(|regex: &RegExp| Ok(Value::from(regex.unicode)))
    // }

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
    pub(crate) fn test(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let arg_str = args
            .get(0)
            .expect("could not get argument")
            .to_string(ctx)?;
        let mut last_index = this.get_field("lastIndex").to_index(ctx)?;
        let result = if let Some(object) = this.as_object() {
            let regex = object.as_regexp().unwrap();
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
        } else {
            panic!("object is not a regexp")
        };
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
    pub(crate) fn exec(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let arg_str = args
            .get(0)
            .expect("could not get argument")
            .to_string(ctx)?;
        let mut last_index = this.get_field("lastIndex").to_index(ctx)?;
        let result = if let Some(object) = this.as_object() {
            let regex = object.as_regexp().unwrap();
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
        } else {
            panic!("object is not a regexp")
        };
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
    pub(crate) fn r#match(this: &Value, arg: RcString, ctx: &mut Context) -> Result<Value> {
        let (matcher, flags) = if let Some(object) = this.as_object() {
            let regex = object.as_regexp().unwrap();
            (regex.matcher.clone(), regex.flags.clone())
        } else {
            panic!("object is not a regexp")
        };
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
    pub(crate) fn to_string(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
        let (body, flags) = if let Some(object) = this.as_object() {
            let regex = object.as_regexp().unwrap();
            (regex.original_source.clone(), regex.flags.clone())
        } else {
            panic!("object is not an object")
        };
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
    pub(crate) fn match_all(this: &Value, arg_str: String) -> Result<Value> {
        let matches = if let Some(object) = this.as_object() {
            let regex = object.as_regexp().unwrap();
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
        } else {
            panic!("object is not a regexp")
        };

        let length = matches.len();
        let result = Value::from(matches);
        result.set_field("length", Value::from(length));
        result.set_data(ObjectData::Array);

        Ok(result)
    }
}
