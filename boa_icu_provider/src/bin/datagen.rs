#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]

use std::{error::Error, fs::File};

use boa_icu_provider::data_root;
use icu_datagen::{
    all_keys_with_experimental, datagen, BakedOptions, CldrLocaleSubset, Out, SourceData,
};
use icu_normalizer::provider::{
    CanonicalCompositionsV1Marker, CanonicalDecompositionDataV1Marker,
    CanonicalDecompositionTablesV1Marker, CompatibilityDecompositionSupplementV1Marker,
    CompatibilityDecompositionTablesV1Marker,
};
use icu_properties::provider::{IdContinueV1Marker, IdStartV1Marker};
use icu_provider::KeyedDataMarker;
use icu_provider_adapters::fallback::provider::{
    CollationFallbackSupplementV1Marker, LocaleFallbackLikelySubtagsV1Marker,
    LocaleFallbackParentsV1Marker,
};

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new()
        .env()
        .with_level(log::LevelFilter::Info)
        .init()?;

    let source_data = SourceData::default()
        .with_cldr_for_tag(SourceData::LATEST_TESTED_CLDR_TAG, CldrLocaleSubset::Modern)?
        .with_icuexport_for_tag(SourceData::LATEST_TESTED_ICUEXPORT_TAG)?
        .with_collations(vec![String::from("search*")]);

    let full_blob_out = Out::Blob(Box::new(File::create(
        data_root().join("icudata.postcard"),
    )?));

    let normalization_out = Out::Baked {
        mod_directory: data_root().join("min"),
        options: {
            let mut opt = BakedOptions::default();
            opt.use_separate_crates = true;
            opt.overwrite = true;
            opt.pretty = true;
            opt
        },
    };

    datagen(
        None,
        &[
            CanonicalDecompositionDataV1Marker::KEY,
            CanonicalDecompositionTablesV1Marker::KEY,
            CanonicalCompositionsV1Marker::KEY,
            CompatibilityDecompositionSupplementV1Marker::KEY,
            CompatibilityDecompositionTablesV1Marker::KEY,
            LocaleFallbackLikelySubtagsV1Marker::KEY,
            LocaleFallbackParentsV1Marker::KEY,
            CollationFallbackSupplementV1Marker::KEY,
            IdContinueV1Marker::KEY,
            IdStartV1Marker::KEY,
        ],
        &source_data,
        [normalization_out].into(),
    )?;

    datagen(
        None,
        &all_keys_with_experimental(),
        &source_data,
        [full_blob_out].into(),
    )?;

    Ok(())
}
