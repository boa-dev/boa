//! This module implements the global `Intl` object.
//!
//! `Intl` is a built-in object that has properties and methods for i18n. It's not a function object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma402/#intl-object

use crate::{
    builtins::intl::date_time_format::DateTimeFormat,
    builtins::{Array, BuiltIn, JsArgs},
    object::ObjectInitializer,
    property::Attribute,
    symbol::WellKnownSymbols,
    Context, JsResult, JsString, JsValue,
};

pub mod date_time_format;
#[cfg(test)]
mod tests;

use boa_profiler::Profiler;
use indexmap::IndexSet;
use rustc_hash::FxHashMap;
use tap::{Conv, Pipe};

/// JavaScript `Intl` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Intl;

impl BuiltIn for Intl {
    const NAME: &'static str = "Intl";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let string_tag = WellKnownSymbols::to_string_tag();
        let date_time_format = DateTimeFormat::init(context);
        ObjectInitializer::new(context)
            .function(Self::get_canonical_locales, "getCanonicalLocales", 1)
            .property(
                string_tag,
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                "DateTimeFormat",
                date_time_format,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
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

/// `MatcherRecord` type aggregates unicode `locale` string and unicode locale `extension`.
///
/// This is a return value for `lookup_matcher` and `best_fit_matcher` subroutines.
#[derive(Debug)]
struct MatcherRecord {
    locale: JsString,
    extension: JsString,
}

/// Returns the position of the first found unicode locale extension in a given string.
///
/// If no extensions found, return the length of requested locale
fn get_leftmost_unicode_extension_pos(requested_locale: &str) -> usize {
    let ext_sep = "-u-";
    let src_locale = requested_locale.to_lowercase();
    let pos = src_locale.find(ext_sep);
    match pos {
        Some(idx) => idx,
        None => src_locale.len(),
    }
}

/// Trims unciode locale extensions from a given string if any.
///
/// For example:
///
/// - `ja-Jpan-JP-u-ca-japanese-hc-h12` becomes `ja-Jpan-JP`
/// - `fr-FR` becomes `fr-FR`
fn trim_unicode_extensions(requested_locale: &str) -> JsString {
    let trim_pos = get_leftmost_unicode_extension_pos(requested_locale);
    JsString::new(&requested_locale[..trim_pos])
}

/// Extracts unciode locale extensions from a given string if any.
///
/// For example:
///
/// - `ja-Jpan-JP-u-ca-japanese-hc-h12` becomes `-u-ca-japanese-hc-h12`
/// - `en-US` becomes an empty string
fn extract_unicode_extensions(requested_locale: &str) -> JsString {
    let trim_pos = get_leftmost_unicode_extension_pos(requested_locale);
    JsString::new(&requested_locale[trim_pos..])
}

/// The `DefaultLocale` abstract operation returns a String value representing the structurally
/// valid and canonicalized Unicode BCP 47 locale identifier for the host environment's current
/// locale.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-defaultlocale
fn default_locale() -> JsString {
    // FIXME get locale from environment
    JsString::new("en-US")
}

/// The `BestAvailableLocale` abstract operation compares the provided argument `locale`,
/// which must be a String value with a structurally valid and canonicalized Unicode BCP 47
/// locale identifier, against the locales in `availableLocales` and returns either the longest
/// non-empty prefix of `locale` that is an element of `availableLocales`, or undefined if
/// there is no such element.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-bestavailablelocale
fn best_available_locale(available_locales: &[JsString], locale: &JsString) -> Option<JsString> {
    // 1. Let candidate be locale.
    let mut candidate = locale.clone();
    // 2. Repeat
    loop {
        // a. If availableLocales contains an element equal to candidate, return candidate.
        if available_locales.contains(&candidate) {
            return Some(candidate);
        }

        // b. Let pos be the character index of the last occurrence of "-" (U+002D) within candidate. If that character does not occur, return undefined.
        let pos = candidate.rfind('-');
        match pos {
            Some(ind) => {
                // c. If pos ≥ 2 and the character "-" occurs at index pos-2 of candidate, decrease pos by 2.
                let tmp_candidate = candidate[..ind].to_string();
                let prev_dash = tmp_candidate.rfind('-').unwrap_or(ind);
                let trim_ind = if ind >= 2 && prev_dash == ind - 2 {
                    ind - 2
                } else {
                    ind
                };
                // d. Let candidate be the substring of candidate from position 0, inclusive, to position pos, exclusive.
                candidate = JsString::new(&candidate[..trim_ind]);
            }
            None => return None,
        }
    }
}

/// The `LookupMatcher` abstract operation compares `requestedLocales`, which must be a `List`
/// as returned by `CanonicalizeLocaleList`, against the locales in `availableLocales` and
/// determines the best available language to meet the request.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-lookupmatcher
fn lookup_matcher(available_locales: &[JsString], requested_locales: &[JsString]) -> MatcherRecord {
    // 1. Let result be a new Record.
    // 2. For each element locale of requestedLocales, do
    for locale_str in requested_locales {
        // a. Let noExtensionsLocale be the String value that is locale with any Unicode locale
        //    extension sequences removed.
        let no_extensions_locale = trim_unicode_extensions(locale_str);

        // b. Let availableLocale be ! BestAvailableLocale(availableLocales, noExtensionsLocale).
        let available_locale = best_available_locale(available_locales, &no_extensions_locale);

        // c. If availableLocale is not undefined, then
        if let Some(available_locale) = available_locale {
            // i. Set result.[[locale]] to availableLocale.
            // Assignment deferred. See return statement below.
            // ii. If locale and noExtensionsLocale are not the same String value, then
            let maybe_ext = if locale_str.eq(&no_extensions_locale) {
                JsString::empty()
            } else {
                // 1. Let extension be the String value consisting of the substring of the Unicode
                //    locale extension sequence within locale.
                // 2. Set result.[[extension]] to extension.
                extract_unicode_extensions(locale_str)
            };

            // iii. Return result.
            return MatcherRecord {
                locale: available_locale,
                extension: maybe_ext,
            };
        }
    }

    // 3. Let defLocale be ! DefaultLocale().
    // 4. Set result.[[locale]] to defLocale.
    // 5. Return result.
    MatcherRecord {
        locale: default_locale(),
        extension: JsString::empty(),
    }
}

/// The `BestFitMatcher` abstract operation compares `requestedLocales`, which must be a `List`
/// as returned by `CanonicalizeLocaleList`, against the locales in `availableLocales` and
/// determines the best available language to meet the request. The algorithm is implementation
/// dependent, but should produce results that a typical user of the requested locales would
/// perceive as at least as good as those produced by the `LookupMatcher` abstract operation.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-bestfitmatcher
fn best_fit_matcher(
    available_locales: &[JsString],
    requested_locales: &[JsString],
) -> MatcherRecord {
    lookup_matcher(available_locales, requested_locales)
}

/// `Keyword` structure is a pair of keyword key and keyword value.
#[derive(Debug)]
struct Keyword {
    key: JsString,
    value: JsString,
}

/// `UniExtRecord` structure represents unicode extension records.
///
/// It contains the list of unicode `extension` attributes and the list of `keywords`.
///
/// For example:
///
/// - `-u-nu-thai` has no attributes and the list of keywords contains `(nu:thai)` pair.
#[allow(dead_code)]
#[derive(Debug)]
struct UniExtRecord {
    attributes: Vec<JsString>, // never read at this point
    keywords: Vec<Keyword>,
}

/// The `UnicodeExtensionComponents` abstract operation returns the attributes and keywords from
/// `extension`, which must be a String value whose contents are a `Unicode locale extension`
/// sequence.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-unicode-extension-components
fn unicode_extension_components(extension: &JsString) -> UniExtRecord {
    // 1. Let attributes be a new empty List.
    let mut attributes = Vec::<JsString>::new();

    // 2. Let keywords be a new empty List.
    let mut keywords = Vec::<Keyword>::new();

    // 3. Let keyword be undefined.
    let mut keyword: Option<Keyword> = None;

    // 4. Let size be the length of extension.
    let size = extension.len();

    // 5. Let k be 3.
    let mut k = 3;

    // 6. Repeat, while k < size,
    while k < size {
        // a. Let e be ! StringIndexOf(extension, "-", k).
        let e = extension.index_of(&JsString::new("-"), k);

        // b. If e = -1, let len be size - k; else let len be e - k.
        let len = match e {
            Some(pos) => pos - k,
            None => size - k,
        };

        // c. Let subtag be the String value equal to the substring of extension consisting of the
        // code units at indices k (inclusive) through k + len (exclusive).
        let subtag = JsString::new(&extension[k..k + len]);

        // d. If keyword is undefined and len ≠ 2, then
        if keyword.is_none() && len != 2 {
            // i. If subtag is not an element of attributes, then
            if !attributes.contains(&subtag) {
                // 1. Append subtag to attributes.
                attributes.push(subtag);
            }
        // e. Else if len = 2, then
        } else if len == 2 {
            // i. If keyword is not undefined and keywords does not contain an element
            // whose [[Key]] is the same as keyword.[[Key]], then
            //     1. Append keyword to keywords.
            if let Some(keyword_val) = keyword {
                let has_key = keywords.iter().any(|elem| elem.key == keyword_val.key);
                if !has_key {
                    keywords.push(keyword_val);
                }
            };

            // ii. Set keyword to the Record { [[Key]]: subtag, [[Value]]: "" }.
            keyword = Some(Keyword {
                key: subtag,
                value: JsString::empty(),
            });
        // f. Else,
        } else {
            // i. If keyword.[[Value]] is the empty String, then
            //      1. Set keyword.[[Value]] to subtag.
            // ii. Else,
            //      1. Set keyword.[[Value]] to the string-concatenation of keyword.[[Value]], "-", and subtag.
            if let Some(keyword_val) = keyword {
                let new_keyword_val = if keyword_val.value.is_empty() {
                    subtag
                } else {
                    JsString::new(format!("{}-{subtag}", keyword_val.value))
                };

                keyword = Some(Keyword {
                    key: keyword_val.key,
                    value: new_keyword_val,
                });
            };
        }

        // g. Let k be k + len + 1.
        k = k + len + 1;
    }

    // 7. If keyword is not undefined and keywords does not contain an element whose [[Key]] is
    // the same as keyword.[[Key]], then
    //      a. Append keyword to keywords.
    if let Some(keyword_val) = keyword {
        let has_key = keywords.iter().any(|elem| elem.key == keyword_val.key);
        if !has_key {
            keywords.push(keyword_val);
        }
    };

    // 8. Return the Record { [[Attributes]]: attributes, [[Keywords]]: keywords }.
    UniExtRecord {
        attributes,
        keywords,
    }
}

/// The `InsertUnicodeExtensionAndCanonicalize` abstract operation inserts `extension`, which must
/// be a Unicode locale extension sequence, into `locale`, which must be a String value with a
/// structurally valid and canonicalized Unicode BCP 47 locale identifier.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-insert-unicode-extension-and-canonicalize
fn insert_unicode_extension_and_canonicalize(locale: &str, extension: &str) -> JsString {
    // TODO 1. Assert: locale does not contain a substring that is a Unicode locale extension sequence.
    // TODO 2. Assert: extension is a Unicode locale extension sequence.
    // TODO 3. Assert: tag matches the unicode_locale_id production.
    // 4. Let privateIndex be ! StringIndexOf(locale, "-x-", 0).
    let private_index = locale.find("-x-");
    let new_locale = match private_index {
        // 5. If privateIndex = -1, then
        None => {
            // a. Let locale be the string-concatenation of locale and extension.
            locale.to_owned() + extension
        }
        // 6. Else,
        Some(idx) => {
            // a. Let preExtension be the substring of locale from position 0, inclusive,
            // to position privateIndex, exclusive.
            let pre_extension = &locale[0..idx];

            // b. Let postExtension be the substring of locale from position privateIndex to
            // the end of the string.
            let post_extension = &locale[idx..];

            // c. Let locale be the string-concatenation of preExtension, extension,
            // and postExtension.
            pre_extension.to_owned() + extension + post_extension
        }
    };

    // TODO 7. Assert: ! IsStructurallyValidLanguageTag(locale) is true.
    // 8. Return ! CanonicalizeUnicodeLocaleId(locale).
    Intl::canonicalize_locale(&new_locale)
}

/// `LocaleDataRecord` is the type of `locale_data` argument in `resolve_locale` subroutine.
///
/// It is an alias for a map where key is a string and value is another map.
///
/// Value of that inner map is a vector of strings representing locale parameters.
type LocaleDataRecord = FxHashMap<JsString, FxHashMap<JsString, Vec<JsString>>>;

/// `DateTimeFormatRecord` type aggregates `locale_matcher` selector and `properties` map.
///
/// It is used as a type of `options` parameter in `resolve_locale` subroutine.
#[derive(Debug)]
struct DateTimeFormatRecord {
    pub(crate) locale_matcher: JsString,
    pub(crate) properties: FxHashMap<JsString, JsValue>,
}

/// `ResolveLocaleRecord` type consists of unicode `locale` string, `data_locale` string and `properties` map.
///
/// This is a return value for `resolve_locale` subroutine.
#[derive(Debug)]
struct ResolveLocaleRecord {
    pub(crate) locale: JsString,
    pub(crate) properties: FxHashMap<JsString, JsValue>,
    pub(crate) data_locale: JsString,
}

/// The `ResolveLocale` abstract operation compares a BCP 47 language priority list
/// `requestedLocales` against the locales in `availableLocales` and determines the best
/// available language to meet the request. `availableLocales`, `requestedLocales`, and
/// `relevantExtensionKeys` must be provided as `List` values, options and `localeData` as Records.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-resolvelocale
#[allow(dead_code)]
fn resolve_locale(
    available_locales: &[JsString],
    requested_locales: &[JsString],
    options: &DateTimeFormatRecord,
    relevant_extension_keys: &[JsString],
    locale_data: &LocaleDataRecord,
    context: &mut Context,
) -> ResolveLocaleRecord {
    // 1. Let matcher be options.[[localeMatcher]].
    let matcher = &options.locale_matcher;
    // 2. If matcher is "lookup", then
    //    a. Let r be ! LookupMatcher(availableLocales, requestedLocales).
    // 3. Else,
    //    a. Let r be ! BestFitMatcher(availableLocales, requestedLocales).
    let r = if matcher.eq(&JsString::new("lookup")) {
        lookup_matcher(available_locales, requested_locales)
    } else {
        best_fit_matcher(available_locales, requested_locales)
    };

    // 4. Let foundLocale be r.[[locale]].
    let mut found_locale = r.locale;

    // 5. Let result be a new Record.
    let mut result = ResolveLocaleRecord {
        locale: JsString::empty(),
        properties: FxHashMap::default(),
        data_locale: JsString::empty(),
    };

    // 6. Set result.[[dataLocale]] to foundLocale.
    result.data_locale = found_locale.clone();

    // 7. If r has an [[extension]] field, then
    let keywords = if r.extension.is_empty() {
        Vec::<Keyword>::new()
    } else {
        // a. Let components be ! UnicodeExtensionComponents(r.[[extension]]).
        let components = unicode_extension_components(&r.extension);
        // b. Let keywords be components.[[Keywords]].
        components.keywords
    };

    // 8. Let supportedExtension be "-u".
    let mut supported_extension = JsString::new("-u");

    // 9. For each element key of relevantExtensionKeys, do
    for key in relevant_extension_keys {
        // a. Let foundLocaleData be localeData.[[<foundLocale>]].
        // TODO b. Assert: Type(foundLocaleData) is Record.
        let found_locale_data = match locale_data.get(&found_locale) {
            Some(locale_value) => locale_value.clone(),
            None => FxHashMap::default(),
        };

        // c. Let keyLocaleData be foundLocaleData.[[<key>]].
        // TODO d. Assert: Type(keyLocaleData) is List.
        let key_locale_data = match found_locale_data.get(key) {
            Some(locale_vec) => locale_vec.clone(),
            None => Vec::new(),
        };

        // e. Let value be keyLocaleData[0].
        // TODO f. Assert: Type(value) is either String or Null.
        let mut value = match key_locale_data.get(0) {
            Some(first_elt) => JsValue::String(first_elt.clone()),
            None => JsValue::null(),
        };

        // g. Let supportedExtensionAddition be "".
        let mut supported_extension_addition = JsString::empty();

        // h. If r has an [[extension]] field, then
        if !r.extension.is_empty() {
            // i. If keywords contains an element whose [[Key]] is the same as key, then
            //      1. Let entry be the element of keywords whose [[Key]] is the same as key.
            let maybe_entry = keywords.iter().find(|elem| key.eq(&elem.key));
            if let Some(entry) = maybe_entry {
                // 2. Let requestedValue be entry.[[Value]].
                let requested_value = &entry.value;

                // 3. If requestedValue is not the empty String, then
                if !requested_value.is_empty() {
                    // a. If keyLocaleData contains requestedValue, then
                    if key_locale_data.contains(requested_value) {
                        // i. Let value be requestedValue.
                        value = JsValue::String(JsString::new(requested_value));
                        // ii. Let supportedExtensionAddition be the string-concatenation
                        // of "-", key, "-", and value.
                        supported_extension_addition =
                            JsString::concat_array(&["-", key, "-", requested_value]);
                    }
                // 4. Else if keyLocaleData contains "true", then
                } else if key_locale_data.contains(&JsString::new("true")) {
                    // a. Let value be "true".
                    value = JsValue::String(JsString::new("true"));
                    // b. Let supportedExtensionAddition be the string-concatenation of "-" and key.
                    supported_extension_addition = JsString::concat_array(&["-", key]);
                }
            }
        }

        // i. If options has a field [[<key>]], then
        if options.properties.contains_key(key) {
            // i. Let optionsValue be options.[[<key>]].
            // TODO ii. Assert: Type(optionsValue) is either String, Undefined, or Null.
            let mut options_value = options
                .properties
                .get(key)
                .unwrap_or(&JsValue::undefined())
                .clone();

            // iii. If Type(optionsValue) is String, then
            if options_value.is_string() {
                // TODO 1. Let optionsValue be the string optionsValue after performing the
                // algorithm steps to transform Unicode extension values to canonical syntax
                // per Unicode Technical Standard #35 LDML § 3.2.1 Canonical Unicode Locale
                // Identifiers, treating key as ukey and optionsValue as uvalue productions.

                // TODO 2. Let optionsValue be the string optionsValue after performing the
                // algorithm steps to replace Unicode extension values with their canonical
                // form per Unicode Technical Standard #35 LDML § 3.2.1 Canonical Unicode
                // Locale Identifiers, treating key as ukey and optionsValue as uvalue
                // productions.

                // 3. If optionsValue is the empty String, then
                if let Some(options_val_str) = options_value.as_string() {
                    if options_val_str.is_empty() {
                        // a. Let optionsValue be "true".
                        options_value = JsValue::String(JsString::new("true"));
                    }
                }
            }

            // iv. If keyLocaleData contains optionsValue, then
            let options_val_str = options_value
                .to_string(context)
                .unwrap_or_else(|_| JsString::empty());
            if key_locale_data.contains(&options_val_str) {
                // 1. If SameValue(optionsValue, value) is false, then
                if !options_value.eq(&value) {
                    // a. Let value be optionsValue.
                    value = options_value;

                    // b. Let supportedExtensionAddition be "".
                    supported_extension_addition = JsString::empty();
                }
            }
        }

        // j. Set result.[[<key>]] to value.
        result.properties.insert(key.clone(), value);

        // k. Append supportedExtensionAddition to supportedExtension.
        supported_extension = JsString::concat(supported_extension, &supported_extension_addition);
    }

    // 10. If the number of elements in supportedExtension is greater than 2, then
    if supported_extension.len() > 2 {
        // a. Let foundLocale be InsertUnicodeExtensionAndCanonicalize(foundLocale, supportedExtension).
        found_locale =
            insert_unicode_extension_and_canonicalize(&found_locale, &supported_extension);
    }

    // 11. Set result.[[locale]] to foundLocale.
    result.locale = found_locale;

    // 12. Return result.
    result
}
