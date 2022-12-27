use std::fmt::Debug;

use icu_collator::{Collator, CollatorError, CollatorOptions};
use icu_list::{ListError, ListFormatter, ListLength};
use icu_locid_transform::{LocaleCanonicalizer, LocaleExpander, LocaleTransformError};
use icu_provider::{
    yoke::{trait_hack::YokeTraitHack, Yokeable},
    zerofrom::ZeroFrom,
    AnyProvider, AsDeserializingBufferProvider, AsDowncastingAnyProvider, BufferProvider,
    DataError, DataLocale, DataProvider, DataRequest, DataResponse, KeyedDataMarker, MaybeSendSync,
};
use serde::Deserialize;

use crate::builtins::intl::list_format::ListFormatType;

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

impl<M> DataProvider<M> for BoaProvider<'_>
where
    M: KeyedDataMarker + 'static,
    for<'de> YokeTraitHack<<M::Yokeable as Yokeable<'de>>::Output>: Deserialize<'de>,
    for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
    M::Yokeable: ZeroFrom<'static, M::Yokeable> + MaybeSendSync,
{
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<M>, DataError> {
        match *self {
            BoaProvider::Buffer(provider) => provider.as_deserializing().load(req),
            BoaProvider::Any(provider) => provider.as_downcasting().load(req),
        }
    }
}

impl BoaProvider<'_> {
    /// Creates a new [`LocaleCanonicalizer`] from the provided [`DataProvider`].
    pub(crate) fn try_new_locale_canonicalizer(
        &self,
    ) -> Result<LocaleCanonicalizer, LocaleTransformError> {
        match self {
            BoaProvider::Buffer(buffer) => {
                LocaleCanonicalizer::try_new_with_buffer_provider(&**buffer)
            }
            BoaProvider::Any(any) => LocaleCanonicalizer::try_new_with_any_provider(&**any),
        }
    }

    /// Creates a new [`LocaleExpander`] from the provided [`DataProvider`].
    pub(crate) fn try_new_locale_expander(&self) -> Result<LocaleExpander, LocaleTransformError> {
        match self {
            BoaProvider::Buffer(buffer) => LocaleExpander::try_new_with_buffer_provider(&**buffer),
            BoaProvider::Any(any) => LocaleExpander::try_new_with_any_provider(&**any),
        }
    }

    /// Creates a new [`ListFormatter`] from the provided [`DataProvider`] and options.
    pub(crate) fn try_new_list_formatter(
        &self,
        locale: &DataLocale,
        typ: ListFormatType,
        style: ListLength,
    ) -> Result<ListFormatter, ListError> {
        match self {
            BoaProvider::Buffer(buf) => match typ {
                ListFormatType::Conjunction => {
                    ListFormatter::try_new_and_with_length_with_buffer_provider(
                        &**buf, locale, style,
                    )
                }
                ListFormatType::Disjunction => {
                    ListFormatter::try_new_or_with_length_with_buffer_provider(
                        &**buf, locale, style,
                    )
                }
                ListFormatType::Unit => {
                    ListFormatter::try_new_unit_with_length_with_buffer_provider(
                        &**buf, locale, style,
                    )
                }
            },
            BoaProvider::Any(any) => match typ {
                ListFormatType::Conjunction => {
                    ListFormatter::try_new_and_with_length_with_any_provider(&**any, locale, style)
                }
                ListFormatType::Disjunction => {
                    ListFormatter::try_new_or_with_length_with_any_provider(&**any, locale, style)
                }
                ListFormatType::Unit => {
                    ListFormatter::try_new_unit_with_length_with_any_provider(&**any, locale, style)
                }
            },
        }
    }

    /// Creates a new [`Collator`] from the provided [`DataProvider`] and options.
    pub(crate) fn try_new_collator(
        &self,
        locale: &DataLocale,
        options: CollatorOptions,
    ) -> Result<Collator, CollatorError> {
        match self {
            BoaProvider::Buffer(buf) => {
                Collator::try_new_with_buffer_provider(&**buf, locale, options)
            }
            BoaProvider::Any(any) => Collator::try_new_with_any_provider(&**any, locale, options),
        }
    }
}

/// Collection of tools initialized from a [`DataProvider`] that are used
/// for the functionality of `Intl`.
pub(crate) struct Icu<'provider> {
    provider: BoaProvider<'provider>,
    locale_canonicalizer: LocaleCanonicalizer,
    locale_expander: LocaleExpander,
}

impl Debug for Icu<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Icu")
            .field("provider", &self.provider)
            .field("locale_canonicalizer", &"LocaleCanonicalizer")
            .field("locale_expander", &"LocaleExpander")
            .finish()
    }
}

impl<'provider> Icu<'provider> {
    /// Creates a new [`Icu`] from a valid [`BoaProvider`]
    ///
    /// # Errors
    ///
    /// This method will return an error if any of the tools
    /// required cannot be constructed.
    pub(crate) fn new(
        provider: BoaProvider<'provider>,
    ) -> Result<Icu<'provider>, LocaleTransformError> {
        Ok(Self {
            locale_canonicalizer: provider.try_new_locale_canonicalizer()?,
            locale_expander: provider.try_new_locale_expander()?,
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

    /// Gets the inner icu data provider
    pub(crate) const fn provider(&self) -> BoaProvider<'provider> {
        self.provider
    }
}
