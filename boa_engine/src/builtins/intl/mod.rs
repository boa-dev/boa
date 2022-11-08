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
    error::JsNativeError,
    object::{JsObject, ObjectInitializer},
    property::Attribute,
    symbol::WellKnownSymbols,
    Context, JsResult, JsValue,
};

pub mod date_time_format;
#[cfg(test)]
mod tests;

use boa_profiler::Profiler;
use icu_locale_canonicalizer::LocaleCanonicalizer;
use icu_locid::{locale, Locale};
use indexmap::IndexSet;
use rustc_hash::FxHashMap;
use tap::{Conv, Pipe, TapOptional};

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
    /// `Intl.getCanonicalLocales ( locales )`
    ///
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
        let ll = canonicalize_locale_list(args, context)?;

        // 2. Return CreateArrayFromList(ll).
        Ok(JsValue::Object(Array::create_array_from_list(
            ll.into_iter().map(|loc| loc.to_string().into()),
            context,
        )))
    }
}

/// `MatcherRecord` type aggregates unicode `locale` string and unicode locale `extension`.
///
/// This is a return value for `lookup_matcher` and `best_fit_matcher` subroutines.
#[derive(Debug)]
struct MatcherRecord {
    locale: String,
    extension: String,
}

/// Abstract operation `DefaultLocale ( )`
///
/// Returns a String value representing the structurally valid and canonicalized
/// Unicode BCP 47 locale identifier for the host environment's current locale.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-defaultlocale
fn default_locale(canonicalizer: &LocaleCanonicalizer) -> Locale {
    sys_locale::get_locale()
        .and_then(|loc| loc.parse::<Locale>().ok())
        .tap_some_mut(|loc| canonicalize_unicode_locale_id(loc, canonicalizer))
        .unwrap_or(locale!("en-US"))
}

/// Abstract operation `BestAvailableLocale ( availableLocales, locale )`
///
/// Compares the provided argument `locale`, which must be a String value with a
/// structurally valid and canonicalized Unicode BCP 47 locale identifier, against
/// the locales in `availableLocales` and returns either the longest non-empty prefix
/// of `locale` that is an element of `availableLocales`, or undefined if there is no
/// such element.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-bestavailablelocale
fn best_available_locale<'a>(available_locales: &'_ [&'_ str], locale: &'a str) -> Option<&'a str> {
    // 1. Let candidate be locale.
    let mut candidate = locale;
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
                candidate = &candidate[..trim_ind];
            }
            None => return None,
        }
    }
}

/// Abstract operation `LookupMatcher ( availableLocales, requestedLocales )`
///
/// Compares `requestedLocales`, which must be a `List` as returned by `CanonicalizeLocaleList`,
/// against the locales in `availableLocales` and determines the best available language to
/// meet the request.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-lookupmatcher
fn lookup_matcher(
    available_locales: &[&str],
    requested_locales: &[&str],
    canonicalizer: &LocaleCanonicalizer,
) -> MatcherRecord {
    // 1. Let result be a new Record.
    // 2. For each element locale of requestedLocales, do
    for locale_str in requested_locales {
        // a. Let noExtensionsLocale be the String value that is locale with any Unicode locale
        //    extension sequences removed.
        let locale: Locale = locale_str.parse().expect("Locale parsing failed");
        let no_extensions_locale = locale.id.to_string();

        // b. Let availableLocale be ! BestAvailableLocale(availableLocales, noExtensionsLocale).
        let available_locale = best_available_locale(available_locales, &no_extensions_locale);

        // c. If availableLocale is not undefined, then
        if let Some(available_locale) = available_locale {
            // i. Set result.[[locale]] to availableLocale.
            // Assignment deferred. See return statement below.
            // ii. If locale and noExtensionsLocale are not the same String value, then
            let maybe_ext = if locale_str.eq(&no_extensions_locale) {
                String::new()
            } else {
                // 1. Let extension be the String value consisting of the substring of the Unicode
                //    locale extension sequence within locale.
                // 2. Set result.[[extension]] to extension.
                locale.extensions.to_string()
            };

            // iii. Return result.
            return MatcherRecord {
                locale: available_locale.into(),
                extension: maybe_ext,
            };
        }
    }

    // 3. Let defLocale be ! DefaultLocale().
    // 4. Set result.[[locale]] to defLocale.
    // 5. Return result.
    MatcherRecord {
        locale: default_locale(canonicalizer).to_string(),
        extension: String::new(),
    }
}

/// Abstract operation `BestFitMatcher ( availableLocales, requestedLocales )`
///
/// Compares `requestedLocales`, which must be a `List` as returned by `CanonicalizeLocaleList`,
/// against the locales in `availableLocales` and determines the best available language to
/// meet the request. The algorithm is implementation dependent, but should produce results
/// that a typical user of the requested locales would perceive as at least as good as those
/// produced by the `LookupMatcher` abstract operation.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-bestfitmatcher
fn best_fit_matcher(
    available_locales: &[&str],
    requested_locales: &[&str],
    canonicalizer: &LocaleCanonicalizer,
) -> MatcherRecord {
    lookup_matcher(available_locales, requested_locales, canonicalizer)
}

/// `Keyword` structure is a pair of keyword key and keyword value.
#[derive(Debug)]
struct Keyword {
    key: String,
    value: String,
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
    attributes: Vec<String>, // never read at this point
    keywords: Vec<Keyword>,
}

/// Abstract operation `UnicodeExtensionComponents ( extension )`
///
/// Returns the attributes and keywords from `extension`, which must be a String
/// value whose contents are a `Unicode locale extension` sequence.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-unicode-extension-components
fn unicode_extension_components(extension: &str) -> UniExtRecord {
    // 1. Let attributes be a new empty List.
    let mut attributes: Vec<String> = Vec::new();

    // 2. Let keywords be a new empty List.
    let mut keywords: Vec<Keyword> = Vec::new();

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
            Some(pos) => pos,
            None => size - k,
        };

        // c. Let subtag be the String value equal to the substring of extension consisting of the
        // code units at indices k (inclusive) through k + len (exclusive).
        let subtag = &extension[k..k + len];

        // d. If keyword is undefined and len ≠ 2, then
        if keyword.is_none() && len != 2 {
            // i. If subtag is not an element of attributes, then
            if !attributes.iter().any(|s| s == subtag) {
                // 1. Append subtag to attributes.
                attributes.push(subtag.to_string());
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
                key: subtag.into(),
                value: String::new(),
            });
        // f. Else,
        } else {
            // i. If keyword.[[Value]] is the empty String, then
            //      1. Set keyword.[[Value]] to subtag.
            // ii. Else,
            //      1. Set keyword.[[Value]] to the string-concatenation of keyword.[[Value]], "-", and subtag.
            if let Some(keyword_val) = keyword {
                let new_keyword_val = if keyword_val.value.is_empty() {
                    subtag.into()
                } else {
                    format!("{}-{subtag}", keyword_val.value)
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

/// Abstract operation `InsertUnicodeExtensionAndCanonicalize ( locale, extension )`
///
/// Inserts `extension`, which must be a Unicode locale extension sequence, into
/// `locale`, which must be a String value with a structurally valid and canonicalized
/// Unicode BCP 47 locale identifier.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-insert-unicode-extension-and-canonicalize
fn insert_unicode_extension_and_canonicalize(
    locale: &str,
    extension: &str,
    canonicalizer: &LocaleCanonicalizer,
) -> String {
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

    // 7. Assert: ! IsStructurallyValidLanguageTag(locale) is true.
    let mut new_locale = new_locale
        .parse()
        .expect("Assert: ! IsStructurallyValidLanguageTag(locale) is true.");

    // 8. Return ! CanonicalizeUnicodeLocaleId(locale).
    canonicalize_unicode_locale_id(&mut new_locale, canonicalizer);
    new_locale.to_string()
}

/// Abstract operation `CanonicalizeLocaleList ( locales )`
///
/// Converts an array of [`JsValue`]s containing structurally valid
/// [Unicode BCP 47 locale identifiers][bcp-47] into their [canonical form][canon].
///
/// For efficiency, this returns a [`Vec`] of [`Locale`]s instead of a [`Vec`] of
/// [`String`]s, since [`Locale`] allows us to modify individual parts of the locale
/// without scanning the whole string again.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-canonicalizelocalelist
/// [bcp-47]: https://unicode.org/reports/tr35/#Unicode_locale_identifier
/// [canon]: https://unicode.org/reports/tr35/#LocaleId_Canonicalization
fn canonicalize_locale_list(args: &[JsValue], context: &mut Context) -> JsResult<Vec<Locale>> {
    // 1. If locales is undefined, then
    let locales = args.get_or_undefined(0);
    if locales.is_undefined() {
        // a. Return a new empty List.
        return Ok(Vec::new());
    }

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
                return Err(JsNativeError::typ()
                    .with_message("locale should be a String or Object")
                    .into());
            }
            // iii. If Type(kValue) is Object and kValue has an [[InitializedLocale]] internal slot, then
            // TODO: handle checks for InitializedLocale internal slot (there should be an if statement here)
            // 1. Let tag be kValue.[[Locale]].
            // iv. Else,
            // 1. Let tag be ? ToString(kValue).
            // v. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
            let mut tag = k_value
                .to_string(context)?
                .to_std_string()
                .ok()
                .and_then(|tag| tag.parse().ok())
                .ok_or_else(|| {
                    JsNativeError::range()
                        .with_message("locale is not a structurally valid language tag")
                })?;

            // vi. Let canonicalizedTag be CanonicalizeUnicodeLocaleId(tag).
            canonicalize_unicode_locale_id(&mut tag, context.icu().locale_canonicalizer());
            seen.insert(tag);
            // vii. If canonicalizedTag is not an element of seen, append canonicalizedTag as the last element of seen.
        }
        // d. Increase k by 1.
    }

    // 8. Return seen.
    Ok(seen.into_iter().collect())
}

/// `LocaleDataRecord` is the type of `locale_data` argument in `resolve_locale` subroutine.
///
/// It is an alias for a map where key is a string and value is another map.
///
/// Value of that inner map is a vector of strings representing locale parameters.
type LocaleDataRecord = FxHashMap<String, FxHashMap<String, Vec<String>>>;

/// `DateTimeFormatRecord` type aggregates `locale_matcher` selector and `properties` map.
///
/// It is used as a type of `options` parameter in `resolve_locale` subroutine.
#[derive(Debug)]
struct DateTimeFormatRecord {
    pub(crate) locale_matcher: String,
    pub(crate) properties: FxHashMap<String, JsValue>,
}

/// `ResolveLocaleRecord` type consists of unicode `locale` string, `data_locale` string and `properties` map.
///
/// This is a return value for `resolve_locale` subroutine.
#[derive(Debug)]
struct ResolveLocaleRecord {
    pub(crate) locale: String,
    pub(crate) properties: FxHashMap<String, JsValue>,
    pub(crate) data_locale: String,
}

/// Abstract operation `ResolveLocale ( availableLocales, requestedLocales, options, relevantExtensionKeys, localeData )`
///
/// Compares a BCP 47 language priority list `requestedLocales` against the locales
/// in `availableLocales` and determines the best available language to meet the request.
/// `availableLocales`, `requestedLocales`, and `relevantExtensionKeys` must be provided as
/// `List` values, options and `localeData` as Records.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-resolvelocale
#[allow(dead_code)]
fn resolve_locale(
    available_locales: &[&str],
    requested_locales: &[&str],
    options: &DateTimeFormatRecord,
    relevant_extension_keys: &[&str],
    locale_data: &LocaleDataRecord,
    context: &mut Context,
) -> ResolveLocaleRecord {
    // 1. Let matcher be options.[[localeMatcher]].
    let matcher = &options.locale_matcher;
    // 2. If matcher is "lookup", then
    //    a. Let r be ! LookupMatcher(availableLocales, requestedLocales).
    // 3. Else,
    //    a. Let r be ! BestFitMatcher(availableLocales, requestedLocales).
    let r = if matcher == "lookup" {
        lookup_matcher(
            available_locales,
            requested_locales,
            context.icu().locale_canonicalizer(),
        )
    } else {
        best_fit_matcher(
            available_locales,
            requested_locales,
            context.icu().locale_canonicalizer(),
        )
    };

    // 4. Let foundLocale be r.[[locale]].
    let mut found_locale = r.locale;

    // 5. Let result be a new Record.
    let mut result = ResolveLocaleRecord {
        locale: String::new(),
        properties: FxHashMap::default(),
        data_locale: String::new(),
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
    let mut supported_extension = String::from("-u");

    // 9. For each element key of relevantExtensionKeys, do
    for &key in relevant_extension_keys {
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
            Some(first_elt) => first_elt.clone().into(),
            None => JsValue::null(),
        };

        // g. Let supportedExtensionAddition be "".
        let mut supported_extension_addition = String::new();

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
                    if key_locale_data.iter().any(|s| s == requested_value) {
                        // i. Let value be requestedValue.
                        value = requested_value.clone().into();
                        // ii. Let supportedExtensionAddition be the string-concatenation
                        // of "-", key, "-", and value.
                        supported_extension_addition = format!("-{key}-{requested_value}");
                    }
                // 4. Else if keyLocaleData contains "true", then
                } else if key_locale_data.iter().any(|s| s == "true") {
                    // a. Let value be "true".
                    value = "true".into();
                    // b. Let supportedExtensionAddition be the string-concatenation of "-" and key.
                    supported_extension_addition = format!("-{key}");
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
                        options_value = "true".into();
                    }
                }
            }

            // iv. If keyLocaleData contains optionsValue, then
            let options_val_str = options_value
                .to_string(context)
                .unwrap_or_else(|_| "".into())
                .to_std_string_escaped();
            if key_locale_data.iter().any(|s| s == &options_val_str) {
                // 1. If SameValue(optionsValue, value) is false, then
                if !options_value.eq(&value) {
                    // a. Let value be optionsValue.
                    value = options_value;

                    // b. Let supportedExtensionAddition be "".
                    supported_extension_addition = String::new();
                }
            }
        }

        // j. Set result.[[<key>]] to value.
        result.properties.insert(key.to_string(), value);

        // k. Append supportedExtensionAddition to supportedExtension.
        supported_extension.push_str(&supported_extension_addition);
    }

    // 10. If the number of elements in supportedExtension is greater than 2, then
    if supported_extension.len() > 2 {
        // a. Let foundLocale be InsertUnicodeExtensionAndCanonicalize(foundLocale, supportedExtension).
        found_locale = insert_unicode_extension_and_canonicalize(
            &found_locale,
            &supported_extension,
            context.icu().locale_canonicalizer(),
        );
    }

    // 11. Set result.[[locale]] to foundLocale.
    result.locale = found_locale;

    // 12. Return result.
    result
}

#[allow(unused)]
pub(crate) enum GetOptionType {
    String,
    Boolean,
}

/// Abstract operation `GetOption ( options, property, type, values, fallback )`
///
/// Extracts the value of the property named `property` from the provided `options` object,
/// converts it to the required `type`, checks whether it is one of a `List` of allowed
/// `values`, and fills in a `fallback` value if necessary. If `values` is
/// undefined, there is no fixed set of values and any is permitted.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-getoption
#[allow(unused)]
pub(crate) fn get_option(
    options: &JsObject,
    property: &str,
    r#type: &GetOptionType,
    values: &[&str],
    fallback: &JsValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. Assert: Type(options) is Object.
    // 2. Let value be ? Get(options, property).
    let mut value = options.get(property, context)?;

    // 3. If value is undefined, return fallback.
    if value.is_undefined() {
        return Ok(fallback.clone());
    }

    // 4. Assert: type is "boolean" or "string".
    // 5. If type is "boolean", then
    //      a. Set value to ! ToBoolean(value).
    // 6. If type is "string", then
    //      a. Set value to ? ToString(value).
    // 7. If values is not undefined and values does not contain an element equal to value,
    // throw a RangeError exception.
    value = match r#type {
        GetOptionType::Boolean => JsValue::Boolean(value.to_boolean()),
        GetOptionType::String => {
            let string_value = value.to_string(context)?.to_std_string_escaped();
            if !values.is_empty() && !values.contains(&string_value.as_str()) {
                return Err(JsNativeError::range()
                    .with_message("GetOption: values array does not contain value")
                    .into());
            }
            JsValue::String(string_value.into())
        }
    };

    // 8. Return value.
    Ok(value)
}

/// Abstract operation `GetNumberOption ( options, property, minimum, maximum, fallback )`
///
/// Extracts the value of the property named `property` from the provided `options`
/// object, converts it to a `Number value`, checks whether it is in the allowed range,
/// and fills in a `fallback` value if necessary.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-getnumberoption
#[allow(unused)]
pub(crate) fn get_number_option(
    options: &JsObject,
    property: &str,
    minimum: f64,
    maximum: f64,
    fallback: Option<f64>,
    context: &mut Context,
) -> JsResult<Option<f64>> {
    // 1. Assert: Type(options) is Object.
    // 2. Let value be ? Get(options, property).
    let value = options.get(property, context)?;

    // 3. Return ? DefaultNumberOption(value, minimum, maximum, fallback).
    default_number_option(&value, minimum, maximum, fallback, context)
}

/// Abstract operation `DefaultNumberOption ( value, minimum, maximum, fallback )`
///
/// Converts `value` to a `Number value`, checks whether it is in the allowed range,
/// and fills in a `fallback` value if necessary.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-defaultnumberoption
#[allow(unused)]
pub(crate) fn default_number_option(
    value: &JsValue,
    minimum: f64,
    maximum: f64,
    fallback: Option<f64>,
    context: &mut Context,
) -> JsResult<Option<f64>> {
    // 1. If value is undefined, return fallback.
    if value.is_undefined() {
        return Ok(fallback);
    }

    // 2. Set value to ? ToNumber(value).
    let value = value.to_number(context)?;

    // 3. If value is NaN or less than minimum or greater than maximum, throw a RangeError exception.
    if value.is_nan() || value < minimum || value > maximum {
        return Err(JsNativeError::range()
            .with_message("DefaultNumberOption: value is out of range.")
            .into());
    }

    // 4. Return floor(value).
    Ok(Some(value.floor()))
}

/// Abstract operation `CanonicalizeUnicodeLocaleId ( locale )`.
///
/// This function differs slightly from the specification by modifying in-place
/// the provided [`Locale`] instead of creating a new canonicalized copy.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-canonicalizeunicodelocaleid
fn canonicalize_unicode_locale_id(locale: &mut Locale, canonicalizer: &LocaleCanonicalizer) {
    canonicalizer.canonicalize(locale);
}
