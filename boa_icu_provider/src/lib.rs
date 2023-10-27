//! Boa's **`boa_icu_provider`** exports the default data provider used by its `Intl` implementation.
//!
//! # Crate Overview
//! This crate exports the function `buffer`, which contains an extensive dataset of locale data to
//! enable `Intl` functionality in the engine. The set of locales included is precisely the ["modern"]
//! subset of locales in the [Unicode Common Locale Data Repository][cldr].
//!
//! If you need to support the full set of locales, you can check out the [ICU4X guide] about
//! generating custom data providers. Boa supports plugging both [`BufferProvider`]s or [`AnyProvider`]s
//! generated by the tool.
//!
//! ["modern"]: https://github.com/unicode-org/cldr-json/tree/main/cldr-json/cldr-localenames-modern/main
//! [cldr]: https://github.com/unicode-org/
//! [ICU4X guide]: https://github.com/unicode-org/icu4x/blob/main/docs/tutorials/data_management.md
//! [`BufferProvider`]: icu_provider::BufferProvider
//! [`AnyProvider`]: icu_provider::AnyProvider
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![warn(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,

    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    missing_docs,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,

    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,

    // clippy allowed by default
    clippy::dbg_macro,

    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(elided_lifetimes_in_paths)]
#![cfg_attr(not(feature = "bin"), no_std)]

/// Gets the path to the directory where the generated data is stored.
#[cfg(feature = "bin")]
#[must_use]
#[doc(hidden)]
pub fn data_root() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("data")
}

/// Gets the default data provider stored as a [`BufferProvider`].
///
/// [`BufferProvider`]: icu_provider::BufferProvider
#[must_use]
pub fn buffer() -> &'static impl icu_provider::BufferProvider {
    use icu_provider_adapters::fallback::LocaleFallbackProvider;
    use icu_provider_blob::BlobDataProvider;
    use once_cell::sync::Lazy;

    static PROVIDER: Lazy<LocaleFallbackProvider<BlobDataProvider>> = Lazy::new(|| {
        let blob = BlobDataProvider::try_new_from_static_blob(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/icudata.postcard"
        )))
        .expect("The statically compiled data file should be valid.");
        LocaleFallbackProvider::try_new_with_buffer_provider(blob)
            .expect("The statically compiled data file should be valid.")
    });

    &*PROVIDER
}
