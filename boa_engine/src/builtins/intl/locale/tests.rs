// use icu_datetime::{pattern::CoarseHourCycle, provider::calendar::TimeLengthsV1Marker, options::preferences::HourCycle};
// use icu_locid::{
//     extensions::{
//         unicode::{Key, Value},
//         Extensions,
//     },
//     extensions_unicode_key as key, extensions_unicode_value as value, langid, locale, LanguageIdentifier, Locale,
// };
// use icu_provider::{hello_world::HelloWorldV1Marker, DataProvider, DataRequest, AsDowncastingAnyProvider, DataLocale};
// use serde::{de::value::StrDeserializer, Deserialize, Serialize};

// use crate::{
//     builtins::intl::{
//         locale::{best_available_locale, best_fit_matcher, default_locale, lookup_matcher},
//         ExtensionKey, Service,
//     },
//     context::icu::{BoaProvider, Icu},
// };

// #[test]
// fn best_avail_loc() {
//     let provider = icu_testdata::any();
//     let provider = provider.as_downcasting();

//     assert_eq!(
//         best_available_locale::<CardinalV1Marker>(langid!("en"), &provider),
//         Some(langid!("en"))
//     );

//     assert_eq!(
//         best_available_locale::<CardinalV1Marker>(langid!("es-ES"), &provider),
//         Some(langid!("es"))
//     );

//     assert_eq!(
//         best_available_locale::<CardinalV1Marker>(langid!("kr"), &provider),
//         None
//     );
// }

// #[test]
// fn lookup_match() {
//     let icu = Icu::new(BoaProvider::Any(Box::new(icu_testdata::any()))).unwrap();

//     // requested: []

//     let res = lookup_matcher::<CardinalV1Marker>(&[], &icu);
//     assert_eq!(res, default_locale(icu.locale_canonicalizer()));
//     assert!(res.extensions.is_empty());

//     // requested: [fr-FR-u-hc-h12]
//     let req: Locale = "fr-FR-u-hc-h12".parse().unwrap();

//     let res = lookup_matcher::<CardinalV1Marker>(&[req.clone()], &icu);
//     assert_eq!(res.id, langid!("fr"));
//     assert_eq!(res.extensions, req.extensions);

//     // requested: [kr-KR-u-hc-h12, gr-GR-u-hc-h24-x-4a, es-ES-valencia-u-ca-gregory, uz-Cyrl]
//     let kr: Locale = "kr-KR-u-hc-h12".parse().unwrap();
//     let gr: Locale = "gr-GR-u-hc-h24-x-4a".parse().unwrap();
//     let en: Locale = "es-ES-valencia-u-ca-gregory".parse().unwrap();
//     let uz = locale!("uz-Cyrl");
//     let req = vec![kr, gr, en.clone(), uz];

//     let res = best_fit_matcher::<CardinalV1Marker>(&req, &icu);
//     assert_eq!(res.id, langid!("es"));
//     assert_eq!(res.extensions, en.extensions);
// }

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
