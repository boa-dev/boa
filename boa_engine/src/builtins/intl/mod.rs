//! This module implements the global `Intl` object.
//!
//! `Intl` is a built-in object that has properties and methods for i18n. It's not a function object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma402/#intl-object

use crate::{
    builtins::{Array, BuiltIn, JsArgs},
    object::ObjectInitializer,
    property::Attribute,
    symbol::WellKnownSymbols,
    Context, JsResult, JsString, JsValue,
};
use boa_profiler::Profiler;
use indexmap::IndexSet;
use tap::{Conv, Pipe};

/// JavaScript `Intl` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Intl;

impl BuiltIn for Intl {
    const NAME: &'static str = "Intl";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let string_tag = WellKnownSymbols::to_string_tag();
        ObjectInitializer::new(context)
            .function(Self::get_canonical_locales, "getCanonicalLocales", 1)
            .property(
                string_tag,
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build()
            .conv::<JsValue>()
            .pipe(Some)
    }
}

impl Intl {
    fn canonicalize_locale(locale: &str) -> JsString {
        JsString::new(locale)
    }

    fn canonicalize_locale_list(
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Vec<JsString>> {
        // https://tc39.es/ecma402/#sec-canonicalizelocalelist
        // 1. If locales is undefined, then
        let locales = args.get_or_undefined(0);
        if locales.is_undefined() {
            // a. Return a new empty List.
            return Ok(Vec::new());
        }

        let locales = &args[0];

        // 2. Let seen be a new empty List.
        let mut seen = IndexSet::new();

        // 3. If Type(locales) is String or Type(locales) is Object and locales has an [[InitializedLocale]] internal slot, then
        // TODO: check if Type(locales) is object and handle the internal slots
        let o = if locales.is_string() {
            // a. Let O be CreateArrayFromList(« locales »).
            Array::create_array_from_list([locales.clone()], context)
        } else {
            // 4. Else,
            // a. Let O be ? ToObject(locales).
            locales.to_object(context)?
        };

        // 5. Let len be ? ToLength(? Get(O, "length")).
        let len = o.length_of_array_like(context)?;

        // 6 Let k be 0.
        // 7. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ToString(k).
            // b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.has_property(k, context)?;
            // c. If kPresent is true, then
            if k_present {
                // i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(k, context)?;
                // ii. If Type(kValue) is not String or Object, throw a TypeError exception.
                if !(k_value.is_object() || k_value.is_string()) {
                    return context.throw_type_error("locale should be a String or Object");
                }
                // iii. If Type(kValue) is Object and kValue has an [[InitializedLocale]] internal slot, then
                // TODO: handle checks for InitializedLocale internal slot (there should be an if statement here)
                // 1. Let tag be kValue.[[Locale]].
                // iv. Else,
                // 1. Let tag be ? ToString(kValue).
                let tag = k_value.to_string(context)?;
                // v. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
                // TODO: implement `IsStructurallyValidLanguageTag`

                // vi. Let canonicalizedTag be CanonicalizeUnicodeLocaleId(tag).
                seen.insert(Self::canonicalize_locale(&tag));
                // vii. If canonicalizedTag is not an element of seen, append canonicalizedTag as the last element of seen.
            }
            // d. Increase k by 1.
        }

        // 8. Return seen.
        Ok(seen.into_iter().collect::<Vec<JsString>>())
    }

    /// Returns an array containing the canonical locale names.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN docs][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.getcanonicallocales
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/getCanonicalLocales
    pub(crate) fn get_canonical_locales(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let ll be ? CanonicalizeLocaleList(locales).
        let ll = Self::canonicalize_locale_list(args, context)?;
        // 2. Return CreateArrayFromList(ll).
        Ok(JsValue::Object(Array::create_array_from_list(
            ll.into_iter().map(Into::into),
            context,
        )))
    }
}
