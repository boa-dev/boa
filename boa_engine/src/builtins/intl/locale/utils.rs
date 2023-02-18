use crate::{
    builtins::{
        intl::{
            options::{coerce_options_to_object, get_option, IntlOptions, LocaleMatcher},
            Service,
        },
        Array,
    },
    context::{icu::Icu, BoaProvider},
    object::JsObject,
    string::utf16,
    Context, JsNativeError, JsResult, JsValue,
};

use icu_collator::provider::CollationMetadataV1Marker;
use icu_locid::{
    extensions::unicode::{Key, Value},
    subtags::Variants,
    LanguageIdentifier, Locale,
};
use icu_locid_transform::LocaleCanonicalizer;
use icu_provider::{DataLocale, DataProvider, DataRequest, DataRequestMetadata, KeyedDataMarker};
use indexmap::IndexSet;

use tap::TapOptional;

/// Abstract operation `DefaultLocale ( )`
///
/// Returns a String value representing the structurally valid and canonicalized
/// Unicode BCP 47 locale identifier for the host environment's current locale.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-defaultlocale
pub(crate) fn default_locale(canonicalizer: &LocaleCanonicalizer) -> Locale {
    sys_locale::get_locale()
        .and_then(|loc| loc.parse::<Locale>().ok())
        .tap_some_mut(|loc| {
            canonicalizer.canonicalize(loc);
        })
        .unwrap_or_default()
}

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
pub(crate) fn canonicalize_locale_list(
    locales: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<Vec<Locale>> {
    // 1. If locales is undefined, then
    if locales.is_undefined() {
        // a. Return a new empty List.
        return Ok(Vec::default());
    }

    // 2. Let seen be a new empty List.
    let mut seen = IndexSet::new();

    // 3. If Type(locales) is String or Type(locales) is Object and locales has an [[InitializedLocale]] internal slot, then
    let o = if locales.is_string()
        || locales
            .as_object()
            .map_or(false, |o| o.borrow().is_locale())
    {
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
            let mut tag = if let Some(tag) = k_value
                .as_object()
                .and_then(|obj| obj.borrow().as_locale().cloned())
            {
                // 1. Let tag be kValue.[[Locale]].
                tag
            }
            // iv. Else,
            else {
                // 1. Let tag be ? ToString(kValue).
                k_value
                    .to_string(context)?
                    .to_std_string_escaped()
                    .parse()
                    // v. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
                    .map_err(|_| {
                        JsNativeError::range()
                            .with_message("locale is not a structurally valid language tag")
                    })?
            };

            // vi. Let canonicalizedTag be CanonicalizeUnicodeLocaleId(tag).
            context.icu().locale_canonicalizer().canonicalize(&mut tag);

            // vii. If canonicalizedTag is not an element of seen, append canonicalizedTag as the last element of seen.
            seen.insert(tag);
        }
        // d. Increase k by 1.
    }

    // 8. Return seen.
    Ok(seen.into_iter().collect())
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
pub(crate) fn best_available_locale<M: KeyedDataMarker>(
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
            // `metadata.locale` returns None when the provider doesn't have a fallback mechanism,
            // but supports the required locale. However, if the provider has a fallback mechanism,
            // this will return `Some(locale)`, where the locale is the used locale after applying
            // the fallback algorithm, even if the used locale is exactly the same as the required
            // locale.
            match req.metadata.locale {
                Some(loc)
                    if loc == candidate
                // TODO: ugly hack to accept locales that fallback to "und" in the collator service
                || (loc.is_empty() && M::KEY.path() == CollationMetadataV1Marker::KEY.path()) =>
                {
                    return Some(candidate.into_locale().id)
                }
                None => return Some(candidate.into_locale().id),
                _ => {}
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

/// Returns the locale resolved by the `provider` after using the ICU4X fallback
/// algorithm with `candidate` (if the provider supports this), or None if the locale is not
/// supported.
pub(crate) fn best_locale_for_provider<M: KeyedDataMarker>(
    candidate: LanguageIdentifier,
    provider: &(impl DataProvider<M> + ?Sized),
) -> Option<LanguageIdentifier> {
    let response = DataProvider::<M>::load(
        provider,
        DataRequest {
            locale: &DataLocale::from(&candidate),
            metadata: DataRequestMetadata::default(),
        },
    )
    .ok()?;

    if candidate == LanguageIdentifier::UND {
        return Some(LanguageIdentifier::UND);
    }

    response
        .metadata
        .locale
        .map(|dl| {
            // TODO: ugly hack to accept locales that fallback to "und" in the collator service
            if M::KEY.path() == CollationMetadataV1Marker::KEY.path() && dl.is_empty() {
                candidate.clone()
            } else {
                dl.into_locale().id
            }
        })
        .or(Some(candidate))
        .filter(|loc| loc != &LanguageIdentifier::UND)
}

/// Abstract operation [`LookupMatcher ( availableLocales, requestedLocales )`][spec]
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
/// [spec]: https://tc39.es/ecma402/#sec-lookupmatcher
fn lookup_matcher<'provider, M: KeyedDataMarker>(
    requested_locales: &[Locale],
    icu: &Icu<'provider>,
) -> Locale
where
    BoaProvider<'provider>: DataProvider<M>,
{
    // 1. Let result be a new Record.
    // 2. For each element locale of requestedLocales, do
    for locale in requested_locales {
        // a. Let noExtensionsLocale be the String value that is locale with any Unicode locale
        //    extension sequences removed.
        let mut locale = locale.clone();
        let id = std::mem::take(&mut locale.id);

        // b. Let availableLocale be ! BestAvailableLocale(availableLocales, noExtensionsLocale).
        let available_locale = best_available_locale::<M>(id, &icu.provider());

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

/// Abstract operation [`BestFitMatcher ( availableLocales, requestedLocales )`][spec]
///
/// Compares `requestedLocales`, which must be a `List` as returned by `CanonicalizeLocaleList`,
/// against the locales in `availableLocales` and determines the best available language to
/// meet the request. The algorithm is implementation dependent, but should produce results
/// that a typical user of the requested locales would perceive as at least as good as those
/// produced by the `LookupMatcher` abstract operation.
///
/// [spec]: https://tc39.es/ecma402/#sec-bestfitmatcher
fn best_fit_matcher<'provider, M: KeyedDataMarker>(
    requested_locales: &[Locale],
    icu: &Icu<'provider>,
) -> Locale
where
    BoaProvider<'provider>: DataProvider<M>,
{
    for mut locale in requested_locales
        .iter()
        .cloned()
        .chain(std::iter::once_with(|| {
            default_locale(icu.locale_canonicalizer())
        }))
    {
        let id = std::mem::take(&mut locale.id);

        if let Some(available) = best_locale_for_provider(id, &icu.provider()) {
            locale.id = available;

            return locale;
        }
    }
    Locale::default()
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
pub(in crate::builtins::intl) fn resolve_locale<'provider, S>(
    requested_locales: &[Locale],
    options: &mut IntlOptions<S::LocaleOptions>,
    icu: &Icu<'provider>,
) -> Locale
where
    S: Service,
    BoaProvider<'provider>: DataProvider<S::LangMarker>,
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
    icu.locale_canonicalizer().canonicalize(&mut found_locale);
    found_locale
}

/// Abstract operation [`LookupSupportedLocales ( availableLocales, requestedLocales )`][spec]
///
/// Returns the subset of the provided BCP 47 language priority list requestedLocales for which
/// `availableLocales` has a matching locale when using the BCP 47 Lookup algorithm. Locales appear
/// in the same order in the returned list as in `requestedLocales`.
///
/// # Note
///
/// This differs a bit from the spec, since we don't have an `[[AvailableLocales]]`
/// list to compare with. However, we can do data requests to a [`DataProvider`]
/// in order to see if a certain [`Locale`] is supported.
///
/// [spec]: https://tc39.es/ecma402/#sec-lookupsupportedlocales
fn lookup_supported_locales<M: KeyedDataMarker>(
    requested_locales: &[Locale],
    provider: &impl DataProvider<M>,
) -> Vec<Locale> {
    // 1. Let subset be a new empty List.
    // 2. For each element locale of requestedLocales, do
    //     a. Let noExtensionsLocale be the String value that is locale with any Unicode locale extension sequences removed.
    //     b. Let availableLocale be ! BestAvailableLocale(availableLocales, noExtensionsLocale).
    //     c. If availableLocale is not undefined, append locale to the end of subset.
    // 3. Return subset.
    requested_locales
        .iter()
        .cloned()
        .filter(|loc| best_available_locale(loc.id.clone(), provider).is_some())
        .collect()
}

/// Abstract operation [`BestFitSupportedLocales ( availableLocales, requestedLocales )`][spec]
///
/// Returns the subset of the provided BCP 47 language priority list `requestedLocales` for which
/// `availableLocales` has a matching locale when using the Best Fit Matcher algorithm. Locales appear
/// in the same order in the returned list as in requestedLocales.
///
/// [spec]: https://tc39.es/ecma402/#sec-bestfitsupportedlocales
fn best_fit_supported_locales<M: KeyedDataMarker>(
    requested_locales: &[Locale],
    provider: &impl DataProvider<M>,
) -> Vec<Locale> {
    requested_locales
        .iter()
        .cloned()
        .filter(|loc| best_locale_for_provider(loc.id.clone(), provider).is_some())
        .collect()
}

/// Abstract operation [`SupportedLocales ( availableLocales, requestedLocales, options )`][spec]
///
/// Returns the subset of the provided BCP 47 language priority list requestedLocales for which
/// availableLocales has a matching locale
///
/// [spec]: https://tc39.es/ecma402/#sec-supportedlocales
pub(in crate::builtins::intl) fn supported_locales<'ctx, 'icu: 'ctx, M: KeyedDataMarker>(
    requested_locales: &[Locale],
    options: &JsValue,
    context: &'ctx mut Context<'icu>,
) -> JsResult<JsObject>
where
    BoaProvider<'icu>: DataProvider<M>,
{
    // 1. Set options to ? CoerceOptionsToObject(options).
    let options = coerce_options_to_object(options, context)?;

    // 2. Let matcher be ? GetOption(options, "localeMatcher", string, « "lookup", "best fit" », "best fit").
    let matcher = get_option::<LocaleMatcher>(&options, utf16!("localeMatcher"), false, context)?
        .unwrap_or_default();

    let elements = match matcher {
        // 4. Else,
        //     a. Let supportedLocales be LookupSupportedLocales(availableLocales, requestedLocales).
        LocaleMatcher::Lookup => {
            lookup_supported_locales(requested_locales, &context.icu().provider())
        }
        // 3. If matcher is "best fit", then
        //     a. Let supportedLocales be BestFitSupportedLocales(availableLocales, requestedLocales).
        LocaleMatcher::BestFit => {
            best_fit_supported_locales(requested_locales, &context.icu().provider())
        }
    };

    // 5. Return CreateArrayFromList(supportedLocales).
    Ok(Array::create_array_from_list(
        elements.into_iter().map(|loc| loc.to_string().into()),
        context,
    ))
}

/// Validates that the unicode extension `key` with `value` is a valid extension value for the
/// `language`.
pub(in crate::builtins::intl) fn validate_extension<M: KeyedDataMarker>(
    language: LanguageIdentifier,
    key: Key,
    value: &Value,
    provider: &impl DataProvider<M>,
) -> bool {
    let mut locale = DataLocale::from(language);
    locale.set_unicode_ext(key, value.clone());
    let request = DataRequest {
        locale: &locale,
        metadata: DataRequestMetadata::default(),
    };

    DataProvider::load(provider, request)
        .ok()
        .map(|res| res.metadata.locale.unwrap_or_else(|| locale.clone()))
        .filter(|loc| loc == &locale)
        .is_some()
}

#[cfg(test)]
mod tests {
    use icu_locid::{langid, locale, Locale};
    use icu_plurals::provider::CardinalV1Marker;
    use icu_provider::AsDeserializingBufferProvider;

    use crate::{
        builtins::intl::locale::utils::{
            best_available_locale, best_fit_matcher, default_locale, lookup_matcher,
        },
        context::icu::{BoaProvider, Icu},
    };

    #[test]
    fn best_avail_loc() {
        let provider = boa_icu_provider::buffer();
        let provider = provider.as_deserializing();

        assert_eq!(
            best_available_locale::<CardinalV1Marker>(langid!("en"), &provider),
            Some(langid!("en"))
        );

        assert_eq!(
            best_available_locale::<CardinalV1Marker>(langid!("es-ES"), &provider),
            Some(langid!("es"))
        );

        assert_eq!(
            best_available_locale::<CardinalV1Marker>(langid!("kr"), &provider),
            None
        );
    }

    #[test]
    fn lookup_match() {
        let provider = boa_icu_provider::buffer();
        let icu = Icu::new(BoaProvider::Buffer(provider)).unwrap();

        // requested: []

        let res = lookup_matcher::<CardinalV1Marker>(&[], &icu);
        assert_eq!(res, default_locale(icu.locale_canonicalizer()));
        assert!(res.extensions.is_empty());

        // requested: [fr-FR-u-hc-h12]
        let requested: Locale = "fr-FR-u-hc-h12".parse().unwrap();

        let result = lookup_matcher::<CardinalV1Marker>(&[requested.clone()], &icu);
        assert_eq!(result.id, langid!("fr"));
        assert_eq!(result.extensions, requested.extensions);

        // requested: [kr-KR-u-hc-h12, gr-GR-u-hc-h24-x-4a, es-ES-valencia-u-ca-gregory, uz-Cyrl]
        let kr = "kr-KR-u-hc-h12".parse().unwrap();
        let gr = "gr-GR-u-hc-h24-x-4a".parse().unwrap();
        let es: Locale = "es-ES-valencia-u-ca-gregory".parse().unwrap();
        let uz = locale!("uz-Cyrl");
        let requested = vec![kr, gr, es.clone(), uz];

        let res = best_fit_matcher::<CardinalV1Marker>(&requested, &icu);
        assert_eq!(res.id, langid!("es"));
        assert_eq!(res.extensions, es.extensions);
    }
}
