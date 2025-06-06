#![allow(missing_docs, rustdoc::missing_crate_level_docs)]

use std::path::Path;
use std::{error::Error, fs::File};

use icu_provider_export::blob_exporter::BlobExporter;
use icu_provider_export::prelude::*;
use icu_provider_source::{CoverageLevel, SourceDataProvider};

/// Path to the directory where the exported data lives.
const EXPORT_PATH: &str = "core/icu_provider/data";

/// List of services used by `Intl` components.
///
/// This must be kept in sync with the list of implemented services for `Intl`.
const SERVICES: &[(&str, &[DataMarkerInfo])] = &[
    ("icu_casemap", icu_casemap::provider::MARKERS),
    ("icu_collator", icu_collator::provider::MARKERS),
    ("icu_datetime", icu_datetime::provider::MARKERS),
    ("icu_decimal", icu_decimal::provider::MARKERS),
    ("icu_list", icu_list::provider::MARKERS),
    ("icu_locale", icu_locale::provider::MARKERS),
    ("icu_normalizer", icu_normalizer::provider::MARKERS),
    ("icu_plurals", icu_plurals::provider::MARKERS),
    ("icu_segmenter", icu_segmenter::provider::MARKERS),
];

fn export_for_service(
    service: &str,
    markers: &[DataMarkerInfo],
    provider: &SourceDataProvider,
    driver: ExportDriver,
) -> Result<(), Box<dyn Error>> {
    log::info!("Generating ICU4X data for service `{service}` with markers: {markers:#?}");

    let export_path = Path::new(EXPORT_PATH);
    let export_file = export_path.join(format!("{service}.postcard"));

    driver.with_markers(markers.iter().copied()).export(
        provider,
        BlobExporter::new_with_sink(Box::new(File::create(export_file)?)),
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

    let provider = &SourceDataProvider::new();
    let locales = provider
        .locales_for_coverage_levels([CoverageLevel::Modern])?
        .into_iter()
        .map(DataLocaleFamily::with_descendants);
    // .chain([langid!("en-US")]);

    let driver = ExportDriver::new(
        locales,
        DeduplicationStrategy::RetainBaseLanguages.into(),
        LocaleFallbacker::try_new_unstable(provider)?,
    )
    .with_additional_collations([String::from("search*")])
    .with_recommended_segmenter_models();

    for (service, keys) in SERVICES {
        export_for_service(service, keys, provider, driver.clone())?;
    }

    Ok(())
}
