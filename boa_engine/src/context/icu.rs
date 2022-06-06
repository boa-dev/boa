use std::fmt::Debug;

use icu_locid_transform::{LocaleCanonicalizer, LocaleTransformError};
use icu_provider::{
    yoke::{trait_hack::YokeTraitHack, Yokeable},
    zerofrom::ZeroFrom,
    AnyProvider, AsDeserializingBufferProvider, AsDowncastingAnyProvider, BufferProvider,
    DataError, DataProvider, DataRequest, DataResponse, KeyedDataMarker, MaybeSendSync,
};
use serde::Deserialize;

pub enum BoaProvider {
    Buffer(Box<dyn BufferProvider>),
    Any(Box<dyn AnyProvider>),
}

impl Debug for BoaProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buffer(_) => f.debug_tuple("Buffer").field(&"_").finish(),
            Self::Any(_) => f.debug_tuple("Any").field(&"_").finish(),
        }
    }
}

impl<M> DataProvider<M> for BoaProvider
where
    M: KeyedDataMarker + 'static,
    for<'de> YokeTraitHack<<M::Yokeable as Yokeable<'de>>::Output>: Deserialize<'de>,
    for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
    M::Yokeable: ZeroFrom<'static, M::Yokeable>,
    M::Yokeable: MaybeSendSync,
{
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<M>, DataError> {
        match self {
            BoaProvider::Buffer(provider) => provider.as_deserializing().load(req),
            BoaProvider::Any(provider) => provider.as_downcasting().load(req),
        }
    }
}

impl BoaProvider {
    /// Creates a new [`LocaleCanonicalizer`] from the provided [`DataProvider`].
    pub(crate) fn try_new_locale_canonicalizer(
        &self,
    ) -> Result<LocaleCanonicalizer, LocaleTransformError> {
        match self {
            BoaProvider::Buffer(buffer) => {
                LocaleCanonicalizer::try_new_with_buffer_provider(buffer)
            }
            BoaProvider::Any(any) => LocaleCanonicalizer::try_new_with_any_provider(any),
        }
    }
}

/// Collection of tools initialized from a [`DataProvider`] that are used
/// for the functionality of `Intl`.
pub(crate) struct Icu<P> {
    provider: P,
    locale_canonicalizer: LocaleCanonicalizer,
}

impl<P: Debug> Debug for Icu<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Icu")
            .field("provider", &self.provider)
            .field("locale_canonicalizer", &"LocaleCanonicalizer")
            .finish()
    }
}

impl<P> Icu<P> {
    /// Gets the [`LocaleCanonicalizer`] tool.
    pub(crate) fn locale_canonicalizer(&self) -> &LocaleCanonicalizer {
        &self.locale_canonicalizer
    }

    /// Gets the inner icu data provider
    #[allow(unused)]
    pub(crate) fn provider(&self) -> &P {
        &self.provider
    }
}

impl Icu<BoaProvider> {
    /// Creates a new [`Icu`] from a valid [`BoaProvider`]
    ///
    /// # Errors
    ///
    /// This method will return an error if any of the tools
    /// required cannot be constructed.
    pub(crate) fn new(provider: BoaProvider) -> Result<Self, LocaleTransformError> {
        let locale_canonicalizer = provider.try_new_locale_canonicalizer()?;
        Ok(Self {
            provider,
            locale_canonicalizer,
        })
    }
}
