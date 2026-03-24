//! This module implements the global `Intl.DisplayNames` object.
//!
//! # TODO
//! - Implement the constructor following `InitializeDisplayNames` (§12.1.1)
//! - Implement `of()` (§12.3.3)
//! - Implement `resolvedOptions()` (§12.3.4)
//!
//! [spec]: https://tc39.es/ecma402/#intl-displaynames-objects

use crate::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        intl::options::EmptyPreferences,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
};

use boa_gc::{Finalize, Trace};
use icu_experimental::displaynames::provider::LocaleDisplayNamesV1;
// use icu_experimental::displaynames::{Fallback, LanguageDisplay, RegionDisplayNames, Style};
//use icu_locale::Locale;

mod options;
//uncomment when the constructor is implemented.
//pub(crate) use options::DisplayNamesType;

use super::{
    Service,
    locale::{canonicalize_locale_list, filter_locales},
};

#[derive(Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct DisplayNames {
    //locale: Locale,
    // style: Style,
    // typ: DisplayNamesType,
    // fallback: Fallback,
    // language_display: Option<LanguageDisplay>,
    // native: RegionDisplayNames,
}

// NOTE:
// `Intl.DisplayNames` supports multiple display name categories (language,
// region, script, currency, calendar, datetimefield)[https://docs.rs/icu/latest/icu/experimental/displaynames/provider/index.html#structs], each backed by different ICU providers.
// However, the `Service` trait is required for `supportedLocalesOf`,
// which only depends on locale availability. Therefore, we use
// `LocaleDisplayNamesV1` as a general marker here, while actual
// formatting dispatches on `type` at runtime inside `of()`.

impl Service for DisplayNames {
    //`LocaleDisplayNamesV1`
    // is used here as a temporary stand-in for `supportedLocalesOf` locale
    // availability only, and may be replaced by a more specific marker in the future as ICU4X's DisplayNames API design finalizes
    type LangMarker = LocaleDisplayNamesV1;
    type Preferences = EmptyPreferences;
}

impl IntrinsicObject for DisplayNames {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(
                Self::supported_locales_of,
                js_string!("supportedLocalesOf"),
                1,
            )
            .property(
                JsSymbol::to_string_tag(),
                js_string!("Intl.DisplayNames"),
                Attribute::CONFIGURABLE,
            )
            .method(Self::of, js_string!("of"), 1)
            .method(Self::resolved_options, js_string!("resolvedOptions"), 0)
            .build();
    }
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DisplayNames {
    const NAME: JsString = StaticJsStrings::DISPLAY_NAMES;
}

impl BuiltInConstructor for DisplayNames {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 3;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::display_names;

    ///The `DisplayNames` constructor is the %Intl.DisplayNames% intrinsic object and a standard built-in property of the Intl object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl-displaynames-constructor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DisplayNames
    // TODO: implement §12.1.1
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Intl.DisplayNames must be called with new")
                .into());
        }

        let _locales = args.get_or_undefined(0);
        let _options = args.get_or_undefined(1);

        // 2. Let displayNames be ? OrdinaryCreateFromConstructor(NewTarget, "%Intl.DisplayNames.prototype%", « [[InitializedDisplayNames]], [[Locale]], [[Style]], [[Type]], [[Fallback]], [[LanguageDisplay]], [[Fields]] »).
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::display_names,
            context,
        )?;
        let display_names_format = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            Self{
            //locale,
            // style,
            // typ,
            // fallback,
            // language_display,
            // native,
            },
        );

        // 3. Let optionsResolution be ? ResolveOptions(%Intl.DisplayNames%, %Intl.DisplayNames%.[[LocaleData]], locales, options, « require-options »).
        // 4. Set options to optionsResolution.[[Options]].
        // 5. Let r be optionsResolution.[[ResolvedLocale]].
        // 6. Let style be ? GetOption(options, "style", string, « "narrow", "short", "long" », "long").
        // 7. Set displayNames.[[Style]] to style.
        // 8. Let type be ? GetOption(options, "type", string, « "language", "region", "script", "currency", "calendar", "dateTimeField" », undefined).
        // 9. If type is undefined, throw a TypeError exception.
        // 10. Set displayNames.[[Type]] to type.
        // 11. Let fallback be ? GetOption(options, "fallback", string, « "code", "none" », "code").
        // 12. Set displayNames.[[Fallback]] to fallback.
        // 13. Set displayNames.[[Locale]] to r.[[Locale]].
        // 14. Let resolvedLocaleData be r.[[LocaleData]].
        // 15. Let types be resolvedLocaleData.[[types]].
        // 16. Assert: types is a Record (see 12.2.3).
        // 17. Let languageDisplay be ? GetOption(options, "languageDisplay", string, « "dialect", "standard" », "dialect").
        // 18. Let typeFields be types.[[<type>]].
        // 19. Assert: typeFields is a Record (see 12.2.3).
        // 20. If type is "language", then
        //     a. Set displayNames.[[LanguageDisplay]] to languageDisplay.
        //     b. Set typeFields to typeFields.[[<languageDisplay>]].
        //     c. Assert: typeFields is a Record (see 12.2.3).
        // 21. Let styleFields be typeFields.[[<style>]].
        // 22. Assert: styleFields is a Record (see 12.2.3).
        // 23. Set displayNames.[[Fields]] to styleFields.

        // 24. Return displayNames.
        Ok(display_names_format.into())
    }
}

impl DisplayNames {
    /// `Intl.DisplayNames.supportedLocalesOf` ( locales [ , options ] )[spec].
    ///
    /// The `Intl.DisplayNames.supportedLocalesOf()` static method returns
    /// an array containing those of the provided locales that are supported in
    /// display names without having to fall back to the runtime's default locale.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.DisplayNames.supportedLocalesOf
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DisplayNames/supportedLocalesOf
    fn supported_locales_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. Let availableLocales be %Intl.DisplayNames%.[[AvailableLocales]].
        // 2. Let requestedLocales be ? CanonicalizeLocaleList(locales).
        let requested_locales = canonicalize_locale_list(locales, context)?;

        //3. Return ? FilterLocales(availableLocales, requestedLocales, options).
        filter_locales::<Self>(requested_locales, options, context).map(JsValue::from)
    }

    /// [`Intl.DisplayNames.prototype.of ( code )`][spec].
    ///
    /// The `of()` method of Intl.DisplayNames instances receives a code and returns a string
    /// based on the locale and options provided when instantiating this Intl.DisplayNames object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.DisplayNames.prototype.of
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DisplayNames/of
    // TODO: implement §12.3.3
    fn of(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // 1. Let displayNames be this value.
        // 2. Perform ? RequireInternalSlot(displayNames, [[InitializedDisplayNames]]).
        // 3. Let code be ? ToString(code).
        // 4. Set code to ? CanonicalCodeForDisplayNames(displayNames.[[Type]], code).
        // 5. Let fields be displayNames.[[Fields]].
        // 6. If fields has a field [[<code>]], return fields.[[<code>]].
        // 7. If displayNames.[[Fallback]] is "code", return code.
        // 8. Return undefined.
        Err(JsNativeError::typ()
            .with_message("Intl.DisplayNames.prototype.of is not yet implemented")
            .into())
    }

    /// [`Intl.DisplayNames.prototype.resolvedOptions ( )`][spec].
    ///
    /// This function provides access to the locale and options computed during initialization of the object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.DisplayNames.prototype.resolvedOptions
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DisplayNames/resolvedOptions
    // TODO: implement §12.3.2
    fn resolved_options(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let displayNames be this value.
        // 2. Perform ? RequireInternalSlot(displayNames, [[InitializedDisplayNames]]).
        // 3. Let options be OrdinaryObjectCreate(%Object.prototype%).
        // 4. For each row of Table 18, except the header row, in table order, do
        //    a. Let p be the Property value of the current row.
        //    b. Let v be the value of displayNames's internal slot whose name is the Internal Slot value of the current row.
        //    c. If v is not undefined, then
        //       i. Perform ! CreateDataPropertyOrThrow(options, p, v).
        // 5. Return options.
        Err(JsNativeError::typ()
            .with_message("Intl.DisplayNames.prototype.resolvedOptions is not yet implemented")
            .into())
    }
}
