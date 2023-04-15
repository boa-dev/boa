#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]

use std::{error::Error, fs::File};

use boa_icu_provider::data_root;
use icu_datagen::{all_keys_with_experimental, datagen, CldrLocaleSubset, Out, SourceData};

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new()
        .env()
        .with_level(log::LevelFilter::Info)
        .init()?;

    let source_data = SourceData::default()
        .with_cldr_for_tag(SourceData::LATEST_TESTED_CLDR_TAG, CldrLocaleSubset::Modern)?
        .with_icuexport_for_tag(SourceData::LATEST_TESTED_ICUEXPORT_TAG)?
        .with_collations(vec![String::from("search*")]);

    let blob_out = Out::Blob(Box::new(File::create(
        data_root().join("icudata.postcard"),
    )?));

    datagen(
        None,
        &all_keys_with_experimental(),
        &source_data,
        [blob_out].into(),
    )
    .map_err(Into::into)
}
