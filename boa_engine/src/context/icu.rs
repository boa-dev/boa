use std::fmt::Debug;

use icu_casemap::CaseMapper;
use icu_locid_transform::{LocaleCanonicalizer, LocaleExpander, LocaleTransformError};
use icu_normalizer::{ComposingNormalizer, DecomposingNormalizer, NormalizerError};
use icu_provider::{
    AnyProvider, AsDeserializingBufferProvider, AsDowncastingAnyProvider, BufferProvider,
    DataError, DataProvider, DataRequest, DataResponse, KeyedDataMarker, MaybeSendSync,
};
use serde::Deserialize;
use thiserror::Error;
use yoke::{trait_hack::YokeTraitHack, Yokeable};
use zerofrom::ZeroFrom;

use crate::builtins::string::StringNormalizers;

/// ICU4X data provider used in boa.
///
/// Providers can be either [`BufferProvider`]s or [`AnyProvider`]s.
///
/// The [`icu_provider`] documentation has more information about data providers.
#[derive(Clone, Copy)]
pub enum BoaProvider<'a> {
    /// A [`BufferProvider`] data provider.
    Buffer(&'a dyn BufferProvider),
    /// An [`AnyProvider`] data provider.
    Any(&'a dyn AnyProvider),
}

impl Debug for BoaProvider<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buffer(_) => f.debug_tuple("Buffer").field(&"..").finish(),
            Self::Any(_) => f.debug_tuple("Any").field(&"..").finish(),
        }
    }
}

// This blanket implementation mirrors the `DataProvider` implementations of `BufferProvider` and
// `AnyProvider`, which allows us to use `unstable` constructors in a stable way.
impl<M> DataProvider<M> for BoaProvider<'_>
where
    M: KeyedDataMarker + 'static,
    for<'de> YokeTraitHack<<M::Yokeable as Yokeable<'de>>::Output>: Deserialize<'de>,
    for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
    M::Yokeable: ZeroFrom<'static, M::Yokeable> + MaybeSendSync,
{
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<M>, DataError> {
        match self {
            BoaProvider::Buffer(provider) => provider.as_deserializing().load(req),
            BoaProvider::Any(provider) => provider.as_downcasting().load(req),
        }
    }
}

/// Error thrown when the engine cannot initialize the ICU tools from a data provider.
#[derive(Debug, Error)]
pub enum IcuError {
    /// Failed to create the locale transform tools.
    #[error("could not construct the locale transform tools")]
    LocaleTransform(#[from] LocaleTransformError),
    /// Failed to create the string normalization tools.
    #[error("could not construct the string normalization tools")]
    Normalizer(#[from] NormalizerError),
    /// Failed to create the case mapping tools.
    #[error("could not construct the case mapping tools")]
    CaseMap(#[from] DataError),
}

/// Collection of tools initialized from a [`DataProvider`] that are used for the functionality of
/// `Intl`.
pub(crate) struct Icu<'provider> {
    provider: BoaProvider<'provider>,
    locale_canonicalizer: LocaleCanonicalizer,
    locale_expander: LocaleExpander,
    string_normalizers: StringNormalizers,
    case_mapper: CaseMapper,
}

impl Debug for Icu<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Icu")
            .field("provider", &self.provider)
            .field("locale_canonicalizer", &self.locale_canonicalizer)
            .field("locale_expander", &self.locale_expander)
            .field("string_normalizers", &self.string_normalizers)
            .field("string_normalizercase_mapper", &self.case_mapper)
            .finish()
    }
}

impl<'provider> Icu<'provider> {
    /// Creates a new [`Icu`] from a valid [`BoaProvider`]
    ///
    /// # Errors
    ///
    /// Returns an error if any of the tools required cannot be constructed.
    pub(crate) fn new(provider: BoaProvider<'provider>) -> Result<Icu<'provider>, IcuError> {
        Ok(Self {
            locale_canonicalizer: LocaleCanonicalizer::try_new_unstable(&provider)?,
            locale_expander: LocaleExpander::try_new_extended_unstable(&provider)?,
            string_normalizers: StringNormalizers {
                nfc: ComposingNormalizer::try_new_nfc_unstable(&provider)?,
                nfkc: ComposingNormalizer::try_new_nfkc_unstable(&provider)?,
                nfd: DecomposingNormalizer::try_new_nfd_unstable(&provider)?,
                nfkd: DecomposingNormalizer::try_new_nfkd_unstable(&provider)?,
            },
            case_mapper: CaseMapper::try_new_unstable(&provider)?,
            provider,
        })
    }

    /// Gets the [`LocaleCanonicalizer`] tool.
    pub(crate) const fn locale_canonicalizer(&self) -> &LocaleCanonicalizer {
        &self.locale_canonicalizer
    }

    /// Gets the [`LocaleExpander`] tool.
    pub(crate) const fn locale_expander(&self) -> &LocaleExpander {
        &self.locale_expander
    }

    /// Gets the [`StringNormalizers`] tools.
    pub(crate) const fn string_normalizers(&self) -> &StringNormalizers {
        &self.string_normalizers
    }

    /// Gets the [`CaseMapper`] tool.
    pub(crate) const fn case_mapper(&self) -> &CaseMapper {
        &self.case_mapper
    }

    /// Gets the inner icu data provider
    pub(crate) const fn provider(&self) -> BoaProvider<'provider> {
        self.provider
    }
}
