use std::fmt::Debug;

use icu_collator::{Collator, CollatorError, CollatorOptions};
use icu_list::{ListError, ListFormatter, ListLength};
use icu_locid_transform::{LocaleCanonicalizer, LocaleExpander, LocaleTransformError};
use icu_normalizer::{ComposingNormalizer, DecomposingNormalizer, NormalizerError};
use icu_provider::{
    AnyProvider, AsDeserializingBufferProvider, AsDowncastingAnyProvider, BufferProvider,
    DataError, DataLocale, DataProvider, DataRequest, DataResponse, KeyedDataMarker, MaybeSendSync,
};
use icu_segmenter::{GraphemeClusterSegmenter, SegmenterError, SentenceSegmenter, WordSegmenter};
use serde::Deserialize;
use thiserror::Error;
use yoke::{trait_hack::YokeTraitHack, Yokeable};
use zerofrom::ZeroFrom;

use crate::builtins::{
    intl::{
        list_format::ListFormatType,
        segmenter::{Granularity, NativeSegmenter},
    },
    string::StringNormalizers,
};

/// ICU4X data provider used in boa.
///
/// An icu provider can be constructed from either [`BufferProvider`]s or [`AnyProvider`]s.
///
/// The [`icu_provider`] documentation has more information about data providers.
#[derive(Debug)]
pub struct IcuProvider<'a> {
    provider: Inner<'a>,
    locale_canonicalizer: LocaleCanonicalizer,
    locale_expander: LocaleExpander,
    string_normalizers: StringNormalizers,
}

#[derive(Copy, Clone)]
enum Inner<'a> {
    /// A [`BufferProvider`] data provider.
    Buffer(&'a dyn BufferProvider),
    /// An [`AnyProvider`] data provider.
    Any(&'a dyn AnyProvider),
}

impl Debug for Inner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buffer(_) => f.debug_tuple("Buffer").field(&"_").finish(),
            Self::Any(_) => f.debug_tuple("Any").field(&"_").finish(),
        }
    }
}

impl<'a> IcuProvider<'a> {
    /// Creates a new [`IcuProvider`] from a [`BufferProvider`].
    ///
    /// # Errors
    ///
    /// This returns `Err` if the provided provider doesn't have the required locale information
    /// to construct the tools required by `Intl`. Note that this doesn't
    /// mean that the provider will successfully construct all `Intl` services; that check is made
    /// until the creation of an instance of a service.
    pub fn from_buffer_provider<P: BufferProvider>(
        provider: &'a P,
    ) -> Result<IcuProvider<'a>, IcuError> {
        let locale_canonicalizer = LocaleCanonicalizer::try_new_with_buffer_provider(provider)?;
        let locale_expander = LocaleExpander::try_new_with_buffer_provider(provider)?;
        let string_normalizers = StringNormalizers {
            nfc: ComposingNormalizer::try_new_nfc_with_buffer_provider(provider)?,
            nfkc: ComposingNormalizer::try_new_nfkc_with_buffer_provider(provider)?,
            nfd: DecomposingNormalizer::try_new_nfd_with_buffer_provider(provider)?,
            nfkd: DecomposingNormalizer::try_new_nfkd_with_buffer_provider(provider)?,
        };

        Ok(Self {
            provider: Inner::Buffer(provider),
            locale_canonicalizer,
            locale_expander,
            string_normalizers,
        })
    }

    /// Creates a new [`IcuProvider`] from an [`AnyProvider`].
    ///
    /// # Errors
    ///
    /// This returns `Err` if the provided provider doesn't have the required locale information
    /// to construct the tools required by `Intl`. Note that this doesn't
    /// mean that the provider will successfully construct all `Intl` services; that check is made
    /// until the creation of an instance of a service.
    pub fn from_any_provider<P: AnyProvider>(provider: &'a P) -> Result<IcuProvider<'a>, IcuError> {
        let locale_canonicalizer = LocaleCanonicalizer::try_new_with_any_provider(provider)?;
        let locale_expander = LocaleExpander::try_new_with_any_provider(provider)?;
        let string_normalizers = StringNormalizers {
            nfc: ComposingNormalizer::try_new_nfc_with_any_provider(provider)?,
            nfkc: ComposingNormalizer::try_new_nfkc_with_any_provider(provider)?,
            nfd: DecomposingNormalizer::try_new_nfd_with_any_provider(provider)?,
            nfkd: DecomposingNormalizer::try_new_nfkd_with_any_provider(provider)?,
        };
        Ok(Self {
            provider: Inner::Any(provider),
            locale_canonicalizer,
            locale_expander,
            string_normalizers,
        })
    }
}

impl IcuProvider<'_> {
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

    /// Creates a new [`ListFormatter`] from the provided [`DataProvider`] and options.
    pub(crate) fn try_new_list_formatter(
        &self,
        locale: &DataLocale,
        typ: ListFormatType,
        style: ListLength,
    ) -> Result<ListFormatter, ListError> {
        match self.provider {
            Inner::Buffer(buf) => match typ {
                ListFormatType::Conjunction => {
                    ListFormatter::try_new_and_with_length_with_buffer_provider(buf, locale, style)
                }
                ListFormatType::Disjunction => {
                    ListFormatter::try_new_or_with_length_with_buffer_provider(buf, locale, style)
                }
                ListFormatType::Unit => {
                    ListFormatter::try_new_unit_with_length_with_buffer_provider(buf, locale, style)
                }
            },
            Inner::Any(any) => match typ {
                ListFormatType::Conjunction => {
                    ListFormatter::try_new_and_with_length_with_any_provider(any, locale, style)
                }
                ListFormatType::Disjunction => {
                    ListFormatter::try_new_or_with_length_with_any_provider(any, locale, style)
                }
                ListFormatType::Unit => {
                    ListFormatter::try_new_unit_with_length_with_any_provider(any, locale, style)
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
        match self.provider {
            Inner::Buffer(buf) => Collator::try_new_with_buffer_provider(buf, locale, options),
            Inner::Any(any) => Collator::try_new_with_any_provider(any, locale, options),
        }
    }

    /// Creates a new [`NativeSegmenter`] from the provided [`DataProvider`] and options.
    pub(crate) fn try_new_segmenter(
        &self,
        granularity: Granularity,
    ) -> Result<NativeSegmenter, SegmenterError> {
        match granularity {
            Granularity::Grapheme => match self.provider {
                Inner::Buffer(buf) => GraphemeClusterSegmenter::try_new_with_buffer_provider(buf),
                Inner::Any(any) => GraphemeClusterSegmenter::try_new_with_any_provider(any),
            }
            .map(|seg| NativeSegmenter::Grapheme(Box::new(seg))),
            Granularity::Word => match self.provider {
                Inner::Buffer(buf) => WordSegmenter::try_new_auto_with_buffer_provider(buf),
                Inner::Any(any) => WordSegmenter::try_new_auto_with_any_provider(any),
            }
            .map(|seg| NativeSegmenter::Word(Box::new(seg))),
            Granularity::Sentence => match self.provider {
                Inner::Buffer(buf) => SentenceSegmenter::try_new_with_buffer_provider(buf),
                Inner::Any(any) => SentenceSegmenter::try_new_with_any_provider(any),
            }
            .map(|seg| NativeSegmenter::Sentence(Box::new(seg))),
        }
    }
}

impl<M> DataProvider<M> for IcuProvider<'_>
where
    M: KeyedDataMarker + 'static,
    for<'de> YokeTraitHack<<M::Yokeable as Yokeable<'de>>::Output>: Deserialize<'de>,
    for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
    M::Yokeable: ZeroFrom<'static, M::Yokeable> + MaybeSendSync,
{
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<M>, DataError> {
        match self.provider {
            Inner::Buffer(provider) => provider.as_deserializing().load(req),
            Inner::Any(provider) => provider.as_downcasting().load(req),
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
}
