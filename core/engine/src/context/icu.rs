use std::fmt::Debug;

use icu_casemap::CaseMapper;
use icu_locid_transform::{LocaleCanonicalizer, LocaleExpander, LocaleTransformError};
use icu_normalizer::{ComposingNormalizer, DecomposingNormalizer, NormalizerError};
use icu_provider::{
    AnyProvider, AnyResponse, BufferMarker, BufferProvider, DataError, DataKey, DataProvider,
    DataRequest, DataResponse, KeyedDataMarker, MaybeSendSync,
};
use serde::Deserialize;
use thiserror::Error;
use yoke::{trait_hack::YokeTraitHack, Yokeable};
use zerofrom::ZeroFrom;

use crate::builtins::string::StringNormalizers;

/// A response data from a [`DataProvider`], erased to unify both types
/// of providers into a single trait.
enum ErasedResponse {
    Any(AnyResponse),
    Buffer(DataResponse<BufferMarker>),
}

/// A [`DataProvider`] that can be either a [`BufferProvider`] or an [`AnyProvider`] ...
/// or both! (please don't).
trait ErasedProvider {
    /// Loads data according to the key and request.
    fn load_data(&self, key: DataKey, req: DataRequest<'_>) -> Result<ErasedResponse, DataError>;
}

/// Wraps a [`BufferProvider`] in order to be able to implement [`ErasedProvider`] for it.
struct BufferProviderWrapper<T>(T);

impl<T: BufferProvider> ErasedProvider for BufferProviderWrapper<T> {
    fn load_data(&self, key: DataKey, req: DataRequest<'_>) -> Result<ErasedResponse, DataError> {
        self.0.load_buffer(key, req).map(ErasedResponse::Buffer)
    }
}

/// Wraps an [`AnyProvider`] in order to be able to implement [`ErasedProvider`] for it.
struct AnyProviderWrapper<T>(T);
impl<T: AnyProvider> ErasedProvider for AnyProviderWrapper<T> {
    fn load_data(&self, key: DataKey, req: DataRequest<'_>) -> Result<ErasedResponse, DataError> {
        self.0.load_any(key, req).map(ErasedResponse::Any)
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

/// Custom [`DataProvider`] for `Intl` that caches some utilities.
pub(crate) struct IntlProvider {
    inner_provider: Box<dyn ErasedProvider>,
    locale_canonicalizer: LocaleCanonicalizer,
    locale_expander: LocaleExpander,
    string_normalizers: StringNormalizers,
    case_mapper: CaseMapper,
}

impl<M> DataProvider<M> for IntlProvider
where
    M: KeyedDataMarker + 'static,
    for<'de> YokeTraitHack<<M::Yokeable as Yokeable<'de>>::Output>: Deserialize<'de> + Clone,
    M::Yokeable: ZeroFrom<'static, M::Yokeable> + MaybeSendSync,
{
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<M>, DataError> {
        match self.inner_provider.load_data(M::KEY, req)? {
            ErasedResponse::Any(response) => {
                response.downcast().map_err(|e| e.with_req(M::KEY, req))
            }
            ErasedResponse::Buffer(response) => {
                let buffer_format = response.metadata.buffer_format.ok_or_else(|| {
                    DataError::custom("BufferProvider didn't set BufferFormat")
                        .with_req(M::KEY, req)
                })?;
                Ok(DataResponse {
                    metadata: response.metadata,
                    payload: response
                        .payload
                        .map(|p| p.into_deserialized(buffer_format))
                        .transpose()
                        .map_err(|e| e.with_req(M::KEY, req))?,
                })
            }
        }
    }
}

impl Debug for IntlProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Icu")
            .field("locale_canonicalizer", &self.locale_canonicalizer)
            .field("locale_expander", &self.locale_expander)
            .field("string_normalizers", &self.string_normalizers)
            .field("string_normalizercase_mapper", &self.case_mapper)
            .finish_non_exhaustive()
    }
}

impl IntlProvider {
    /// Creates a new [`IntlProvider`] from a [`BufferProvider`].
    ///
    /// # Errors
    ///
    /// Returns an error if any of the tools required cannot be constructed.
    pub(crate) fn try_new_with_buffer_provider(
        provider: (impl BufferProvider + 'static),
    ) -> Result<IntlProvider, IcuError> {
        Ok(Self {
            locale_canonicalizer: LocaleCanonicalizer::try_new_with_buffer_provider(&provider)?,
            locale_expander: LocaleExpander::try_new_with_buffer_provider(&provider)?,
            string_normalizers: StringNormalizers {
                nfc: ComposingNormalizer::try_new_nfc_with_buffer_provider(&provider)?,
                nfkc: ComposingNormalizer::try_new_nfkc_with_buffer_provider(&provider)?,
                nfd: DecomposingNormalizer::try_new_nfd_with_buffer_provider(&provider)?,
                nfkd: DecomposingNormalizer::try_new_nfkd_with_buffer_provider(&provider)?,
            },
            case_mapper: CaseMapper::try_new_with_buffer_provider(&provider)?,
            inner_provider: Box::new(BufferProviderWrapper(provider)),
        })
    }

    /// Creates a new [`IntlProvider`] from an [`AnyProvider`].
    ///
    /// # Errors
    ///
    /// Returns an error if any of the tools required cannot be constructed.
    pub(crate) fn try_new_with_any_provider(
        provider: (impl AnyProvider + 'static),
    ) -> Result<IntlProvider, IcuError> {
        Ok(Self {
            locale_canonicalizer: LocaleCanonicalizer::try_new_with_any_provider(&provider)?,
            locale_expander: LocaleExpander::try_new_extended_with_any_provider(&provider)?,
            string_normalizers: StringNormalizers {
                nfc: ComposingNormalizer::try_new_nfc_with_any_provider(&provider)?,
                nfkc: ComposingNormalizer::try_new_nfkc_with_any_provider(&provider)?,
                nfd: DecomposingNormalizer::try_new_nfd_with_any_provider(&provider)?,
                nfkd: DecomposingNormalizer::try_new_nfkd_with_any_provider(&provider)?,
            },
            case_mapper: CaseMapper::try_new_with_any_provider(&provider)?,
            inner_provider: Box::new(AnyProviderWrapper(provider)),
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
}
