#![allow(missing_docs, rustdoc::missing_crate_level_docs)]

use std::{error::Error, fs::File, path::Path};

use icu_datagen::blob_exporter::BlobExporter;
use icu_datagen::prelude::*;
use icu_provider::data_key;

const KEYS_LEN: usize = 129;

/// List of keys used by `Intl` components.
///
/// This must be kept in sync with the list of implemented components of `Intl`.
const KEYS: [DataKey; KEYS_LEN] = {
    const CENTINEL_KEY: DataKey = data_key!("centinel@1");
    const SERVICES: [&[DataKey]; 9] = [
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

    let mut array = [CENTINEL_KEY; KEYS_LEN];

    let mut offset = 0;
    let mut service_idx = 0;

    while service_idx < SERVICES.len() {
        let service = SERVICES[service_idx];
        let mut idx = 0;
        while idx < service.len() {
            array[offset + idx] = service[idx];
            idx += 1;
        }

        offset += service.len();
        service_idx += 1;
    }

    assert!(offset == array.len());

    array
};

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

    log::info!("Generating ICU4X data for keys: {:#?}", KEYS);

    let provider = DatagenProvider::new_latest_tested();
    let locales = provider
        .locales_for_coverage_levels([CoverageLevel::Modern])?
        .into_iter()
        .chain([langid!("en-US")]);

    DatagenDriver::new()
        .with_keys(KEYS)
        .with_locales_and_fallback(locales.map(LocaleFamily::with_descendants), {
            let mut options = FallbackOptions::default();
            options.deduplication_strategy = Some(DeduplicationStrategy::None);
            options
        })
        .with_additional_collations([String::from("search*")])
        .with_recommended_segmenter_models()
        .export(
            &provider,
            BlobExporter::new_v2_with_sink(Box::new(File::create(path.join("icudata.postcard"))?)),
        )?;

    Ok(())
}
