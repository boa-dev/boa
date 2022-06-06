use crate::{
    builtins::{
        intl::{
            options::{IntlOptions, LocaleMatcher},
            Service,
        },
        Array, JsArgs,
    },
    context::icu::Icu,
    Context, JsNativeError, JsResult, JsValue,
};

use icu_locid::{subtags::Variants, LanguageIdentifier, Locale};
use icu_locid_transform::LocaleCanonicalizer;
use icu_provider::{DataProvider, DataRequest, DataRequestMetadata, KeyedDataMarker};
use indexmap::IndexSet;

use tap::TapOptional;

/// Abstract operation `CanonicalizeLocaleList ( locales )`
///
/// Converts an array of [`JsValue`]s containing structurally valid
/// [Unicode BCP 47 locale identifiers][bcp-47] into their [canonical form][canon].
///
/// For efficiency, this returns [`Locale`]s instead of [`String`]s, since
/// `Locale` allows us to modify individual parts of the locale without scanning
/// the whole string again.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-canonicalizelocalelist
/// [bcp-47]: https://unicode.org/reports/tr35/#Unicode_locale_identifier
/// [canon]: https://unicode.org/reports/tr35/#LocaleId_Canonicalization
pub(in crate::builtins::intl) fn canonicalize_locale_list(
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<IndexSet<Locale>> {
    // 1. If locales is undefined, then
    let locales = args.get_or_undefined(0);
    if locales.is_undefined() {
        // a. Return a new empty List.
        return Ok(IndexSet::default());
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
                .to_std_string_escaped()
                .parse()
                .map_err(|_| {
                    JsNativeError::range()
                        .with_message("locale is not a structurally valid language tag")
                })?;

            // vi. Let canonicalizedTag be CanonicalizeUnicodeLocaleId(tag).
            context.icu().locale_canonicalizer().canonicalize(&mut tag);

            // vii. If canonicalizedTag is not an element of seen, append canonicalizedTag as the last element of seen.
            seen.insert(tag);
        }
        // d. Increase k by 1.
    }

    // 8. Return seen.
    Ok(seen)
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
pub(in crate::builtins::intl) fn resolve_locale<S, P>(
    requested_locales: &[Locale],
    options: &mut IntlOptions<S::Options>,
    icu: &Icu<P>,
) -> Locale
where
    S: Service<P>,
    P: DataProvider<S::LangMarker>,
{
    // 1. Let matcher be options.[[localeMatcher]].
    // 2. If matcher is "lookup", then
    //    a. Let r be ! LookupMatcher(availableLocales, requestedLocales).
    // 3. Else,
    //    a. Let r be ! BestFitMatcher(availableLocales, requestedLocales).
    // 4. Let foundLocale be r.[[locale]].
    let mut found_locale = if options.matcher == LocaleMatcher::Lookup {
        lookup_matcher::<S::LangMarker>(requested_locales, icu)
    } else {
        best_fit_matcher::<S::LangMarker>(requested_locales, icu)
    };

    // From here, the spec differs significantly from the implementation,
    // since ICU4X allows us to skip some steps and modularize the
    // extension resolution algorithm. However, the original spec is left here
    // for completion purposes.

    // 5. Let result be a new Record.
    // 6. Set result.[[dataLocale]] to foundLocale.
    // 7. If r has an [[extension]] field, then
    //     a. Let components be ! UnicodeExtensionComponents(r.[[extension]]).
    //     b. Let keywords be components.[[Keywords]].
    // 9. For each element key of relevantExtensionKeys, do
    //     a. Let foundLocaleData be localeData.[[<foundLocale>]].
    //     b. Assert: Type(foundLocaleData) is Record.
    //     c. Let keyLocaleData be foundLocaleData.[[<key>]].
    //     d. Assert: Type(keyLocaleData) is List.
    //     e. Let value be keyLocaleData[0].
    //     f. Assert: Type(value) is either String or Null.
    //     g. Let supportedExtensionAddition be "".
    //     h. If r has an [[extension]] field, then
    //         i. If keywords contains an element whose [[Key]] is the same as key, then
    //             1. Let entry be the element of keywords whose [[Key]] is the same as key.
    //             2. Let requestedValue be entry.[[Value]].
    //             3. If requestedValue is not the empty String, then
    //                 a. If keyLocaleData contains requestedValue, then
    //                     i. Let value be requestedValue.
    //                     ii. Let supportedExtensionAddition be the string-concatenation of "-", key, "-", and value.
    //             4. Else if keyLocaleData contains "true", then
    //                 a. Let value be "true".
    //                 b. Let supportedExtensionAddition be the string-concatenation of "-" and key.
    //     i. If options has a field [[<key>]], then
    //         i. Let optionsValue be options.[[<key>]].
    //         ii. Assert: Type(optionsValue) is either String, Undefined, or Null.
    //         iii. If Type(optionsValue) is String, then
    //             1. Let optionsValue be the string optionsValue after performing the algorithm steps to transform
    //                Unicode extension values to canonical syntax per Unicode Technical Standard #35 LDML § 3.2.1
    //                Canonical Unicode Locale Identifiers, treating key as ukey and optionsValue as uvalue productions.
    //             2. Let optionsValue be the string optionsValue after performing the algorithm steps to replace
    //                Unicode extension values with their canonical form per Unicode Technical Standard #35 LDML § 3.2.1
    //                Canonical Unicode Locale Identifiers, treating key as ukey and optionsValue as uvalue productions.
    //             3. If optionsValue is the empty String, then
    //                 a. Let optionsValue be "true".
    //         iv. If keyLocaleData contains optionsValue, then
    //             1. If SameValue(optionsValue, value) is false, then
    //                 a. Let value be optionsValue.
    //                 b. Let supportedExtensionAddition be "".
    //     j. Set result.[[<key>]] to value.
    //     k. Append supportedExtensionAddition to supportedExtension.
    // 10. If the number of elements in supportedExtension is greater than 2, then
    //     a. Let foundLocale be InsertUnicodeExtensionAndCanonicalize(foundLocale, supportedExtension).
    // 11. Set result.[[locale]] to foundLocale.

    // 12. Return result.
    S::resolve(
        &mut found_locale,
        &mut options.service_options,
        icu.provider(),
    );
    found_locale
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
        .tap_some_mut(|loc| {
            canonicalizer.canonicalize(loc);
        })
        .unwrap_or_default()
}

/// Abstract operation `BestAvailableLocale ( availableLocales, locale )`
///
/// Compares the provided argument `locale`, which must be a String value with a
/// structurally valid and canonicalized Unicode BCP 47 locale identifier, against
/// the locales in `availableLocales` and returns either the longest non-empty prefix
/// of `locale` that is an element of `availableLocales`, or undefined if there is no
/// such element.
///
/// We only work with language identifiers, which have the same semantics
/// but are a bit easier to manipulate.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-bestavailablelocale
fn best_available_locale<M: KeyedDataMarker>(
    candidate: LanguageIdentifier,
    provider: &(impl DataProvider<M> + ?Sized),
) -> Option<LanguageIdentifier> {
    // 1. Let candidate be locale.
    let mut candidate = candidate.into();
    // 2. Repeat
    loop {
        // a. If availableLocales contains an element equal to candidate, return candidate.
        // ICU4X requires doing data requests in order to check if a locale
        // is part of the set of supported locales.
        let response = DataProvider::<M>::load(
            provider,
            DataRequest {
                locale: &candidate,
                metadata: DataRequestMetadata::default(),
            },
        );

        if let Ok(req) = response {
            let metadata = req.metadata;

            // `metadata.locale` returns None when the provider doesn't have a
            // fallback mechanism, but supports the required locale.
            // However, if the provider has a fallback mechanism, this will return
            // `Some(locale)`, where the locale is the used locale after applying
            // the fallback algorithm, even if the used locale is exactly the same
            // as the required locale.
            if metadata.locale.is_none() || metadata.locale.as_ref() == Some(&candidate) {
                return Some(candidate.get_langid());
            }
        }

        // b. Let pos be the character index of the last occurrence of "-" (U+002D) within candidate. If that character does not occur, return undefined.
        // c. If pos ≥ 2 and the character "-" occurs at index pos-2 of candidate, decrease pos by 2.
        // d. Let candidate be the substring of candidate from position 0, inclusive, to position pos, exclusive.
        //
        // Since the definition of `LanguageIdentifier` allows us to manipulate it
        // without using strings, we can replace these steps by a simpler
        // algorithm.

        if candidate.has_variants() {
            let mut variants = candidate
                .clear_variants()
                .iter()
                .copied()
                .collect::<Vec<_>>();
            variants.pop();
            candidate.set_variants(Variants::from_vec_unchecked(variants));
        } else if candidate.region().is_some() {
            candidate.set_region(None);
        } else if candidate.script().is_some() {
            candidate.set_script(None);
        } else {
            return None;
        }
    }
}

/// Abstract operation `LookupMatcher ( availableLocales, requestedLocales )`
///
/// Compares `requestedLocales`, which must be a `List` as returned by `CanonicalizeLocaleList`,
/// against the locales in `availableLocales` and determines the best available language to
/// meet the request.
///
/// # Note
///
/// This differs a bit from the spec, since we don't have an `[[AvailableLocales]]`
/// list to compare with. However, we can do data requests to a [`DataProvider`]
/// in order to see if a certain [`Locale`] is supported.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-lookupmatcher
fn lookup_matcher<M: KeyedDataMarker>(
    requested_locales: &[Locale],
    icu: &Icu<impl DataProvider<M>>,
) -> Locale {
    // 1. Let result be a new Record.
    // 2. For each element locale of requestedLocales, do
    for locale in requested_locales {
        // a. Let noExtensionsLocale be the String value that is locale with any Unicode locale
        //    extension sequences removed.
        let mut locale = locale.clone();
        let id = std::mem::take(&mut locale.id);

        // b. Let availableLocale be ! BestAvailableLocale(availableLocales, noExtensionsLocale).
        let available_locale = best_available_locale::<M>(id, icu.provider());

        // c. If availableLocale is not undefined, then
        if let Some(available_locale) = available_locale {
            // i. Set result.[[locale]] to availableLocale.
            // Assignment deferred. See return statement below.
            // ii. If locale and noExtensionsLocale are not the same String value, then
            // 1. Let extension be the String value consisting of the substring of the Unicode
            //    locale extension sequence within locale.
            // 2. Set result.[[extension]] to extension.
            locale.id = available_locale;

            // iii. Return result.
            return locale;
        }
    }

    // 3. Let defLocale be ! DefaultLocale().
    // 4. Set result.[[locale]] to defLocale.
    // 5. Return result.
    default_locale(icu.locale_canonicalizer())
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
fn best_fit_matcher<M: KeyedDataMarker>(
    requested_locales: &[Locale],
    icu: &Icu<impl DataProvider<M>>,
) -> Locale {
    lookup_matcher::<M>(requested_locales, icu)
}
