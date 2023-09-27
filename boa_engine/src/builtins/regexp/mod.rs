//! Boa's implementation of ECMAScript's global `RegExp` object.
//!
//! The `RegExp` object is used for matching text with a pattern.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-regexp-constructor
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp

use crate::{
    builtins::{array::Array, string, BuiltInObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{
        internal_methods::get_prototype_from_constructor, JsObject, Object, ObjectData, ObjectKind,
        CONSTRUCTOR,
    },
    property::{Attribute, PropertyDescriptorBuilder},
    realm::Realm,
    string::{common::StaticJsStrings, utf16, CodePoint},
    symbol::JsSymbol,
    value::JsValue,
    Context, JsArgs, JsResult, JsString,
};
use boa_parser::lexer::regex::RegExpFlags;
use boa_profiler::Profiler;
use regress::{Flags, Range, Regex};
use std::str::FromStr;

use super::{BuiltInBuilder, BuiltInConstructor, IntrinsicObject};

mod regexp_string_iterator;
pub(crate) use regexp_string_iterator::RegExpStringIterator;
#[cfg(test)]
mod tests;

/// The internal representation of a `RegExp` object.
#[derive(Debug, Clone)]
pub struct RegExp {
    /// Regex matcher.
    matcher: Regex,
    flags: RegExpFlags,
    original_source: JsString,
    original_flags: JsString,
}

impl IntrinsicObject for RegExp {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_species = BuiltInBuilder::callable(realm, Self::get_species)
            .name(js_string!("get [Symbol.species]"))
            .build();

        let flag_attributes = Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE;

        let get_has_indices = BuiltInBuilder::callable(realm, Self::get_has_indices)
            .name(js_string!("get hasIndices"))
            .build();
        let get_global = BuiltInBuilder::callable(realm, Self::get_global)
            .name(js_string!("get global"))
            .build();
        let get_ignore_case = BuiltInBuilder::callable(realm, Self::get_ignore_case)
            .name(js_string!("get ignoreCase"))
            .build();
        let get_multiline = BuiltInBuilder::callable(realm, Self::get_multiline)
            .name(js_string!("get multiline"))
            .build();
        let get_dot_all = BuiltInBuilder::callable(realm, Self::get_dot_all)
            .name(js_string!("get dotAll"))
            .build();
        let get_unicode = BuiltInBuilder::callable(realm, Self::get_unicode)
            .name(js_string!("get unicode"))
            .build();
        let get_sticky = BuiltInBuilder::callable(realm, Self::get_sticky)
            .name(js_string!("get sticky"))
            .build();
        let get_flags = BuiltInBuilder::callable(realm, Self::get_flags)
            .name(js_string!("get flags"))
            .build();
        let get_source = BuiltInBuilder::callable(realm, Self::get_source)
            .name(js_string!("get source"))
            .build();
        let regexp = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_accessor(
                JsSymbol::species(),
                Some(get_species),
                None,
                Attribute::CONFIGURABLE,
            )
            .property(js_string!("lastIndex"), 0, Attribute::all())
            .method(Self::test, js_string!("test"), 1)
            .method(Self::exec, js_string!("exec"), 1)
            .method(Self::to_string, js_string!("toString"), 0)
            .method(
                Self::r#match,
                (JsSymbol::r#match(), js_string!("[Symbol.match]")),
                1,
            )
            .method(
                Self::match_all,
                (JsSymbol::match_all(), js_string!("[Symbol.matchAll]")),
                1,
            )
            .method(
                Self::replace,
                (JsSymbol::replace(), js_string!("[Symbol.replace]")),
                2,
            )
            .method(
                Self::search,
                (JsSymbol::search(), js_string!("[Symbol.search]")),
                1,
            )
            .method(
                Self::split,
                (JsSymbol::split(), js_string!("[Symbol.split]")),
                2,
            )
            .accessor(
                js_string!("hasIndices"),
                Some(get_has_indices),
                None,
                flag_attributes,
            )
            .accessor(
                js_string!("global"),
                Some(get_global),
                None,
                flag_attributes,
            )
            .accessor(
                js_string!("ignoreCase"),
                Some(get_ignore_case),
                None,
                flag_attributes,
            )
            .accessor(
                js_string!("multiline"),
                Some(get_multiline),
                None,
                flag_attributes,
            )
            .accessor(
                js_string!("dotAll"),
                Some(get_dot_all),
                None,
                flag_attributes,
            )
            .accessor(
                js_string!("unicode"),
                Some(get_unicode),
                None,
                flag_attributes,
            )
            .accessor(
                js_string!("sticky"),
                Some(get_sticky),
                None,
                flag_attributes,
            )
            .accessor(js_string!("flags"), Some(get_flags), None, flag_attributes)
            .accessor(
                js_string!("source"),
                Some(get_source),
                None,
                flag_attributes,
            );

        #[cfg(feature = "annex-b")]
        let regexp = regexp.method(Self::compile, js_string!("compile"), 2);

        regexp.build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for RegExp {
    const NAME: JsString = StaticJsStrings::REG_EXP;
}

impl BuiltInConstructor for RegExp {
    const LENGTH: usize = 2;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::regexp;

    /// `22.2.3.1 RegExp ( pattern, flags )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexp-pattern-flags
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let pattern = args.get_or_undefined(0);
        let flags = args.get_or_undefined(1);

        // 1. Let patternIsRegExp be ? IsRegExp(pattern).
        let pattern_is_regexp = Self::is_reg_exp(pattern, context)?;

        // 2. If NewTarget is undefined, then
        // 3. Else, let newTarget be NewTarget.
        if new_target.is_undefined() {
            // a. Let newTarget be the active function object.
            let new_target = context
                .active_function_object()
                .map_or(JsValue::undefined(), JsValue::new);

            // b. If patternIsRegExp is true and flags is undefined, then
            if let Some(pattern) = pattern_is_regexp {
                if flags.is_undefined() {
                    // i. Let patternConstructor be ? Get(pattern, "constructor").
                    let pattern_constructor = pattern.get(CONSTRUCTOR, context)?;

                    // ii. If SameValue(newTarget, patternConstructor) is true, return pattern.
                    if JsValue::same_value(&new_target, &pattern_constructor) {
                        return Ok(pattern.clone().into());
                    }
                }
            }
        }

        // 4. If pattern is an Object and pattern has a [[RegExpMatcher]] internal slot, then
        let (p, f) = if let Some(pattern) = pattern
            .as_object()
            .map(JsObject::borrow)
            .as_deref()
            .and_then(Object::as_regexp)
        {
            // a. Let P be pattern.[[OriginalSource]].
            let p = pattern.original_source.clone().into();

            // b. If flags is undefined, let F be pattern.[[OriginalFlags]].
            let f = if flags.is_undefined() {
                pattern.original_flags.clone().into()
            // c. Else, let F be flags.
            } else {
                flags.clone()
            };

            (p, f)
        } else if let Some(pattern) = pattern_is_regexp {
            // a. Let P be ? Get(pattern, "source").
            let p = pattern.get(js_string!("source"), context)?;

            // b. If flags is undefined, then
            let f = if flags.is_undefined() {
                // i. Let F be ? Get(pattern, "flags").
                pattern.get(js_string!("flags"), context)?
            // c. Else,
            } else {
                // i. Let F be flags.
                flags.clone()
            };

            (p, f)
        // 6. Else,
        } else {
            // a. Let P be pattern.
            // b. Let F be flags.
            (pattern.clone(), flags.clone())
        };

        // 7. Let O be ? RegExpAlloc(newTarget).
        let o = Self::alloc(new_target, context)?;

        // 8.Return ? RegExpInitialize(O, P, F).
        Self::initialize(o, &p, &f, context)
    }
}

impl RegExp {
    /// `7.2.8 IsRegExp ( argument )`
    ///
    /// This modified to return the object if it's `true`, [`None`] otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isregexp
    pub(crate) fn is_reg_exp<'a>(
        argument: &'a JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<Option<&'a JsObject>> {
        // 1. If argument is not an Object, return false.
        let Some(argument) = argument.as_object() else {
            return Ok(None);
        };

        // 2. Let matcher be ? Get(argument, @@match).
        let matcher = argument.get(JsSymbol::r#match(), context)?;

        // 3. If matcher is not undefined, return ToBoolean(matcher).
        if !matcher.is_undefined() {
            return Ok(matcher.to_boolean().then_some(argument));
        }

        // 4. If argument has a [[RegExpMatcher]] internal slot, return true.
        if argument.is_regexp() {
            return Ok(Some(argument));
        }

        // 5. Return false.
        Ok(None)
    }

    /// `22.2.3.2.1 RegExpAlloc ( newTarget )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexpalloc
    pub(crate) fn alloc(new_target: &JsValue, context: &mut Context<'_>) -> JsResult<JsObject> {
        // 1. Let obj be ? OrdinaryCreateFromConstructor(newTarget, "%RegExp.prototype%", « [[RegExpMatcher]], [[OriginalSource]], [[OriginalFlags]] »).
        let proto =
            get_prototype_from_constructor(new_target, StandardConstructors::regexp, context)?;
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ObjectData::ordinary(),
        );

        // 2. Perform ! DefinePropertyOrThrow(obj, "lastIndex", PropertyDescriptor { [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: false }).
        obj.define_property_or_throw(
            utf16!("lastIndex"),
            PropertyDescriptorBuilder::new()
                .writable(true)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )
        .expect("this cannot fail per spec");

        // 3. Return obj.
        Ok(obj)
    }

    /// `22.2.3.2.2 RegExpInitialize ( obj, pattern, flags )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexpinitialize
    pub(crate) fn initialize(
        obj: JsObject,
        pattern: &JsValue,
        flags: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If pattern is undefined, let P be the empty String.
        // 2. Else, let P be ? ToString(pattern).
        let p = if pattern.is_undefined() {
            js_string!()
        } else {
            pattern.to_string(context)?
        };

        // 3. If flags is undefined, let F be the empty String.
        // 4. Else, let F be ? ToString(flags).
        let f = if flags.is_undefined() {
            js_string!()
        } else {
            flags.to_string(context)?
        };

        // 5. If F contains any code unit other than "g", "i", "m", "s", "u", or "y"
        //    or if it contains the same code unit more than once, throw a SyntaxError exception.
        // TODO: Should directly parse the JsString instead of converting to String
        let flags = match RegExpFlags::from_str(&f.to_std_string_escaped()) {
            Err(msg) => return Err(JsNativeError::syntax().with_message(msg).into()),
            Ok(result) => result,
        };

        // 10. If u is true, then
        //     a. Let patternText be StringToCodePoints(P).
        // 11. Else,
        //     a. Let patternText be the result of interpreting each of P's 16-bit elements as a Unicode BMP code point. UTF-16 decoding is not applied to the elements.
        // 12. Let parseResult be ParsePattern(patternText, u).
        // 13. If parseResult is a non-empty List of SyntaxError objects, throw a SyntaxError exception.
        // 14. Assert: parseResult is a Pattern Parse Node.
        // 15. Set obj.[[OriginalSource]] to P.
        // 16. Set obj.[[OriginalFlags]] to F.
        // 17. Let capturingGroupsCount be CountLeftCapturingParensWithin(parseResult).
        // 18. Let rer be the RegExp Record { [[IgnoreCase]]: i, [[Multiline]]: m, [[DotAll]]: s, [[Unicode]]: u, [[CapturingGroupsCount]]: capturingGroupsCount }.
        // 19. Set obj.[[RegExpRecord]] to rer.
        // 20. Set obj.[[RegExpMatcher]] to CompilePattern of parseResult with argument rer.
        let matcher =
            match Regex::from_unicode(p.code_points().map(CodePoint::as_u32), Flags::from(flags)) {
                Err(error) => {
                    return Err(JsNativeError::syntax()
                        .with_message(format!("failed to create matcher: {}", error.text))
                        .into());
                }
                Ok(val) => val,
            };

        let regexp = Self {
            matcher,
            flags,
            original_source: p,
            original_flags: f,
        };

        // Safe to directly initialize since previous assertions ensure `obj` is a `Regexp` object.
        *obj.borrow_mut().kind_mut() = ObjectKind::RegExp(Box::new(regexp));

        // 16. Perform ? Set(obj, "lastIndex", +0𝔽, true).
        obj.set(utf16!("lastIndex"), 0, true, context)?;

        // 16. Return obj.
        Ok(obj.into())
    }

    /// `22.2.3.2.4 RegExpCreate ( P, F )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexpcreate
    pub(crate) fn create(p: &JsValue, f: &JsValue, context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let obj be ? RegExpAlloc(%RegExp%).
        let obj = Self::alloc(&context.global_object().get(Self::NAME, context)?, context)?;

        // 2. Return ? RegExpInitialize(obj, P, F).
        Self::initialize(obj, p, f, context)
    }

    /// `get RegExp [ @@species ]`
    ///
    /// The `RegExp [ @@species ]` accessor property returns the `RegExp` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-regexp-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    fn regexp_has_flag(this: &JsValue, flag: u8, context: &mut Context<'_>) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(regexp) = object.borrow().as_regexp() {
                return Ok(JsValue::new(match flag {
                    b'd' => regexp.flags.contains(RegExpFlags::HAS_INDICES),
                    b'g' => regexp.flags.contains(RegExpFlags::GLOBAL),
                    b'm' => regexp.flags.contains(RegExpFlags::MULTILINE),
                    b's' => regexp.flags.contains(RegExpFlags::DOT_ALL),
                    b'i' => regexp.flags.contains(RegExpFlags::IGNORE_CASE),
                    b'u' => regexp.flags.contains(RegExpFlags::UNICODE),
                    b'y' => regexp.flags.contains(RegExpFlags::STICKY),
                    _ => unreachable!(),
                }));
            }

            if JsObject::equals(
                object,
                &context.intrinsics().constructors().regexp().prototype(),
            ) {
                return Ok(JsValue::undefined());
            }
        }

        let name = match flag {
            b'd' => "hasIndices",
            b'g' => "global",
            b'm' => "multiline",
            b's' => "dotAll",
            b'i' => "ignoreCase",
            b'u' => "unicode",
            b'y' => "sticky",
            _ => unreachable!(),
        };

        Err(JsNativeError::typ()
            .with_message(format!(
                "RegExp.prototype.{name} getter called on non-RegExp object",
            ))
            .into())
    }

    /// `get RegExp.prototype.hasIndices`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-regexp.prototype.hasindices
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/global
    pub(crate) fn get_has_indices(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Self::regexp_has_flag(this, b'd', context)
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
    pub(crate) fn get_global(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Self::regexp_has_flag(this, b'g', context)
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
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Self::regexp_has_flag(this, b'i', context)
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
    pub(crate) fn get_multiline(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Self::regexp_has_flag(this, b'm', context)
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
    pub(crate) fn get_dot_all(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Self::regexp_has_flag(this, b's', context)
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
    pub(crate) fn get_unicode(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Self::regexp_has_flag(this, b'u', context)
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
    pub(crate) fn get_sticky(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Self::regexp_has_flag(this, b'y', context)
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
    pub(crate) fn get_flags(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let R be the this value.
        // 2. If Type(R) is not Object, throw a TypeError exception.
        if let Some(object) = this.as_object() {
            // 3. Let result be the empty String.
            let mut result = String::new();

            // 4. Let hasIndices be ToBoolean(? Get(R, "hasIndices")).
            // 5. If hasIndices is true, append the code unit 0x0064 (LATIN SMALL LETTER D) as the last code unit of result.
            if object.get(utf16!("hasIndices"), context)?.to_boolean() {
                result.push('d');
            }

            // 6. Let global be ! ToBoolean(? Get(R, "global")).
            // 7. If global is true, append the code unit 0x0067 (LATIN SMALL LETTER G) as the last code unit of result.
            if object.get(utf16!("global"), context)?.to_boolean() {
                result.push('g');
            }
            // 8. Let ignoreCase be ! ToBoolean(? Get(R, "ignoreCase")).
            // 9. If ignoreCase is true, append the code unit 0x0069 (LATIN SMALL LETTER I) as the last code unit of result.
            if object.get(utf16!("ignoreCase"), context)?.to_boolean() {
                result.push('i');
            }

            // 10. Let multiline be ! ToBoolean(? Get(R, "multiline")).
            // 11. If multiline is true, append the code unit 0x006D (LATIN SMALL LETTER M) as the last code unit of result.
            if object.get(utf16!("multiline"), context)?.to_boolean() {
                result.push('m');
            }

            // 12. Let dotAll be ! ToBoolean(? Get(R, "dotAll")).
            // 13. If dotAll is true, append the code unit 0x0073 (LATIN SMALL LETTER S) as the last code unit of result.
            if object.get(utf16!("dotAll"), context)?.to_boolean() {
                result.push('s');
            }
            // 14. Let unicode be ! ToBoolean(? Get(R, "unicode")).
            // 15. If unicode is true, append the code unit 0x0075 (LATIN SMALL LETTER U) as the last code unit of result.
            if object.get(utf16!("unicode"), context)?.to_boolean() {
                result.push('u');
            }

            // 16. Let sticky be ! ToBoolean(? Get(R, "sticky")).
            // 17. If sticky is true, append the code unit 0x0079 (LATIN SMALL LETTER Y) as the last code unit of result.
            if object.get(utf16!("sticky"), context)?.to_boolean() {
                result.push('y');
            }

            // 18. Return result.
            return Ok(js_string!(result).into());
        }

        Err(JsNativeError::typ()
            .with_message("RegExp.prototype.flags getter called on non-object")
            .into())
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
    pub(crate) fn get_source(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let R be the this value.
        // 2. If Type(R) is not Object, throw a TypeError exception.
        let Some(object) = this.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("RegExp.prototype.source method called on incompatible value")
                .into());
        };

        let object = object.borrow();

        match object.as_regexp() {
            // 3. If R does not have an [[OriginalSource]] internal slot, then
            None => {
                // a. If SameValue(R, %RegExp.prototype%) is true, return "(?:)".
                // b. Otherwise, throw a TypeError exception.
                if JsValue::same_value(
                    this,
                    &JsValue::new(context.intrinsics().constructors().regexp().prototype()),
                ) {
                    Ok(JsValue::new(js_string!("(?:)")))
                } else {
                    Err(JsNativeError::typ()
                        .with_message("RegExp.prototype.source method called on incompatible value")
                        .into())
                }
            }
            // 4. Assert: R has an [[OriginalFlags]] internal slot.
            Some(re) => {
                // 5. Let src be R.[[OriginalSource]].
                // 6. Let flags be R.[[OriginalFlags]].
                // 7. Return EscapeRegExpPattern(src, flags).
                Ok(Self::escape_pattern(
                    &re.original_source,
                    &re.original_flags,
                ))
            }
        }
    }

    /// `22.2.3.2.5 EscapeRegExpPattern ( P, F )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-escaperegexppattern
    fn escape_pattern(src: &JsString, _flags: &JsString) -> JsValue {
        if src.is_empty() {
            js_string!("(?:)").into()
        } else {
            let mut s = Vec::with_capacity(src.len());
            let mut buf = [0; 2];
            for c in src.code_points() {
                match c {
                    CodePoint::Unicode('/') => s.extend_from_slice(utf16!(r"\/")),
                    CodePoint::Unicode('\n') => s.extend_from_slice(utf16!(r"\\n")),
                    CodePoint::Unicode('\r') => s.extend_from_slice(utf16!(r"\\r")),
                    CodePoint::Unicode(c) => s.extend_from_slice(c.encode_utf16(&mut buf)),
                    CodePoint::UnpairedSurrogate(surr) => s.push(surr),
                }
            }

            JsValue::new(js_string!(&s[..]))
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
    pub(crate) fn test(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let R be the this value.
        // 2. If Type(R) is not Object, throw a TypeError exception.
        let this = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("RegExp.prototype.test method called on incompatible value")
        })?;

        // 3. Let string be ? ToString(S).
        let arg_str = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?;

        // 4. Let match be ? RegExpExec(R, string).
        let m = Self::abstract_exec(this, arg_str, context)?;

        // 5. If match is not null, return true; else return false.
        if m.is_some() {
            Ok(JsValue::new(true))
        } else {
            Ok(JsValue::new(false))
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
    pub(crate) fn exec(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let R be the this value.
        // 2. Perform ? RequireInternalSlot(R, [[RegExpMatcher]]).
        let obj = this
            .as_object()
            .filter(|obj| obj.is_regexp())
            .ok_or_else(|| {
                JsNativeError::typ().with_message("RegExp.prototype.exec called with invalid value")
            })?;

        // 3. Let S be ? ToString(string).
        let arg_str = args.get_or_undefined(0).to_string(context)?;

        // 4. Return ? RegExpBuiltinExec(R, S).
        (Self::abstract_builtin_exec(obj, &arg_str, context)?)
            .map_or_else(|| Ok(JsValue::null()), |v| Ok(v.into()))
    }

    /// `22.2.5.2.1 RegExpExec ( R, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexpexec
    pub(crate) fn abstract_exec(
        this: &JsObject,
        input: JsString,
        context: &mut Context<'_>,
    ) -> JsResult<Option<JsObject>> {
        // 1. Assert: Type(R) is Object.
        // 2. Assert: Type(S) is String.

        // 3. Let exec be ? Get(R, "exec").
        let exec = this.get(utf16!("exec"), context)?;

        // 4. If IsCallable(exec) is true, then
        if let Some(exec) = exec.as_callable() {
            // a. Let result be ? Call(exec, R, « S »).
            let result = exec.call(&this.clone().into(), &[input.into()], context)?;

            // b. If Type(result) is neither Object nor Null, throw a TypeError exception.
            if !result.is_object() && !result.is_null() {
                return Err(JsNativeError::typ()
                    .with_message("regexp exec returned neither object nor null")
                    .into());
            }

            // c. Return result.
            return Ok(result.as_object().cloned());
        }

        // 5. Perform ? RequireInternalSlot(R, [[RegExpMatcher]]).
        if !this.is_regexp() {
            return Err(JsNativeError::typ()
                .with_message("RegExpExec called with invalid value")
                .into());
        }

        // 6. Return ? RegExpBuiltinExec(R, S).
        Self::abstract_builtin_exec(this, &input, context)
    }

    /// `22.2.7.2 RegExpBuiltinExec ( R, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexpbuiltinexec
    pub(crate) fn abstract_builtin_exec(
        this: &JsObject,
        input: &JsString,
        context: &mut Context<'_>,
    ) -> JsResult<Option<JsObject>> {
        let rx = {
            let obj = this.borrow();
            if let Some(rx) = obj.as_regexp() {
                rx.clone()
            } else {
                return Err(JsNativeError::typ()
                    .with_message("RegExpBuiltinExec called with invalid value")
                    .into());
            }
        };

        // 1. Let length be the length of S.
        let length = input.len() as u64;

        // 2. Let lastIndex be ℝ(? ToLength(? Get(R, "lastIndex"))).
        let mut last_index = this.get(utf16!("lastIndex"), context)?.to_length(context)?;

        // 3. Let flags be R.[[OriginalFlags]].
        let flags = &rx.original_flags;

        // 4. If flags contains "g", let global be true; else let global be false.
        let global = flags.contains(&('g' as u16));

        // 5. If flags contains "y", let sticky be true; else let sticky be false.
        let sticky = flags.contains(&('y' as u16));

        // 6. If flags contains "d", let hasIndices be true; else let hasIndices be false.
        let has_indices = flags.contains(&('d' as u16));

        // 7. If global is false and sticky is false, set lastIndex to 0.
        if !global && !sticky {
            last_index = 0;
        }

        // 8. Let matcher be R.[[RegExpMatcher]].
        let matcher = &rx.matcher;

        // 9. If flags contains "u" or flags contains "v", let fullUnicode be true; else let fullUnicode be false.
        let full_unicode = flags.contains(&('u' as u16)) || flags.contains(&('v' as u16));

        // TODO:
        // 11. If fullUnicode is true, let input be StringToCodePoints(S). Otherwise, let input be a List whose elements are the code units that are the elements of S.
        // 12. NOTE: Each element of input is considered to be a character.

        // 10. Let matchSucceeded be false.
        // 13. Repeat, while matchSucceeded is false,
        let lossy_input = input.to_std_string_escaped();
        let (match_value, last_byte_index) = loop {
            // a. If lastIndex > length, then
            if last_index > length {
                // i. If global is true or sticky is true, then
                if global || sticky {
                    // 1. Perform ? Set(R, "lastIndex", +0𝔽, true).
                    this.set(utf16!("lastIndex"), 0, true, context)?;
                }

                // ii. Return null.
                return Ok(None);
            }

            // b. Let inputIndex be the index into input of the character that was obtained from element lastIndex of S.
            // Check if last_index is a valid utf8 index into input.
            // TODO: avoid converting to String
            let last_byte_index = match String::from_utf16(&input[..last_index as usize]) {
                Ok(s) => s.len(),
                Err(_) => {
                    return Err(JsNativeError::typ()
                        .with_message("Failed to get byte index from utf16 encoded string")
                        .into())
                }
            };
            // c. Let r be matcher(input, inputIndex).
            let r = matcher.find_from(&lossy_input, last_byte_index).next();

            match r {
                // d. If r is failure, then
                None => {
                    // i. If sticky is true, then
                    if sticky {
                        // 1. Perform ? Set(R, "lastIndex", +0𝔽, true).
                        this.set(utf16!("lastIndex"), 0, true, context)?;

                        // 2. Return null.
                        return Ok(None);
                    }

                    // ii. Set lastIndex to AdvanceStringIndex(S, lastIndex, fullUnicode).
                    last_index = advance_string_index(input, last_index, full_unicode);
                }

                Some(m) => {
                    // d. If r is failure, then
                    #[allow(clippy::if_not_else)]
                    if m.start() != last_byte_index {
                        // i. If sticky is true, then
                        if sticky {
                            // 1. Perform ? Set(R, "lastIndex", +0𝔽, true).
                            this.set(utf16!("lastIndex"), 0, true, context)?;

                            // 2. Return null.
                            return Ok(None);
                        }

                        // ii. Set lastIndex to AdvanceStringIndex(S, lastIndex, fullUnicode).
                        last_index = advance_string_index(input, last_index, full_unicode);
                    // e. Else,
                    } else {
                        // i. Assert: r is a State.
                        // ii. Set matchSucceeded to true.
                        break (m, last_byte_index);
                    }
                }
            }
        };

        // 14. Let e be r's endIndex value.
        let mut e = match_value.end();

        // 15. If fullUnicode is true, set e to GetStringIndex(S, e).
        // TODO: disabled for now until we have UTF-16 support
        if false {
            // e is an index into the Input character list, derived from S, matched by matcher.
            // Let eUTF be the smallest index into S that corresponds to the character at element e of Input.
            // If e is greater than or equal to the number of elements in Input, then eUTF is the number of code units in S.
            // b. Set e to eUTF.
            e = input.get(..e).map_or_else(|| input.len(), <[u16]>::len);
        }

        // 16. If global is true or sticky is true, then
        if global || sticky {
            // a. Perform ? Set(R, "lastIndex", 𝔽(e), true).
            this.set(
                utf16!("lastIndex"),
                lossy_input[..e].encode_utf16().count(),
                true,
                context,
            )?;
        }

        // 17. Let n be the number of elements in r's captures List.
        let n = match_value.captures.len() as u64;
        // 18. Assert: n = R.[[RegExpRecord]].[[CapturingGroupsCount]].
        // 19. Assert: n < 232 - 1.
        debug_assert!(n < 23u64.pow(2) - 1);

        // 20. Let A be ! ArrayCreate(n + 1).
        // 21. Assert: The mathematical value of A's "length" property is n + 1.
        let a = Array::array_create(n + 1, None, context)?;

        // 22. Perform ! CreateDataPropertyOrThrow(A, "index", 𝔽(lastIndex)).
        a.create_data_property_or_throw(utf16!("index"), last_index, context)
            .expect("this CreateDataPropertyOrThrow call must not fail");

        // 23. Perform ! CreateDataPropertyOrThrow(A, "input", S).
        a.create_data_property_or_throw(utf16!("input"), input.clone(), context)
            .expect("this CreateDataPropertyOrThrow call must not fail");

        // 24. Let match be the Match Record { [[StartIndex]]: lastIndex, [[EndIndex]]: e }.
        // Immediately convert it to an array according to 22.2.7.7 GetMatchIndexPair(S, match)
        // 1. Assert: match.[[StartIndex]] ≤ match.[[EndIndex]] ≤ the length of S.
        // 2. Return CreateArrayFromList(« 𝔽(match.[[StartIndex]]), 𝔽(match.[[EndIndex]]) »).
        let match_record = Array::create_array_from_list(
            [match_value.start().into(), match_value.end().into()],
            context,
        );

        // 25. Let indices be a new empty List.
        let indices = Array::array_create(n + 1, None, context)?;

        // 27. Append match to indices.
        indices
            .create_data_property_or_throw(0, match_record, context)
            .expect("this CreateDataPropertyOrThrow call must not fail");

        // 28. Let matchedSubstr be GetMatchString(S, match).
        let matched_substr = js_string!(&lossy_input[last_byte_index..e]);

        // 29. Perform ! CreateDataPropertyOrThrow(A, "0", matchedSubstr).
        a.create_data_property_or_throw(0, matched_substr, context)
            .expect("this CreateDataPropertyOrThrow call must not fail");

        let mut named_groups = match_value
            .named_groups()
            .collect::<Vec<(&str, Option<Range>)>>();
        // Strict mode requires groups to be created in a sorted order
        named_groups.sort_by(|(name_x, _), (name_y, _)| name_x.cmp(name_y));

        // Combines:
        // 26. Let groupNames be a new empty List.
        // 30. If R contains any GroupName, then
        // 31. Else,
        // 33. For each integer i such that 1 ≤ i ≤ n, in ascending order, do
        #[allow(clippy::if_not_else)]
        let (groups, group_names) = if !named_groups.clone().is_empty() {
            // a. Let groups be OrdinaryObjectCreate(null).
            let groups = JsObject::with_null_proto();
            let group_names = JsObject::with_null_proto();

            // e. If the ith capture of R was defined with a GroupName, then
            // i. Let s be the CapturingGroupName of that GroupName.
            // ii. Perform ! CreateDataPropertyOrThrow(groups, s, capturedValue).
            // iii. Append s to groupNames.
            for (name, range) in named_groups {
                let name = js_string!(name);
                if let Some(range) = range {
                    // TODO: Full UTF-16 regex support
                    let value = js_string!(&lossy_input[range.clone()]);

                    groups
                        .create_data_property_or_throw(name.clone(), value, context)
                        .expect("this CreateDataPropertyOrThrow call must not fail");

                    // 22.2.7.8 MakeMatchIndicesIndexPairArray ( S, indices, groupNames, hasGroups )
                    // a. Let matchIndices be indices[i].
                    // b. If matchIndices is not undefined, then
                    // i. Let matchIndexPair be GetMatchIndexPair(S, matchIndices).
                    // d. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(i)), matchIndexPair).
                    group_names
                        .create_data_property_or_throw(
                            name.clone(),
                            Array::create_array_from_list(
                                [range.start.into(), range.end.into()],
                                context,
                            ),
                            context,
                        )
                        .expect("this CreateDataPropertyOrThrow call must not fail");
                } else {
                    groups
                        .create_data_property_or_throw(name.clone(), JsValue::undefined(), context)
                        .expect("this CreateDataPropertyOrThrow call must not fail");

                    // 22.2.7.8 MakeMatchIndicesIndexPairArray ( S, indices, groupNames, hasGroups )
                    // c. Else,
                    // i. Let matchIndexPair be undefined.
                    // d. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(i)), matchIndexPair).
                    group_names
                        .create_data_property_or_throw(name, JsValue::undefined(), context)
                        .expect("this CreateDataPropertyOrThrow call must not fail");
                }
            }

            (groups.into(), group_names.into())
        } else {
            // a. Let groups be undefined.
            (JsValue::undefined(), JsValue::undefined())
        };

        // 22.2.7.8 MakeMatchIndicesIndexPairArray ( S, indices, groupNames, hasGroups )
        // 8. Perform ! CreateDataPropertyOrThrow(A, "groups", groups).
        indices
            .create_data_property_or_throw(utf16!("groups"), group_names, context)
            .expect("this CreateDataPropertyOrThrow call must not fail");

        // 32. Perform ! CreateDataPropertyOrThrow(A, "groups", groups).
        a.create_data_property_or_throw(utf16!("groups"), groups, context)
            .expect("this CreateDataPropertyOrThrow call must not fail");

        // 27. For each integer i such that i ≥ 1 and i ≤ n, in ascending order, do
        for i in 1..=n {
            // a. Let captureI be ith element of r's captures List.
            let capture = match_value.group(i as usize);

            // b. If captureI is undefined, let capturedValue be undefined.
            // c. Else if fullUnicode is true, then
            // d. Else,
            // TODO: Full UTF-16 regex support
            let captured_value = capture.clone().map_or_else(JsValue::undefined, |range| {
                js_string!(&lossy_input[range]).into()
            });

            // e. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(i)), capturedValue).
            a.create_data_property_or_throw(i, captured_value.clone(), context)
                .expect("this CreateDataPropertyOrThrow call must not fail");

            // 22.2.7.8 MakeMatchIndicesIndexPairArray ( S, indices, groupNames, hasGroups )
            if has_indices {
                // b. If matchIndices is not undefined, then
                // i. Let matchIndexPair be GetMatchIndexPair(S, matchIndices).
                // c. Else,
                // i. Let matchIndexPair be undefined.
                let indices_range = capture.map_or_else(JsValue::undefined, |range| {
                    Array::create_array_from_list([range.start.into(), range.end.into()], context)
                        .into()
                });

                // d. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(i)), matchIndexPair).
                indices
                    .create_data_property_or_throw(i, indices_range, context)
                    .expect("this CreateDataPropertyOrThrow call must not fail");
            }
        }

        // 34. If hasIndices is true, then
        // a. Let indicesArray be MakeMatchIndicesIndexPairArray(S, indices, groupNames, hasGroups).
        // b. Perform ! CreateDataPropertyOrThrow(A, "indices", indicesArray).
        if has_indices {
            a.create_data_property_or_throw(utf16!("indices"), indices, context)
                .expect("this CreateDataPropertyOrThrow call must not fail");
        }

        // 35. Return A.
        Ok(Some(a))
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
    pub(crate) fn r#match(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let rx be the this value.
        // 2. If Type(rx) is not Object, throw a TypeError exception.
        let rx = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("RegExp.prototype.match method called on incompatible value")
        })?;

        // 3. Let S be ? ToString(string).
        let arg_str = args.get_or_undefined(0).to_string(context)?;

        // 4. Let global be ! ToBoolean(? Get(rx, "global")).
        let global = rx.get(utf16!("global"), context)?.to_boolean();

        // 5. If global is false, then
        #[allow(clippy::if_not_else)]
        if !global {
            // a. Return ? RegExpExec(rx, S).
            (Self::abstract_exec(rx, arg_str, context)?)
                .map_or_else(|| Ok(JsValue::null()), |v| Ok(v.into()))
        // 6. Else,
        } else {
            // a. Assert: global is true.

            // b. Let fullUnicode be ! ToBoolean(? Get(rx, "unicode")).
            let unicode = rx.get(utf16!("unicode"), context)?.to_boolean();

            // c. Perform ? Set(rx, "lastIndex", +0𝔽, true).
            rx.set(utf16!("lastIndex"), 0, true, context)?;

            // d. Let A be ! ArrayCreate(0).
            let a =
                Array::array_create(0, None, context).expect("this ArrayCreate call must not fail");

            // e. Let n be 0.
            let mut n = 0;

            // f. Repeat,
            loop {
                // i. Let result be ? RegExpExec(rx, S).
                let result = Self::abstract_exec(rx, arg_str.clone(), context)?;

                // ii. If result is null, then
                // iii. Else,
                if let Some(result) = result {
                    // 1. Let matchStr be ? ToString(? Get(result, "0")).
                    let match_str = result.get(0, context)?.to_string(context)?;

                    // 2. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(n)), matchStr).
                    a.create_data_property_or_throw(n, match_str.clone(), context)
                        .expect("this CreateDataPropertyOrThrow call must not fail");

                    // 3. If matchStr is the empty String, then
                    if match_str.is_empty() {
                        // a. Let thisIndex be ℝ(? ToLength(? Get(rx, "lastIndex"))).
                        let this_index =
                            rx.get(utf16!("lastIndex"), context)?.to_length(context)?;

                        // b. Let nextIndex be AdvanceStringIndex(S, thisIndex, fullUnicode).
                        let next_index = advance_string_index(&arg_str, this_index, unicode);

                        // c. Perform ? Set(rx, "lastIndex", 𝔽(nextIndex), true).
                        rx.set(utf16!("lastIndex"), JsValue::new(next_index), true, context)?;
                    }

                    // 4. Set n to n + 1.
                    n += 1;
                } else {
                    // 1. If n = 0, return null.
                    if n == 0 {
                        return Ok(JsValue::null());
                    }
                    // 2. Return A.
                    return Ok(a.into());
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
    pub(crate) fn to_string(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let (body, flags) = if let Some(object) = this.as_object() {
            let object = object.borrow();
            let regex = object.as_regexp().ok_or_else(|| {
                JsNativeError::typ().with_message(format!(
                    "Method RegExp.prototype.toString called on incompatible receiver {}",
                    this.display()
                ))
            })?;
            (regex.original_source.clone(), regex.original_flags.clone())
        } else {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "Method RegExp.prototype.toString called on incompatible receiver {}",
                    this.display()
                ))
                .into());
        };
        Ok(js_string!(utf16!("/"), &body, utf16!("/"), &flags).into())
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
    pub(crate) fn match_all(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let R be the this value.
        // 2. If Type(R) is not Object, throw a TypeError exception.
        let regexp = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("RegExp.prototype.match_all method called on incompatible value")
        })?;

        // 3. Let S be ? ToString(string).
        let arg_str = args.get_or_undefined(0).to_string(context)?;

        // 4. Let C be ? SpeciesConstructor(R, %RegExp%).
        let c = regexp.species_constructor(StandardConstructors::regexp, context)?;

        // 5. Let flags be ? ToString(? Get(R, "flags")).
        let flags = regexp.get(utf16!("flags"), context)?.to_string(context)?;

        // 6. Let matcher be ? Construct(C, « R, flags »).
        let matcher = c.construct(&[this.clone(), flags.clone().into()], Some(&c), context)?;

        // 7. Let lastIndex be ? ToLength(? Get(R, "lastIndex")).
        let last_index = regexp
            .get(utf16!("lastIndex"), context)?
            .to_length(context)?;

        // 8. Perform ? Set(matcher, "lastIndex", lastIndex, true).
        matcher.set(utf16!("lastIndex"), last_index, true, context)?;

        // 9. If flags contains "g", let global be true.
        // 10. Else, let global be false.
        let global = flags.contains(&('g' as u16));

        // 11. If flags contains "u", let fullUnicode be true.
        // 12. Else, let fullUnicode be false.
        let unicode = flags.contains(&('u' as u16));

        // 13. Return ! CreateRegExpStringIterator(matcher, S, global, fullUnicode).
        Ok(RegExpStringIterator::create_regexp_string_iterator(
            matcher.clone(),
            arg_str,
            global,
            unicode,
            context,
        ))
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
    pub(crate) fn replace(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // Helper enum.
        enum CallableOrString<'a> {
            FunctionalReplace(&'a JsObject),
            ReplaceValue(JsString),
        }

        // 1. Let rx be the this value.
        // 2. If rx is not an Object, throw a TypeError exception.
        let rx = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message(
                "RegExp.prototype[Symbol.replace] method called on incompatible value",
            )
        })?;

        // 3. Let S be ? ToString(string).
        let s = args.get_or_undefined(0).to_string(context)?;

        // 4. Let lengthS be the length of S.
        let length_s = s.len();

        let replace_value = args.get_or_undefined(1);

        // 5. Let functionalReplace be IsCallable(replaceValue).
        let functional_replace = replace_value.as_callable();

        // 6. If functionalReplace is false, then
        let replace_value = if let Some(callable) = functional_replace {
            CallableOrString::FunctionalReplace(callable)
        } else {
            // a. Set replaceValue to ? ToString(replaceValue).
            CallableOrString::ReplaceValue(replace_value.to_string(context)?)
        };

        // 7. Let flags be ? ToString(? Get(rx, "flags")).
        let flags = rx.get(utf16!("flags"), context)?.to_string(context)?;

        // 8. If flags contains "g", let global be true. Otherwise, let global be false.
        let global = flags.as_slice().contains(&u16::from(b'g'));

        // 9. If global is true, then
        let full_unicode = if global {
            // a. If flags contains "u", let fullUnicode be true. Otherwise, let fullUnicode be false.
            let full_unicode = flags.contains(&u16::from(b'u'));

            // b. Perform ? Set(rx, "lastIndex", +0𝔽, true).
            rx.set(utf16!("lastIndex"), 0, true, context)?;

            full_unicode
        } else {
            false
        };

        // 10. Let results be a new empty List.
        let mut results = Vec::new();

        // SKIPPED: 11. Let done be false.
        //
        // NOTE(HalidOdat): We don't keep track of `done`, we just break when done is true.

        // 12. Repeat, while done is false,
        loop {
            // a. Let result be ? RegExpExec(rx, S).
            let result = Self::abstract_exec(rx, s.clone(), context)?;

            // b. If result is null, set done to true.
            let Some(result) = result else {
                // SKIPPED: 1. Set done to true.
                break;
            };

            // c. Else,
            //  i. Append result to results.
            results.push(result.clone());

            //  ii. If global is false, then
            if !global {
                // SKIPPED: 1. Set done to true.
                break;
            }

            //  iii. Else,
            //    1. Let matchStr be ? ToString(? Get(result, "0")).
            let match_str = result.get(0, context)?.to_string(context)?;

            //    2. If matchStr is the empty String, then
            if match_str.is_empty() {
                // a. Let thisIndex be ℝ(? ToLength(? Get(rx, "lastIndex"))).
                let this_index = rx.get(utf16!("lastIndex"), context)?.to_length(context)?;

                // b. Let nextIndex be AdvanceStringIndex(S, thisIndex, fullUnicode).
                let next_index = advance_string_index(&s, this_index, full_unicode);

                // c. Perform ? Set(rx, "lastIndex", 𝔽(nextIndex), true).
                rx.set(utf16!("lastIndex"), JsValue::new(next_index), true, context)?;
            }
        }

        // 16. If nextSourcePosition ≥ lengthS, return accumulatedResult.
        // 17. Return the string-concatenation of accumulatedResult and the substring of S from nextSourcePosition.

        // 13. Let accumulatedResult be the empty String.
        let mut accumulated_result = vec![];

        // 14. Let nextSourcePosition be 0.
        let mut next_source_position = 0;

        // 15. For each element result of results, do
        for result in results {
            // a. Let resultLength be ? LengthOfArrayLike(result).
            let result_length = result.length_of_array_like(context)? as i64;

            // b. Let nCaptures be max(resultLength - 1, 0).
            let n_captures = std::cmp::max(result_length - 1, 0);

            // c. Let matched be ? ToString(? Get(result, "0")).
            let matched = result.get(0, context)?.to_string(context)?;

            // d. Let matchLength be the length of matched.
            let match_length = matched.len();

            // e. Let position be ? ToIntegerOrInfinity(? Get(result, "index")).
            let position = result
                .get(utf16!("index"), context)?
                .to_integer_or_infinity(context)?;

            // f. Set position to the result of clamping position between 0 and lengthS.
            let position = position.clamp_finite(0, length_s as i64) as usize;

            // g. Let captures be a new empty List.
            let mut captures = Vec::new();

            // h. Let n be 1.
            // i. Repeat, while n ≤ nCaptures,
            for n in 1..=n_captures {
                // i. Let capN be ? Get(result, ! ToString(𝔽(n))).
                let mut cap_n = result.get(n, context)?;

                // ii. If capN is not undefined, then
                if !cap_n.is_undefined() {
                    // 1. Set capN to ? ToString(capN).
                    cap_n = cap_n.to_string(context)?.into();
                }

                // iii. Append capN to captures.
                captures.push(cap_n);

                // iv. NOTE: When n = 1, the preceding step puts the first element into captures (at index 0).
                //     More generally, the nth capture (the characters captured by the nth set of capturing parentheses)
                //     is at captures[n - 1].
                //
                // v. Set n to n + 1.
            }

            // j. Let namedCaptures be ? Get(result, "groups").
            let mut named_captures = result.get(utf16!("groups"), context)?;

            let replacement = match replace_value {
                // k. If functionalReplace is true, then
                CallableOrString::FunctionalReplace(replace_value) => {
                    // i. Let replacerArgs be the list-concatenation of « matched », captures, and « 𝔽(position), S ».
                    let mut replacer_args = vec![JsValue::new(matched)];
                    replacer_args.extend(captures);
                    replacer_args.push(position.into());
                    replacer_args.push(s.clone().into());

                    // ii. If namedCaptures is not undefined, then
                    if !named_captures.is_undefined() {
                        // 1. Append namedCaptures to replacerArgs.
                        replacer_args.push(named_captures);
                    }

                    // iii. Let replValue be ? Call(replaceValue, undefined, replacerArgs).
                    let repl_value =
                        replace_value.call(&JsValue::undefined(), &replacer_args, context)?;

                    // iv. Let replacement be ? ToString(replValue).
                    repl_value.to_string(context)?
                }
                // l. Else,
                CallableOrString::ReplaceValue(ref replace_value) => {
                    // i. If namedCaptures is not undefined, then
                    if !named_captures.is_undefined() {
                        // 1. Set namedCaptures to ? ToObject(namedCaptures).
                        named_captures = named_captures.to_object(context)?.into();
                    }

                    // ii. Let replacement be ? GetSubstitution(matched, S, position, captures, namedCaptures, replaceValue).
                    string::get_substitution(
                        &matched,
                        &s,
                        position,
                        &captures,
                        &named_captures,
                        replace_value,
                        context,
                    )?
                }
            };

            // m. If position ≥ nextSourcePosition, then
            if position >= next_source_position {
                // i. NOTE: position should not normally move backwards.
                //    If it does, it is an indication of an ill-behaving RegExp subclass or use of
                //    an access triggered side-effect to change the global flag or other characteristics of rx.
                //    In such cases, the corresponding substitution is ignored.

                // ii. Set accumulatedResult to the string-concatenation of accumulatedResult, the substring of S from nextSourcePosition to position, and replacement.
                accumulated_result.extend_from_slice(&s[next_source_position..position]);
                accumulated_result.extend_from_slice(&replacement);

                // iii. Set nextSourcePosition to position + matchLength.
                next_source_position = position + match_length;
            }
        }

        // 16. If nextSourcePosition ≥ lengthS, return accumulatedResult.
        if next_source_position >= length_s {
            return Ok(js_string!(accumulated_result).into());
        }

        // 17. Return the string-concatenation of accumulatedResult and the substring of S from nextSourcePosition.
        Ok(js_string!(&accumulated_result[..], &s[next_source_position..]).into())
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
    pub(crate) fn search(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let rx be the this value.
        // 2. If Type(rx) is not Object, throw a TypeError exception.
        let rx = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("RegExp.prototype[Symbol.search] method called on incompatible value")
        })?;

        // 3. Let S be ? ToString(string).
        let arg_str = args.get_or_undefined(0).to_string(context)?;

        // 4. Let previousLastIndex be ? Get(rx, "lastIndex").
        let previous_last_index = rx.get(utf16!("lastIndex"), context)?;

        // 5. If SameValue(previousLastIndex, +0𝔽) is false, then
        if !JsValue::same_value(&previous_last_index, &JsValue::new(0)) {
            // a. Perform ? Set(rx, "lastIndex", +0𝔽, true).
            rx.set(utf16!("lastIndex"), 0, true, context)?;
        }

        // 6. Let result be ? RegExpExec(rx, S).
        let result = Self::abstract_exec(rx, arg_str, context)?;

        // 7. Let currentLastIndex be ? Get(rx, "lastIndex").
        let current_last_index = rx.get(utf16!("lastIndex"), context)?;

        // 8. If SameValue(currentLastIndex, previousLastIndex) is false, then
        if !JsValue::same_value(&current_last_index, &previous_last_index) {
            // a. Perform ? Set(rx, "lastIndex", previousLastIndex, true).
            rx.set(utf16!("lastIndex"), previous_last_index, true, context)?;
        }

        // 9. If result is null, return -1𝔽.
        // 10. Return ? Get(result, "index").
        result.map_or_else(
            || Ok(JsValue::new(-1)),
            |result| result.get(utf16!("index"), context),
        )
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
    pub(crate) fn split(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let rx be the this value.
        // 2. If Type(rx) is not Object, throw a TypeError exception.
        let rx = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("RegExp.prototype.split method called on incompatible value")
        })?;

        // 3. Let S be ? ToString(string).
        let arg_str = args.get_or_undefined(0).to_string(context)?;

        // 4. Let C be ? SpeciesConstructor(rx, %RegExp%).
        let constructor = rx.species_constructor(StandardConstructors::regexp, context)?;

        // 5. Let flags be ? ToString(? Get(rx, "flags")).
        let flags = rx.get(utf16!("flags"), context)?.to_string(context)?;

        // 6. If flags contains "u", let unicodeMatching be true.
        // 7. Else, let unicodeMatching be false.
        let unicode = flags.contains(&('u' as u16));

        // 8. If flags contains "y", let newFlags be flags.
        // 9. Else, let newFlags be the string-concatenation of flags and "y".
        let new_flags = if flags.contains(&('y' as u16)) {
            flags
        } else {
            js_string!(&flags, utf16!("y"))
        };

        // 10. Let splitter be ? Construct(C, « rx, newFlags »).
        let splitter = constructor.construct(
            &[this.clone(), new_flags.into()],
            Some(&constructor),
            context,
        )?;

        // 11. Let A be ! ArrayCreate(0).
        let a = Array::array_create(0, None, context).expect("this ArrayCreate call must not fail");

        // 12. Let lengthA be 0.
        let mut length_a = 0;

        // 13. If limit is undefined, let lim be 2^32 - 1; else let lim be ℝ(? ToUint32(limit)).
        let limit = args.get_or_undefined(1);
        let lim = if limit.is_undefined() {
            u32::MAX
        } else {
            limit.to_u32(context)?
        };

        // 14. If lim is 0, return A.
        if lim == 0 {
            return Ok(a.into());
        }

        // 15. Let size be the length of S.
        let size = arg_str.len() as u64;

        // 16. If size is 0, then
        if size == 0 {
            // a. Let z be ? RegExpExec(splitter, S).
            let result = Self::abstract_exec(&splitter, arg_str.clone(), context)?;

            // b. If z is not null, return A.
            if result.is_some() {
                return Ok(a.into());
            }

            // c. Perform ! CreateDataPropertyOrThrow(A, "0", S).
            a.create_data_property_or_throw(0, arg_str, context)
                .expect("this CreateDataPropertyOrThrow call must not fail");

            // d. Return A.
            return Ok(a.into());
        }

        // 17. Let p be 0.
        // 18. Let q be p.
        let mut p = 0;
        let mut q = p;

        // 19. Repeat, while q < size,
        while q < size {
            // a. Perform ? Set(splitter, "lastIndex", 𝔽(q), true).
            splitter.set(utf16!("lastIndex"), JsValue::new(q), true, context)?;

            // b. Let z be ? RegExpExec(splitter, S).
            let result = Self::abstract_exec(&splitter, arg_str.clone(), context)?;

            // c. If z is null, set q to AdvanceStringIndex(S, q, unicodeMatching).
            // d. Else,
            if let Some(result) = result {
                // i. Let e be ℝ(? ToLength(? Get(splitter, "lastIndex"))).
                let mut e = splitter
                    .get(utf16!("lastIndex"), context)?
                    .to_length(context)?;

                // ii. Set e to min(e, size).
                e = std::cmp::min(e, size);

                // iii. If e = p, set q to AdvanceStringIndex(S, q, unicodeMatching).
                // iv. Else,
                if e == p {
                    q = advance_string_index(&arg_str, q, unicode);
                } else {
                    // 1. Let T be the substring of S from p to q.
                    let arg_str_substring = js_string!(&arg_str[p as usize..q as usize]);

                    // 2. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(lengthA)), T).
                    a.create_data_property_or_throw(length_a, arg_str_substring, context)
                        .expect("this CreateDataPropertyOrThrow call must not fail");

                    // 3. Set lengthA to lengthA + 1.
                    length_a += 1;

                    // 4. If lengthA = lim, return A.
                    if length_a == lim {
                        return Ok(a.into());
                    }

                    // 5. Set p to e.
                    p = e;

                    // 6. Let numberOfCaptures be ? LengthOfArrayLike(z).
                    let mut number_of_captures = result.length_of_array_like(context)? as isize;

                    // 7. Set numberOfCaptures to max(numberOfCaptures - 1, 0).
                    number_of_captures = std::cmp::max(number_of_captures - 1, 0);

                    // 8. Let i be 1.
                    // 9. Repeat, while i ≤ numberOfCaptures,
                    for i in 1..=number_of_captures {
                        // a. Let nextCapture be ? Get(z, ! ToString(𝔽(i))).
                        let next_capture = result.get(i, context)?;

                        // b. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(lengthA)), nextCapture).
                        a.create_data_property_or_throw(length_a, next_capture, context)
                            .expect("this CreateDataPropertyOrThrow call must not fail");

                        // d. Set lengthA to lengthA + 1.
                        length_a += 1;

                        // e. If lengthA = lim, return A.
                        if length_a == lim {
                            return Ok(a.into());
                        }
                    }

                    // 10. Set q to p.
                    q = p;
                }
            } else {
                q = advance_string_index(&arg_str, q, unicode);
            }
        }

        // 20. Let T be the substring of S from p to size.
        let arg_str_substring = js_string!(&arg_str[p as usize..size as usize]);

        // 21. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(lengthA)), T).
        a.create_data_property_or_throw(length_a, arg_str_substring, context)
            .expect("this CreateDataPropertyOrThrow call must not fail");

        // 22. Return A.
        Ok(a.into())
    }

    /// [`RegExp.prototype.compile ( pattern, flags )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexp.prototype.compile
    #[cfg(feature = "annex-b")]
    fn compile(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[RegExpMatcher]]).
        let this = this
            .as_object()
            .filter(|o| o.borrow().is_regexp())
            .cloned()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("`RegExp.prototype.compile` cannot be called for a non-object")
            })?;
        let pattern = args.get_or_undefined(0);
        let flags = args.get_or_undefined(1);
        // 3. If pattern is an Object and pattern has a [[RegExpMatcher]] internal slot, then
        let (pattern, flags) = if let Some((p, f)) = pattern.as_object().and_then(|o| {
            let o = o.borrow();
            o.as_regexp()
                .map(|rx| (rx.original_source.clone(), rx.original_flags.clone()))
        }) {
            //     a. If flags is not undefined, throw a TypeError exception.
            if !flags.is_undefined() {
                return Err(JsNativeError::typ()
                    .with_message(
                        "`RegExp.prototype.compile` cannot be \
                called with both a RegExp initializer and new flags",
                    )
                    .into());
            }
            //     b. Let P be pattern.[[OriginalSource]].
            //     c. Let F be pattern.[[OriginalFlags]].
            (p.into(), f.into())
        } else {
            // 4. Else,
            //     a. Let P be pattern.
            //     b. Let F be flags.
            (pattern.clone(), flags.clone())
        };
        // 5. Return ? RegExpInitialize(O, P, F).
        Self::initialize(this, &pattern, &flags, context)
    }
}

/// `22.2.5.2.3 AdvanceStringIndex ( S, index, unicode )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-advancestringindex
fn advance_string_index(s: &JsString, index: u64, unicode: bool) -> u64 {
    // Regress only works with utf8, so this function differs from the spec.

    // 1. Assert: index ≤ 2^53 - 1.

    // 2. If unicode is false, return index + 1.
    if !unicode {
        return index + 1;
    }

    // 3. Let length be the number of code units in S.
    let length = s.len() as u64;

    // 4. If index + 1 ≥ length, return index + 1.
    if index + 1 > length {
        return index + 1;
    }

    // 5. Let cp be ! CodePointAt(S, index).
    let code_point = s.code_point_at(index as usize);

    index + code_point.code_unit_count() as u64
}
