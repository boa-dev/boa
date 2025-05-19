use icu_datetime::provider::{calendar::TimeLengthsV1Marker, pattern::CoarseHourCycle};
use icu_locale::{
    extensions_unicode_key as key, extensions_unicode_value as value, locale,
    preferences::extensions::unicode::keywords::HourCycle, Locale,
};
use icu_plurals::provider::CardinalV1Marker;
use icu_provider::{
    DataIdentifierBorrowed, DataLocale, DataProvider, DataRequest, DataRequestMetadata,
};

use crate::{
    builtins::intl::{
        locale::{default_locale, resolve_locale},
        options::{IntlOptions, LocaleMatcher},
        Service,
    },
    context::icu::IntlProvider,
};

#[derive(Debug)]
struct TestOptions {
    hc: Option<HourCycle>,
}

struct TestService;

impl Service for TestService {
    type LangMarker = CardinalV1Marker;

    type LocaleOptions = TestOptions;

    fn resolve(locale: &mut Locale, options: &mut Self::LocaleOptions, provider: &IntlProvider) {
        let loc_hc = locale
            .extensions
            .unicode
            .keywords
            .get(&key!("hc"))
            .and_then(|v| HourCycle::try_from(v).ok());
        let hc = options.hc.or(loc_hc).unwrap_or_else(|| {
            let locale = &DataLocale::from(&*locale);
            let req = DataRequest {
                id: DataIdentifierBorrowed::for_locale(locale),
                metadata: DataRequestMetadata::default(),
            };
            let preferred = DataProvider::<TimeLengthsV1Marker>::load(provider, req)
                .unwrap()
                .payload
                .get()
                .preferred_hour_cycle;
            match preferred {
                CoarseHourCycle::H11H12 => HourCycle::H11,
                CoarseHourCycle::H23H24 => HourCycle::H23,
            }
        });
        locale
            .extensions
            .unicode
            .keywords
            .set(key!("hc"), hc.into());
        options.hc = Some(hc);
    }
}

#[test]
fn locale_resolution() {
    let provider = IntlProvider::try_new_buffer(boa_icu_provider::buffer());
    let mut default = default_locale(provider.locale_canonicalizer().unwrap());
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
    let locale = resolve_locale::<TestService>([], &mut options, &provider).unwrap();
    assert_eq!(locale, default);

    // test best fit
    let mut options = IntlOptions {
        matcher: LocaleMatcher::BestFit,
        service_options: TestOptions {
            hc: Some(HourCycle::H11),
        },
    };

    let locale = resolve_locale::<TestService>([], &mut options, &provider).unwrap();
    assert_eq!(locale, default);

    // requested: [es-ES]
    let mut options = IntlOptions {
        matcher: LocaleMatcher::Lookup,
        service_options: TestOptions { hc: None },
    };

    let locale =
        resolve_locale::<TestService>([locale!("es-AR")], &mut options, &provider).unwrap();
    assert_eq!(locale, "es-u-hc-h23".parse().unwrap());
}
