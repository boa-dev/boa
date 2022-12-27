use icu_datetime::{
    options::preferences::HourCycle, pattern::CoarseHourCycle,
    provider::calendar::TimeLengthsV1Marker,
};
use icu_locid::{
    extensions::unicode::Value, extensions_unicode_key as key, extensions_unicode_value as value,
    locale, Locale,
};
use icu_plurals::provider::CardinalV1Marker;
use icu_provider::{DataLocale, DataProvider, DataRequest, DataRequestMetadata};

use crate::{
    builtins::intl::{
        locale::{best_locale_for_provider, default_locale, resolve_locale},
        options::{IntlOptions, LocaleMatcher},
        Service,
    },
    context::icu::{BoaProvider, Icu},
};

#[derive(Debug)]
struct TestOptions {
    hc: Option<HourCycle>,
}

struct TestService;

impl Service for TestService {
    type LangMarker = CardinalV1Marker;

    type LocaleOptions = TestOptions;

    fn resolve(locale: &mut Locale, options: &mut Self::LocaleOptions, provider: BoaProvider<'_>) {
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
            let preferred = DataProvider::<TimeLengthsV1Marker>::load(&provider, req)
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

#[test]
fn locale_resolution() {
    let provider = boa_icu_provider::buffer();
    let icu = Icu::new(BoaProvider::Buffer(provider)).unwrap();
    let mut default = default_locale(icu.locale_canonicalizer());
    default
        .extensions
        .unicode
        .keywords
        .set(key!("hc"), value!("h11"));

    // test lookup
    let mut options = IntlOptions {
        matcher: LocaleMatcher::Lookup,
        service_options: TestOptions {
            hc: Some(HourCycle::H11),
        },
    };
    let locale = resolve_locale::<TestService>(&[], &mut options, &icu);
    assert_eq!(locale, default);

    // test best fit
    let mut options = IntlOptions {
        matcher: LocaleMatcher::BestFit,
        service_options: TestOptions {
            hc: Some(HourCycle::H11),
        },
    };

    let locale = resolve_locale::<TestService>(&[], &mut options, &icu);
    let best = best_locale_for_provider::<<TestService as Service>::LangMarker>(
        default.id.clone(),
        &icu.provider(),
    )
    .unwrap();
    let mut best = Locale::from(best);
    best.extensions = locale.extensions.clone();
    assert_eq!(locale, best);

    // requested: [es-ES]
    let mut options = IntlOptions {
        matcher: LocaleMatcher::Lookup,
        service_options: TestOptions { hc: None },
    };

    let locale = resolve_locale::<TestService>(&[locale!("es-AR")], &mut options, &icu);
    assert_eq!(locale, "es-u-hc-h23".parse().unwrap());
}
