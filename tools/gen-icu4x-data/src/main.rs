#![allow(missing_docs, rustdoc::missing_crate_level_docs)]

use std::{error::Error, fs::File, path::Path};

use icu_datagen::{CoverageLevel, DatagenDriver, DatagenProvider};
use icu_plurals::provider::{PluralRangesV1, PluralRangesV1Marker};
use icu_provider::{
    datagen::{ExportMarker, IterableDynamicDataProvider},
    dynutil::UpcastDataPayload,
    prelude::*,
};
use icu_provider_blob::export::BlobExporter;

/// Hack that associates the `und` locale with an empty plural ranges data.
/// This enables the default behaviour for all locales without data.
#[derive(Debug)]
struct PluralRangesFallbackHack(DatagenProvider);

// We definitely don't want to import dependencies just to do `T::default`.
#[allow(clippy::default_trait_access)]
impl DynamicDataProvider<AnyMarker> for PluralRangesFallbackHack {
    fn load_data(
        &self,
        key: DataKey,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<AnyMarker>, DataError> {
        if req.locale.is_und() && key.hashed() == PluralRangesV1Marker::KEY.hashed() {
            let payload = <AnyMarker as UpcastDataPayload<PluralRangesV1Marker>>::upcast(
                DataPayload::from_owned(PluralRangesV1 {
                    ranges: Default::default(),
                }),
            );
            Ok(DataResponse {
                metadata: DataResponseMetadata::default(),
                payload: Some(payload),
            })
        } else {
            self.0.load_data(key, req)
        }
    }
}

#[allow(clippy::default_trait_access)]
impl DynamicDataProvider<ExportMarker> for PluralRangesFallbackHack {
    fn load_data(
        &self,
        key: DataKey,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<ExportMarker>, DataError> {
        if req.locale.is_und() && key.hashed() == PluralRangesV1Marker::KEY.hashed() {
            let payload = <ExportMarker as UpcastDataPayload<PluralRangesV1Marker>>::upcast(
                DataPayload::from_owned(PluralRangesV1 {
                    ranges: Default::default(),
                }),
            );
            Ok(DataResponse {
                metadata: DataResponseMetadata::default(),
                payload: Some(payload),
            })
        } else {
            self.0.load_data(key, req)
        }
    }
}

impl IterableDynamicDataProvider<ExportMarker> for PluralRangesFallbackHack {
    fn supported_locales_for_key(&self, key: DataKey) -> Result<Vec<DataLocale>, DataError> {
        if key.hashed() == PluralRangesV1Marker::KEY.hashed() {
            let mut locales = self.0.supported_locales_for_key(key)?;
            locales.push(DataLocale::default());
            Ok(locales)
        } else {
            self.0.supported_locales_for_key(key)
        }
    }
}

/// List of keys used by `Intl` components.
///
/// This must be kept in sync with the list of implemented components of `Intl`.
const KEYS: [&[DataKey]; 9] = [
    icu_casemap::provider::KEYS,
    icu_collator::provider::KEYS,
    icu_datetime::provider::KEYS,
    icu_decimal::provider::KEYS,
    icu_list::provider::KEYS,
    icu_locid_transform::provider::KEYS,
    icu_normalizer::provider::KEYS,
    icu_plurals::provider::KEYS,
    icu_segmenter::provider::KEYS,
];

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new()
        .env()
        .with_level(log::LevelFilter::Info)
        .init()?;

    let path = Path::new("core/icu_provider/data");

    // Removal will throw an error if the directory doesn't exist, hence
    // why we can ignore the error.
    let _unused = std::fs::remove_dir_all(path);
    std::fs::create_dir_all(path)?;

    log::info!("Generating ICU4X data for keys: {:?}", KEYS);

    let provider = DatagenProvider::new_latest_tested();

    DatagenDriver::new()
        .with_keys(KEYS.into_iter().flatten().copied())
        .with_locales(provider.locales_for_coverage_levels([CoverageLevel::Modern])?)
        .with_additional_collations([String::from("search*")])
        .with_recommended_segmenter_models()
        .export(
            &PluralRangesFallbackHack(provider),
            BlobExporter::new_with_sink(Box::new(File::create(path.join("icudata.postcard"))?)),
        )?;

    Ok(())
}
