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

use super::Service;

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
