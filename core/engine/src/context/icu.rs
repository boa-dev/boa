use std::{cell::OnceCell, fmt::Debug};

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

use crate::{builtins::string::StringNormalizers, JsError, JsNativeError};

/// A [`DataProvider`] that can be either a [`BufferProvider`] or an [`AnyProvider`].
pub(crate) enum ErasedProvider {
    Any(Box<dyn AnyProvider>),
    Buffer(Box<dyn BufferProvider>),
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

impl From<IcuError> for JsNativeError {
    fn from(value: IcuError) -> Self {
        JsNativeError::typ().with_message(value.to_string())
    }
}

impl From<IcuError> for JsError {
    fn from(value: IcuError) -> Self {
        JsNativeError::from(value).into()
    }
}

/// Custom [`DataProvider`] for `Intl` that caches some utilities.
pub(crate) struct IntlProvider {
    inner_provider: ErasedProvider,
    locale_canonicalizer: OnceCell<LocaleCanonicalizer>,
    locale_expander: OnceCell<LocaleExpander>,
    string_normalizers: OnceCell<StringNormalizers>,
    case_mapper: OnceCell<CaseMapper>,
}

impl<M> DataProvider<M> for IntlProvider
where
    M: KeyedDataMarker + 'static,
    for<'de> YokeTraitHack<<M::Yokeable as Yokeable<'de>>::Output>: Deserialize<'de> + Clone,
    M::Yokeable: ZeroFrom<'static, M::Yokeable> + MaybeSendSync,
{
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<M>, DataError> {
        match &self.inner_provider {
            ErasedProvider::Any(any) => any.as_downcasting().load(req),
            ErasedProvider::Buffer(buffer) => buffer.as_deserializing().load(req),
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
    ) -> IntlProvider {
        Self {
            locale_canonicalizer: OnceCell::new(),
            locale_expander: OnceCell::new(),
            string_normalizers: OnceCell::new(),
            case_mapper: OnceCell::new(),
            inner_provider: ErasedProvider::Buffer(Box::new(provider)),
        }
    }

    /// Creates a new [`IntlProvider`] from an [`AnyProvider`].
    ///
    /// # Errors
    ///
    /// Returns an error if any of the tools required cannot be constructed.
    pub(crate) fn try_new_with_any_provider(
        provider: (impl AnyProvider + 'static),
    ) -> IntlProvider {
        Self {
            locale_canonicalizer: OnceCell::new(),
            locale_expander: OnceCell::new(),
            string_normalizers: OnceCell::new(),
            case_mapper: OnceCell::new(),
            inner_provider: ErasedProvider::Any(Box::new(provider)),
        }
    }

    /// Gets the [`LocaleCanonicalizer`] tool.
    pub(crate) fn locale_canonicalizer(&self) -> Result<&LocaleCanonicalizer, IcuError> {
        if let Some(lc) = self.locale_canonicalizer.get() {
            return Ok(lc);
        }
        let lc = match &self.inner_provider {
            ErasedProvider::Any(a) => LocaleCanonicalizer::try_new_with_any_provider(a)?,
            ErasedProvider::Buffer(b) => LocaleCanonicalizer::try_new_with_buffer_provider(b)?,
        };
        Ok(self.locale_canonicalizer.get_or_init(|| lc))
    }

    /// Gets the [`LocaleExpander`] tool.
    pub(crate) fn locale_expander(&self) -> Result<&LocaleExpander, IcuError> {
        if let Some(le) = self.locale_expander.get() {
            return Ok(le);
        }
        let le = match &self.inner_provider {
            ErasedProvider::Any(a) => LocaleExpander::try_new_with_any_provider(a)?,
            ErasedProvider::Buffer(b) => LocaleExpander::try_new_with_buffer_provider(b)?,
        };
        Ok(self.locale_expander.get_or_init(|| le))
    }

    /// Gets the [`StringNormalizers`] tools.
    pub(crate) fn string_normalizers(&self) -> Result<&StringNormalizers, IcuError> {
        if let Some(sn) = self.string_normalizers.get() {
            return Ok(sn);
        }
        let sn = match &self.inner_provider {
            ErasedProvider::Any(a) => StringNormalizers {
                nfc: ComposingNormalizer::try_new_nfc_with_any_provider(a)?,
                nfkc: ComposingNormalizer::try_new_nfkc_with_any_provider(a)?,
                nfd: DecomposingNormalizer::try_new_nfd_with_any_provider(a)?,
                nfkd: DecomposingNormalizer::try_new_nfkd_with_any_provider(a)?,
            },
            ErasedProvider::Buffer(b) => StringNormalizers {
                nfc: ComposingNormalizer::try_new_nfc_with_buffer_provider(b)?,
                nfkc: ComposingNormalizer::try_new_nfkc_with_buffer_provider(b)?,
                nfd: DecomposingNormalizer::try_new_nfd_with_buffer_provider(b)?,
                nfkd: DecomposingNormalizer::try_new_nfkd_with_buffer_provider(b)?,
            },
        };
        Ok(self.string_normalizers.get_or_init(|| sn))
    }

    /// Gets the [`CaseMapper`] tool.
    pub(crate) fn case_mapper(&self) -> Result<&CaseMapper, IcuError> {
        if let Some(cm) = self.case_mapper.get() {
            return Ok(cm);
        }
        let cm = match &self.inner_provider {
            ErasedProvider::Any(a) => CaseMapper::try_new_with_any_provider(a)?,
            ErasedProvider::Buffer(b) => CaseMapper::try_new_with_buffer_provider(b)?,
        };
        Ok(self.case_mapper.get_or_init(|| cm))
    }

    /// Gets the inner erased provider.
    pub(crate) fn erased_provider(&self) -> &ErasedProvider {
        &self.inner_provider
    }
}
