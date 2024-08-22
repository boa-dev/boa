use crate::{
    builtins::{
        intl::{
            options::{coerce_options_to_object, IntlOptions, LocaleMatcher},
            Service,
        },
        options::get_option,
        Array,
    },
    context::icu::IntlProvider,
    js_string,
    object::JsObject,
    Context, JsNativeError, JsResult, JsValue,
};

use boa_macros::js_str;
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
    context: &mut Context,
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
            .map_or(false, |o| o.borrow().is::<Locale>())
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
        // c. If kPresent is true, then
        // c.i. Let kValue be ? Get(O, Pk).
        if let Some(k_value) = o.try_get(k, context)? {
            // ii. If Type(kValue) is not String or Object, throw a TypeError exception.
            if !(k_value.is_object() || k_value.is_string()) {
                return Err(JsNativeError::typ()
                    .with_message("locale should be a String or Object")
                    .into());
            }
            // iii. If Type(kValue) is Object and kValue has an [[InitializedLocale]] internal slot, then
            let mut tag = if let Some(tag) = k_value
                .as_object()
                .and_then(|obj| obj.borrow().downcast_ref::<Locale>().cloned())
            {
                // 1. Let tag be kValue.[[Locale]].
                tag
            }
            // iv. Else,
            else {
                // 1. Let tag be ? ToString(kValue).
                let k_value = k_value.to_string(context)?.to_std_string_escaped();
                if k_value.contains('_') {
                    return Err(JsNativeError::range()
                        .with_message("locale is not a structurally valid language tag")
                        .into());
                }

                k_value
                    .parse()
                    // v. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
                    .map_err(|_| {
                        JsNativeError::range()
                            .with_message("locale is not a structurally valid language tag")
                    })?
            };

            // vi. Let canonicalizedTag be CanonicalizeUnicodeLocaleId(tag).
            context
                .intl_provider()
                .locale_canonicalizer()?
                .canonicalize(&mut tag);

            // vii. If canonicalizedTag is not an element of seen, append canonicalizedTag as the last element of seen.
            seen.insert(tag);
        }
        // d. Increase k by 1.
    }

    // 8. Return seen.
    Ok(seen.into_iter().collect())
}

/// Abstract operation [`LookupMatchingLocaleByPrefix ( availableLocales, requestedLocales )`][prefix]
/// and [`LookupMatchingLocaleByBestFit ( availableLocales, requestedLocales )`][best]
///
/// Compares `requestedLocales`, which must be a `List` as returned by `CanonicalizeLocaleList`,
/// against the locales in `availableLocales` and determines the best available language to
/// meet the request.
///
/// # Notes
///
/// - This differs a bit from the spec, since we don't have an `[[AvailableLocales]]`
///   list to compare with. However, we can do data requests to a [`DataProvider`]
///   in order to see if a certain [`Locale`] is supported.
///
/// - Calling this function with a singleton `KeyedDataMarker` will always return `None`.
///
/// [prefix]: https://tc39.es/ecma402/#sec-lookupmatchinglocalebyprefix
/// [best]: https://tc39.es/ecma402/#sec-lookupmatchinglocalebybestfit
pub(crate) fn lookup_matching_locale_by_prefix<M: KeyedDataMarker>(
    requested_locales: impl IntoIterator<Item = Locale>,
    provider: &IntlProvider,
) -> Option<Locale>
where
    IntlProvider: DataProvider<M>,
{
    // 1. For each element locale of requestedLocales, do
    for locale in requested_locales {
        // a. Let extension be empty.
        // b. If locale contains a Unicode locale extension sequence, then
        //     i. Set extension to the Unicode locale extension sequence of locale.
        //     ii. Set locale to the String value that is locale with any Unicode locale extension sequences removed.
        let mut locale = locale.clone();
        let id = std::mem::take(&mut locale.id);
        locale.extensions.transform.clear();
        locale.extensions.private.clear();

        // c. Let prefix be locale.
        let mut prefix = id.into();

        // d. Repeat, while prefix is not the empty String,
        // We don't use a `while !prefix.is_und()` because it could be that prefix is und at the start,
        // so we need to make the request at least once.
        loop {
            // i. If availableLocales contains prefix, return the Record { [[locale]]: prefix, [[extension]]: extension }.
            // ICU4X requires doing data requests in order to check if a locale
            // is part of the set of supported locales.
            let response = DataProvider::<M>::load(
                provider,
                DataRequest {
                    locale: &prefix,
                    metadata: {
                        let mut metadata = DataRequestMetadata::default();
                        metadata.silent = true;
                        metadata
                    },
                },
            );

            if let Ok(req) = response {
                // `metadata.locale` returns None when the provider doesn't have a fallback mechanism,
                // but supports the required locale. However, if the provider has a fallback mechanism,
                // this will return `Some(locale)`, where the locale is the used locale after applying
                // the fallback algorithm, even if the used locale is exactly the same as the required
                // locale.
                match req.metadata.locale {
                    Some(loc) if loc.get_langid() == prefix.get_langid() => {
                        locale.id = loc.into_locale().id;
                        return Some(locale);
                    }
                    None => {
                        locale.id = prefix.into_locale().id;
                        return Some(locale);
                    }
                    _ => {}
                }
            }

            // ii. If prefix contains "-" (code unit 0x002D HYPHEN-MINUS), let pos be the index into prefix of the last occurrence of "-"; else let pos be 0.
            // iii. Repeat, while pos ≥ 2 and the substring of prefix from pos - 2 to pos - 1 is "-",
            //     1. Set pos to pos - 2.
            // iv. Set prefix to the substring of prefix from 0 to pos.
            // Since the definition of `LanguageIdentifier` allows us to manipulate it
            // without using strings, we can replace these steps by a simpler
            // algorithm.
            if prefix.has_variants() {
                let mut variants = prefix.clear_variants().iter().copied().collect::<Vec<_>>();
                variants.pop();
                prefix.set_variants(Variants::from_vec_unchecked(variants));
            } else if prefix.region().is_some() {
                prefix.set_region(None);
            } else if prefix.script().is_some() {
                prefix.set_script(None);
            } else {
                break;
            }
        }
    }

    // 2. Return undefined.
    None
}

/// Abstract operation [`LookupMatchingLocaleByBestFit ( availableLocales, requestedLocales )`][spec]
///
/// Compares `requestedLocales`, which must be a `List` as returned by `CanonicalizeLocaleList`,
/// against the locales in `availableLocales` and determines the best available language to
/// meet the request. The algorithm is implementation dependent, but should produce results
/// that a typical user of the requested locales would perceive as at least as good as those
/// produced by the `LookupMatcher` abstract operation.
///
/// [spec]: https://tc39.es/ecma402/#sec-bestfitmatcher
fn lookup_matching_locale_by_best_fit<M: KeyedDataMarker>(
    requested_locales: impl IntoIterator<Item = Locale>,
    provider: &IntlProvider,
) -> Option<Locale>
where
    IntlProvider: DataProvider<M>,
{
    for mut locale in requested_locales {
        let id = std::mem::take(&mut locale.id);

        // Only leave unicode extensions when returning the locale.
        locale.extensions.transform.clear();
        locale.extensions.private.clear();

        let Ok(response) = DataProvider::<M>::load(
            provider,
            DataRequest {
                locale: &DataLocale::from(&id),
                metadata: {
                    let mut md = DataRequestMetadata::default();
                    md.silent = true;
                    md
                },
            },
        ) else {
            continue;
        };

        if id == LanguageIdentifier::UND {
            return Some(locale);
        }

        if let Some(id) = response
            .metadata
            .locale
            .map(|dl| dl.into_locale().id)
            .or(Some(id))
            .filter(|loc| loc != &LanguageIdentifier::UND)
        {
            locale.id = id;
            return Some(locale);
        }
    }
    None
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
pub(in crate::builtins::intl) fn resolve_locale<S>(
    requested_locales: impl IntoIterator<Item = Locale>,
    options: &mut IntlOptions<S::LocaleOptions>,
    provider: &IntlProvider,
) -> JsResult<Locale>
where
    S: Service,
    IntlProvider: DataProvider<S::LangMarker>,
{
    // 1. Let matcher be options.[[localeMatcher]].
    // 2. If matcher is "lookup", then
    //     a. Let r be LookupMatchingLocaleByPrefix(availableLocales, requestedLocales).
    // 3. Else,
    //     a. Let r be LookupMatchingLocaleByBestFit(availableLocales, requestedLocales).
    // 4. If r is undefined, set r to the Record { [[locale]]: DefaultLocale(), [[extension]]: empty }.
    let found_locale = if options.matcher == LocaleMatcher::Lookup {
        lookup_matching_locale_by_prefix::<S::LangMarker>(requested_locales, provider)
    } else {
        lookup_matching_locale_by_best_fit::<S::LangMarker>(requested_locales, provider)
    };

    let mut found_locale = if let Some(loc) = found_locale {
        loc
    } else {
        default_locale(provider.locale_canonicalizer()?)
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
    S::resolve(&mut found_locale, &mut options.service_options, provider);
    provider
        .locale_canonicalizer()?
        .canonicalize(&mut found_locale);
    Ok(found_locale)
}

/// Abstract operation [`FilterLocales ( availableLocales, requestedLocales, options )`][spec]
///
/// Returns the subset of the provided BCP 47 language priority list requestedLocales for which
/// availableLocales has a matching locale.
///
/// # Note
///
/// Calling this function with a singleton `KeyedDataMarker` will always return `None`.
///
/// [spec]: https://tc39.es/ecma402/#sec-supportedlocales
pub(in crate::builtins::intl) fn filter_locales<M: KeyedDataMarker>(
    requested_locales: Vec<Locale>,
    options: &JsValue,
    context: &mut Context,
) -> JsResult<JsObject>
where
    IntlProvider: DataProvider<M>,
{
    // 1. Set options to ? CoerceOptionsToObject(options).
    let options = coerce_options_to_object(options, context)?;

    // 2. Let matcher be ? GetOption(options, "localeMatcher", string, « "lookup", "best fit" », "best fit").
    let matcher = get_option(&options, js_str!("localeMatcher"), context)?.unwrap_or_default();

    // 3. Let subset be a new empty List.
    let mut subset = Vec::with_capacity(requested_locales.len());

    // 4. For each element locale of requestedLocales, do
    for locale in requested_locales {
        // a. Let noExtensionsLocale be the String value that is locale with any Unicode locale extension sequences removed.
        let mut no_ext_loc = locale.clone();
        no_ext_loc.extensions.unicode.clear();
        let loc_match = match matcher {
            // b. If matcher is "lookup", then
            //     i. Let match be LookupMatchingLocaleByPrefix(availableLocales, noExtensionsLocale).
            LocaleMatcher::Lookup => {
                lookup_matching_locale_by_prefix([no_ext_loc], context.intl_provider())
            }
            // c. Else,
            //     i. Let match be LookupMatchingLocaleByBestFit(availableLocales, noExtensionsLocale).
            LocaleMatcher::BestFit => {
                lookup_matching_locale_by_best_fit([no_ext_loc], context.intl_provider())
            }
        };

        // d. If match is not undefined, append locale to subset.
        if loc_match.is_some() {
            subset.push(locale);
        }
    }

    // 5. Return CreateArrayFromList(subset).
    Ok(Array::create_array_from_list(
        subset
            .into_iter()
            .map(|loc| js_string!(loc.to_string()).into()),
        context,
    ))
}

/// Validates that the unicode extension `key` with `value` is a valid extension value for the
/// `language`.
///
/// # Note
///
/// Calling this function with a singleton `KeyedDataMarker` will always return `None`.
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

#[cfg(all(test, feature = "intl_bundled"))]
mod tests {
    use icu_locid::{langid, locale, Locale};
    use icu_plurals::provider::CardinalV1Marker;

    use crate::{
        builtins::intl::locale::utils::{
            lookup_matching_locale_by_best_fit, lookup_matching_locale_by_prefix,
        },
        context::icu::IntlProvider,
    };

    #[test]
    fn best_fit() {
        let icu = &IntlProvider::try_new_with_buffer_provider(boa_icu_provider::buffer());

        assert_eq!(
            lookup_matching_locale_by_best_fit::<CardinalV1Marker>([locale!("en")], icu),
            Some(locale!("en"))
        );

        assert_eq!(
            lookup_matching_locale_by_best_fit::<CardinalV1Marker>([locale!("es-ES")], icu),
            Some(locale!("es"))
        );

        assert_eq!(
            lookup_matching_locale_by_best_fit::<CardinalV1Marker>([locale!("kr")], icu),
            None
        );
    }

    #[test]
    fn lookup_match() {
        let icu = &IntlProvider::try_new_with_buffer_provider(boa_icu_provider::buffer());

        // requested: [fr-FR-u-hc-h12]
        let requested: Locale = "fr-FR-u-hc-h12".parse().unwrap();

        let result =
            lookup_matching_locale_by_prefix::<CardinalV1Marker>([requested.clone()], icu).unwrap();
        assert_eq!(result.id, langid!("fr"));
        assert_eq!(result.extensions, requested.extensions);

        // requested: [kr-KR-u-hc-h12, gr-GR-u-hc-h24-x-4a, es-ES-valencia-u-ca-gregory, uz-Cyrl]
        let kr = "kr-KR-u-hc-h12".parse().unwrap();
        let gr = "gr-GR-u-hc-h24-x-4a".parse().unwrap();
        let es: Locale = "es-ES-valencia-u-ca-gregory".parse().unwrap();
        let uz = locale!("uz-Cyrl");
        let requested = vec![kr, gr, es.clone(), uz];

        let res = lookup_matching_locale_by_best_fit::<CardinalV1Marker>(requested, icu).unwrap();
        assert_eq!(res.id, langid!("es"));
        assert_eq!(res.extensions, es.extensions);
    }
}
