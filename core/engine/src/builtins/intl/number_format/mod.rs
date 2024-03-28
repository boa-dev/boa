use std::borrow::Cow;

use boa_gc::{Finalize, Trace};
use boa_macros::js_str;
use boa_profiler::Profiler;
use fixed_decimal::{FixedDecimal, FloatPrecision, SignDisplay};
use icu_decimal::{
    options::{FixedDecimalFormatterOptions, GroupingStrategy},
    provider::DecimalSymbolsV1Marker,
    FixedDecimalFormatter, FormattedFixedDecimal,
};

mod options;
use icu_locid::{
    extensions::unicode::{key, Value},
    Locale,
};
use num_bigint::BigInt;
use num_traits::Num;
pub(crate) use options::*;

use crate::{
    builtins::{
        builder::BuiltInBuilder, options::get_option, string::is_trimmable_whitespace,
        BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{
        internal_methods::get_prototype_from_constructor, FunctionObjectBuilder, JsFunction,
        ObjectInitializer,
    },
    property::{Attribute, PropertyDescriptor},
    realm::Realm,
    string::common::StaticJsStrings,
    value::PreferredType,
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
    NativeFunction,
};

use super::{
    locale::{canonicalize_locale_list, resolve_locale, supported_locales, validate_extension},
    options::{coerce_options_to_object, IntlOptions},
    Service,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Trace, Finalize, JsData)]
// Safety: `NumberFormat` only contains non-traceable types.
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct NumberFormat {
    locale: Locale,
    formatter: FixedDecimalFormatter,
    numbering_system: Option<Value>,
    unit_options: UnitFormatOptions,
    digit_options: DigitFormatOptions,
    notation: Notation,
    use_grouping: GroupingStrategy,
    sign_display: SignDisplay,
    bound_format: Option<JsFunction>,
}

impl NumberFormat {
    /// [`FormatNumeric ( numberFormat, x )`][full] and [`FormatNumericToParts ( numberFormat, x )`][parts].
    ///
    /// The returned struct implements `Writable`, allowing to either write the number as a full
    /// string or by parts.
    ///
    /// [full]: https://tc39.es/ecma402/#sec-formatnumber
    /// [parts]: https://tc39.es/ecma402/#sec-formatnumbertoparts
    fn format<'a>(&'a self, value: &'a mut FixedDecimal) -> FormattedFixedDecimal<'a> {
        // TODO: Missing support from ICU4X for Percent/Currency/Unit formatting.
        // TODO: Missing support from ICU4X for Scientific/Engineering/Compact notation.

        self.digit_options.format_fixed_decimal(value);
        value.apply_sign_display(self.sign_display);

        self.formatter.format(value)
    }
}

#[derive(Debug, Clone)]
pub(super) struct NumberFormatLocaleOptions {
    numbering_system: Option<Value>,
}

impl Service for NumberFormat {
    type LangMarker = DecimalSymbolsV1Marker;

    type LocaleOptions = NumberFormatLocaleOptions;

    fn resolve(
        locale: &mut Locale,
        options: &mut Self::LocaleOptions,
        provider: &crate::context::icu::IntlProvider,
    ) {
        let numbering_system = options
            .numbering_system
            .take()
            .filter(|nu| {
                validate_extension::<Self::LangMarker>(locale.id.clone(), key!("nu"), nu, provider)
            })
            .or_else(|| {
                locale
                    .extensions
                    .unicode
                    .keywords
                    .get(&key!("nu"))
                    .cloned()
                    .filter(|nu| {
                        validate_extension::<Self::LangMarker>(
                            locale.id.clone(),
                            key!("nu"),
                            nu,
                            provider,
                        )
                    })
            });

        locale.extensions.unicode.clear();

        if let Some(nu) = numbering_system.clone() {
            locale.extensions.unicode.keywords.set(key!("nu"), nu);
        }

        options.numbering_system = numbering_system;
    }
}

impl IntrinsicObject for NumberFormat {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_format = BuiltInBuilder::callable(realm, Self::get_format)
            .name(js_string!("get format"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(
                Self::supported_locales_of,
                js_string!("supportedLocalesOf"),
                1,
            )
            .property(
                JsSymbol::to_string_tag(),
                js_string!("Intl.NumberFormat"),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("format"),
                Some(get_format),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::resolved_options, js_string!("resolvedOptions"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for NumberFormat {
    const NAME: JsString = StaticJsStrings::NUMBER_FORMAT;
}

impl BuiltInConstructor for NumberFormat {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::number_format;

    /// [`Intl.NumberFormat ( [ locales [ , options ] ] )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.numberformat
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.
        let new_target_inner = &if new_target.is_undefined() {
            context
                .active_function_object()
                .unwrap_or_else(|| {
                    context
                        .intrinsics()
                        .constructors()
                        .number_format()
                        .constructor()
                })
                .into()
        } else {
            new_target.clone()
        };

        // 2. Let numberFormat be ? OrdinaryCreateFromConstructor(newTarget, "%Intl.NumberFormat.prototype%", « [[InitializedNumberFormat]], [[Locale]], [[DataLocale]], [[NumberingSystem]], [[Style]], [[Unit]], [[UnitDisplay]], [[Currency]], [[CurrencyDisplay]], [[CurrencySign]], [[MinimumIntegerDigits]], [[MinimumFractionDigits]], [[MaximumFractionDigits]], [[MinimumSignificantDigits]], [[MaximumSignificantDigits]], [[RoundingType]], [[Notation]], [[CompactDisplay]], [[UseGrouping]], [[SignDisplay]], [[RoundingIncrement]], [[RoundingMode]], [[ComputedRoundingPriority]], [[TrailingZeroDisplay]], [[BoundFormat]] »).
        let prototype = get_prototype_from_constructor(
            new_target_inner,
            StandardConstructors::number_format,
            context,
        )?;

        // 3. Perform ? InitializeNumberFormat(numberFormat, locales, options).

        // `InitializeNumberFormat ( numberFormat, locales, options )`
        // https://tc39.es/ecma402/#sec-initializenumberformat

        // 1. Let requestedLocales be ? CanonicalizeLocaleList(locales).
        let requested_locales = canonicalize_locale_list(locales, context)?;
        // 2. Set options to ? CoerceOptionsToObject(options).
        let options = coerce_options_to_object(options, context)?;

        // 3. Let opt be a new Record.

        // 4. Let matcher be ? GetOption(options, "localeMatcher", string, « "lookup", "best fit" », "best fit").
        // 5. Set opt.[[localeMatcher]] to matcher.
        let matcher = get_option(&options, js_str!("localeMatcher"), context)?.unwrap_or_default();

        // 6. Let numberingSystem be ? GetOption(options, "numberingSystem", string, empty, undefined).
        // 7. If numberingSystem is not undefined, then
        //     a. If numberingSystem cannot be matched by the type Unicode locale nonterminal, throw a RangeError exception.
        // 8. Set opt.[[nu]] to numberingSystem.
        let numbering_system = get_option(&options, js_str!("numberingSystem"), context)?;

        let mut intl_options = IntlOptions {
            matcher,
            service_options: NumberFormatLocaleOptions { numbering_system },
        };

        // 9. Let localeData be %Intl.NumberFormat%.[[LocaleData]].
        // 10. Let r be ResolveLocale(%Intl.NumberFormat%.[[AvailableLocales]], requestedLocales, opt, %Intl.NumberFormat%.[[RelevantExtensionKeys]], localeData).
        let locale = resolve_locale::<Self>(
            &requested_locales,
            &mut intl_options,
            context.intl_provider(),
        );

        // 11. Set numberFormat.[[Locale]] to r.[[locale]].
        // 12. Set numberFormat.[[DataLocale]] to r.[[dataLocale]].
        // 13. Set numberFormat.[[NumberingSystem]] to r.[[nu]].

        // 14. Perform ? SetNumberFormatUnitOptions(numberFormat, options).
        let unit_options = UnitFormatOptions::from_options(&options, context)?;

        // 15. Let style be numberFormat.[[Style]].
        // 16. If style is "currency", then
        let (min_fractional, max_fractional) = if unit_options.style() == Style::Currency {
            // TODO: Missing support from ICU4X
            // a. Let currency be numberFormat.[[Currency]].
            // b. Let cDigits be CurrencyDigits(currency).
            // c. Let mnfdDefault be cDigits.
            // d. Let mxfdDefault be cDigits.
            return Err(JsNativeError::typ().with_message("unimplemented").into());
        } else {
            // 17. Else,
            (
                // a. Let mnfdDefault be 0.
                0,
                // b. If style is "percent", then
                if unit_options.style() == Style::Percent {
                    // i. Let mxfdDefault be 0.
                    0
                } else {
                    // c. Else,
                    //    i. Let mxfdDefault be 3.
                    3
                },
            )
        };

        // 18. Let notation be ? GetOption(options, "notation", string, « "standard", "scientific", "engineering", "compact" », "standard").
        // 19. Set numberFormat.[[Notation]] to notation.
        let notation = get_option(&options, js_str!("notation"), context)?.unwrap_or_default();

        // 20. Perform ? SetNumberFormatDigitOptions(numberFormat, options, mnfdDefault, mxfdDefault, notation).
        let digit_options = DigitFormatOptions::from_options(
            &options,
            min_fractional,
            max_fractional,
            notation,
            context,
        )?;

        // 21. Let compactDisplay be ? GetOption(options, "compactDisplay", string, « "short", "long" », "short").
        let compact_display =
            get_option(&options, js_str!("compactDisplay"), context)?.unwrap_or_default();

        // 22. Let defaultUseGrouping be "auto".
        let mut default_use_grouping = GroupingStrategy::Auto;

        let notation = match notation {
            NotationKind::Standard => Notation::Standard,
            NotationKind::Scientific => Notation::Scientific,
            NotationKind::Engineering => Notation::Engineering,
            // 23. If notation is "compact", then
            NotationKind::Compact => {
                // b. Set defaultUseGrouping to "min2".
                default_use_grouping = GroupingStrategy::Min2;

                // a. Set numberFormat.[[CompactDisplay]] to compactDisplay.
                Notation::Compact {
                    display: compact_display,
                }
            }
        };

        // 24. NOTE: For historical reasons, the strings "true" and "false" are accepted and replaced with the default value.
        // 25. Let useGrouping be ? GetBooleanOrStringNumberFormatOption(options, "useGrouping",
        //     « "min2", "auto", "always", "true", "false" », defaultUseGrouping).
        // 26. If useGrouping is "true" or useGrouping is "false", set useGrouping to defaultUseGrouping.
        // 27. If useGrouping is true, set useGrouping to "always".
        // 28. Set numberFormat.[[UseGrouping]] to useGrouping.
        // useGrouping requires special handling because of the "true" and "false" exceptions.
        // We could also modify the `OptionType` interface but it complicates it a lot just for
        // a single exception.
        let use_grouping = 'block: {
            // GetBooleanOrStringNumberFormatOption ( options, property, stringValues, fallback )
            // <https://tc39.es/ecma402/#sec-getbooleanorstringnumberformatoption>

            // 1. Let value be ? Get(options, property).
            let value = options.get(js_str!("useGrouping"), context)?;

            // 2. If value is undefined, return fallback.
            if value.is_undefined() {
                break 'block default_use_grouping;
            }
            // 3. If value is true, return true.
            if let &JsValue::Boolean(true) = &value {
                break 'block GroupingStrategy::Always;
            }

            // 4. If ToBoolean(value) is false, return false.
            if !value.to_boolean() {
                break 'block GroupingStrategy::Never;
            }

            // 5. Set value to ? ToString(value).
            // 6. If stringValues does not contain value, throw a RangeError exception.
            // 7. Return value.
            match value.to_string(context)?.to_std_string_escaped().as_str() {
                "min2" => GroupingStrategy::Min2,
                "auto" => GroupingStrategy::Auto,
                "always" => GroupingStrategy::Always,
                // special handling for historical reasons
                "true" | "false" => default_use_grouping,
                _ => {
                    return Err(JsNativeError::range()
                        .with_message(
                            "expected one of `min2`, `auto`, `always`, `true`, or `false`",
                        )
                        .into())
                }
            }
        };

        // 29. Let signDisplay be ? GetOption(options, "signDisplay", string, « "auto", "never", "always", "exceptZero", "negative" », "auto").
        // 30. Set numberFormat.[[SignDisplay]] to signDisplay.
        let sign_display =
            get_option(&options, js_str!("signDisplay"), context)?.unwrap_or(SignDisplay::Auto);

        let formatter = FixedDecimalFormatter::try_new_unstable(
            context.intl_provider(),
            &locale.clone().into(),
            {
                let mut options = FixedDecimalFormatterOptions::default();
                options.grouping_strategy = use_grouping;
                options
            },
        )
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;

        let number_format = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            NumberFormat {
                locale,
                numbering_system: intl_options.service_options.numbering_system,
                formatter,
                unit_options,
                digit_options,
                notation,
                use_grouping,
                sign_display,
                bound_format: None,
            },
        );

        // 31. Return unused.

        // 4. If the implementation supports the normative optional constructor mode of 4.3 Note 1, then
        //     a. Let this be the this value.
        //     b. Return ? ChainNumberFormat(numberFormat, NewTarget, this).
        // ChainNumberFormat ( numberFormat, newTarget, this )
        // <https://tc39.es/ecma402/#sec-chainnumberformat>

        let this = context.vm.frame().this(&context.vm);
        let Some(this_obj) = this.as_object() else {
            return Ok(number_format.into());
        };

        let constructor = context
            .intrinsics()
            .constructors()
            .number_format()
            .constructor();

        // 1. If newTarget is undefined and ? OrdinaryHasInstance(%Intl.NumberFormat%, this) is true, then
        if new_target.is_undefined()
            && JsValue::ordinary_has_instance(&constructor.into(), &this, context)?
        {
            let fallback_symbol = context
                .intrinsics()
                .objects()
                .intl()
                .borrow()
                .data
                .fallback_symbol();

            // a. Perform ? DefinePropertyOrThrow(this, %Intl%.[[FallbackSymbol]], PropertyDescriptor{ [[Value]]: numberFormat, [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }).
            this_obj.define_property_or_throw(
                fallback_symbol,
                PropertyDescriptor::builder()
                    .value(number_format)
                    .writable(false)
                    .enumerable(false)
                    .configurable(false),
                context,
            )?;
            // b. Return this.
            Ok(this)
        } else {
            // 2. Return numberFormat.
            Ok(number_format.into())
        }
    }
}

impl NumberFormat {
    /// [`Intl.NumberFormat.supportedLocalesOf ( locales [ , options ] )`][spec].
    ///
    /// Returns an array containing those of the provided locales that are supported in number format
    /// without having to fall back to the runtime's default locale.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.numberformat.supportedlocalesof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/supportedLocalesOf
    fn supported_locales_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. Let availableLocales be %Intl.NumberFormat%.[[AvailableLocales]].
        // 2. Let requestedLocales be ? CanonicalizeLocaleList(locales).
        let requested_locales = canonicalize_locale_list(locales, context)?;

        // 3. Return ? SupportedLocales(availableLocales, requestedLocales, options).
        supported_locales::<<Self as Service>::LangMarker>(&requested_locales, options, context)
            .map(JsValue::from)
    }

    /// [`get Intl.NumberFormat.prototype.format`][spec].
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.numberformat.prototype.format
    fn get_format(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let nf be the this value.
        // 2. If the implementation supports the normative optional constructor mode of 4.3 Note 1, then
        //     a. Set nf to ? UnwrapNumberFormat(nf).
        // 3. Perform ? RequireInternalSlot(nf, [[InitializedNumberFormat]]).
        let nf = unwrap_number_format(this, context)?;
        let nf_clone = nf.clone();
        let mut nf = nf.borrow_mut();

        let bound_format = if let Some(f) = nf.data.bound_format.clone() {
            f
        } else {
            // 4. If nf.[[BoundFormat]] is undefined, then
            //     a. Let F be a new built-in function object as defined in Number Format Functions (15.5.2).
            //     b. Set F.[[NumberFormat]] to nf.
            //     c. Set nf.[[BoundFormat]] to F.
            let bound_format = FunctionObjectBuilder::new(
                context.realm(),
                // Number Format Functions
                // <https://tc39.es/ecma402/#sec-number-format-functions>
                NativeFunction::from_copy_closure_with_captures(
                    |_, args, nf, context| {
                        // 1. Let nf be F.[[NumberFormat]].
                        // 2. Assert: Type(nf) is Object and nf has an [[InitializedNumberFormat]] internal slot.

                        // 3. If value is not provided, let value be undefined.
                        let value = args.get_or_undefined(0);

                        // 4. Let x be ? ToIntlMathematicalValue(value).
                        let mut x = to_intl_mathematical_value(value, context)?;

                        // 5. Return FormatNumeric(nf, x).
                        Ok(js_string!(nf.borrow().data.format(&mut x).to_string()).into())
                    },
                    nf_clone,
                ),
            )
            .length(2)
            .build();

            nf.data.bound_format = Some(bound_format.clone());
            bound_format
        };

        // 5. Return nf.[[BoundFormat]].
        Ok(bound_format.into())
    }

    /// [`Intl.NumberFormat.prototype.resolvedOptions ( )`][spec].
    ///
    /// Returns a new object with properties reflecting the locale and options computed during the
    /// construction of the current `Intl.NumberFormat` object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.numberformat.prototype.resolvedoptions
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/resolvedOptions
    fn resolved_options(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // This function provides access to the locale and options computed during initialization of the object.

        // 1. Let nf be the this value.
        // 2. If the implementation supports the normative optional constructor mode of 4.3 Note 1, then
        //     a. Set nf to ? UnwrapNumberFormat(nf).
        // 3. Perform ? RequireInternalSlot(nf, [[InitializedNumberFormat]]).
        let nf = unwrap_number_format(this, context)?;
        let nf = nf.borrow();
        let nf = &nf.data;

        // 4. Let options be OrdinaryObjectCreate(%Object.prototype%).
        // 5. For each row of Table 12, except the header row, in table order, do
        //     a. Let p be the Property value of the current row.
        //     b. Let v be the value of nf's internal slot whose name is the Internal Slot value of the current row.
        //     c. If v is not undefined, then
        //         i. If there is a Conversion value in the current row, then
        //             1. Assert: The Conversion value of the current row is number.
        //             2. Set v to 𝔽(v).
        //         ii. Perform ! CreateDataPropertyOrThrow(options, p, v).
        let mut options = ObjectInitializer::new(context);
        options.property(
            js_string!("locale"),
            js_string!(nf.locale.to_string()),
            Attribute::all(),
        );
        if let Some(nu) = &nf.numbering_system {
            options.property(
                js_string!("numberingSystem"),
                js_string!(nu.to_string()),
                Attribute::all(),
            );
        }

        options.property(
            js_string!("style"),
            nf.unit_options.style().to_js_string(),
            Attribute::all(),
        );

        match &nf.unit_options {
            UnitFormatOptions::Currency {
                currency,
                display,
                sign,
            } => {
                options.property(
                    js_string!("currency"),
                    currency.to_js_string(),
                    Attribute::all(),
                );
                options.property(
                    js_string!("currencyDisplay"),
                    display.to_js_string(),
                    Attribute::all(),
                );
                options.property(
                    js_string!("currencySign"),
                    sign.to_js_string(),
                    Attribute::all(),
                );
            }
            UnitFormatOptions::Unit { unit, display } => {
                options.property(js_string!("unit"), unit.to_js_string(), Attribute::all());
                options.property(
                    js_string!("unitDisplay"),
                    display.to_js_string(),
                    Attribute::all(),
                );
            }
            UnitFormatOptions::Decimal | UnitFormatOptions::Percent => {}
        }

        options.property(
            js_string!("minimumIntegerDigits"),
            nf.digit_options.minimum_integer_digits,
            Attribute::all(),
        );

        if let Some(Extrema { minimum, maximum }) = nf.digit_options.rounding_type.fraction_digits()
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

        if let Some(Extrema { minimum, maximum }) =
            nf.digit_options.rounding_type.significant_digits()
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

        let use_grouping = match nf.use_grouping {
            GroupingStrategy::Auto => js_string!("auto").into(),
            GroupingStrategy::Never => JsValue::from(false),
            GroupingStrategy::Always => js_string!("always").into(),
            GroupingStrategy::Min2 => js_string!("min2").into(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("unsupported useGrouping value")
                    .into())
            }
        };

        options
            .property(js_string!("useGrouping"), use_grouping, Attribute::all())
            .property(
                js_string!("notation"),
                nf.notation.kind().to_js_string(),
                Attribute::all(),
            );

        if let Notation::Compact { display } = nf.notation {
            options.property(
                js_string!("compactDisplay"),
                display.to_js_string(),
                Attribute::all(),
            );
        }

        let sign_display = match nf.sign_display {
            SignDisplay::Auto => js_string!("auto"),
            SignDisplay::Never => js_string!("never"),
            SignDisplay::Always => js_string!("always"),
            SignDisplay::ExceptZero => js_string!("exceptZero"),
            SignDisplay::Negative => js_string!("negative"),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("unsupported signDisplay value")
                    .into())
            }
        };

        options
            .property(js_string!("signDisplay"), sign_display, Attribute::all())
            .property(
                js_string!("roundingIncrement"),
                nf.digit_options.rounding_increment.to_u16(),
                Attribute::all(),
            )
            .property(
                js_string!("roundingPriority"),
                nf.digit_options.rounding_priority.to_js_string(),
                Attribute::all(),
            )
            .property(
                js_string!("trailingZeroDisplay"),
                nf.digit_options.trailing_zero_display.to_js_string(),
                Attribute::all(),
            );

        // 6. Return options.
        Ok(options.build().into())
    }
}

/// Abstract operation [`UnwrapNumberFormat ( nf )`][spec].
///
/// This also checks that the returned object is a `NumberFormat`, which skips the
/// call to `RequireInternalSlot`.
///
/// [spec]: https://tc39.es/ecma402/#sec-unwrapnumberformat
fn unwrap_number_format(nf: &JsValue, context: &mut Context) -> JsResult<JsObject<NumberFormat>> {
    // 1. If Type(nf) is not Object, throw a TypeError exception.
    let nf_o = nf.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("value was not an `Intl.NumberFormat` object")
    })?;

    if let Ok(nf) = nf_o.clone().downcast::<NumberFormat>() {
        // 3. Return nf.
        return Ok(nf);
    }

    // 2. If nf does not have an [[InitializedNumberFormat]] internal slot and ? OrdinaryHasInstance(%Intl.NumberFormat%, nf)
    //    is true, then
    let constructor = context
        .intrinsics()
        .constructors()
        .number_format()
        .constructor();
    if JsValue::ordinary_has_instance(&constructor.into(), nf, context)? {
        let fallback_symbol = context
            .intrinsics()
            .objects()
            .intl()
            .borrow()
            .data
            .fallback_symbol();

        //    a. Return ? Get(nf, %Intl%.[[FallbackSymbol]]).
        let nf = nf_o.get(fallback_symbol, context)?;
        if let JsValue::Object(nf) = nf {
            if let Ok(nf) = nf.downcast::<NumberFormat>() {
                return Ok(nf);
            }
        }
    }

    Err(JsNativeError::typ()
        .with_message("object was not an `Intl.NumberFormat` object")
        .into())
}

/// Abstract operation [`ToIntlMathematicalValue ( value )`][spec].
///
/// [spec]: https://tc39.es/ecma402/#sec-tointlmathematicalvalue
fn to_intl_mathematical_value(value: &JsValue, context: &mut Context) -> JsResult<FixedDecimal> {
    // 1. Let primValue be ? ToPrimitive(value, number).
    let prim_value = value.to_primitive(context, PreferredType::Number)?;

    // TODO: Add support in `FixedDecimal` for infinity and NaN, which
    // should remove the returned errors.
    match prim_value {
        // 2. If Type(primValue) is BigInt, return ℝ(primValue).
        JsValue::BigInt(bi) => {
            let bi = bi.to_string();
            FixedDecimal::try_from(bi.as_bytes())
                .map_err(|err| JsNativeError::range().with_message(err.to_string()).into())
        }
        // 3. If Type(primValue) is String, then
        //     a. Let str be primValue.
        JsValue::String(s) => {
            // 5. Let text be StringToCodePoints(str).
            // 6. Let literal be ParseText(text, StringNumericLiteral).
            // 7. If literal is a List of errors, return not-a-number.
            // 8. Let intlMV be the StringIntlMV of literal.
            // 9. If intlMV is a mathematical value, then
            //     a. Let rounded be RoundMVResult(abs(intlMV)).
            //     b. If rounded is +∞𝔽 and intlMV < 0, return negative-infinity.
            //     c. If rounded is +∞𝔽, return positive-infinity.
            //     d. If rounded is +0𝔽 and intlMV < 0, return negative-zero.
            //     e. If rounded is +0𝔽, return 0.
            js_string_to_fixed_decimal(&s).ok_or_else(|| {
                JsNativeError::syntax()
                    .with_message("could not parse the provided string")
                    .into()
            })
        }
        // 4. Else,
        other => {
            // a. Let x be ? ToNumber(primValue).
            // b. If x is -0𝔽, return negative-zero.
            // c. Let str be Number::toString(x, 10).
            let x = other.to_number(context)?;

            FixedDecimal::try_from_f64(x, FloatPrecision::Floating)
                .map_err(|err| JsNativeError::range().with_message(err.to_string()).into())
        }
    }
}

/// Abstract operation [`StringToNumber ( str )`][spec], but specialized for the conversion
/// to a `FixedDecimal`.
///
/// [spec]: https://tc39.es/ecma262/#sec-stringtonumber
// TODO: Introduce `Infinity` and `NaN` to `FixedDecimal` to make this operation
// infallible.
pub(crate) fn js_string_to_fixed_decimal(string: &JsString) -> Option<FixedDecimal> {
    // 1. Let text be ! StringToCodePoints(str).
    // 2. Let literal be ParseText(text, StringNumericLiteral).
    let Ok(string) = string.to_std_string() else {
        // 3. If literal is a List of errors, return NaN.
        return None;
    };
    // 4. Return StringNumericValue of literal.
    let string = string.trim_matches(is_trimmable_whitespace);
    match string {
        "" => return Some(FixedDecimal::from(0)),
        "-Infinity" | "Infinity" | "+Infinity" => return None,
        _ => {}
    }

    let mut s = string.bytes();
    let base = match (s.next(), s.next()) {
        (Some(b'0'), Some(b'b' | b'B')) => Some(2),
        (Some(b'0'), Some(b'o' | b'O')) => Some(8),
        (Some(b'0'), Some(b'x' | b'X')) => Some(16),
        // Make sure that no further variants of "infinity" are parsed.
        (Some(b'i' | b'I'), _) => {
            return None;
        }
        _ => None,
    };

    // Parse numbers that begin with `0b`, `0o` and `0x`.
    let s = if let Some(base) = base {
        let string = &string[2..];
        if string.is_empty() {
            return None;
        }
        let int = BigInt::from_str_radix(string, base).ok()?;
        let int_str = int.to_string();

        Cow::Owned(int_str)
    } else {
        Cow::Borrowed(string)
    };

    FixedDecimal::try_from(s.as_bytes()).ok()
}
