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
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use core::fmt::Debug;

use icu_provider::{BufferMarker, BufferProvider, DataError, DataErrorKind, DataKey, DataResponse};
use icu_provider_adapters::{fallback::LocaleFallbackProvider, fork::MultiForkByKeyProvider};
use icu_provider_blob::BlobDataProvider;
use once_cell::sync::{Lazy, OnceCell};

/// A buffer provider that is lazily deserialized at the first data request.
///
/// The provider must specify the list of keys it supports, to avoid deserializing the
/// buffer for unknown keys.
struct LazyBufferProvider {
    provider: OnceCell<BlobDataProvider>,
    bytes: &'static [u8],
    valid_keys: &'static [DataKey],
}

impl Debug for LazyBufferProvider {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LazyBufferProvider")
            .field("provider", &self.provider)
            .field("bytes", &"[...]")
            .field("valid_keys", &self.valid_keys)
            .finish()
    }
}

impl BufferProvider for LazyBufferProvider {
    fn load_buffer(
        &self,
        key: DataKey,
        req: icu_provider::DataRequest<'_>,
    ) -> Result<DataResponse<BufferMarker>, DataError> {
        if !self.valid_keys.contains(&key) {
            return Err(DataErrorKind::MissingDataKey.with_key(key));
        }

        let Ok(provider) = self
            .provider
            .get_or_try_init(|| BlobDataProvider::try_new_from_static_blob(self.bytes))
        else {
            return Err(DataErrorKind::Custom.with_str_context("invalid blob data provider"));
        };

        provider.load_buffer(key, req)
    }
}

/// A macro that creates a [`LazyBufferProvider`] from an icu4x crate.
macro_rules! provider_from_icu_crate {
    ($service:path) => {
        paste::paste! {
            LazyBufferProvider {
                provider: OnceCell::new(),
                bytes: include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/data/",
                    stringify!($service),
                    ".postcard",
                )),
                valid_keys: $service::provider::KEYS,
            }
        }
    };
}

/// Boa's default buffer provider.
static PROVIDER: Lazy<LocaleFallbackProvider<MultiForkByKeyProvider<LazyBufferProvider>>> =
    Lazy::new(|| {
        let provider = MultiForkByKeyProvider::new(alloc::vec![
            provider_from_icu_crate!(icu_casemap),
            provider_from_icu_crate!(icu_collator),
            provider_from_icu_crate!(icu_datetime),
            provider_from_icu_crate!(icu_decimal),
            provider_from_icu_crate!(icu_list),
            provider_from_icu_crate!(icu_locid_transform),
            provider_from_icu_crate!(icu_normalizer),
            provider_from_icu_crate!(icu_plurals),
            provider_from_icu_crate!(icu_segmenter),
        ]);
        LocaleFallbackProvider::try_new_with_buffer_provider(provider)
            .expect("The statically compiled data file should be valid.")
    });

/// Gets the default data provider stored as a [`BufferProvider`].
///
/// [`BufferProvider`]: icu_provider::BufferProvider
#[must_use]
pub fn buffer() -> &'static impl BufferProvider {
    &*PROVIDER
}
