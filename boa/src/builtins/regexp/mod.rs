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

pub mod regexp_string_iterator;

use crate::{
    builtins::{array::Array, BuiltIn},
    gc::{empty_trace, Finalize, Trace},
    object::{ConstructorBuilder, FunctionBuilder, GcObject, ObjectData, PROTOTYPE},
    property::{Attribute, DataDescriptor},
    symbol::WellKnownSymbols,
    value::{IntegerOrInfinity, Value},
    BoaProfiler, Context, JsString, Result,
};
use regexp_string_iterator::RegExpStringIterator;
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
        let get_source = FunctionBuilder::new(context, Self::get_source)
            .name("get source")
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
            Self::r#match,
            (WellKnownSymbols::match_(), "[Symbol.match]"),
            1,
        )
        .method(
            Self::match_all,
            (WellKnownSymbols::match_all(), "[Symbol.matchAll]"),
            1,
        )
        .method(
            Self::replace,
            (WellKnownSymbols::replace(), "[Symbol.replace]"),
            2,
        )
        .method(
            Self::search,
            (WellKnownSymbols::search(), "[Symbol.search]"),
            1,
        )
        .method(
            Self::split,
            (WellKnownSymbols::split(), "[Symbol.split]"),
            2,
        )
        .accessor("global", Some(get_global), None, flag_attributes)
        .accessor("ignoreCase", Some(get_ignore_case), None, flag_attributes)
        .accessor("multiline", Some(get_multiline), None, flag_attributes)
        .accessor("dotAll", Some(get_dot_all), None, flag_attributes)
        .accessor("unicode", Some(get_unicode), None, flag_attributes)
        .accessor("sticky", Some(get_sticky), None, flag_attributes)
        .accessor("flags", Some(get_flags), None, flag_attributes)
        .accessor("source", Some(get_source), None, flag_attributes)
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

    /// `get RegExp.prototype.source`
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
    pub(crate) fn get_source(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let R be the this value.
        // 2. If Type(R) is not Object, throw a TypeError exception.
        if let Some(object) = this.as_object() {
            let object = object.borrow();

            match object.as_regexp() {
                // 3. If R does not have an [[OriginalSource]] internal slot, then
                None => {
                    // a. If SameValue(R, %RegExp.prototype%) is true, return "(?:)".
                    // b. Otherwise, throw a TypeError exception.
                    if Value::same_value(
                        this,
                        &Value::from(context.standard_objects().regexp_object().prototype()),
                    ) {
                        Ok(Value::from("(?:)"))
                    } else {
                        context.throw_type_error(
                            "RegExp.prototype.source method called on incompatible value",
                        )
                    }
                }
                // 4. Assert: R has an [[OriginalFlags]] internal slot.
                Some(re) => {
                    // 5. Let src be R.[[OriginalSource]].
                    // 6. Let flags be R.[[OriginalFlags]].
                    // 7. Return EscapeRegExpPattern(src, flags).
                    RegExp::escape_pattern(&re.original_source, &re.original_flags)
                }
            }
        } else {
            context.throw_type_error("RegExp.prototype.source method called on incompatible value")
        }
    }

    /// `22.2.3.2.5 EscapeRegExpPattern ( P, F )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-escaperegexppattern
    fn escape_pattern(src: &str, _flags: &str) -> Result<Value> {
        if src.is_empty() {
            Ok(Value::from("(?:)"))
        } else {
            let mut s = String::from("");

            for c in src.chars() {
                match c {
                    '/' => s.push_str("\\/"),
                    '\n' => s.push_str("\\\\n"),
                    '\r' => s.push_str("\\\\r"),
                    _ => s.push(c),
                }
            }

            Ok(Value::from(s))
        }
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
    pub(crate) fn test(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let R be the this value.
        // 2. If Type(R) is not Object, throw a TypeError exception.
        if !this.is_object() {
            return context
                .throw_type_error("RegExp.prototype.test method called on incompatible value");
        }

        // 3. Let string be ? ToString(S).
        let arg_str = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?;

        // 4. Let match be ? RegExpExec(R, string).
        let m = Self::abstract_exec(this, arg_str, context)?;

        // 5. If match is not null, return true; else return false.
        if !m.is_null() {
            Ok(Value::Boolean(true))
        } else {
            Ok(Value::Boolean(false))
        }
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
        // 1. Let R be the this value.
        // 2. Perform ? RequireInternalSlot(R, [[RegExpMatcher]]).
        {
            let obj = this.as_object().unwrap_or_default();
            let obj = obj.borrow();
            obj.as_regexp().ok_or_else(|| {
                context.construct_type_error("RegExp.prototype.exec called with invalid value")
            })?;
        }

        // 3. Let S be ? ToString(string).
        let arg_str = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?;

        // 4. Return ? RegExpBuiltinExec(R, S).
        Self::abstract_builtin_exec(this, arg_str, context)
    }

    /// `22.2.5.2.1 RegExpExec ( R, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexpexec
    pub(crate) fn abstract_exec(
        this: &Value,
        input: JsString,
        context: &mut Context,
    ) -> Result<Value> {
        // 1. Assert: Type(R) is Object.
        let object = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("RegExpExec called with invalid value"))?;
        // 2. Assert: Type(S) is String.

        // 3. Let exec be ? Get(R, "exec").
        let exec = this.get_field("exec", context)?;

        // 4. If IsCallable(exec) is true, then
        if exec.is_function() {
            // a. Let result be ? Call(exec, R, « S »).
            let result = context.call(&exec, this, &[input.into()])?;

            // b. If Type(result) is neither Object nor Null, throw a TypeError exception.
            if !result.is_object() && !result.is_null() {
                return context.throw_type_error("regexp exec returned neither object nor null");
            }

            // c. Return result.
            return Ok(result);
        }

        // 5. Perform ? RequireInternalSlot(R, [[RegExpMatcher]]).
        object
            .borrow()
            .as_regexp()
            .ok_or_else(|| context.construct_type_error("RegExpExec called with invalid value"))?;

        // 6. Return ? RegExpBuiltinExec(R, S).
        Self::abstract_builtin_exec(this, input, context)
    }

    /// `22.2.5.2.2 RegExpBuiltinExec ( R, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexpbuiltinexec
    pub(crate) fn abstract_builtin_exec(
        this: &Value,
        input: JsString,
        context: &mut Context,
    ) -> Result<Value> {
        // 1. Assert: R is an initialized RegExp instance.
        let rx = {
            let obj = this.as_object().unwrap_or_default();
            let obj = obj.borrow();
            obj.as_regexp()
                .ok_or_else(|| {
                    context.construct_type_error("RegExpBuiltinExec called with invalid value")
                })?
                .clone()
        };

        // 2. Assert: Type(S) is String.

        // 3. Let length be the number of code units in S.
        // Regress only works with utf8. According to the spec we would use the utf16 encoded count.
        let length = input.chars().count();

        // 4. Let lastIndex be ℝ(? ToLength(? Get(R, "lastIndex"))).
        let mut last_index = this.get_field("lastIndex", context)?.to_length(context)?;

        // 5. Let flags be R.[[OriginalFlags]].
        let flags = rx.original_flags;

        // 6. If flags contains "g", let global be true; else let global be false.
        let global = flags.contains('g');

        // 7. If flags contains "y", let sticky be true; else let sticky be false.
        let sticky = flags.contains('y');

        // 8. If global is false and sticky is false, set lastIndex to 0.
        if !global && !sticky {
            last_index = 0;
        }

        // 9. Let matcher be R.[[RegExpMatcher]].
        let matcher = rx.matcher;

        // 10. If flags contains "u", let fullUnicode be true; else let fullUnicode be false.
        let unicode = flags.contains('u');

        // 11. Let matchSucceeded be false.
        // 12. Repeat, while matchSucceeded is false,
        let match_value = loop {
            // a. If lastIndex > length, then
            if last_index > length {
                // i. If global is true or sticky is true, then
                if global || sticky {
                    // 1. Perform ? Set(R, "lastIndex", +0𝔽, true).
                    this.set_field("lastIndex", 0, true, context)?;
                }

                // ii. Return null.
                return Ok(Value::null());
            }

            // b. Let r be matcher(S, lastIndex).
            // Check if last_index is a valid utf8 index into input.
            if input.get(last_index..).is_none() {
                return Ok(Value::null());
            }
            let r = matcher.find_from(&input, last_index).next();

            match r {
                // c. If r is failure, then
                None => {
                    // i. If sticky is true, then
                    if sticky {
                        // 1. Perform ? Set(R, "lastIndex", +0𝔽, true).
                        this.set_field("lastIndex", 0, true, context)?;

                        // 2. Return null.
                        return Ok(Value::null());
                    }

                    // ii. Set lastIndex to AdvanceStringIndex(S, lastIndex, fullUnicode).
                    last_index = advance_string_index(input.clone(), last_index, unicode);
                }

                Some(m) => {
                    // c. If r is failure, then
                    // d. Else,
                    if m.start() != last_index {
                        // i. If sticky is true, then
                        if sticky {
                            // 1. Perform ? Set(R, "lastIndex", +0𝔽, true).
                            this.set_field("lastIndex", 0, true, context)?;

                            // 2. Return null.
                            return Ok(Value::null());
                        }

                        // ii. Set lastIndex to AdvanceStringIndex(S, lastIndex, fullUnicode).
                        last_index = advance_string_index(input.clone(), last_index, unicode);
                    } else {
                        //i. Assert: r is a State.
                        //ii. Set matchSucceeded to true.
                        break m;
                    }
                }
            }
        };

        // 13. Let e be r's endIndex value.
        let mut e = match_value.end();

        // 14. If fullUnicode is true, then
        if unicode {
            // e is an index into the Input character list, derived from S, matched by matcher.
            // Let eUTF be the smallest index into S that corresponds to the character at element e of Input.
            // If e is greater than or equal to the number of elements in Input, then eUTF is the number of code units in S.
            // b. Set e to eUTF.
            // Regress only works with utf8. According to the spec we would use the utf16 encoded count.
            e = input.split_at(e).0.chars().count() + 1;
        }

        // 15. If global is true or sticky is true, then
        if global || sticky {
            // a. Perform ? Set(R, "lastIndex", 𝔽(e), true).
            this.set_field("lastIndex", e, true, context)?;
        }

        // 16. Let n be the number of elements in r's captures List. (This is the same value as 22.2.2.1's NcapturingParens.)
        // 17. Assert: n < 23^2 - 1.
        let n: u32 = match_value.captures.len() as u32;

        // 18. Let A be ! ArrayCreate(n + 1).
        // 19. Assert: The mathematical value of A's "length" property is n + 1.
        let a = Array::array_create(n + 1, None, context);

        // 20. Perform ! CreateDataPropertyOrThrow(A, "index", 𝔽(lastIndex)).
        a.set_property(
            "index",
            DataDescriptor::new(match_value.start(), Attribute::all()),
        );

        // 21. Perform ! CreateDataPropertyOrThrow(A, "input", S).
        a.set_property(
            "input",
            DataDescriptor::new(input.clone(), Attribute::all()),
        );

        // 22. Let matchedSubstr be the substring of S from lastIndex to e.
        let matched_substr = if let Some(s) = input.get(match_value.range()) {
            s
        } else {
            ""
        };

        // 23. Perform ! CreateDataPropertyOrThrow(A, "0", matchedSubstr).
        a.set_property("0", DataDescriptor::new(matched_substr, Attribute::all()));

        // TODO: named capture groups
        // 24. If R contains any GroupName, then
        //     a. Let groups be ! OrdinaryObjectCreate(null).
        // 25. Else,
        //     a. Let groups be undefined.
        let groups = Value::undefined();

        // 26. Perform ! CreateDataPropertyOrThrow(A, "groups", groups).
        a.set_property("groups", DataDescriptor::new(groups, Attribute::all()));

        // 27. For each integer i such that i ≥ 1 and i ≤ n, in ascending order, do
        for i in 1..=n {
            // a. Let captureI be ith element of r's captures List.
            let capture = match_value.group(i as usize);

            let captured_value = match capture {
                // b. If captureI is undefined, let capturedValue be undefined.
                None => Value::undefined(),
                // c. Else if fullUnicode is true, then
                // d. Else,
                Some(range) => {
                    if let Some(s) = input.get(range) {
                        s.into()
                    } else {
                        "".into()
                    }
                }
            };

            // e. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(i)), capturedValue).
            a.set_property(i, DataDescriptor::new(captured_value, Attribute::all()));

            // TODO: named capture groups
            // f. If the ith capture of R was defined with a GroupName, then
            // i. Let s be the CapturingGroupName of the corresponding RegExpIdentifierName.
            // ii. Perform ! CreateDataPropertyOrThrow(groups, s, capturedValue).
        }

        // 28. Return A.
        Ok(a)
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
    pub(crate) fn r#match(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let rx be the this value.
        // 2. If Type(rx) is not Object, throw a TypeError exception.
        if !this.is_object() {
            return context
                .throw_type_error("RegExp.prototype.match method called on incompatible value");
        }

        // 3. Let S be ? ToString(string).
        let arg_str = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?;

        // 4. Let global be ! ToBoolean(? Get(rx, "global")).
        let global = this.get_field("global", context)?.to_boolean();

        // 5. If global is false, then
        // 6. Else,
        if !global {
            // a. Return ? RegExpExec(rx, S).
            Self::abstract_exec(this, arg_str, context)
        } else {
            // a. Assert: global is true.

            // b. Let fullUnicode be ! ToBoolean(? Get(rx, "unicode")).
            let unicode = this.get_field("unicode", context)?.to_boolean();

            // c. Perform ? Set(rx, "lastIndex", +0𝔽, true).
            this.set_field("lastIndex", Value::from(0), true, context)?;

            // d. Let A be ! ArrayCreate(0).
            let a = Array::array_create(0, None, context);

            // e. Let n be 0.
            let mut n = 0;

            // f. Repeat,
            loop {
                // i. Let result be ? RegExpExec(rx, S).
                let result = Self::abstract_exec(this, arg_str.clone(), context)?;

                // ii. If result is null, then
                // iii. Else,
                if result.is_null() {
                    // 1. If n = 0, return null.
                    // 2. Return A.
                    if n == 0 {
                        return Ok(Value::null());
                    } else {
                        return Ok(a);
                    }
                } else {
                    // 1. Let matchStr be ? ToString(? Get(result, "0")).
                    let match_str = result.get_field("0", context)?.to_string(context)?;

                    // 2. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(n)), matchStr).
                    Array::add_to_array_object(&a, &[match_str.clone().into()], context)?;

                    // 3. If matchStr is the empty String, then
                    if match_str.is_empty() {
                        // a. Let thisIndex be ℝ(? ToLength(? Get(rx, "lastIndex"))).
                        let this_index =
                            this.get_field("lastIndex", context)?.to_length(context)?;

                        // b. Let nextIndex be AdvanceStringIndex(S, thisIndex, fullUnicode).
                        let next_index = advance_string_index(arg_str.clone(), this_index, unicode);

                        // c. Perform ? Set(rx, "lastIndex", 𝔽(nextIndex), true).
                        this.set_field("lastIndex", Value::from(next_index), true, context)?;
                    }

                    // 4. Set n to n + 1.
                    n += 1;
                }
            }
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
        Ok(format!("/{}/{}", body, flags).into())
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
    pub(crate) fn match_all(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let R be the this value.
        // 2. If Type(R) is not Object, throw a TypeError exception.
        if !this.is_object() {
            return context.throw_type_error(
                "RegExp.prototype.match_all method called on incompatible value",
            );
        }

        // 3. Let S be ? ToString(string).
        let arg_str = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?;

        // 4. Let C be ? SpeciesConstructor(R, %RegExp%).
        let c = this
            .as_object()
            .unwrap_or_default()
            .species_constructor(context.standard_objects().regexp_object().clone(), context)?;

        // 5. Let flags be ? ToString(? Get(R, "flags")).
        let flags = this.get_field("flags", context)?.to_string(context)?;

        // 6. Let matcher be ? Construct(C, « R, flags »).
        let matcher = RegExp::constructor(&c, &[this.clone(), flags.clone().into()], context)?;

        // 7. Let lastIndex be ? ToLength(? Get(R, "lastIndex")).
        let last_index = this.get_field("lastIndex", context)?.to_length(context)?;

        // 8. Perform ? Set(matcher, "lastIndex", lastIndex, true).
        matcher.set_field("lastIndex", last_index, true, context)?;

        // 9. If flags contains "g", let global be true.
        // 10. Else, let global be false.
        let global = flags.contains('g');

        // 11. If flags contains "u", let fullUnicode be true.
        // 12. Else, let fullUnicode be false.
        let unicode = flags.contains('u');

        // 13. Return ! CreateRegExpStringIterator(matcher, S, global, fullUnicode).
        RegExpStringIterator::create_regexp_string_iterator(
            &matcher, arg_str, global, unicode, context,
        )
    }

    /// `RegExp.prototype [ @@replace ] ( string, replaceValue )`
    ///
    /// The [@@replace]() method replaces some or all matches of a this pattern in a string by a replacement,
    /// and returns the result of the replacement as a new string.
    /// The replacement can be a string or a function to be called for each match.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexp.prototype-@@replace
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/@@replace
    pub(crate) fn replace(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let rx be the this value.
        // 2. If Type(rx) is not Object, throw a TypeError exception.
        if !this.is_object() {
            return context.throw_type_error(
                "RegExp.prototype[Symbol.replace] method called on incompatible value",
            );
        }

        // 3. Let S be ? ToString(string).
        let arg_str = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?;

        // 4. Let lengthS be the number of code unit elements in S.
        // Regress only works with utf8. According to the spec we would use the utf16 encoded count.
        let length_arg_str = arg_str.chars().count();

        // 5. Let functionalReplace be IsCallable(replaceValue).
        let replace_value = args.get(1).cloned().unwrap_or_default();
        let functional_replace = replace_value.is_function();

        // 6. If functionalReplace is false, then
        // a. Set replaceValue to ? ToString(replaceValue).

        // 7. Let global be ! ToBoolean(? Get(rx, "global")).
        let global = this.get_field("global", context)?.to_boolean();

        // 8. If global is true, then
        // a. Let fullUnicode be ! ToBoolean(? Get(rx, "unicode")).
        let unicode = this.get_field("unicode", context)?.to_boolean();
        if global {
            // b. Perform ? Set(rx, "lastIndex", +0𝔽, true).
            this.set_field("lastIndex", 0, true, context)?;
        }

        //  9. Let results be a new empty List.
        let mut results = Vec::new();

        // 10. Let done be false.
        // 11. Repeat, while done is false,
        loop {
            // a. Let result be ? RegExpExec(rx, S).
            let result = Self::abstract_exec(this, arg_str.clone(), context)?;

            // b. If result is null, set done to true.
            // c. Else,
            if result.is_null() {
                break;
            } else {
                // i. Append result to the end of results.
                results.push(result.clone());

                // ii. If global is false, set done to true.
                // iii. Else,
                if !global {
                    break;
                } else {
                    // 1. Let matchStr be ? ToString(? Get(result, "0")).
                    let match_str = result.get_field("0", context)?.to_string(context)?;

                    // 2. If matchStr is the empty String, then
                    if match_str.is_empty() {
                        // a. Let thisIndex be ℝ(? ToLength(? Get(rx, "lastIndex"))).
                        let this_index =
                            this.get_field("lastIndex", context)?.to_length(context)?;

                        // b. Let nextIndex be AdvanceStringIndex(S, thisIndex, fullUnicode).
                        let next_index = advance_string_index(arg_str.clone(), this_index, unicode);

                        // c. Perform ? Set(rx, "lastIndex", 𝔽(nextIndex), true).
                        this.set_field("lastIndex", Value::from(next_index), true, context)?;
                    }
                }
            }
        }

        // 12. Let accumulatedResult be the empty String.
        let mut accumulated_result = JsString::new("");

        // 13. Let nextSourcePosition be 0.
        let mut next_source_position = 0;

        // 14. For each element result of results, do
        for result in results {
            // a. Let resultLength be ? LengthOfArrayLike(result).
            let result_length = result.get_field("length", context)?.to_length(context)? as isize;

            // b. Let nCaptures be max(resultLength - 1, 0).
            let n_captures = std::cmp::max(result_length - 1, 0);

            // c. Let matched be ? ToString(? Get(result, "0")).
            let matched = result.get_field("0", context)?.to_string(context)?;

            // d. Let matchLength be the number of code units in matched.
            // Regress only works with utf8. According to the spec we would use the utf16 encoded count.
            let match_length = matched.chars().count();

            // e. Let position be ? ToIntegerOrInfinity(? Get(result, "index")).
            let position = result
                .get_field("index", context)?
                .to_integer_or_infinity(context)?;

            // f. Set position to the result of clamping position between 0 and lengthS.
            //position = position.
            let position = match position {
                IntegerOrInfinity::Integer(i) => {
                    if i < 0 {
                        0
                    } else if i as usize > length_arg_str {
                        length_arg_str
                    } else {
                        i as usize
                    }
                }
                IntegerOrInfinity::PositiveInfinity => length_arg_str,
                IntegerOrInfinity::NegativeInfinity => 0,
            };

            // h. Let captures be a new empty List.
            let mut captures = Vec::new();

            // g. Let n be 1.
            // i. Repeat, while n ≤ nCaptures,
            for n in 1..=n_captures {
                // i. Let capN be ? Get(result, ! ToString(𝔽(n))).
                let mut cap_n = result.get_field(n.to_string(), context)?;

                // ii. If capN is not undefined, then
                if !cap_n.is_undefined() {
                    // 1. Set capN to ? ToString(capN).
                    cap_n = cap_n.to_string(context)?.into();
                }

                // iii. Append capN as the last element of captures.
                captures.push(cap_n);

                // iv. Set n to n + 1.
            }

            // j. Let namedCaptures be ? Get(result, "groups").
            let mut named_captures = result.get_field("groups", context)?;

            // k. If functionalReplace is true, then
            // l. Else,
            let replacement: JsString;
            if functional_replace {
                // i. Let replacerArgs be « matched ».
                let mut replacer_args = vec![Value::from(matched)];

                // ii. Append in List order the elements of captures to the end of the List replacerArgs.
                replacer_args.extend(captures);

                // iii. Append 𝔽(position) and S to replacerArgs.
                replacer_args.push(position.into());
                replacer_args.push(arg_str.clone().into());

                // iv. If namedCaptures is not undefined, then
                if !named_captures.is_undefined() {
                    // 1. Append namedCaptures as the last element of replacerArgs.
                    replacer_args.push(named_captures);
                }

                // v. Let replValue be ? Call(replaceValue, undefined, replacerArgs).
                let repl_value = context.call(&replace_value, &Value::Undefined, &replacer_args)?;

                // vi. Let replacement be ? ToString(replValue).
                replacement = repl_value.to_string(context)?;
            } else {
                // i. If namedCaptures is not undefined, then
                if !named_captures.is_undefined() {
                    // 1. Set namedCaptures to ? ToObject(namedCaptures).
                    named_captures = named_captures.to_object(context)?.into();
                }

                // ii. Let replacement be ? GetSubstitution(matched, S, position, captures, namedCaptures, replaceValue).
                replacement = crate::builtins::string::get_substitution(
                    matched.to_string(),
                    arg_str.to_string(),
                    position,
                    captures,
                    named_captures,
                    replace_value.to_string(context)?.to_string(),
                )?;
            }

            // m. If position ≥ nextSourcePosition, then
            if position >= next_source_position {
                // i. NOTE: position should not normally move backwards.
                //    If it does, it is an indication of an ill-behaving RegExp subclass
                //    or use of an access triggered side-effect to change the global flag or other characteristics of rx.
                //    In such cases, the corresponding substitution is ignored.
                // ii. Set accumulatedResult to the string-concatenation of accumulatedResult,
                //     the substring of S from nextSourcePosition to position, and replacement.
                accumulated_result = format!(
                    "{}{}{}",
                    accumulated_result,
                    arg_str.get(next_source_position..position).unwrap(),
                    replacement
                )
                .into();

                // iii. Set nextSourcePosition to position + matchLength.
                next_source_position = position + match_length;
            }
        }

        // 15. If nextSourcePosition ≥ lengthS, return accumulatedResult.
        if next_source_position >= length_arg_str {
            return Ok(accumulated_result.into());
        }

        // 16. Return the string-concatenation of accumulatedResult and the substring of S from nextSourcePosition.
        Ok(format!(
            "{}{}",
            accumulated_result,
            arg_str.get(next_source_position..).unwrap()
        )
        .into())
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
        let result = Self::abstract_exec(this, arg_str, context)?;

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

    /// `RegExp.prototype [ @@split ] ( string, limit )`
    ///
    /// The [@@split]() method splits a String object into an array of strings by separating the string into substrings.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexp.prototype-@@split
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/@@split
    pub(crate) fn split(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let rx be the this value.
        // 2. If Type(rx) is not Object, throw a TypeError exception.
        if !this.is_object() {
            return context
                .throw_type_error("RegExp.prototype.split method called on incompatible value");
        }

        // 3. Let S be ? ToString(string).
        let arg_str = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?;

        // 4. Let C be ? SpeciesConstructor(rx, %RegExp%).
        let constructor = this
            .as_object()
            .unwrap_or_default()
            .species_constructor(context.standard_objects().regexp_object().clone(), context)?;

        // 5. Let flags be ? ToString(? Get(rx, "flags")).
        let flags = this.get_field("flags", context)?.to_string(context)?;

        // 6. If flags contains "u", let unicodeMatching be true.
        // 7. Else, let unicodeMatching be false.
        let unicode = flags.contains('u');

        // 8. If flags contains "y", let newFlags be flags.
        // 9. Else, let newFlags be the string-concatenation of flags and "y".
        let new_flags = if flags.contains('y') {
            flags.to_string()
        } else {
            format!("{}{}", flags, 'y')
        };

        // 10. Let splitter be ? Construct(C, « rx, newFlags »).
        let splitter =
            RegExp::constructor(&constructor, &[this.clone(), new_flags.into()], context)?;

        // 11. Let A be ! ArrayCreate(0).
        let a = Array::array_create(0, None, context);

        // 12. Let lengthA be 0.
        let mut length_a = 0;

        // 13. If limit is undefined, let lim be 2^32 - 1; else let lim be ℝ(? ToUint32(limit)).
        let limit = args.get(1).cloned().unwrap_or_default();
        let lim = if limit.is_undefined() {
            u32::MAX
        } else {
            limit.to_u32(context)?
        };

        // 14. If lim is 0, return A.
        if lim == 0 {
            return Ok(a);
        }

        // 15. Let size be the length of S.
        let size = arg_str.chars().count();

        // 16. If size is 0, then
        if size == 0 {
            // a. Let z be ? RegExpExec(splitter, S).
            let result = Self::abstract_exec(&splitter, arg_str.clone(), context)?;

            // b. If z is not null, return A.
            if !result.is_null() {
                return Ok(a);
            }

            // c. Perform ! CreateDataPropertyOrThrow(A, "0", S).
            Array::add_to_array_object(&a, &[Value::from(arg_str)], context)?;

            // d. Return A.
            return Ok(a);
        }

        // 17. Let p be 0.
        // 18. Let q be p.
        let mut p = 0;
        let mut q = p;

        // 19. Repeat, while q < size,
        while q < size {
            // a. Perform ? Set(splitter, "lastIndex", 𝔽(q), true).
            splitter.set_field("lastIndex", Value::from(q), true, context)?;

            // b. Let z be ? RegExpExec(splitter, S).
            let result = Self::abstract_exec(&splitter, arg_str.clone(), context)?;

            // c. If z is null, set q to AdvanceStringIndex(S, q, unicodeMatching).
            // d. Else,
            if result.is_null() {
                q = advance_string_index(arg_str.clone(), q, unicode);
            } else {
                // i. Let e be ℝ(? ToLength(? Get(splitter, "lastIndex"))).
                let mut e = splitter
                    .get_field("lastIndex", context)?
                    .to_length(context)?;

                // ii. Set e to min(e, size).
                e = std::cmp::min(e, size);

                // iii. If e = p, set q to AdvanceStringIndex(S, q, unicodeMatching).
                // iv. Else,
                if e == p {
                    q = advance_string_index(arg_str.clone(), q, unicode);
                } else {
                    // 1. Let T be the substring of S from p to q.
                    //let arg_str_substring = arg_str
                    //    .get(p..q)
                    //    .expect("invalid index into string to split");
                    let arg_str_substring: String = arg_str.chars().skip(p).take(q - p).collect();

                    // 2. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(lengthA)), T).
                    Array::add_to_array_object(&a, &[Value::from(arg_str_substring)], context)?;

                    // 3. Set lengthA to lengthA + 1.
                    length_a += 1;

                    // 4. If lengthA = lim, return A.
                    if length_a == lim {
                        return Ok(a);
                    }

                    // 5. Set p to e.
                    p = e;

                    // 6. Let numberOfCaptures be ? LengthOfArrayLike(z).
                    let mut number_of_captures =
                        result.get_field("length", context)?.to_length(context)?;

                    // 7. Set numberOfCaptures to max(numberOfCaptures - 1, 0).
                    number_of_captures = if number_of_captures == 0 {
                        0
                    } else {
                        std::cmp::max(number_of_captures - 1, 0)
                    };

                    // 8. Let i be 1.
                    // 9. Repeat, while i ≤ numberOfCaptures,
                    for i in 1..=number_of_captures {
                        // a. Let nextCapture be ? Get(z, ! ToString(𝔽(i))).
                        let next_capture = result.get_field(i.to_string(), context)?;

                        // b. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(lengthA)), nextCapture).
                        Array::add_to_array_object(&a, &[next_capture], context)?;

                        // d. Set lengthA to lengthA + 1.
                        length_a += 1;

                        // e. If lengthA = lim, return A.
                        if length_a == lim {
                            return Ok(a);
                        }
                    }

                    // 10. Set q to p.
                    q = p;
                }
            }
        }

        // 20. Let T be the substring of S from p to size.
        let arg_str_substring: String = arg_str.chars().skip(p).take(size - p).collect();

        // 21. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(lengthA)), T).
        Array::add_to_array_object(&a, &[Value::from(arg_str_substring)], context)?;

        // 22. Return A.
        Ok(a)
    }
}

/// `22.2.5.2.3 AdvanceStringIndex ( S, index, unicode )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-advancestringindex
fn advance_string_index(s: JsString, index: usize, unicode: bool) -> usize {
    // Regress only works with utf8, so this function differs from the spec.

    // 1. Assert: index ≤ 2^53 - 1.

    // 2. If unicode is false, return index + 1.
    if !unicode {
        return index + 1;
    }

    // 3. Let length be the number of code units in S.
    let length = s.chars().count();

    // 4. If index + 1 ≥ length, return index + 1.
    if index + 1 > length {
        return index + 1;
    }

    // 5. Let cp be ! CodePointAt(S, index).
    let offset = if let Some(c) = s.chars().nth(index) {
        c.len_utf8()
    } else {
        1
    };

    index + offset
}
