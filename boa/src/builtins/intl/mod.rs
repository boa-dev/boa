//! This module implements the global `Intl` object.
//!
//! `Intl` is a built-in object that has properties and methods for i18n. It's not a function object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma402/#intl-object

use crate::{
    builtins::{
        BuiltIn,
        Array
    },
    object::ObjectInitializer, property::Attribute, symbol::WellKnownSymbols,
    BoaProfiler, Context, JsValue, JsResult, JsString
};

#[cfg(test)]
mod tests;

/// JavaScript `Intl` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Intl;

impl BuiltIn for Intl {
    const NAME: &'static str = "Intl";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);

    fn init(context: &mut Context) -> JsValue {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let string_tag = WellKnownSymbols::to_string_tag();
        let object = ObjectInitializer::new(context)
            .function(Self::get_canonical_locales, "getCanonicalLocales", 1)
            .property(
                string_tag,
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();

        object.into()
    }
}

impl Intl {
    fn canonicalize_locale(locale: &str) -> JsResult<String> {
        Ok(String::from(locale))
    }

    fn canonicalize_locale_list(args: &[JsValue], context: &mut Context) -> JsResult<Vec<String>> {
        // https://tc39.es/ecma402/#sec-canonicalizelocalelist
        // 1. If locales is undefined, then
        if (args.len() == 0) || (args[0].is_undefined()) {
            // a. Return a new empty List.
            return Ok(Vec::new());
        }

        let locales = &args[0];

        // 2. Let seen be a new empty List.
        let mut seen = Vec::new();

        // 3. If Type(locales) is String or Type(locales) is Object and locales has an [[InitializedLocale]] internal slot, then
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
                    return Err(context.throw_type_error("locale should be a String or Object").unwrap_err());
                }
                // iii. If Type(kValue) is Object and kValue has an [[InitializedLocale]] internal slot, then
                // 1. Let tag be kValue.[[Locale]].
                // iv. Else,
                // 1. Let tag be ? ToString(kValue).
                let tag_s = k_value.to_string(context)?;
                let tag = tag_s.as_str();
                // v. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
                // vi. Let canonicalizedTag be CanonicalizeUnicodeLocaleId(tag).
                if let Ok(x) = Self::canonicalize_locale(tag) {
                    seen.push(x);
                };
                // vii. If canonicalizedTag is not an element of seen, append canonicalizedTag as the last element of seen.
            }
            // d. Increase k by 1.
        };

        seen.sort_unstable();
        seen.dedup();

        // 8. Return seen.
        Ok(seen)
    }

    /// Returns an array containing the canonical locale names.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN docs][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.getcanonicallocales
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/getCanonicalLocales
    pub(crate) fn get_canonical_locales(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let ll be ? CanonicalizeLocaleList(locales).
        let ll = Self::canonicalize_locale_list(args, context)?;
        // 2. Return CreateArrayFromList(ll).
        Ok(
            JsValue::Object(
                Array::create_array_from_list(
            ll.iter()
                        .map(|s| JsString::new(s).into())
                        .collect::<Vec<JsValue>>(),
                    context
                )
            )
        )
    }
}