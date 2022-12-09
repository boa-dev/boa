#![allow(clippy::string_lit_as_bytes)]

use boa_profiler::Profiler;
use icu_collator::CaseFirst;
use icu_datetime::options::preferences::HourCycle;
use icu_locid::{
    extensions::unicode::Value,
    extensions_unicode_key as key, extensions_unicode_value as value,
    subtags::{Language, Region, Script},
};
use tap::{Conv, Pipe};

#[cfg(test)]
mod tests;

mod utils;
pub(super) use utils::*;

use crate::{
    builtins::{BuiltIn, JsArgs},
    context::intrinsics::StandardConstructors,
    js_string,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        JsObject, ObjectData,
    },
    property::Attribute,
    symbol::WellKnownSymbols,
    Context, JsNativeError, JsResult, JsString, JsValue,
};

use super::options::{coerce_options_to_object, get_option};

#[derive(Debug, Clone)]
pub(crate) struct Locale;

impl BuiltIn for Locale {
    const NAME: &'static str = "Locale";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let base_name = FunctionBuilder::native(context, Self::base_name)
            .name("get baseName")
            .constructor(false)
            .build();

        let calendar = FunctionBuilder::native(context, Self::calendar)
            .name("get calendar")
            .constructor(false)
            .build();

        let case_first = FunctionBuilder::native(context, Self::case_first)
            .name("get caseFirst")
            .constructor(false)
            .build();

        let collation = FunctionBuilder::native(context, Self::collation)
            .name("get collation")
            .constructor(false)
            .build();

        let hour_cycle = FunctionBuilder::native(context, Self::hour_cycle)
            .name("get hourCycle")
            .constructor(false)
            .build();

        let numeric = FunctionBuilder::native(context, Self::numeric)
            .name("get numeric")
            .constructor(false)
            .build();

        let numbering_system = FunctionBuilder::native(context, Self::numbering_system)
            .name("get numberingSystem")
            .constructor(false)
            .build();

        let language = FunctionBuilder::native(context, Self::language)
            .name("get language")
            .constructor(false)
            .build();

        let script = FunctionBuilder::native(context, Self::script)
            .name("get script")
            .constructor(false)
            .build();

        let region = FunctionBuilder::native(context, Self::region)
            .name("get region")
            .constructor(false)
            .build();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().locale().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .property(
            WellKnownSymbols::to_string_tag(),
            "Intl.Locale",
            Attribute::CONFIGURABLE,
        )
        .method(Self::maximize, "maximize", 0)
        .method(Self::minimize, "minimize", 0)
        .method(Self::to_string, "toString", 0)
        .accessor("baseName", Some(base_name), None, Attribute::CONFIGURABLE)
        .accessor("calendar", Some(calendar), None, Attribute::CONFIGURABLE)
        .accessor("caseFirst", Some(case_first), None, Attribute::CONFIGURABLE)
        .accessor("collation", Some(collation), None, Attribute::CONFIGURABLE)
        .accessor("hourCycle", Some(hour_cycle), None, Attribute::CONFIGURABLE)
        .accessor("numeric", Some(numeric), None, Attribute::CONFIGURABLE)
        .accessor(
            "numberingSystem",
            Some(numbering_system),
            None,
            Attribute::CONFIGURABLE,
        )
        .accessor("language", Some(language), None, Attribute::CONFIGURABLE)
        .accessor("script", Some(script), None, Attribute::CONFIGURABLE)
        .accessor("region", Some(region), None, Attribute::CONFIGURABLE)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl Locale {
    pub(crate) const LENGTH: usize = 1;

    /// Constructor [`Intl.Locale ( tag [ , options ] )`][spec].
    ///
    /// Constructor for `Locale` objects.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.Locale
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("cannot call `Intl.Locale` constructor without `new`")
                .into());
        }

        let tag = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 2. Let relevantExtensionKeys be %Locale%.[[RelevantExtensionKeys]].
        // 3. Let internalSlotsList be « [[InitializedLocale]], [[Locale]], [[Calendar]], [[Collation]], [[HourCycle]], [[NumberingSystem]] ».
        // 4. If relevantExtensionKeys contains "kf", then
        //     a. Append [[CaseFirst]] as the last element of internalSlotsList.
        // 5. If relevantExtensionKeys contains "kn", then
        //     a. Append [[Numeric]] as the last element of internalSlotsList.

        // 7. If Type(tag) is not String or Object, throw a TypeError exception.
        if !(tag.is_object() || tag.is_string()) {
            return Err(JsNativeError::typ()
                .with_message("Intl.Locale: `tag` should be a String or Object")
                .into());
        }

        // 8. If Type(tag) is Object and tag has an [[InitializedLocale]] internal slot, then

        let mut tag = if let Some(tag) = tag
            .as_object()
            .and_then(|obj| obj.borrow().as_locale().cloned())
        {
            // a. Let tag be tag.[[Locale]].
            tag
        }
        // 9. Else,
        else {
            // a. Let tag be ? ToString(tag).
            tag.to_string(context)?
                .to_std_string_escaped()
                .parse()
                .map_err(|_| {
                    JsNativeError::range()
                        .with_message("Intl.Locale: `tag` is not a structurally valid language tag")
                })?
        };

        // 10. Set options to ? CoerceOptionsToObject(options).
        let options = &coerce_options_to_object(options, context)?;

        // 11. Set tag to ? ApplyOptionsToTag(tag, options).

        // Abstract operation [`ApplyOptionsToTag ( tag, options )`][https://tc39.es/ecma402/#sec-apply-options-to-tag]
        {
            // 1. Assert: Type(tag) is String.
            // 2. Assert: Type(options) is Object.
            // 3. If ! IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
            // 4. Let language be ? GetOption(options, "language", string, empty, undefined).
            // 5. If language is not undefined, then
            let language = get_option::<JsString>(options, "language", false, context)?
                // a. If language does not match the unicode_language_subtag production, throw a RangeError exception.
                .map(|s| s.to_std_string_escaped().parse::<Language>())
                .transpose()
                .map_err(|e| JsNativeError::range().with_message(e.to_string()))?;

            // 6. Let script be ? GetOption(options, "script", string, empty, undefined).
            // 7. If script is not undefined, then
            let script = get_option::<JsString>(options, "script", false, context)?
                .map(|s| s.to_std_string_escaped().parse::<Script>())
                .transpose()
                // a. If script does not match the unicode_script_subtag production, throw a RangeError exception.
                .map_err(|e| JsNativeError::range().with_message(e.to_string()))?;

            // 8. Let region be ? GetOption(options, "region", string, empty, undefined).
            // 9. If region is not undefined, then
            let region = get_option::<JsString>(options, "region", false, context)?
                .map(|s| s.to_std_string_escaped().parse::<Region>())
                .transpose()
                // a. If region does not match the unicode_region_subtag production, throw a RangeError exception.
                .map_err(|e| JsNativeError::range().with_message(e.to_string()))?;

            // 10. Set tag to ! CanonicalizeUnicodeLocaleId(tag).
            context.icu().locale_canonicalizer().canonicalize(&mut tag);

            // Skipping some boilerplate since this is easier to do using the `Locale` type, but putting the
            // spec for completion.
            // 11. Assert: tag matches the unicode_locale_id production.
            // 12. Let languageId be the substring of tag corresponding to the unicode_language_id production.
            // 13. If language is not undefined, then
            //     a. Set languageId to languageId with the substring corresponding to the unicode_language_subtag production replaced by the string language.
            // 14. If script is not undefined, then
            //     a. If languageId does not contain a unicode_script_subtag production, then
            //         i. Set languageId to the string-concatenation of the unicode_language_subtag production of languageId, "-", script, and the rest of languageId.
            //     b. Else,
            //         i. Set languageId to languageId with the substring corresponding to the unicode_script_subtag production replaced by the string script.
            // 15. If region is not undefined, then
            //     a. If languageId does not contain a unicode_region_subtag production, then
            //         i. Set languageId to the string-concatenation of the unicode_language_subtag production of languageId, the substring corresponding to "-"` and the `unicode_script_subtag` production if present, `"-", region, and the rest of languageId.
            //     b. Else,
            //         i. Set languageId to languageId with the substring corresponding to the unicode_region_subtag production replaced by the string region.
            // 16. Set tag to tag with the substring corresponding to the unicode_language_id production replaced by the string languageId.

            if let Some(language) = language {
                tag.id.language = language;
            }
            if let Some(script) = script {
                tag.id.script = Some(script);
            }
            if let Some(region) = region {
                tag.id.region = Some(region);
            }
            // 17. Return ! CanonicalizeUnicodeLocaleId(tag).
            context.icu().locale_canonicalizer().canonicalize(&mut tag);
        }

        // 12. Let opt be a new Record.
        // 13. Let calendar be ? GetOption(options, "calendar", string, empty, undefined).
        // 14. If calendar is not undefined, then
        // 15. Set opt.[[ca]] to calendar.
        let ca = if let Some(s) = get_option::<JsString>(options, "calendar", false, context)? {
            // a. If calendar does not match the Unicode Locale Identifier type nonterminal, throw a RangeError exception.
            if s.is_empty() {
                return Err(JsNativeError::range()
                    .with_message("Intl.Locale: `calendar` cannot be empty")
                    .into());
            }
            Some(
                s.to_std_string_escaped()
                    .parse::<Value>()
                    .map_err(|e| JsNativeError::range().with_message(e.to_string()))?,
            )
        } else {
            None
        };

        // 16. Let collation be ? GetOption(options, "collation", string, empty, undefined).
        // 17. If collation is not undefined, then
        // 18. Set opt.[[co]] to collation.
        let co = if let Some(s) = get_option::<JsString>(options, "collation", false, context)? {
            // a. If collation does not match the Unicode Locale Identifier type nonterminal, throw a RangeError exception.
            if s.is_empty() {
                return Err(JsNativeError::range()
                    .with_message("Intl.Locale: `collation` cannot be empty")
                    .into());
            }
            Some(
                s.to_std_string_escaped()
                    .parse::<Value>()
                    .map_err(|e| JsNativeError::range().with_message(e.to_string()))?,
            )
        } else {
            None
        };

        // 19. Let hc be ? GetOption(options, "hourCycle", string, « "h11", "h12", "h23", "h24" », undefined).
        // 20. Set opt.[[hc]] to hc.
        let hc =
            get_option::<HourCycle>(options, "hourCycle", false, context)?.map(|hc| match hc {
                HourCycle::H24 => value!("h24"),
                HourCycle::H23 => value!("h23"),
                HourCycle::H12 => value!("h12"),
                HourCycle::H11 => value!("h11"),
            });

        // 21. Let kf be ? GetOption(options, "caseFirst", string, « "upper", "lower", "false" », undefined).
        // 22. Set opt.[[kf]] to kf.
        let kf =
            get_option::<CaseFirst>(options, "caseFirst", false, context)?.map(|kf| match kf {
                CaseFirst::UpperFirst => value!("upper"),
                CaseFirst::LowerFirst => value!("lower"),
                CaseFirst::Off => value!("false"),
                _ => unreachable!(),
            });

        // 23. Let kn be ? GetOption(options, "numeric", boolean, empty, undefined).
        // 24. If kn is not undefined, set kn to ! ToString(kn).
        // 25. Set opt.[[kn]] to kn.
        let kn = get_option::<bool>(options, "numeric", false, context)?.map(|b| {
            if b {
                value!("true")
            } else {
                value!("false")
            }
        });

        // 26. Let numberingSystem be ? GetOption(options, "numberingSystem", string, empty, undefined).
        // 27. If numberingSystem is not undefined, then
        // 28. Set opt.[[nu]] to numberingSystem.
        let nu =
            if let Some(s) = get_option::<JsString>(options, "numberingSystem", false, context)? {
                // a. If numberingSystem does not match the Unicode Locale Identifier type nonterminal, throw a RangeError exception.
                if s.is_empty() {
                    return Err(JsNativeError::range()
                        .with_message("Intl.Locale: `numberingSystem` cannot be empty")
                        .into());
                }
                Some(
                    s.to_std_string_escaped()
                        .parse::<Value>()
                        .map_err(|e| JsNativeError::range().with_message(e.to_string()))?,
                )
            } else {
                None
            };

        // 29. Let r be ! ApplyUnicodeExtensionToTag(tag, opt, relevantExtensionKeys).
        // 30. Set locale.[[Locale]] to r.[[locale]].
        if let Some(ca) = ca {
            // 31. Set locale.[[Calendar]] to r.[[ca]].
            tag.extensions.unicode.keywords.set(key!("ca"), ca);
        }
        if let Some(co) = co {
            // 32. Set locale.[[Collation]] to r.[[co]].
            tag.extensions.unicode.keywords.set(key!("co"), co);
        }
        if let Some(hc) = hc {
            // 33. Set locale.[[HourCycle]] to r.[[hc]].
            tag.extensions.unicode.keywords.set(key!("hc"), hc);
        }
        if let Some(kf) = kf {
            // 34. If relevantExtensionKeys contains "kf", then
            //     a. Set locale.[[CaseFirst]] to r.[[kf]].
            tag.extensions.unicode.keywords.set(key!("kf"), kf);
        }
        if let Some(kn) = kn {
            // 35. If relevantExtensionKeys contains "kn", then
            //     a. If SameValue(r.[[kn]], "true") is true or r.[[kn]] is the empty String, then
            //         i. Set locale.[[Numeric]] to true.
            //     b. Else,
            //         i. Set locale.[[Numeric]] to false.
            tag.extensions.unicode.keywords.set(key!("kn"), kn);
        }
        if let Some(nu) = nu {
            // 36. Set locale.[[NumberingSystem]] to r.[[nu]].
            tag.extensions.unicode.keywords.set(key!("nu"), nu);
        }

        context.icu().locale_canonicalizer().canonicalize(&mut tag);

        // 6. Let locale be ? OrdinaryCreateFromConstructor(NewTarget, "%Locale.prototype%", internalSlotsList).
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::locale, context)?;
        let locale = JsObject::from_proto_and_data(prototype, ObjectData::locale(tag));

        // 37. Return locale.
        Ok(locale.into())
    }

    /// [`Intl.Locale.prototype.maximize ( )`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.maximize
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/maximize
    pub(crate) fn maximize(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("`maximize` can only be called on a `Locale` object")
        })?;
        let mut loc = loc
            .as_locale()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("`maximize` can only be called on a `Locale` object")
            })?
            .clone();

        // 3. Let maximal be the result of the Add Likely Subtags algorithm applied to loc.[[Locale]]. If an error is signaled, set maximal to loc.[[Locale]].
        context.icu().locale_expander().maximize(&mut loc);

        // 4. Return ! Construct(%Locale%, maximal).
        let prototype = context.intrinsics().constructors().locale().prototype();
        Ok(JsObject::from_proto_and_data(prototype, ObjectData::locale(loc)).into())
    }

    /// [`Intl.Locale.prototype.minimize ( )`][spec]
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.minimize
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/minimize
    pub(crate) fn minimize(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("`minimize` can only be called on a `Locale` object")
        })?;
        let mut loc = loc
            .as_locale()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("`minimize` can only be called on a `Locale` object")
            })?
            .clone();

        // 3. Let minimal be the result of the Remove Likely Subtags algorithm applied to loc.[[Locale]]. If an error is signaled, set minimal to loc.[[Locale]].
        context.icu().locale_expander().minimize(&mut loc);

        // 4. Return ! Construct(%Locale%, minimal).
        let prototype = context.intrinsics().constructors().locale().prototype();
        Ok(JsObject::from_proto_and_data(prototype, ObjectData::locale(loc)).into())
    }

    /// [`Intl.Locale.prototype.toString ( )`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.toString
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/toString
    pub(crate) fn to_string(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("`toString` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ().with_message("`toString` can only be called on a `Locale` object")
        })?;

        // 3. Return loc.[[Locale]].
        Ok(js_string!(loc.to_string()).into())
    }

    /// [`get Intl.Locale.prototype.baseName`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/baseName
    /// [mdn]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.baseName
    pub(crate) fn base_name(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get baseName` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get baseName` can only be called on a `Locale` object")
        })?;

        // 3. Let locale be loc.[[Locale]].
        // 4. Return the substring of locale corresponding to the unicode_language_id production.
        Ok(js_string!(loc.id.to_string()).into())
    }

    /// [`get Intl.Locale.prototype.calendar`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/calendar
    /// [mdn]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.calendar
    pub(crate) fn calendar(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get calendar` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get calendar` can only be called on a `Locale` object")
        })?;

        // 3. Return loc.[[Calendar]].
        Ok(loc
            .extensions
            .unicode
            .keywords
            .get(&key!("ca"))
            .map(|v| js_string!(v.to_string()).into())
            .unwrap_or_default())
    }

    /// [`get Intl.Locale.prototype.caseFirst`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/calendar
    /// [mdn]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.calendar
    pub(crate) fn case_first(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get caseFirst` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get caseFirst` can only be called on a `Locale` object")
        })?;

        // 3. Return loc.[[CaseFirst]].
        Ok(loc
            .extensions
            .unicode
            .keywords
            .get(&key!("kf"))
            .map(|v| js_string!(v.to_string()).into())
            .unwrap_or_default())
    }

    /// [`get Intl.Locale.prototype.collation`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/collation
    /// [mdn]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.collation
    pub(crate) fn collation(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get collation` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get collation` can only be called on a `Locale` object")
        })?;

        // 3. Return loc.[[Collation]].
        Ok(loc
            .extensions
            .unicode
            .keywords
            .get(&key!("co"))
            .map(|v| js_string!(v.to_string()).into())
            .unwrap_or_default())
    }

    /// [`get Intl.Locale.prototype.hourCycle`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/hourCycle
    /// [mdn]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.hourCycle
    pub(crate) fn hour_cycle(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get hourCycle` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get hourCycle` can only be called on a `Locale` object")
        })?;

        // 3. Return loc.[[HourCycle]].
        Ok(loc
            .extensions
            .unicode
            .keywords
            .get(&key!("hc"))
            .map(|v| js_string!(v.to_string()).into())
            .unwrap_or_default())
    }

    /// [`get Intl.Locale.prototype.numeric`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/numeric
    /// [mdn]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.numeric
    pub(crate) fn numeric(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get numeric` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get numeric` can only be called on a `Locale` object")
        })?;

        // 3. Return loc.[[Numeric]].
        let kn = loc
            .extensions
            .unicode
            .keywords
            .get(&key!("kn"))
            .map(Value::as_tinystr_slice);
        Ok(JsValue::Boolean(match kn {
            Some([]) => true,
            Some([kn]) if kn == "true" => true,
            _ => false,
        }))
    }

    /// [`get Intl.Locale.prototype.numberingSystem`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/numeric
    /// [mdn]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.numeric
    pub(crate) fn numbering_system(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get numberingSystem` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get numberingSystem` can only be called on a `Locale` object")
        })?;

        // 3. Return loc.[[NumberingSystem]].
        Ok(loc
            .extensions
            .unicode
            .keywords
            .get(&key!("nu"))
            .map(|v| js_string!(v.to_string()).into())
            .unwrap_or_default())
    }

    /// [`get Intl.Locale.prototype.language`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/language
    /// [mdn]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.language
    pub(crate) fn language(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get language` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get language` can only be called on a `Locale` object")
        })?;

        // 3. Let locale be loc.[[Locale]].
        // 4. Assert: locale matches the unicode_locale_id production.
        // 5. Return the substring of locale corresponding to the unicode_language_subtag production of the unicode_language_id.
        Ok(js_string!(loc.id.language.to_string()).into())
    }

    /// [`get Intl.Locale.prototype.script`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/script
    /// [mdn]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.script
    pub(crate) fn script(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get script` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get script` can only be called on a `Locale` object")
        })?;

        // 3. Let locale be loc.[[Locale]].
        // 4. Assert: locale matches the unicode_locale_id production.
        // 5. If the unicode_language_id production of locale does not contain the ["-" unicode_script_subtag] sequence, return undefined.
        // 6. Return the substring of locale corresponding to the unicode_script_subtag production of the unicode_language_id.
        Ok(loc
            .id
            .script
            .map(|sc| js_string!(sc.to_string()).into())
            .unwrap_or_default())
    }

    /// [`get Intl.Locale.prototype.region`][spec].
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Locale/region
    /// [mdn]: https://tc39.es/ecma402/#sec-Intl.Locale.prototype.region
    pub(crate) fn region(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let loc be the this value.
        // 2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
        let loc = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get region` can only be called on a `Locale` object")
        })?;
        let loc = loc.as_locale().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`get region` can only be called on a `Locale` object")
        })?;

        // 3. Let locale be loc.[[Locale]].
        // 4. Assert: locale matches the unicode_locale_id production.
        // 5. If the unicode_language_id production of locale does not contain the ["-" unicode_region_subtag] sequence, return undefined.
        // 6. Return the substring of locale corresponding to the unicode_region_subtag production of the unicode_language_id.
        Ok(loc
            .id
            .region
            .map(|sc| js_string!(sc.to_string()).into())
            .unwrap_or_default())
    }
}
