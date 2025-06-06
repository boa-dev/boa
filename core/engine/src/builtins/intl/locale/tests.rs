use icu_decimal::provider::DecimalSymbolsV1;
use icu_locale::{
    extensions::unicode::Value, extensions_unicode_key as key, extensions_unicode_value as value,
    locale, preferences::extensions::unicode::keywords::NumberingSystem, Locale,
};
use icu_plurals::provider::PluralsCardinalV1;
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
    nu: Option<NumberingSystem>,
}

struct TestService;

impl Service for TestService {
    type LangMarker = PluralsCardinalV1;

    type LocaleOptions = TestOptions;

    fn resolve(locale: &mut Locale, options: &mut Self::LocaleOptions, provider: &IntlProvider) {
        let loc_hc = locale
            .extensions
            .unicode
            .keywords
            .get(&key!("nu"))
            .and_then(|v| NumberingSystem::try_from(v.clone()).ok());
        let nu = options.nu.or(loc_hc).unwrap_or_else(|| {
            let locale = &DataLocale::from(&*locale);
            let req = DataRequest {
                id: DataIdentifierBorrowed::for_locale(locale),
                metadata: DataRequestMetadata::default(),
            };
            let data = DataProvider::<DecimalSymbolsV1>::load(provider, req).unwrap();
            let preferred = data.payload.get().numsys();
            NumberingSystem::try_from(Value::try_from_str(preferred).unwrap()).unwrap()
        });
        locale
            .extensions
            .unicode
            .keywords
            .set(key!("nu"), nu.into());
        options.nu = Some(nu);
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
        .set(key!("nu"), value!("latn"));

    // test lookup
    let mut options = IntlOptions {
        matcher: LocaleMatcher::Lookup,
        service_options: TestOptions {
            nu: Some(NumberingSystem::try_from(value!("latn")).unwrap()),
        },
    };
    let locale = resolve_locale::<TestService>([], &mut options, &provider).unwrap();
    assert_eq!(locale, default);

    // test best fit
    let mut options = IntlOptions {
        matcher: LocaleMatcher::BestFit,
        service_options: TestOptions {
            nu: Some(NumberingSystem::try_from(value!("latn")).unwrap()),
        },
    };

    let locale = resolve_locale::<TestService>([], &mut options, &provider).unwrap();
    assert_eq!(locale, default);

    // requested: [es-ES]
    let mut options = IntlOptions {
        matcher: LocaleMatcher::Lookup,
        service_options: TestOptions { nu: None },
    };

    let locale =
        resolve_locale::<TestService>([locale!("bn-Arab")], &mut options, &provider).unwrap();
    assert_eq!(locale, "bn-u-nu-beng".parse().unwrap());
}
