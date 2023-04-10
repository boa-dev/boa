//! Boa's implementation of ECMAScript's global `Intl` object.
//!
//! `Intl` is a built-in object that has properties and methods for i18n. It's not a function object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma402/#intl-object

#![allow(clippy::string_lit_as_bytes)]

use crate::{
    builtins::{Array, BuiltInBuilder, BuiltInObject, IntrinsicObject},
    context::{intrinsics::Intrinsics, BoaProvider},
    object::JsObject,
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
    Context, JsArgs, JsResult, JsValue,
};

use boa_profiler::Profiler;
use icu_provider::KeyedDataMarker;

pub(crate) mod collator;
pub(crate) mod date_time_format;
pub(crate) mod list_format;
pub(crate) mod locale;
pub(crate) mod segmenter;

pub(crate) use self::{
    collator::Collator, date_time_format::DateTimeFormat, list_format::ListFormat, locale::Locale,
    segmenter::Segmenter,
};

mod options;

/// JavaScript `Intl` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Intl;

impl IntrinsicObject for Intl {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

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
                DateTimeFormat::NAME,
                realm
                    .intrinsics()
                    .constructors()
                    .date_time_format()
                    .constructor(),
                DateTimeFormat::ATTRIBUTE,
            )
            .static_method(Self::get_canonical_locales, "getCanonicalLocales", 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().intl()
    }
}

impl BuiltInObject for Intl {
    const NAME: &'static str = "Intl";
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
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);

        // 1. Let ll be ? CanonicalizeLocaleList(locales).
        let ll = locale::canonicalize_locale_list(locales, context)?;

        // 2. Return CreateArrayFromList(ll).
        Ok(JsValue::Object(Array::create_array_from_list(
            ll.into_iter().map(|loc| loc.to_string().into()),
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
    type LangMarker: KeyedDataMarker;

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
    /// new final values.
    /// - If the implementor service doesn't contain any `[[RelevantExtensionKeys]]`, this can be
    /// skipped.
    fn resolve(
        _locale: &mut icu_locid::Locale,
        _options: &mut Self::LocaleOptions,
        _provider: BoaProvider<'_>,
    ) {
    }
}
