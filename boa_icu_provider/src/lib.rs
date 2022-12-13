#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,

    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
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

/// Gets the path to the directory where the generated data is stored.
#[must_use]
pub fn data_root() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("data")
}

use icu_provider_blob::BlobDataProvider;

/// Gets a [`BufferProvider`] that is stored as a Postcard blob.
///
/// This provider does NOT execute locale fallback. Use `LocaleFallbackProvider` from
/// the `icu_provider_adapters` crate for this functionality.
///
/// # Note
///
/// The returned provider internally uses [`Arc`][std::arc::Arc] to share the data between instances,
/// so it is preferrable to clone instead of calling `buffer()` multiple times.
#[must_use]
pub fn blob() -> BlobDataProvider {
    BlobDataProvider::try_new_from_static_blob(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/icudata.postcard"
    )))
    .expect("The statically compiled data file should be valid.")
}
