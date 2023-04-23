// @generated
#[clippy::msrv = "1.61"]
mod fallback;
#[clippy::msrv = "1.61"]
mod normalizer;
#[clippy::msrv = "1.61"]
mod props;
#[clippy::msrv = "1.61"]
use icu_provider::prelude::*;
/// Implement [`DataProvider<M>`] on the given struct using the data
/// hardcoded in this module. This allows the struct to be used with
/// `icu`'s `_unstable` constructors.
///
/// This macro can only be called from its definition-site, i.e. right
/// after `include!`-ing the generated module.
///
/// ```compile_fail
/// struct MyDataProvider;
/// include!("/path/to/generated/mod.rs");
/// impl_data_provider(MyDataProvider);
/// ```
#[allow(unused_macros)]
macro_rules! impl_data_provider {
    ($ provider : path) => {
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_normalizer::provider::CanonicalCompositionsV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu_normalizer::provider::CanonicalCompositionsV1Marker>, DataError> {
                normalizer::comp_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu_normalizer::provider::CanonicalCompositionsV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_normalizer::provider::CanonicalDecompositionDataV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu_normalizer::provider::CanonicalDecompositionDataV1Marker>, DataError> {
                normalizer::nfd_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu_normalizer::provider::CanonicalDecompositionDataV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_normalizer::provider::CanonicalDecompositionTablesV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu_normalizer::provider::CanonicalDecompositionTablesV1Marker>, DataError> {
                normalizer::nfdex_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu_normalizer::provider::CanonicalDecompositionTablesV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_normalizer::provider::CompatibilityDecompositionSupplementV1Marker> for $provider {
            fn load(
                &self,
                req: DataRequest,
            ) -> Result<DataResponse<::icu_normalizer::provider::CompatibilityDecompositionSupplementV1Marker>, DataError> {
                normalizer::nfkd_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| {
                        DataErrorKind::MissingLocale.with_req(::icu_normalizer::provider::CompatibilityDecompositionSupplementV1Marker::KEY, req)
                    })
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_normalizer::provider::CompatibilityDecompositionTablesV1Marker> for $provider {
            fn load(
                &self,
                req: DataRequest,
            ) -> Result<DataResponse<::icu_normalizer::provider::CompatibilityDecompositionTablesV1Marker>, DataError> {
                normalizer::nfkdex_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| {
                        DataErrorKind::MissingLocale.with_req(::icu_normalizer::provider::CompatibilityDecompositionTablesV1Marker::KEY, req)
                    })
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_properties::provider::IdContinueV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu_properties::provider::IdContinueV1Marker>, DataError> {
                props::idc_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu_properties::provider::IdContinueV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_properties::provider::IdStartV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu_properties::provider::IdStartV1Marker>, DataError> {
                props::ids_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu_properties::provider::IdStartV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_provider_adapters::fallback::provider::CollationFallbackSupplementV1Marker> for $provider {
            fn load(
                &self,
                req: DataRequest,
            ) -> Result<DataResponse<::icu_provider_adapters::fallback::provider::CollationFallbackSupplementV1Marker>, DataError> {
                fallback::supplement::co_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| {
                        DataErrorKind::MissingLocale.with_req(
                            ::icu_provider_adapters::fallback::provider::CollationFallbackSupplementV1Marker::KEY,
                            req,
                        )
                    })
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_provider_adapters::fallback::provider::LocaleFallbackLikelySubtagsV1Marker> for $provider {
            fn load(
                &self,
                req: DataRequest,
            ) -> Result<DataResponse<::icu_provider_adapters::fallback::provider::LocaleFallbackLikelySubtagsV1Marker>, DataError> {
                fallback::likelysubtags_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| {
                        DataErrorKind::MissingLocale.with_req(
                            ::icu_provider_adapters::fallback::provider::LocaleFallbackLikelySubtagsV1Marker::KEY,
                            req,
                        )
                    })
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_provider_adapters::fallback::provider::LocaleFallbackParentsV1Marker> for $provider {
            fn load(
                &self,
                req: DataRequest,
            ) -> Result<DataResponse<::icu_provider_adapters::fallback::provider::LocaleFallbackParentsV1Marker>, DataError> {
                fallback::parents_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| {
                        DataErrorKind::MissingLocale.with_req(::icu_provider_adapters::fallback::provider::LocaleFallbackParentsV1Marker::KEY, req)
                    })
            }
        }
    };
}
/// Implement [`AnyProvider`] on the given struct using the data
/// hardcoded in this module. This allows the struct to be used with
/// `icu`'s `_any` constructors.
///
/// This macro can only be called from its definition-site, i.e. right
/// after `include!`-ing the generated module.
///
/// ```compile_fail
/// struct MyAnyProvider;
/// include!("/path/to/generated/mod.rs");
/// impl_any_provider(MyAnyProvider);
/// ```
#[allow(unused_macros)]
macro_rules! impl_any_provider {
    ($ provider : path) => {
        #[clippy::msrv = "1.61"]
        impl AnyProvider for $provider {
            fn load_any(&self, key: DataKey, req: DataRequest) -> Result<AnyResponse, DataError> {
                const CANONICALCOMPOSITIONSV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_normalizer::provider::CanonicalCompositionsV1Marker::KEY.hashed();
                const CANONICALDECOMPOSITIONDATAV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_normalizer::provider::CanonicalDecompositionDataV1Marker::KEY.hashed();
                const CANONICALDECOMPOSITIONTABLESV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_normalizer::provider::CanonicalDecompositionTablesV1Marker::KEY.hashed();
                const COMPATIBILITYDECOMPOSITIONSUPPLEMENTV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_normalizer::provider::CompatibilityDecompositionSupplementV1Marker::KEY.hashed();
                const COMPATIBILITYDECOMPOSITIONTABLESV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_normalizer::provider::CompatibilityDecompositionTablesV1Marker::KEY.hashed();
                const IDCONTINUEV1MARKER: ::icu_provider::DataKeyHash = ::icu_properties::provider::IdContinueV1Marker::KEY.hashed();
                const IDSTARTV1MARKER: ::icu_provider::DataKeyHash = ::icu_properties::provider::IdStartV1Marker::KEY.hashed();
                const COLLATIONFALLBACKSUPPLEMENTV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_provider_adapters::fallback::provider::CollationFallbackSupplementV1Marker::KEY.hashed();
                const LOCALEFALLBACKLIKELYSUBTAGSV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_provider_adapters::fallback::provider::LocaleFallbackLikelySubtagsV1Marker::KEY.hashed();
                const LOCALEFALLBACKPARENTSV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_provider_adapters::fallback::provider::LocaleFallbackParentsV1Marker::KEY.hashed();
                match key.hashed() {
                    CANONICALCOMPOSITIONSV1MARKER => normalizer::comp_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    CANONICALDECOMPOSITIONDATAV1MARKER => normalizer::nfd_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    CANONICALDECOMPOSITIONTABLESV1MARKER => normalizer::nfdex_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    COMPATIBILITYDECOMPOSITIONSUPPLEMENTV1MARKER => normalizer::nfkd_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    COMPATIBILITYDECOMPOSITIONTABLESV1MARKER => normalizer::nfkdex_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    IDCONTINUEV1MARKER => props::idc_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    IDSTARTV1MARKER => props::ids_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    COLLATIONFALLBACKSUPPLEMENTV1MARKER => fallback::supplement::co_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    LOCALEFALLBACKLIKELYSUBTAGSV1MARKER => fallback::likelysubtags_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    LOCALEFALLBACKPARENTSV1MARKER => fallback::parents_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    _ => return Err(DataErrorKind::MissingDataKey.with_req(key, req)),
                }
                .map(|payload| AnyResponse {
                    payload: Some(payload),
                    metadata: Default::default(),
                })
                .ok_or_else(|| DataErrorKind::MissingLocale.with_req(key, req))
            }
        }
    };
}
#[clippy::msrv = "1.61"]
pub struct BakedDataProvider;
impl_data_provider!(BakedDataProvider);
