use icu_datetime::{
    options::preferences::HourCycle, pattern::CoarseHourCycle,
    provider::calendar::TimeLengthsV1Marker,
};
use icu_locid::{
    extensions::unicode::Value, extensions_unicode_key as key, extensions_unicode_value as value,
    Locale,
};
use icu_plurals::provider::CardinalV1Marker;
use icu_provider::{DataLocale, DataProvider, DataRequest, DataRequestMetadata};

use crate::builtins::intl::Service;

struct TestOptions {
    hc: Option<HourCycle>,
}

struct TestService;

impl<P> Service<P> for TestService
where
    P: DataProvider<TimeLengthsV1Marker>,
{
    type LangMarker = CardinalV1Marker;

    type Options = TestOptions;

    fn resolve(locale: &mut Locale, options: &mut Self::Options, provider: &P) {
        let loc_hc = locale
            .extensions
            .unicode
            .keywords
            .get(&key!("hc"))
            .and_then(Value::as_single_subtag)
            .and_then(|s| match &**s {
                "h11" => Some(HourCycle::H11),
                "h12" => Some(HourCycle::H12),
                "h23" => Some(HourCycle::H23),
                "h24" => Some(HourCycle::H24),
                _ => None,
            });
        let hc = options.hc.or(loc_hc).unwrap_or_else(|| {
            let req = DataRequest {
                locale: &DataLocale::from(&*locale),
                metadata: DataRequestMetadata::default(),
            };
            let preferred = DataProvider::<TimeLengthsV1Marker>::load(provider, req)
                .unwrap()
                .take_payload()
                .unwrap()
                .get()
                .preferred_hour_cycle;
            match preferred {
                CoarseHourCycle::H11H12 => HourCycle::H11,
                CoarseHourCycle::H23H24 => HourCycle::H23,
            }
        });
        let hc_value = match hc {
            HourCycle::H11 => value!("h11"),
            HourCycle::H12 => value!("h12"),
            HourCycle::H23 => value!("h23"),
            HourCycle::H24 => value!("h24"),
        };
        locale.extensions.unicode.keywords.set(key!("hc"), hc_value);
        options.hc = Some(hc);
    }
}

// // #[test]
// // fn locale_resolution() {
// //     let mut context = Context::default();

// //     // test lookup
// //     let available_locales = Vec::<JsString>::new();
// //     let requested_locales = Vec::<JsString>::new();
// //     let relevant_extension_keys = Vec::<JsString>::new();
// //     let locale_data = FxHashMap::default();
// //     let options = DateTimeFormatRecord {
// //         locale_matcher: JsString::new("lookup"),
// //         properties: FxHashMap::default(),
// //     };

// //     let locale_record = resolve_locale(
// //         &available_locales,
// //         &requested_locales,
// //         &options,
// //         &relevant_extension_keys,
// //         &locale_data,
// //         &mut context,
// //     );
// //     assert_eq!(
// //         locale_record.locale,
// //         default_locale(context.icu().locale_canonicalizer())
// //             .to_string()
// //             .as_str()
// //     );
// //     assert_eq!(
// //         locale_record.data_locale,
// //         default_locale(context.icu().locale_canonicalizer())
// //             .to_string()
// //             .as_str()
// //     );
// //     assert!(locale_record.properties.is_empty());

// //     // test best fit
// //     let available_locales = Vec::<JsString>::new();
// //     let requested_locales = Vec::<JsString>::new();
// //     let relevant_extension_keys = Vec::<JsString>::new();
// //     let locale_data = FxHashMap::default();
// //     let options = DateTimeFormatRecord {
// //         locale_matcher: JsString::new("best-fit"),
// //         properties: FxHashMap::default(),
// //     };

// //     let locale_record = resolve_locale(
// //         &available_locales,
// //         &requested_locales,
// //         &options,
// //         &relevant_extension_keys,
// //         &locale_data,
// //         &mut context,
// //     );
// //     assert_eq!(
// //         locale_record.locale,
// //         default_locale(context.icu().locale_canonicalizer())
// //             .to_string()
// //             .as_str()
// //     );
// //     assert_eq!(
// //         locale_record.data_locale,
// //         default_locale(context.icu().locale_canonicalizer())
// //             .to_string()
// //             .as_str()
// //     );
// //     assert!(locale_record.properties.is_empty());

// //     // available: [es-ES], requested: [es-ES]
// //     let available_locales = vec![JsString::new("es-ES")];
// //     let requested_locales = vec![JsString::new("es-ES")];
// //     let relevant_extension_keys = Vec::<JsString>::new();
// //     let locale_data = FxHashMap::default();
// //     let options = DateTimeFormatRecord {
// //         locale_matcher: JsString::new("lookup"),
// //         properties: FxHashMap::default(),
// //     };

// //     let locale_record = resolve_locale(
// //         &available_locales,
// //         &requested_locales,
// //         &options,
// //         &relevant_extension_keys,
// //         &locale_data,
// //         &mut context,
// //     );
// //     assert_eq!(locale_record.locale, "es-ES");
// //     assert_eq!(locale_record.data_locale, "es-ES");
// //     assert!(locale_record.properties.is_empty());

// //     // available: [zh-CN], requested: []
// //     let available_locales = vec![JsString::new("zh-CN")];
// //     let requested_locales = Vec::<JsString>::new();
// //     let relevant_extension_keys = Vec::<JsString>::new();
// //     let locale_data = FxHashMap::default();
// //     let options = DateTimeFormatRecord {
// //         locale_matcher: JsString::new("lookup"),
// //         properties: FxHashMap::default(),
// //     };

// //     let locale_record = resolve_locale(
// //         &available_locales,
// //         &requested_locales,
// //         &options,
// //         &relevant_extension_keys,
// //         &locale_data,
// //         &mut context,
// //     );
// //     assert_eq!(
// //         locale_record.locale,
// //         default_locale(context.icu().locale_canonicalizer())
// //             .to_string()
// //             .as_str()
// //     );
// //     assert_eq!(
// //         locale_record.data_locale,
// //         default_locale(context.icu().locale_canonicalizer())
// //             .to_string()
// //             .as_str()
// //     );
// //     assert!(locale_record.properties.is_empty());
// // }
