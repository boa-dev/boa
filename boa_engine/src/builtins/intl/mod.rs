//! Boa's implementation of ECMAScript's global `Intl` object.
//!
//! `Intl` is a built-in object that has properties and methods for i18n. It's not a function object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma402/#intl-object

#![allow(clippy::string_lit_as_bytes)]

use super::JsArgs;
use crate::{
    builtins::intl::date_time_format::DateTimeFormat,
    builtins::{Array, BuiltIn},
    context::BoaProvider,
    object::ObjectInitializer,
    property::Attribute,
    symbol::JsSymbol,
    Context, JsResult, JsValue,
};

use boa_profiler::Profiler;
use icu_provider::KeyedDataMarker;
use tap::{Conv, Pipe};

pub(crate) mod collator;
pub(crate) mod date_time_format;
pub(crate) mod list_format;
pub(crate) mod locale;
mod options;
pub(crate) mod segmenter;

use self::{collator::Collator, list_format::ListFormat, locale::Locale, segmenter::Segmenter};

/// JavaScript `Intl` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Intl;

impl BuiltIn for Intl {
    const NAME: &'static str = "Intl";

    fn init(context: &mut Context<'_>) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let collator = Collator::init(context).expect("initialization should return a constructor");

        let list_format =
            ListFormat::init(context).expect("initialization should return a constructor");

        let locale = Locale::init(context).expect("initialization should return a constructor");

        let segmenter =
            Segmenter::init(context).expect("initialization should return a constructor");

        let date_time_format = DateTimeFormat::init(context);

        ObjectInitializer::new(context)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                "Collator",
                collator,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                "ListFormat",
                list_format,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                "DateTimeFormat",
                date_time_format,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                "Locale",
                locale,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                "Segmenter",
                segmenter,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .function(Self::get_canonical_locales, "getCanonicalLocales", 1)
            .build()
            .conv::<JsValue>()
            .pipe(Some)
    }
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
