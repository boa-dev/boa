mod options;

use boa_macros::utf16;
use boa_profiler::Profiler;
use fixed_decimal::FixedDecimal;
use icu_locid::Locale;
use icu_plurals::{
    provider::CardinalV1Marker, PluralCategory, PluralRuleType, PluralRules as NativePluralRules,
};
use icu_provider::DataLocale;

use crate::{
    builtins::{
        options::get_option, Array, BuiltInBuilder, BuiltInConstructor, BuiltInObject,
        IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, ObjectData, ObjectInitializer},
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};

use super::{
    locale::{canonicalize_locale_list, resolve_locale, supported_locales},
    number_format::{
        f64_to_formatted_fixed_decimal, get_digit_format_options, DigitFormatOptions, Extrema,
        Notation,
    },
    options::{coerce_options_to_object, IntlOptions, LocaleMatcher},
    Service,
};

#[derive(Debug)]
pub struct PluralRules {
    locale: Locale,
    native: NativePluralRules,
    rule_type: PluralRuleType,
    format_options: DigitFormatOptions,
}

impl Service for PluralRules {
    type LangMarker = CardinalV1Marker;

    type LocaleOptions = ();
}

impl IntrinsicObject for PluralRules {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::supported_locales_of, "supportedLocalesOf", 1)
            .property(
                JsSymbol::to_string_tag(),
                "Intl.PluralRules",
                Attribute::CONFIGURABLE,
            )
            .method(Self::resolved_options, "resolvedOptions", 0)
            .method(Self::select, "select", 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for PluralRules {
    const NAME: &'static str = "PluralRules";
}

impl BuiltInConstructor for PluralRules {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plural_rules;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("cannot call `Intl.PluralRules` constructor without `new`")
                .into());
        }
        // 2. Let pluralRules be ? OrdinaryCreateFromConstructor(NewTarget, "%PluralRules.prototype%",
        //    « [[InitializedPluralRules]], [[Locale]], [[Type]], [[MinimumIntegerDigits]],
        //    [[MinimumFractionDigits]], [[MaximumFractionDigits]], [[MinimumSignificantDigits]],
        //    [[MaximumSignificantDigits]], [[RoundingType]], [[RoundingIncrement]], [[RoundingMode]],
        //    [[ComputedRoundingPriority]], [[TrailingZeroDisplay]] »).
        // 3. Return ? InitializePluralRules(pluralRules, locales, options).

        // <https://tc39.es/ecma402/#sec-initializepluralrules>

        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. Let requestedLocales be ? CanonicalizeLocaleList(locales).
        let requested_locales = canonicalize_locale_list(locales, context)?;
        // 2. Set options to ? CoerceOptionsToObject(options).
        let options = coerce_options_to_object(options, context)?;

        // 3. Let opt be a new Record.
        // 4. Let matcher be ? GetOption(options, "localeMatcher", string, « "lookup", "best fit" », "best fit").
        // 5. Set opt.[[localeMatcher]] to matcher.
        let matcher =
            get_option::<LocaleMatcher>(&options, utf16!("localeMatcher"), false, context)?
                .unwrap_or_default();

        // 6. Let t be ? GetOption(options, "type", string, « "cardinal", "ordinal" », "cardinal").
        // 7. Set pluralRules.[[Type]] to t.
        let rule_type = get_option::<PluralRuleType>(&options, utf16!("type"), false, context)?
            .unwrap_or(PluralRuleType::Cardinal);

        // 8. Perform ? SetNumberFormatDigitOptions(pluralRules, options, +0𝔽, 3𝔽, "standard").
        let format_options = get_digit_format_options(&options, 0, 3, Notation::Standard, context)?;

        // 9. Let localeData be %PluralRules%.[[LocaleData]].
        // 10. Let r be ResolveLocale(%PluralRules%.[[AvailableLocales]], requestedLocales, opt, %PluralRules%.[[RelevantExtensionKeys]], localeData).
        // 11. Set pluralRules.[[Locale]] to r.[[locale]].
        let locale = resolve_locale::<Self>(
            &requested_locales,
            &mut IntlOptions {
                matcher,
                ..Default::default()
            },
            context.icu(),
        );

        let native = context
            .icu()
            .provider()
            .try_new_plural_rules(&DataLocale::from(&locale), rule_type)
            .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::plural_rules,
            context,
        )?;

        // 12. Return pluralRules.
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ObjectData::plural_rules(Self {
                locale,
                native,
                rule_type,
                format_options,
            }),
        )
        .into())
    }
}

impl PluralRules {
    /// [`Intl.PluralRules.prototype.select ( value )`][spec].
    ///
    /// Returns a string indicating which plural rule to use for locale-aware formatting of a number.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.pluralrules.prototype.select
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/PluralRules/select
    fn select(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let pr be the this value.
        // 2. Perform ? RequireInternalSlot(pr, [[InitializedPluralRules]]).
        let plural_rules = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message(
                "`resolved_options` can only be called on an `Intl.PluralRules` object",
            )
        })?;
        let plural_rules = plural_rules.as_plural_rules().ok_or_else(|| {
            JsNativeError::typ().with_message(
                "`resolved_options` can only be called on an `Intl.PluralRules` object",
            )
        })?;

        let n = args.get_or_undefined(0).to_number(context)?;

        Ok(plural_category_to_js_string(resolve_plural(plural_rules, n).category).into())
    }
    /// [`Intl.PluralRules.supportedLocalesOf ( locales [ , options ] )`][spec].
    ///
    /// Returns an array containing those of the provided locales that are supported in plural rules
    /// without having to fall back to the runtime's default locale.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.pluralrules.supportedlocalesof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/PluralRules/supportedLocalesOf
    fn supported_locales_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. Let availableLocales be %PluralRules%.[[AvailableLocales]].
        // 2. Let requestedLocales be ? CanonicalizeLocaleList(locales).
        let requested_locales = canonicalize_locale_list(locales, context)?;

        // 3. Return ? SupportedLocales(availableLocales, requestedLocales, options).
        supported_locales::<<Self as Service>::LangMarker>(&requested_locales, options, context)
            .map(JsValue::from)
    }

    /// [`Intl.PluralRules.prototype.resolvedOptions ( )`][spec].
    ///
    /// Returns a new object with properties reflecting the locale and options computed during the
    /// construction of the current `Intl.PluralRules` object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.pluralrules.prototype.resolvedoptions
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/PluralRules/resolvedOptions
    fn resolved_options(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let pr be the this value.
        // 2. Perform ? RequireInternalSlot(pr, [[InitializedPluralRules]]).
        let plural_rules = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message(
                "`resolved_options` can only be called on an `Intl.PluralRules` object",
            )
        })?;
        let plural_rules = plural_rules.as_plural_rules().ok_or_else(|| {
            JsNativeError::typ().with_message(
                "`resolved_options` can only be called on an `Intl.PluralRules` object",
            )
        })?;

        // 3. Let options be OrdinaryObjectCreate(%Object.prototype%).
        // 4. For each row of Table 16, except the header row, in table order, do
        //     a. Let p be the Property value of the current row.
        //     b. Let v be the value of pr's internal slot whose name is the Internal Slot value of the current row.
        //     c. If v is not undefined, then
        //         i. Perform ! CreateDataPropertyOrThrow(options, p, v).
        let mut options = ObjectInitializer::new(context);
        options
            .property(
                js_string!("locale"),
                plural_rules.locale.to_string(),
                Attribute::all(),
            )
            .property(
                js_string!("type"),
                match plural_rules.rule_type {
                    PluralRuleType::Cardinal => "cardinal",
                    PluralRuleType::Ordinal => "ordinal",
                    _ => "unknown",
                },
                Attribute::all(),
            )
            .property(
                js_string!("minimumIntegerDigits"),
                plural_rules.format_options.minimum_integer_digits,
                Attribute::all(),
            );

        if let Some(Extrema { minimum, maximum }) =
            plural_rules.format_options.rounding_type.fraction_digits()
        {
            options
                .property(
                    js_string!("minimumFractionDigits"),
                    minimum,
                    Attribute::all(),
                )
                .property(
                    js_string!("maximumFractionDigits"),
                    maximum,
                    Attribute::all(),
                );
        }

        if let Some(Extrema { minimum, maximum }) = plural_rules
            .format_options
            .rounding_type
            .significant_digits()
        {
            options
                .property(
                    js_string!("minimumSignificantDigits"),
                    minimum,
                    Attribute::all(),
                )
                .property(
                    js_string!("maximumSignificantDigits"),
                    maximum,
                    Attribute::all(),
                );
        }

        options
            .property(
                js_string!("roundingMode"),
                plural_rules.format_options.rounding_mode.to_string(),
                Attribute::all(),
            )
            .property(
                js_string!("roundingIncrement"),
                plural_rules.format_options.rounding_increment,
                Attribute::all(),
            )
            .property(
                js_string!("trailingZeroDisplay"),
                plural_rules
                    .format_options
                    .trailing_zero_display
                    .to_string(),
                Attribute::all(),
            );

        // 5. Let pluralCategories be a List of Strings containing all possible results of PluralRuleSelect
        //    for the selected locale pr.[[Locale]].
        let plural_categories = Array::create_array_from_list(
            plural_rules
                .native
                .categories()
                .map(|category| plural_category_to_js_string(category).into()),
            options.context(),
        );

        // 6. Perform ! CreateDataProperty(options, "pluralCategories", CreateArrayFromList(pluralCategories)).
        options.property(
            js_string!("pluralCategories"),
            plural_categories,
            Attribute::all(),
        );

        // 7. If pr.[[RoundingType]] is morePrecision, then
        //     a. Perform ! CreateDataPropertyOrThrow(options, "roundingPriority", "morePrecision").
        // 8. Else if pr.[[RoundingType]] is lessPrecision, then
        //     a. Perform ! CreateDataPropertyOrThrow(options, "roundingPriority", "lessPrecision").
        // 9. Else,
        //     a. Perform ! CreateDataPropertyOrThrow(options, "roundingPriority", "auto").
        options.property(
            js_string!("roundingPriority"),
            plural_rules.format_options.rounding_priority.to_string(),
            Attribute::all(),
        );

        // 10. Return options.
        Ok(options.build().into())
    }
}

#[derive(Debug)]
#[allow(unused)] // Will be used when we implement `selectRange`
struct ResolvedPlural {
    category: PluralCategory,
    formatted: Option<FixedDecimal>,
}

/// Abstract operation [`ResolvePlural ( pluralRules, n )`][spec]
///
/// Gets the plural corresponding to the number with the provided formatting options.
///
/// [spec]: https://tc39.es/ecma402/#sec-resolveplural
fn resolve_plural(plural_rules: &PluralRules, n: f64) -> ResolvedPlural {
    // 1. Assert: Type(pluralRules) is Object.
    // 2. Assert: pluralRules has an [[InitializedPluralRules]] internal slot.
    // 3. Assert: Type(n) is Number.
    // 4. If n is not a finite Number, then
    if !n.is_finite() {
        // a. Return "other".
        return ResolvedPlural {
            category: PluralCategory::Other,
            formatted: None,
        };
    }

    // 5. Let locale be pluralRules.[[Locale]].
    // 6. Let type be pluralRules.[[Type]].
    // 7. Let res be ! FormatNumericToString(pluralRules, n).
    let fixed = f64_to_formatted_fixed_decimal(n, &plural_rules.format_options);

    // 8. Let s be res.[[FormattedString]].
    // 9. Let operands be ! GetOperands(s).
    // 10. Let p be ! PluralRuleSelect(locale, type, n, operands).
    let category = plural_rules.native.category_for(&fixed);

    // 11. Return the Record { [[PluralCategory]]: p, [[FormattedString]]: s }.
    ResolvedPlural {
        category,
        formatted: Some(fixed),
    }
}

fn plural_category_to_js_string(category: PluralCategory) -> JsString {
    match category {
        PluralCategory::Zero => js_string!("zero"),
        PluralCategory::One => js_string!("one"),
        PluralCategory::Two => js_string!("two"),
        PluralCategory::Few => js_string!("few"),
        PluralCategory::Many => js_string!("many"),
        PluralCategory::Other => js_string!("other"),
    }
}
