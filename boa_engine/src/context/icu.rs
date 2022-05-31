use icu_datagen::transform::cldr::{
    AliasesProvider, CommonDateProvider, LikelySubtagsProvider, PluralsProvider, WeekDataProvider,
};
use icu_datagen::SourceData;
use icu_datetime::provider::{
    calendar::{DatePatternsV1Marker, DateSkeletonPatternsV1Marker, DateSymbolsV1Marker},
    week_data::WeekDataV1Marker,
};
use icu_locale_canonicalizer::{
    provider::{AliasesV1Marker, LikelySubtagsV1Marker},
    LocaleCanonicalizer,
};
use icu_plurals::provider::OrdinalV1Marker;
use icu_provider::inv::InvariantDataProvider;
use icu_provider::prelude::*;

/// Trait encompassing all the required implementations that define
/// a valid icu data provider.
pub trait BoaProvider:
    ResourceProvider<AliasesV1Marker>
    + ResourceProvider<LikelySubtagsV1Marker>
    + ResourceProvider<DateSymbolsV1Marker>
    + ResourceProvider<DatePatternsV1Marker>
    + ResourceProvider<DateSkeletonPatternsV1Marker>
    + ResourceProvider<OrdinalV1Marker>
    + ResourceProvider<WeekDataV1Marker>
{
}

impl<T> BoaProvider for T where
    T: ResourceProvider<AliasesV1Marker>
        + ResourceProvider<LikelySubtagsV1Marker>
        + ResourceProvider<DateSymbolsV1Marker>
        + ResourceProvider<DatePatternsV1Marker>
        + ResourceProvider<DateSkeletonPatternsV1Marker>
        + ResourceProvider<OrdinalV1Marker>
        + ResourceProvider<WeekDataV1Marker>
        + ?Sized
{
}

impl ResourceProvider<AliasesV1Marker> for &dyn BoaProvider {
    fn load_resource(&self, req: &DataRequest) -> Result<DataResponse<AliasesV1Marker>, DataError> {
        let provider = AliasesProvider::from(&SourceData::default());
        let response = provider.load_resource(req);
        if response.is_ok() {
            response
        } else {
            InvariantDataProvider.load_resource(req)
        }
    }
}

impl ResourceProvider<LikelySubtagsV1Marker> for &dyn BoaProvider {
    fn load_resource(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<LikelySubtagsV1Marker>, DataError> {
        let provider = LikelySubtagsProvider::from(&SourceData::default());
        provider.load_resource(req)
    }
}

impl ResourceProvider<DateSymbolsV1Marker> for &dyn BoaProvider {
    fn load_resource(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<DateSymbolsV1Marker>, DataError> {
        let provider = CommonDateProvider::from(&SourceData::default());
        let response = provider.load_resource(req);
        if response.is_ok() {
            response
        } else {
            InvariantDataProvider.load_resource(req)
        }
    }
}

impl ResourceProvider<DatePatternsV1Marker> for &dyn BoaProvider {
    fn load_resource(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<DatePatternsV1Marker>, DataError> {
        let provider = CommonDateProvider::from(&SourceData::default());
        let response = provider.load_resource(req);
        if response.is_ok() {
            response
        } else {
            InvariantDataProvider.load_resource(req)
        }
    }
}

impl ResourceProvider<DateSkeletonPatternsV1Marker> for &dyn BoaProvider {
    fn load_resource(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<DateSkeletonPatternsV1Marker>, DataError> {
        let provider = CommonDateProvider::from(&SourceData::default());
        let response = provider.load_resource(req);
        if response.is_ok() {
            response
        } else {
            InvariantDataProvider.load_resource(req)
        }
    }
}

impl ResourceProvider<OrdinalV1Marker> for &dyn BoaProvider {
    fn load_resource(&self, req: &DataRequest) -> Result<DataResponse<OrdinalV1Marker>, DataError> {
        let provider = PluralsProvider::from(&SourceData::default());
        let response = provider.load_resource(req);
        if response.is_ok() {
            response
        } else {
            InvariantDataProvider.load_resource(req)
        }
    }
}

impl ResourceProvider<WeekDataV1Marker> for &dyn BoaProvider {
    fn load_resource(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<WeekDataV1Marker>, DataError> {
        let provider = WeekDataProvider::from(&SourceData::default());
        let response = provider.load_resource(req);
        if response.is_ok() {
            response
        } else {
            InvariantDataProvider.load_resource(req)
        }
    }
}

/// Collection of tools initialized from a [`BoaProvider`] that are used
/// for the functionality of `Intl`.
#[allow(unused)]
pub(crate) struct Icu {
    provider: Box<dyn BoaProvider>,
    locale_canonicalizer: LocaleCanonicalizer,
}

impl std::fmt::Debug for Icu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        struct Canonicalizer;
        f.debug_struct("Icu")
            .field("locale_canonicalizer", &Canonicalizer)
            .finish()
    }
}

impl Icu {
    /// Create a new [`Icu`] from a valid [`BoaProvider`]
    ///
    /// # Errors
    ///
    /// This method will return an error if any of the tools
    /// required cannot be constructed.
    pub(crate) fn new(provider: Box<dyn BoaProvider>) -> Result<Self, DataError> {
        Ok(Self {
            locale_canonicalizer: LocaleCanonicalizer::new(&*provider)?,
            provider,
        })
    }

    /// Get the [`LocaleCanonicalizer`] tool.
    pub(crate) fn locale_canonicalizer(&self) -> &LocaleCanonicalizer {
        &self.locale_canonicalizer
    }

    /// Get the inner icu data provider
    #[allow(unused)]
    pub(crate) fn provider(&self) -> &dyn BoaProvider {
        self.provider.as_ref()
    }
}
