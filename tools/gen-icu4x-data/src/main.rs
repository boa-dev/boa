#![allow(missing_docs, rustdoc::missing_crate_level_docs)]

use std::path::Path;
use std::{error::Error, fs::File};

use icu_datagen::blob_exporter::BlobExporter;
use icu_datagen::prelude::*;

/// Path to the directory where the exported data lives.
const EXPORT_PATH: &str = "core/icu_provider/data";

/// List of services used by `Intl` components.
///
/// This must be kept in sync with the list of implemented services of `Intl`.
const SERVICES: &[(&str, &[DataKey])] = &[
    ("icu_casemap", icu_casemap::provider::KEYS),
    ("icu_collator", icu_collator::provider::KEYS),
    ("icu_datetime", icu_datetime::provider::KEYS),
    ("icu_decimal", icu_decimal::provider::KEYS),
    ("icu_list", icu_list::provider::KEYS),
    ("icu_locid_transform", icu_locid_transform::provider::KEYS),
    ("icu_normalizer", icu_normalizer::provider::KEYS),
    ("icu_plurals", icu_plurals::provider::KEYS),
    ("icu_segmenter", icu_segmenter::provider::KEYS),
];

fn export_for_service(
    service: &str,
    keys: &[DataKey],
    provider: &DatagenProvider,
    driver: DatagenDriver,
) -> Result<(), Box<dyn Error>> {
    log::info!(
        "Generating ICU4X data for service `{service}` with keys: {:#?}",
        keys
    );

    let export_path = Path::new(EXPORT_PATH);
    let export_file = export_path.join(format!("{service}.postcard"));

    driver.with_keys(keys.iter().copied()).export(
        provider,
        BlobExporter::new_v2_with_sink(Box::new(File::create(export_file)?)),
    )?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new()
        .env()
        .with_level(log::LevelFilter::Info)
        .init()?;

    // Removal will throw an error if the directory doesn't exist, hence
    // why we can ignore the error.
    let _unused = std::fs::remove_dir_all(EXPORT_PATH);
    std::fs::create_dir_all(EXPORT_PATH)?;

    let provider = &DatagenProvider::new_latest_tested();
    let locales = provider
        .locales_for_coverage_levels([CoverageLevel::Modern])?
        .into_iter()
        .chain([langid!("en-US")]);

    let driver = DatagenDriver::new()
        .with_locales_and_fallback(locales.map(LocaleFamily::with_descendants), {
            let mut options = FallbackOptions::default();
            options.deduplication_strategy = Some(DeduplicationStrategy::None);
            options
        })
        .with_additional_collations([String::from("search*")])
        .with_recommended_segmenter_models();

    for (service, keys) in SERVICES {
        export_for_service(service, keys, provider, driver.clone())?;
    }

    Ok(())
}
