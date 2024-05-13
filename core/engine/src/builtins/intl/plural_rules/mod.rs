mod options;

use boa_gc::{Finalize, Trace};
use boa_macros::js_str;
use boa_profiler::Profiler;
use fixed_decimal::FixedDecimal;
use icu_locid::Locale;
use icu_plurals::{
    provider::CardinalV1Marker, PluralCategory, PluralRuleType, PluralRules as NativePluralRules,
    PluralRulesWithRanges,
};
use icu_provider::DataLocale;

use crate::{
    builtins::{
        options::get_option, Array, BuiltInBuilder, BuiltInConstructor, BuiltInObject,
        IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, ObjectInitializer},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsStr, JsString, JsSymbol, JsValue,
};

use super::{
    locale::{canonicalize_locale_list, resolve_locale, supported_locales},
    number_format::{DigitFormatOptions, Extrema, NotationKind},
    options::{coerce_options_to_object, IntlOptions},
    Service,
};

#[derive(Debug, Trace, Finalize, JsData)]
// SAFETY: `PluralRules` doesn't contain any traceable data.
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct PluralRules {
    locale: Locale,
    native: PluralRulesWithRanges<NativePluralRules>,
    rule_type: PluralRuleType,
    format_options: DigitFormatOptions,
}

impl Service for PluralRules {
    type LangMarker = CardinalV1Marker;

    type LocaleOptions = ();
}

impl IntrinsicObject for PluralRules {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(
                Self::supported_locales_of,
                js_string!("supportedLocalesOf"),
                1,
            )
            .property(
                JsSymbol::to_string_tag(),
                js_string!("Intl.PluralRules"),
                Attribute::CONFIGURABLE,
            )
            .method(Self::resolved_options, js_string!("resolvedOptions"), 0)
            .method(Self::select, js_string!("select"), 1)
            .method(Self::select_range, js_string!("selectRange"), 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for PluralRules {
    const NAME: JsString = StaticJsStrings::PLURAL_RULES;
}

impl BuiltInConstructor for PluralRules {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plural_rules;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("cannot call `Intl.PluralRules` constructor without `new`")
                .into());
        }
        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::plural_rules,
            context,
        )?;

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
        let matcher = get_option(&options, js_str!("localeMatcher"), context)?.unwrap_or_default();

        // 6. Let t be ? GetOption(options, "type", string, « "cardinal", "ordinal" », "cardinal").
        // 7. Set pluralRules.[[Type]] to t.
        let rule_type =
            get_option(&options, js_str!("type"), context)?.unwrap_or(PluralRuleType::Cardinal);

        // 8. Perform ? SetNumberFormatDigitOptions(pluralRules, options, +0𝔽, 3𝔽, "standard").
        let format_options =
            DigitFormatOptions::from_options(&options, 0, 3, NotationKind::Standard, context)?;

        // 9. Let localeData be %PluralRules%.[[LocaleData]].
        // 10. Let r be ResolveLocale(%PluralRules%.[[AvailableLocales]], requestedLocales, opt, %PluralRules%.[[RelevantExtensionKeys]], localeData).
        // 11. Set pluralRules.[[Locale]] to r.[[locale]].
        let locale = resolve_locale::<Self>(
            &requested_locales,
            &mut IntlOptions {
                matcher,
                ..Default::default()
            },
            context.intl_provider(),
        );

        let native = match rule_type {
            PluralRuleType::Cardinal => PluralRulesWithRanges::try_new_cardinal_unstable(
                context.intl_provider(),
                &DataLocale::from(&locale),
            ),
            PluralRuleType::Ordinal => PluralRulesWithRanges::try_new_ordinal_unstable(
                context.intl_provider(),
                &DataLocale::from(&locale),
            ),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("unimplemented plural rule type")
                    .into())
            }
        }
        .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;

        // 12. Return pluralRules.
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            Self {
                locale,
                native,
                rule_type,
                format_options,
            },
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
    fn select(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let pr be the this value.
        // 2. Perform ? RequireInternalSlot(pr, [[InitializedPluralRules]]).
        let plural_rules = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`select` can only be called on an `Intl.PluralRules` object")
        })?;
        let plural_rules = plural_rules.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`select` can only be called on an `Intl.PluralRules` object")
        })?;

        let n = args.get_or_undefined(0).to_number(context)?;

        Ok(plural_category_to_js_string(resolve_plural(plural_rules, n).category).into())
    }

    /// [`Intl.PluralRules.prototype.selectRange ( start, end )`][spec].
    ///
    /// Receives two values and returns a string indicating which plural rule to use for
    /// locale-aware formatting of the indicated range.
    ///
    /// More information:
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.pluralrules.prototype.selectrange
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/PluralRules/selectRange
    fn select_range(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let pr be the this value.
        // 2. Perform ? RequireInternalSlot(pr, [[InitializedPluralRules]]).
        let plural_rules = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`select_range` can only be called on an `Intl.PluralRules` object")
        })?;
        let plural_rules = plural_rules.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`select_range` can only be called on an `Intl.PluralRules` object")
        })?;

        // 3. If start is undefined or end is undefined, throw a TypeError exception.
        let x = args.get_or_undefined(0);
        let y = args.get_or_undefined(1);
        if x.is_undefined() || y.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("extremum of range cannot be `undefined`")
                .into());
        }

        // 4. Let x be ? ToNumber(start).
        let x = x.to_number(context)?;
        // 5. Let y be ? ToNumber(end).
        let y = y.to_number(context)?;

        // 6. Return ? ResolvePluralRange(pr, x, y).
        // ResolvePluralRange(pr, x, y)
        // <https://tc39.es/ecma402/#sec-resolvepluralrange>

        // 1. If x is NaN or y is NaN, throw a RangeError exception.
        if x.is_nan() || y.is_nan() {
            return Err(JsNativeError::typ()
                .with_message("extremum of range cannot be NaN")
                .into());
        }

        // 2. Let xp be ResolvePlural(pluralRules, x).
        let x = resolve_plural(plural_rules, x);
        // 3. Let yp be ResolvePlural(pluralRules, y).
        let y = resolve_plural(plural_rules, y);

        // 4. If xp.[[FormattedString]] is yp.[[FormattedString]], then
        if x.formatted == y.formatted {
            // a. Return xp.[[PluralCategory]].
            return Ok(plural_category_to_js_string(x.category).into());
        }

        // 5. Let locale be pluralRules.[[Locale]].
        // 6. Let type be pluralRules.[[Type]].
        // 7. Return PluralRuleSelectRange(locale, type, xp.[[PluralCategory]], yp.[[PluralCategory]]).
        Ok(
            plural_category_to_js_string(plural_rules.native.resolve_range(x.category, y.category))
                .into(),
        )
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
        context: &mut Context,
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
    fn resolved_options(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let pr be the this value.
        // 2. Perform ? RequireInternalSlot(pr, [[InitializedPluralRules]]).
        let plural_rules = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message(
                "`resolved_options` can only be called on an `Intl.PluralRules` object",
            )
        })?;
        let plural_rules = plural_rules.downcast_ref::<Self>().ok_or_else(|| {
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
                js_str!("locale"),
                js_string!(plural_rules.locale.to_string()),
                Attribute::all(),
            )
            .property(
                js_str!("type"),
                match plural_rules.rule_type {
                    PluralRuleType::Cardinal => js_str!("cardinal"),
                    PluralRuleType::Ordinal => js_str!("ordinal"),
                    _ => js_str!("unknown"),
                },
                Attribute::all(),
            )
            .property(
                js_str!("minimumIntegerDigits"),
                plural_rules.format_options.minimum_integer_digits,
                Attribute::all(),
            );

        if let Some(Extrema { minimum, maximum }) =
            plural_rules.format_options.rounding_type.fraction_digits()
        {
            options
                .property(js_str!("minimumFractionDigits"), minimum, Attribute::all())
                .property(js_str!("maximumFractionDigits"), maximum, Attribute::all());
        }

        if let Some(Extrema { minimum, maximum }) = plural_rules
            .format_options
            .rounding_type
            .significant_digits()
        {
            options
                .property(
                    js_str!("minimumSignificantDigits"),
                    minimum,
                    Attribute::all(),
                )
                .property(
                    js_str!("maximumSignificantDigits"),
                    maximum,
                    Attribute::all(),
                );
        }

        options
            .property(
                js_str!("roundingMode"),
                js_string!(plural_rules.format_options.rounding_mode.to_js_string()),
                Attribute::all(),
            )
            .property(
                js_str!("roundingIncrement"),
                plural_rules.format_options.rounding_increment.to_u16(),
                Attribute::all(),
            )
            .property(
                js_str!("trailingZeroDisplay"),
                plural_rules
                    .format_options
                    .trailing_zero_display
                    .to_js_string(),
                Attribute::all(),
            );

        // 5. Let pluralCategories be a List of Strings containing all possible results of PluralRuleSelect
        //    for the selected locale pr.[[Locale]].
        let plural_categories = Array::create_array_from_list(
            plural_rules
                .native
                .rules()
                .categories()
                .map(|category| plural_category_to_js_string(category).into()),
            options.context(),
        );

        // 6. Perform ! CreateDataProperty(options, "pluralCategories", CreateArrayFromList(pluralCategories)).
        options.property(
            js_str!("pluralCategories"),
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
            js_str!("roundingPriority"),
            js_string!(plural_rules.format_options.rounding_priority.to_js_string()),
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
    let fixed = plural_rules.format_options.format_f64(n);

    // 8. Let s be res.[[FormattedString]].
    // 9. Let operands be ! GetOperands(s).
    // 10. Let p be ! PluralRuleSelect(locale, type, n, operands).
    let category = plural_rules.native.rules().category_for(&fixed);

    // 11. Return the Record { [[PluralCategory]]: p, [[FormattedString]]: s }.
    ResolvedPlural {
        category,
        formatted: Some(fixed),
    }
}

fn plural_category_to_js_string(category: PluralCategory) -> JsStr<'static> {
    match category {
        PluralCategory::Zero => js_str!("zero"),
        PluralCategory::One => js_str!("one"),
        PluralCategory::Two => js_str!("two"),
        PluralCategory::Few => js_str!("few"),
        PluralCategory::Many => js_str!("many"),
        PluralCategory::Other => js_str!("other"),
    }
}
