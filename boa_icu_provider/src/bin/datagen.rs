#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]

use std::{error::Error, fs::File};

use boa_icu_provider::data_root;
use icu_datagen::{all_keys, CoverageLevel, DatagenDriver, DatagenProvider};
use icu_provider_blob::export::BlobExporter;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new()
        .env()
        .with_level(log::LevelFilter::Info)
        .init()?;

    let provider = DatagenProvider::new_latest_tested();

    DatagenDriver::new()
        .with_keys(all_keys())
        .with_locales(provider.locales_for_coverage_levels([CoverageLevel::Modern])?)
        .with_additional_collations([String::from("search*")])
        .export(
            &provider,
            BlobExporter::new_with_sink(Box::new(File::create(
                data_root().join("icudata.postcard"),
            )?)),
        )?;

    Ok(())
}
