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
    object::{JsObject, ObjectInitializer},
    property::Attribute,
    symbol::WellKnownSymbols,
    Context, JsResult, JsString, JsValue,
};

pub mod date_time_format;
#[cfg(test)]
mod tests;

use boa_profiler::Profiler;
use indexmap::IndexSet;
use std::cmp::min;
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

struct MatcherRecord {
    locale: String,
    extension: String,
}

fn get_leftmost_ext_pos(requested_locale: &str) -> usize {
    let possible_extensions = vec![
        "-u-ca", // calendar algorithm
        "-u-co", // collation type
        "-u-ka", // collation parameters
        "-u-cu", // currency type
        "-u-nu", // number type
        "-u-va", // common variant type
    ];

    let src_locale = requested_locale.to_lowercase();
    let mut trim_pos = src_locale.len();
    for ext in possible_extensions {
        let pos = src_locale.find(ext);
        trim_pos = match pos {
            Some(idx) => min(trim_pos, idx),
            None => trim_pos,
        };
    }

    trim_pos
}

fn trim_extensions(requested_locale: &str) -> String {
    let trim_pos = get_leftmost_ext_pos(requested_locale);
    requested_locale[..trim_pos].to_string()
}

fn extract_extensions(requested_locale: &str) -> String {
    let trim_pos = get_leftmost_ext_pos(requested_locale);
    requested_locale[trim_pos..].to_string()
}

/// The `DefaultLocale` abstract operation returns a String value representing the structurally
/// valid and canonicalized Unicode BCP 47 locale identifier for the host environment's current
/// locale.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-defaultlocale
fn default_locale() -> String {
    // FIXME get locale from environment
    "en-US".to_string()
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
fn best_available_locale(
    available_locales: &JsValue,
    locale: &str,
    context: &mut Context,
) -> String {
    let avail_locales_obj = available_locales
        .to_object(context)
        .unwrap_or_else(|_| JsObject::empty());
    let avail_locales_len = avail_locales_obj.length_of_array_like(context).unwrap_or(0);
    let mut avail_locales = vec![];
    for index in 0..avail_locales_len as u32 {
        let maybe_locale = avail_locales_obj
            .get(index, context)
            .unwrap_or_else(|_| JsValue::undefined());
        let locale_str = maybe_locale
            .to_string(context)
            .unwrap_or_else(|_| JsString::new(""))
            .to_string();
        avail_locales.push(locale_str);
    }

    // 1. Let candidate be locale.
    let mut candidate = locale.to_string();
    // 2. Repeat
    loop {
        // a. If availableLocales contains an element equal to candidate, return candidate.
        if avail_locales.iter().any(|loc_name| candidate.eq(loc_name)) {
            return candidate;
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
                candidate = candidate[..trim_ind].to_string();
            }
            None => return "undefined".to_string(),
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
fn lookup_matcher(
    available_locales: &JsValue,
    requested_locales: &JsValue,
    context: &mut Context,
) -> MatcherRecord {
    // 1. Let result be a new Record.
    let mut result = MatcherRecord {
        locale: "undefined".to_string(),
        extension: "".to_string(),
    };

    // 2. For each element locale of requestedLocales, do
    let req_locales_obj = requested_locales
        .to_object(context)
        .unwrap_or_else(|_| JsObject::empty());
    let req_locales_len = req_locales_obj.length_of_array_like(context).unwrap_or(0);
    for index in 0..req_locales_len as u32 {
        let maybe_locale = req_locales_obj
            .get(index, context)
            .unwrap_or_else(|_| JsValue::undefined());
        let locale_str = maybe_locale
            .to_string(context)
            .unwrap_or_else(|_| JsString::new(""))
            .to_string();

        // a. Let noExtensionsLocale be the String value that is locale with any Unicode locale
        //    extension sequences removed.
        let no_extensions_locale = trim_extensions(&locale_str);

        // b. Let availableLocale be ! BestAvailableLocale(availableLocales, noExtensionsLocale).
        let available_locale =
            best_available_locale(available_locales, &no_extensions_locale, context);

        // c. If availableLocale is not undefined, then
        if !available_locale.eq("undefined") {
            // i. Set result.[[locale]] to availableLocale.
            result.locale = available_locale;

            // ii. If locale and noExtensionsLocale are not the same String value, then
            if !locale_str.eq(&no_extensions_locale) {
                // 1. Let extension be the String value consisting of the substring of the Unicode
                //    locale extension sequence within locale.
                let extension = extract_extensions(&locale_str);

                // 2. Set result.[[extension]] to extension.
                result.extension = extension;
            }

            // iii. Return result.
            return result;
        }
    }

    // 3. Let defLocale be ! DefaultLocale().
    let def_locale = default_locale();

    // 4. Set result.[[locale]] to defLocale.
    result.locale = def_locale;

    // 5. Return result.
    result
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
    available_locales: &JsValue,
    requested_locales: &JsValue,
    context: &mut Context,
) -> MatcherRecord {
    lookup_matcher(available_locales, requested_locales, context)
}

struct Keyword {
    key: String,
    value: String,
}

#[allow(dead_code)]
struct UniExtRecord {
    attributes: Vec<String>, // never read at this point
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
fn unicode_extension_components(extension: &str) -> UniExtRecord {
    // 1. Let attributes be a new empty List.
    let mut attributes = vec![];

    // 2. Let keywords be a new empty List.
    let mut keywords: Vec<Keyword> = vec![];

    // 3. Let keyword be undefined.
    let mut keyword: Option<Keyword> = None;

    // 4. Let size be the length of extension.
    let size = extension.len();

    // 5. Let k be 3.
    let mut k = 3;

    // 6. Repeat, while k < size,
    while k < size {
        // a. Let e be ! StringIndexOf(extension, "-", k).
        let e = extension[k..].find('-');

        // b. If e = -1, let len be size - k; else let len be e - k.
        let len = match e {
            Some(pos) => pos, // no need to subtract k, since pos is calculated according to the substring
            None => size - k,
        };

        // c. Let subtag be the String value equal to the substring of extension consisting of the
        // code units at indices k (inclusive) through k + len (exclusive).
        let subtag = extension[k..k + len].to_string();

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
                value: "".to_string(),
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
                    keyword_val.value + "-" + &subtag
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

fn js_arr_contains_str(js_arr: &JsValue, str_to_find: &str, context: &mut Context) -> bool {
    let js_arr_obj = js_arr
        .to_object(context)
        .unwrap_or_else(|_| JsObject::empty());
    let js_arr_len = js_arr_obj.length_of_array_like(context).unwrap_or(0);
    for index in 0..js_arr_len as u32 {
        let arr_item = js_arr_obj
            .get(index, context)
            .unwrap_or_else(|_| JsValue::undefined())
            .to_string(context)
            .unwrap_or_else(|_| JsString::new(""))
            .to_string();

        if arr_item.eq(str_to_find) {
            return true;
        }
    }

    false
}

/// The `InsertUnicodeExtensionAndCanonicalize` abstract operation inserts `extension`, which must
/// be a Unicode locale extension sequence, into `locale`, which must be a String value with a
/// structurally valid and canonicalized Unicode BCP 47 locale identifier.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-insert-unicode-extension-and-canonicalize
fn insert_unicode_extension_and_canonicalize(locale: &str, extension: &str) -> String {
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
    Intl::canonicalize_locale(&new_locale).to_string()
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
pub fn resolve_locale(
    available_locales: &JsValue,
    requested_locales: &JsValue,
    options: &JsObject,
    relevant_extension_keys: &JsValue,
    locale_data: &JsValue,
    context: &mut Context,
) -> JsResult<JsObject> {
    // 1. Let matcher be options.[[localeMatcher]].
    let matcher = options
        .get("locale_matcher", context)
        .unwrap_or_else(|_| JsValue::undefined());
    // 2. If matcher is "lookup", then
    //    a. Let r be ! LookupMatcher(availableLocales, requestedLocales).
    // 3. Else,
    //    a. Let r be ! BestFitMatcher(availableLocales, requestedLocales).
    let r = if matcher.eq(&JsValue::String(JsString::new("lookup"))) {
        lookup_matcher(available_locales, requested_locales, context)
    } else {
        best_fit_matcher(available_locales, requested_locales, context)
    };

    // 4. Let foundLocale be r.[[locale]].
    let mut found_locale = r.locale;

    // 5. Let result be a new Record.
    let result = JsObject::empty();

    // 6. Set result.[[dataLocale]] to foundLocale.
    result.set("data_locale", found_locale.clone(), false, context)?;

    // 7. If r has an [[extension]] field, then
    let keywords = if r.extension.is_empty() {
        vec![]
    } else {
        // a. Let components be ! UnicodeExtensionComponents(r.[[extension]]).
        let components = unicode_extension_components(&r.extension);
        // b. Let keywords be components.[[Keywords]].
        components.keywords
    };

    // 8. Let supportedExtension be "-u".
    let mut supported_extension = "-u".to_string();

    // 9. For each element key of relevantExtensionKeys, do
    let rel_ext_keys_obj = relevant_extension_keys
        .to_object(context)
        .unwrap_or_else(|_| JsObject::empty());
    let rel_ext_keys_len = rel_ext_keys_obj.length_of_array_like(context).unwrap_or(0);
    for index in 0..rel_ext_keys_len as u32 {
        let key = rel_ext_keys_obj
            .get(index, context)
            .unwrap_or_else(|_| JsValue::undefined())
            .to_string(context)
            .unwrap_or_else(|_| JsString::new(""))
            .to_string();

        // a. Let foundLocaleData be localeData.[[<foundLocale>]].
        // TODO b. Assert: Type(foundLocaleData) is Record.
        let found_locale_data = locale_data
            .to_object(context)
            .unwrap_or_else(|_| JsObject::empty())
            .get(found_locale.clone(), context)
            .unwrap_or_else(|_| JsValue::undefined());

        // c. Let keyLocaleData be foundLocaleData.[[<key>]].
        // TODO d. Assert: Type(keyLocaleData) is List.
        let key_locale_data = found_locale_data
            .to_object(context)
            .unwrap_or_else(|_| JsObject::empty())
            .get(key.clone(), context)
            .unwrap_or_else(|_| JsValue::undefined());

        // e. Let value be keyLocaleData[0].
        // TODO f. Assert: Type(value) is either String or Null.
        let first_elt_idx: u32 = 0;
        let mut value = key_locale_data
            .to_object(context)
            .unwrap_or_else(|_| JsObject::empty())
            .get(first_elt_idx, context)
            .unwrap_or_else(|_| JsValue::null());

        // g. Let supportedExtensionAddition be "".
        let mut supported_extension_addition = "".to_string();

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
                    let key_locale_data_obj = key_locale_data
                        .to_object(context)
                        .unwrap_or_else(|_| JsObject::empty());
                    let key_locale_data_len = key_locale_data_obj
                        .length_of_array_like(context)
                        .unwrap_or(0);
                    for index in 0..key_locale_data_len as u32 {
                        let key_locale = key_locale_data_obj
                            .get(index, context)
                            .unwrap_or_else(|_| JsValue::undefined())
                            .to_string(context)
                            .unwrap_or_else(|_| JsString::new(""))
                            .to_string();

                        if key_locale.eq(requested_value) {
                            // i. Let value be requestedValue.
                            value = JsValue::String(JsString::new(requested_value));
                            // ii. Let supportedExtensionAddition be the string-concatenation
                            // of "-", key, "-", and value.
                            supported_extension_addition = "-".to_string();
                            supported_extension_addition.push_str(&key);
                            supported_extension_addition.push('-');
                            supported_extension_addition.push_str(requested_value);
                            break;
                        }
                    }
                // 4. Else if keyLocaleData contains "true", then
                } else if js_arr_contains_str(&key_locale_data, "true", context) {
                    // a. Let value be "true".
                    value = JsValue::String(JsString::new("true"));
                    // b. Let supportedExtensionAddition be the string-concatenation of "-" and key.
                    supported_extension_addition = "-".to_string();
                    supported_extension_addition.push_str(&key);
                }
            }
        }

        // i. If options has a field [[<key>]], then
        let opt_key = options
            .get(key.clone(), context)
            .unwrap_or_else(|_| JsValue::undefined());

        if !opt_key.is_undefined() {
            // i. Let optionsValue be options.[[<key>]].
            // TODO ii. Assert: Type(optionsValue) is either String, Undefined, or Null.
            let mut options_value = opt_key;

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
                let options_val_str = options_value
                    .to_string(context)
                    .unwrap_or_else(|_| JsString::new(""))
                    .to_string();
                if options_val_str.is_empty() {
                    // a. Let optionsValue be "true".
                    options_value = JsValue::String(JsString::new("true"));
                }
            }

            // iv. If keyLocaleData contains optionsValue, then
            let options_val_str = options_value
                .to_string(context)
                .unwrap_or_else(|_| JsString::new(""))
                .to_string();
            if js_arr_contains_str(&key_locale_data, &options_val_str, context) {
                // 1. If SameValue(optionsValue, value) is false, then
                if !options_value.eq(&value) {
                    // a. Let value be optionsValue.
                    value = options_value;

                    // b. Let supportedExtensionAddition be "".
                    supported_extension_addition = "".to_string();
                }
            }
        }

        // j. Set result.[[<key>]] to value.
        result.set(key, value, false, context)?;

        // k. Append supportedExtensionAddition to supportedExtension.
        supported_extension.push_str(&supported_extension_addition);
    }

    // 10. If the number of elements in supportedExtension is greater than 2, then
    let ext_elements = unicode_extension_components(&supported_extension);
    if ext_elements.keywords.len() > 2 {
        // a. Let foundLocale be InsertUnicodeExtensionAndCanonicalize(foundLocale, supportedExtension).
        found_locale =
            insert_unicode_extension_and_canonicalize(&found_locale, &supported_extension);
    }

    // 11. Set result.[[locale]] to foundLocale.
    result.set("locale", found_locale, false, context)?;

    // 12. Return result.
    Ok(result)
}
