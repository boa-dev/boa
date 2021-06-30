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
    object::{ConstructorBuilder, FunctionBuilder, GcObject, ObjectData, PROTOTYPE},
    property::{Attribute, DataDescriptor},
    symbol::WellKnownSymbols,
    value::{RcString, Value},
    BoaProfiler, Context, Result,
};
use regress::Regex;

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
    flags: Box<str>,

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

    pub(crate) original_source: Box<str>,
    original_flags: Box<str>,
}

// Only safe while regress::Regex doesn't implement Trace itself.
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

        let get_species = FunctionBuilder::new(context, Self::get_species)
            .name("get [Symbol.species]")
            .constructable(false)
            .callable(true)
            .build();

        let flag_attributes = Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE;

        let get_global = FunctionBuilder::new(context, Self::get_global)
            .name("get global")
            .constructable(false)
            .callable(true)
            .build();
        let get_ignore_case = FunctionBuilder::new(context, Self::get_ignore_case)
            .name("get ignoreCase")
            .constructable(false)
            .callable(true)
            .build();
        let get_multiline = FunctionBuilder::new(context, Self::get_multiline)
            .name("get multiline")
            .constructable(false)
            .callable(true)
            .build();
        let get_dot_all = FunctionBuilder::new(context, Self::get_dot_all)
            .name("get dotAll")
            .constructable(false)
            .callable(true)
            .build();
        let get_unicode = FunctionBuilder::new(context, Self::get_unicode)
            .name("get unicode")
            .constructable(false)
            .callable(true)
            .build();
        let get_sticky = FunctionBuilder::new(context, Self::get_sticky)
            .name("get sticky")
            .constructable(false)
            .callable(true)
            .build();
        let get_flags = FunctionBuilder::new(context, Self::get_flags)
            .name("get flags")
            .constructable(false)
            .callable(true)
            .build();

        let regexp_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().regexp_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .static_accessor(
            WellKnownSymbols::species(),
            Some(get_species),
            None,
            Attribute::CONFIGURABLE,
        )
        .property("lastIndex", 0, Attribute::all())
        .method(Self::test, "test", 1)
        .method(Self::exec, "exec", 1)
        .method(Self::to_string, "toString", 0)
        .method(
            Self::search,
            (WellKnownSymbols::search(), "[Symbol.search]"),
            1,
        )
        .accessor("global", Some(get_global), None, flag_attributes)
        .accessor("ignoreCase", Some(get_ignore_case), None, flag_attributes)
        .accessor("multiline", Some(get_multiline), None, flag_attributes)
        .accessor("dotAll", Some(get_dot_all), None, flag_attributes)
        .accessor("unicode", Some(get_unicode), None, flag_attributes)
        .accessor("sticky", Some(get_sticky), None, flag_attributes)
        .accessor("flags", Some(get_flags), None, flag_attributes)
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
    pub(crate) fn constructor(
        new_target: &Value,
        args: &[Value],
        ctx: &mut Context,
    ) -> Result<Value> {
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.get(&PROTOTYPE.into(), obj.clone().into(), ctx)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| ctx.standard_objects().regexp_object().prototype());
        let this = Value::new_object(ctx);

        this.as_object()
            .expect("this should be an object")
            .set_prototype_instance(prototype.into());
        let arg = args.get(0).ok_or_else(Value::undefined)?;

        let (regex_body, mut regex_flags) = match arg {
            Value::String(ref body) => {
                // first argument is a string -> use it as regex pattern
                (
                    body.to_string().into_boxed_str(),
                    String::new().into_boxed_str(),
                )
            }
            Value::Object(ref obj) => {
                let obj = obj.borrow();
                if let Some(regex) = obj.as_regexp() {
                    // first argument is another `RegExp` object, so copy its pattern and flags
                    (regex.original_source.clone(), regex.original_flags.clone())
                } else {
                    (
                        String::new().into_boxed_str(),
                        String::new().into_boxed_str(),
                    )
                }
            }
            _ => return Err(Value::undefined()),
        };
        // if a second argument is given and it's a string, use it as flags
        if let Some(Value::String(flags)) = args.get(1) {
            regex_flags = flags.to_string().into_boxed_str();
        }

        // parse flags
        let mut sorted_flags = String::new();
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
        }
        if regex_flags.contains('m') {
            multiline = true;
            sorted_flags.push('m');
        }
        if regex_flags.contains('s') {
            dot_all = true;
            sorted_flags.push('s');
        }
        if regex_flags.contains('u') {
            unicode = true;
            sorted_flags.push('u');
        }
        if regex_flags.contains('y') {
            sticky = true;
            sorted_flags.push('y');
        }

        let matcher = match Regex::with_flags(&regex_body, sorted_flags.as_str()) {
            Err(error) => {
                return Err(
                    ctx.construct_syntax_error(format!("failed to create matcher: {}", error.text))
                );
            }
            Ok(val) => val,
        };

        let regexp = RegExp {
            matcher,
            use_last_index: global || sticky,
            flags: sorted_flags.into_boxed_str(),
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

        Ok(this)
    }

    /// `get RegExp [ @@species ]`
    ///
    /// The `RegExp [ @@species ]` accessor property returns the RegExp constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-regexp-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/@@species
    fn get_species(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    #[inline]
    fn regexp_has_flag(this: &Value, flag: char, context: &mut Context) -> Result<Value> {
        if let Some(object) = this.as_object() {
            if let Some(regexp) = object.borrow().as_regexp() {
                return Ok(Value::boolean(match flag {
                    'g' => regexp.global,
                    'm' => regexp.multiline,
                    's' => regexp.dot_all,
                    'i' => regexp.ignore_case,
                    'u' => regexp.unicode,
                    'y' => regexp.sticky,
                    _ => unreachable!(),
                }));
            }

            if GcObject::equals(
                &object,
                &context.standard_objects().regexp_object().prototype,
            ) {
                return Ok(Value::undefined());
            }
        }

        let name = match flag {
            'g' => "global",
            'm' => "multiline",
            's' => "dotAll",
            'i' => "ignoreCase",
            'u' => "unicode",
            'y' => "sticky",
            _ => unreachable!(),
        };

        context.throw_type_error(format!(
            "RegExp.prototype.{} getter called on non-RegExp object",
            name
        ))
    }

    /// `get RegExp.prototype.global`
    ///
    /// The `global` property indicates whether or not the "`g`" flag is used with the regular expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.global
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/global
    pub(crate) fn get_global(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Self::regexp_has_flag(this, 'g', context)
    }

    /// `get RegExp.prototype.ignoreCase`
    ///
    /// The `ignoreCase` property indicates whether or not the "`i`" flag is used with the regular expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.ignorecase
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/ignoreCase
    pub(crate) fn get_ignore_case(
        this: &Value,
        _: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        Self::regexp_has_flag(this, 'i', context)
    }

    /// `get RegExp.prototype.multiline`
    ///
    /// The multiline property indicates whether or not the "m" flag is used with the regular expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.multiline
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/multiline
    pub(crate) fn get_multiline(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Self::regexp_has_flag(this, 'm', context)
    }

    /// `get RegExp.prototype.dotAll`
    ///
    /// The `dotAll` property indicates whether or not the "`s`" flag is used with the regular expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.dotAll
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/dotAll
    pub(crate) fn get_dot_all(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Self::regexp_has_flag(this, 's', context)
    }

    /// `get RegExp.prototype.unicode`
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
    pub(crate) fn get_unicode(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Self::regexp_has_flag(this, 'u', context)
    }

    /// `get RegExp.prototype.sticky`
    ///
    /// This flag indicates that it matches only from the index indicated by the `lastIndex` property
    /// of this regular expression in the target string (and does not attempt to match from any later indexes).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.sticky
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/sticky
    pub(crate) fn get_sticky(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Self::regexp_has_flag(this, 'y', context)
    }

    /// `get RegExp.prototype.flags`
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
    pub(crate) fn get_flags(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        if let Some(object) = this.as_object() {
            let mut result = String::new();
            if object
                .get(&"global".into(), this.clone(), context)?
                .to_boolean()
            {
                result.push('g');
            }
            if object
                .get(&"ignoreCase".into(), this.clone(), context)?
                .to_boolean()
            {
                result.push('i');
            }
            if object
                .get(&"multiline".into(), this.clone(), context)?
                .to_boolean()
            {
                result.push('m');
            }
            if object
                .get(&"dotAll".into(), this.clone(), context)?
                .to_boolean()
            {
                result.push('s');
            }
            if object
                .get(&"unicode".into(), this.clone(), context)?
                .to_boolean()
            {
                result.push('u');
            }
            if object
                .get(&"sticky".into(), this.clone(), context)?
                .to_boolean()
            {
                result.push('y');
            }

            return Ok(result.into());
        }

        context.throw_type_error("RegExp.prototype.flags getter called on non-object")
    }

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
    // pub(crate) fn get_source(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
    //     Ok(this.get_internal_slot("OriginalSource"))
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
    pub(crate) fn test(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 22.2.5.2.2.4 really says to use "toLength" and not "toIndex"
        let mut last_index = this.get_field("lastIndex", context)?.to_length(context)?;
        let result = if let Some(object) = this.as_object() {
            // 3. Let string be ? ToString(S).
            let arg_str = args
                .get(0)
                .cloned()
                .unwrap_or_default()
                .to_string(context)?;

            // 4. Let match be ? RegExpExec(R, string).
            let object = object.borrow();
            if let Some(regex) = object.as_regexp() {
                let result =
                    if let Some(m) = regex.matcher.find_from(arg_str.as_str(), last_index).next() {
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

                // 5. If match is not null, return true; else return false.
                Ok(Value::boolean(result))
            } else {
                return context
                    .throw_type_error("RegExp.prototype.exec method called on incompatible value");
            }
        } else {
            // 2. If Type(R) is not Object, throw a TypeError exception.
            return context
                .throw_type_error("RegExp.prototype.exec method called on incompatible value");
        };
        this.set_field("lastIndex", Value::from(last_index), true, context)?;
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
    pub(crate) fn exec(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 4. Return ? RegExpBuiltinExec(R, S).
        // 22.2.5.2.2.4 really says to use "toLength" and not "toIndex"
        let mut last_index = this.get_field("lastIndex", context)?.to_length(context)?;
        let result = if let Some(object) = this.as_object() {
            let object = object.borrow();
            if let Some(regex) = object.as_regexp() {
                // 3. Let S be ? ToString(string).
                let arg_str = args
                    .get(0)
                    .cloned()
                    .unwrap_or_default()
                    .to_string(context)?;

                let result = {
                    if last_index > arg_str.len() {
                        if regex.use_last_index {
                            last_index = 0;
                        }
                        Value::null()
                    } else if let Some(m) =
                        regex.matcher.find_from(arg_str.as_str(), last_index).next()
                    {
                        if regex.use_last_index {
                            last_index = m.end();
                        }
                        let groups = m.captures.len() + 1;
                        let mut result = Vec::with_capacity(groups);
                        for i in 0..groups {
                            if let Some(range) = m.group(i) {
                                result.push(Value::from(
                                    arg_str.get(range).expect("Could not get slice"),
                                ));
                            } else {
                                result.push(Value::undefined());
                            }
                        }

                        let result = Value::from(result);
                        result.set_property(
                            "index",
                            DataDescriptor::new(m.start(), Attribute::all()),
                        );
                        result
                            .set_property("input", DataDescriptor::new(arg_str, Attribute::all()));
                        result
                    } else {
                        if regex.use_last_index {
                            last_index = 0;
                        }
                        Value::null()
                    }
                };

                Ok(result)
            } else {
                // 2. Perform ? RequireInternalSlot(R, [[RegExpMatcher]]).
                context
                    .throw_type_error("RegExp.prototype.exec method called on incompatible value")
            }
        } else {
            return context.throw_type_error("exec method called on incompatible value");
        };

        this.set_field("lastIndex", Value::from(last_index), true, context)?;
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
    pub(crate) fn r#match(this: &Value, arg: RcString, context: &mut Context) -> Result<Value> {
        let (matcher, flags) = if let Some(object) = this.as_object() {
            let object = object.borrow();
            if let Some(regex) = object.as_regexp() {
                (regex.matcher.clone(), regex.flags.clone())
            } else {
                return context
                    .throw_type_error("RegExp.prototype.exec method called on incompatible value");
            }
        } else {
            return context
                .throw_type_error("RegExp.prototype.match method called on incompatible value");
        };
        if flags.contains('g') {
            let mut matches = Vec::new();
            for mat in matcher.find_iter(&arg) {
                matches.push(Value::from(&arg[mat.range()]));
            }
            if matches.is_empty() {
                return Ok(Value::null());
            }
            Ok(Value::from(matches))
        } else {
            Self::exec(this, &[Value::from(arg)], context)
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
    pub(crate) fn to_string(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        let (body, flags) = if let Some(object) = this.as_object() {
            let object = object.borrow();
            let regex = object.as_regexp().ok_or_else(|| {
                context.construct_type_error(format!(
                    "Method RegExp.prototype.toString called on incompatible receiver {}",
                    this.display()
                ))
            })?;
            (regex.original_source.clone(), regex.flags.clone())
        } else {
            return context.throw_type_error(format!(
                "Method RegExp.prototype.toString called on incompatible receiver {}",
                this.display()
            ));
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
    pub(crate) fn match_all(this: &Value, arg_str: String, context: &mut Context) -> Result<Value> {
        let matches = if let Some(object) = this.as_object() {
            let object = object.borrow();
            if let Some(regex) = object.as_regexp() {
                let mut matches = Vec::new();

                for mat in regex.matcher.find_iter(&arg_str) {
                    let match_vec: Vec<Value> = mat
                        .groups()
                        .map(|group| match group {
                            Some(range) => Value::from(&arg_str[range]),
                            None => Value::undefined(),
                        })
                        .collect();

                    let match_val = Value::from(match_vec);

                    match_val
                        .set_property("index", DataDescriptor::new(mat.start(), Attribute::all()));
                    match_val.set_property(
                        "input",
                        DataDescriptor::new(arg_str.clone(), Attribute::all()),
                    );
                    matches.push(match_val);

                    if !regex.flags.contains('g') {
                        break;
                    }
                }

                matches
            } else {
                return context.throw_type_error(
                    "RegExp.prototype.match_all method called on incompatible value",
                );
            }
        } else {
            return context.throw_type_error(
                "RegExp.prototype.match_all method called on incompatible value",
            );
        };

        let length = matches.len();
        let result = Value::from(matches);
        result.set_field("length", Value::from(length), false, context)?;
        result.set_data(ObjectData::Array);

        Ok(result)
    }

    /// `RegExp.prototype[ @@search ]( string )`
    ///
    /// This method executes a search for a match between a this regular expression and a string.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexp.prototype-@@search
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/@@search
    pub(crate) fn search(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let rx be the this value.
        // 2. If Type(rx) is not Object, throw a TypeError exception.
        if !this.is_object() {
            return context.throw_type_error(
                "RegExp.prototype[Symbol.search] method called on incompatible value",
            );
        }

        // 3. Let S be ? ToString(string).
        let arg_str = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?;

        // 4. Let previousLastIndex be ? Get(rx, "lastIndex").
        let previous_last_index = this.get_field("lastIndex", context)?.to_length(context)?;

        // 5. If SameValue(previousLastIndex, +0𝔽) is false, then
        if previous_last_index != 0 {
            // a. Perform ? Set(rx, "lastIndex", +0𝔽, true).
            this.set_field("lastIndex", 0, true, context)?;
        }

        // 6. Let result be ? RegExpExec(rx, S).
        let result = Self::exec(this, &[Value::from(arg_str)], context)?;

        // 7. Let currentLastIndex be ? Get(rx, "lastIndex").
        let current_last_index = this.get_field("lastIndex", context)?.to_length(context)?;

        // 8. If SameValue(currentLastIndex, previousLastIndex) is false, then
        if current_last_index != previous_last_index {
            // a. Perform ? Set(rx, "lastIndex", previousLastIndex, true).
            this.set_field("lastIndex", previous_last_index, true, context)?;
        }

        // 9. If result is null, return -1𝔽.
        // 10. Return ? Get(result, "index").
        if result.is_null() {
            Ok(Value::from(-1))
        } else {
            result
                .get_field("index", context)
                .map_err(|_| context.construct_type_error("Could not find property `index`"))
        }
    }
}
