use std::{cell::OnceCell, fmt::Debug};

use boa_profiler::Profiler;
use icu_casemap::CaseMapper;
use icu_locale::{LocaleCanonicalizer, LocaleExpander};
use icu_normalizer::{ComposingNormalizer, DecomposingNormalizer};
use icu_provider::{any::MaybeSendSync, prelude::*};
use serde::Deserialize;
use thiserror::Error;
use yoke::Yokeable;
use zerofrom::ZeroFrom;

use crate::{builtins::string::StringNormalizers, JsError, JsNativeError};

/// A [`DataProvider`] that can be either a [`BufferProvider`] or an [`AnyProvider`].
pub(crate) enum ErasedProvider {
    Any(Box<dyn DynamicDryDataProvider<AnyMarker>>),
    Buffer(Box<dyn DynamicDryDataProvider<BufferMarker>>),
}

/// Error thrown when the engine cannot initialize the ICU4X utilities from a data provider.
#[derive(Debug, Error)]
pub enum IcuError {
    /// Failed to create the internationalization utilities.
    #[error("could not construct the internationalization utilities")]
    DataError(#[from] DataError),
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
    M: DataMarker + 'static,
    for<'de> YokeTraitHack<<M::DataStruct as Yokeable<'de>>::Output>: Deserialize<'de> + Clone,
    M::DataStruct: ZeroFrom<'static, M::DataStruct> + MaybeSendSync,
{
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<M>, DataError> {
        match &self.inner_provider {
            ErasedProvider::Any(any) => any.as_any_provider().as_downcasting().load_data(M::INFO, req),
            ErasedProvider::Buffer(buf) => buf.as_deserializing().load_data(M::INFO, req),
        }
    }
}

impl<M> DryDataProvider<M> for IntlProvider
where
    M: DataMarker + 'static,
    for<'de> YokeTraitHack<<M::DataStruct as Yokeable<'de>>::Output>: Deserialize<'de> + Clone,
    M::DataStruct: ZeroFrom<'static, M::DataStruct> + MaybeSendSync,
{
    fn dry_load(&self, req: DataRequest<'_>) -> Result<DataResponseMetadata, DataError> {
        match &self.inner_provider {
            ErasedProvider::Any(any) => any.dry_load_data(M::INFO, req),
            ErasedProvider::Buffer(buf) => buf.dry_load_data(M::INFO, req),
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
    /// Creates a new [`IntlProvider`] from a [`DynamicDryDataProvider<BufferMarker>`].
    ///
    /// # Errors
    ///
    /// Returns an error if any of the tools required cannot be constructed.
    pub(crate) fn try_new_buffer(
        provider: (impl DynamicDryDataProvider<BufferMarker> + 'static),
    ) -> IntlProvider {
        let _timer = Profiler::global().start_event("ICU::try_new_with_buffer_provider", "ICU");
        Self {
            locale_canonicalizer: OnceCell::new(),
            locale_expander: OnceCell::new(),
            string_normalizers: OnceCell::new(),
            case_mapper: OnceCell::new(),
            inner_provider: ErasedProvider::Buffer(Box::new(provider)),
        }
    }

    /// Creates a new [`IntlProvider`] from a [`DynamicDryDataProvider<AnyMarker>`].
    ///
    /// # Errors
    ///
    /// Returns an error if any of the tools required cannot be constructed.
    pub(crate) fn try_new_any(
        provider: (impl DynamicDryDataProvider<AnyMarker> + 'static),
    ) -> IntlProvider {
        let _timer = Profiler::global().start_event("ICU::try_new_with_any_provider", "ICU");
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
        let expander = self.locale_expander()?.clone();
        let lc = match &self.inner_provider {
            ErasedProvider::Any(a) => LocaleCanonicalizer::try_new_with_expander_unstable(&a.as_any_provider().as_downcasting(), expander)?,
            ErasedProvider::Buffer(b) => LocaleCanonicalizer::try_new_with_expander_with_buffer_provider(b, expander)?,
        };
        Ok(self.locale_canonicalizer.get_or_init(|| lc))
    }

    /// Gets the [`LocaleExpander`] tool.
    pub(crate) fn locale_expander(&self) -> Result<&LocaleExpander, IcuError> {
        if let Some(le) = self.locale_expander.get() {
            return Ok(le);
        }
        let le = match &self.inner_provider {
            ErasedProvider::Any(a) => LocaleExpander::try_new_extended_unstable(&a.as_any_provider().as_downcasting())?,
            ErasedProvider::Buffer(b) => LocaleExpander::try_new_extended_with_buffer_provider(b)?,
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
                nfc: ComposingNormalizer::try_new_nfc_unstable(&a.as_any_provider().as_downcasting())?,
                nfkc: ComposingNormalizer::try_new_nfkc_unstable(&a.as_any_provider().as_downcasting())?,
                nfd: DecomposingNormalizer::try_new_nfd_unstable(&a.as_any_provider().as_downcasting())?,
                nfkd: DecomposingNormalizer::try_new_nfkd_unstable(&a.as_any_provider().as_downcasting())?,
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
            ErasedProvider::Any(a) => CaseMapper::try_new_unstable(&a.as_any_provider().as_downcasting())?,
            ErasedProvider::Buffer(b) => CaseMapper::try_new_with_buffer_provider(b)?,
        };
        Ok(self.case_mapper.get_or_init(|| cm))
    }

    /// Gets the inner erased provider.
    pub(crate) fn erased_provider(&self) -> &ErasedProvider {
        &self.inner_provider
    }
}
