//! Boa's implementation of ECMAScript's global `Intl` object.
//!
//! The `Intl` namespace object contains several constructors as well as functionality common to the
//! internationalization constructors and other language sensitive functions. Collectively, they
//! comprise the ECMAScript Internationalization API, which provides language sensitive string
//! comparison, number formatting, date and time formatting, and more.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//!
//! [spec]: https://tc39.es/ecma402/#intl-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl

use crate::{
    Context, JsArgs, JsData, JsResult, JsString, JsValue,
    builtins::{Array, BuiltInBuilder, BuiltInObject, IntrinsicObject},
    context::{icu::IntlProvider, intrinsics::Intrinsics},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
};

use boa_gc::{Finalize, Trace};
use icu_provider::{DataMarker, DataMarkerAttributes};
use static_assertions::const_assert;

pub(crate) mod collator;
pub(crate) mod date_time_format;
pub(crate) mod list_format;
pub(crate) mod locale;
pub(crate) mod number_format;
pub(crate) mod plural_rules;
pub(crate) mod segmenter;

pub(crate) use self::{
    collator::Collator, date_time_format::DateTimeFormat, list_format::ListFormat, locale::Locale,
    number_format::NumberFormat, plural_rules::PluralRules, segmenter::Segmenter,
};

mod options;

// No singletons are allowed as lang markers.
// Hopefully, we'll be able to migrate this to the definition of `Service` in the future
// (https://github.com/rust-lang/rust/issues/76560)
const_assert! {!<Collator as Service>::LangMarker::INFO.is_singleton}
const_assert! {!<ListFormat as Service>::LangMarker::INFO.is_singleton}
const_assert! {!<NumberFormat as Service>::LangMarker::INFO.is_singleton}
const_assert! {!<PluralRules as Service>::LangMarker::INFO.is_singleton}
const_assert! {!<Segmenter as Service>::LangMarker::INFO.is_singleton}

/// JavaScript `Intl` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)]
pub struct Intl {
    fallback_symbol: JsSymbol,
}

impl Intl {
    /// Gets this realm's `Intl` object's `[[FallbackSymbol]]` slot.
    #[must_use]
    pub fn fallback_symbol(&self) -> JsSymbol {
        self.fallback_symbol.clone()
    }

    pub(crate) fn new() -> Option<Self> {
        let fallback_symbol = JsSymbol::new(Some(js_string!("IntlLegacyConstructedSymbol")))?;
        Some(Self { fallback_symbol })
    }
}

impl IntrinsicObject for Intl {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .static_property(
                Collator::NAME,
                realm.intrinsics().constructors().collator().constructor(),
                Collator::ATTRIBUTE,
            )
            .static_property(
                ListFormat::NAME,
                realm
                    .intrinsics()
                    .constructors()
                    .list_format()
                    .constructor(),
                ListFormat::ATTRIBUTE,
            )
            .static_property(
                Locale::NAME,
                realm.intrinsics().constructors().locale().constructor(),
                Locale::ATTRIBUTE,
            )
            .static_property(
                Segmenter::NAME,
                realm.intrinsics().constructors().segmenter().constructor(),
                Segmenter::ATTRIBUTE,
            )
            .static_property(
                PluralRules::NAME,
                realm
                    .intrinsics()
                    .constructors()
                    .plural_rules()
                    .constructor(),
                PluralRules::ATTRIBUTE,
            )
            .static_property(
                DateTimeFormat::NAME,
                realm
                    .intrinsics()
                    .constructors()
                    .date_time_format()
                    .constructor(),
                DateTimeFormat::ATTRIBUTE,
            )
            .static_property(
                NumberFormat::NAME,
                realm
                    .intrinsics()
                    .constructors()
                    .number_format()
                    .constructor(),
                NumberFormat::ATTRIBUTE,
            )
            .static_method(
                Self::get_canonical_locales,
                js_string!("getCanonicalLocales"),
                1,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().intl().upcast()
    }
}

impl BuiltInObject for Intl {
    const NAME: JsString = StaticJsStrings::INTL;
}

impl Intl {
    /// `Intl.getCanonicalLocales ( locales )`
    ///
    /// Returns an array containing the canonical locale names.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN docs][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.getcanonicallocales
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/getCanonicalLocales
    pub(crate) fn get_canonical_locales(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);

        // 1. Let ll be ? CanonicalizeLocaleList(locales).
        let ll = locale::canonicalize_locale_list(locales, context)?;

        // 2. Return CreateArrayFromList(ll).
        Ok(JsValue::new(Array::create_array_from_list(
            ll.into_iter().map(|loc| js_string!(loc.to_string()).into()),
            context,
        )))
    }
}

/// A service component that is part of the `Intl` API.
///
/// This needs to be implemented for every `Intl` service in order to use the functions
/// defined in `locale::utils`, such as locale resolution and selection.
trait Service {
    /// The data marker used by [`resolve_locale`][locale::resolve_locale] to decide
    /// which locales are supported by this service.
    type LangMarker: DataMarker;

    /// The attributes used to resolve the locale.
    const ATTRIBUTES: &'static DataMarkerAttributes = DataMarkerAttributes::empty();

    /// The set of options used in the [`Service::resolve`] method to resolve the provided
    /// locale.
    type LocaleOptions;

    /// Resolves the final value of `locale` from a set of `options`.
    ///
    /// The provided `options` will also be modified with the final values, in case there were
    /// changes in the resolution algorithm.
    ///
    /// # Note
    ///
    /// - A correct implementation must ensure `locale` and `options` are both written with the
    ///   new final values.
    /// - If the implementor service doesn't contain any `[[RelevantExtensionKeys]]`, this can be
    ///   skipped.
    fn resolve(
        _locale: &mut icu_locale::Locale,
        _options: &mut Self::LocaleOptions,
        _provider: &IntlProvider,
    ) {
    }
}
