//! This module implements the global `Intl.DateTimeFormat` object.
//!
//! `Intl.DateTimeFormat` is a built-in object that has properties and methods for date and time i18n.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma402/#datetimeformat-objects


use crate::{
    builtins::{
        intl::{date_time_format::{format::FormatOptions, options::{DateStyle, FormatMatcher, TimeStyle}}, locale::{canonicalize_locale_list, resolve_locale, validate_extension}, options::{coerce_options_to_object, IntlOptions}, Service}, options::{get_option, OptionType}, temporal::system_time_zone_id, BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject, OrdinaryObject
    }, context::{icu::IntlProvider, intrinsics::{Intrinsics, StandardConstructor, StandardConstructors}}, error::JsNativeError, js_error, js_string, object::{internal_methods::get_prototype_from_constructor, JsObject}, realm::Realm, string::StaticJsStrings, Context, JsArgs, JsData, JsResult, JsString, JsValue
};

use boa_gc::{Finalize, Trace};
use icu_calendar::preferences::CalendarAlgorithm;
use icu_datetime::{input::{TimeZone, UtcOffset}, preferences::HourCycle, DateTimeFormatterPreferences};
use icu_datetime::provider::DatetimePatternsDateGregorianV1;
use icu_decimal::preferences::NumberingSystem;
use icu_locale::{extensions::unicode::Value, extensions_unicode_key as key, preferences::PreferenceKey, Locale};
use icu_provider::DataMarkerAttributes;
use icu_time::zone::IanaParser;

mod format;
mod options;

#[derive(Debug, Clone)]
pub enum FormatTimeZone {
    UtcOffset(UtcOffset),
    Identifier(TimeZone),
}

/// JavaScript `Intl.DateTimeFormat` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)] // Safety: No traceable types
pub(crate) struct DateTimeFormat {
    locale: Locale,
    calendar: CalendarAlgorithm,
    numbering_system: NumberingSystem,
    time_zone: FormatTimeZone,
    date_style: DateStyle,
    time_style: TimeStyle,
}

impl Service for DateTimeFormat {
    type LangMarker = DatetimePatternsDateGregorianV1;

    type LocaleOptions = DateTimeFormatterPreferences;

    fn resolve(locale: &mut Locale, options: &mut Self::LocaleOptions, provider: &IntlProvider) {
        let locale_preferences = DateTimeFormatterPreferences::from(&*locale);
        // TODO: Determine if any locale_preferences processing is needed here.

        options.locale_preferences = (&*locale).into();

        // The below handles the [[RelevantExtensionKeys]] of DateTimeFormatters
        // internal slots.
        //
        // See https://tc39.es/ecma402/#sec-intl.datetimeformat-internal-slots

        // Hande LDML unicode key "ca", Calendar algorithm
        options.calendar_algorithm = options
            .calendar_algorithm
            .take()
            .filter(|ca| {
                let attr = DataMarkerAttributes::from_str_or_panic(ca.as_str());
                validate_extension::<Self::LangMarker>(locale.id.clone(), attr, provider)
            })
            .inspect(|ca| {
                if Some(ca) == locale_preferences.calendar_algorithm.as_ref() {
                    if let Some(ca) = ca.unicode_extension_value() {
                        locale.extensions.unicode.keywords.set(key!("ca"), ca);
                    }
                }
            })
            .or_else(|| {
                if let Some(ca) = locale_preferences
                    .calendar_algorithm
                    .as_ref()
                    .and_then(CalendarAlgorithm::unicode_extension_value) {
                        locale.extensions.unicode.keywords.set(key!("ca"), ca);
                    }
                locale_preferences.calendar_algorithm
            });

        // Hande LDML unicode key "nu", Numbering system
        options.numbering_system = options
            .numbering_system
            .take()
            .filter(|nu| {
                let attr = DataMarkerAttributes::from_str_or_panic(nu.as_str());
                validate_extension::<Self::LangMarker>(locale.id.clone(), attr, provider)
            })
            .inspect(|nu| {
                if Some(nu) == locale_preferences.numbering_system.as_ref() {
                    if let Some(nu) = nu.unicode_extension_value() {
                        locale.extensions.unicode.keywords.set(key!("nu"), nu);
                    }
                }
            })
            .or_else(|| {
                if let Some(nu) = locale_preferences
                    .numbering_system
                    .as_ref()
                    .and_then(NumberingSystem::unicode_extension_value) {
                        locale.extensions.unicode.keywords.set(key!("nu"), nu);
                    }
                locale_preferences.numbering_system
            });

        // NOTE (nekevss): issue: this will not support `H24` as ICU4X does
        // not currently support it.
        //
        // track: https://github.com/unicode-org/icu4x/issues/6597
        // Handle LDML unicode key "hc", Hour cycle
        options.hour_cycle = options
            .hour_cycle
            .take()
            .filter(|hc| {
                let attr = DataMarkerAttributes::from_str_or_panic(hc.as_str());
                validate_extension::<Self::LangMarker>(locale.id.clone(), attr, provider)
            })
            .inspect(|hc| {
                if Some(hc) == locale_preferences.hour_cycle.as_ref() {
                    if let Some(hc) = hc.unicode_extension_value() {
                        locale.extensions.unicode.keywords.set(key!("hc"), hc);
                    }
                }
            })
            .or_else(|| {
                if let Some(hc) = locale_preferences
                    .hour_cycle
                    .as_ref()
                    .and_then(HourCycle::unicode_extension_value) {
                        locale.extensions.unicode.keywords.set(key!("hc"), hc);
                    }
                locale_preferences.hour_cycle
            });
    }

}

impl IntrinsicObject for DateTimeFormat {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DateTimeFormat {
    const NAME: JsString = StaticJsStrings::DATE_TIME_FORMAT;
}

impl BuiltInConstructor for DateTimeFormat {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::date_time_format;
    /// The `Intl.DateTimeFormat` constructor is the `%DateTimeFormat%` intrinsic object and a standard built-in property of the `Intl` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#datetimeformat-objects
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // NOTE (nekevss): separate calls to `CreateDateTimeFormat` to avoid clone.
        // 1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.
        let new_target = if new_target.is_undefined() {
            context
                .active_function_object()
                .unwrap_or_else(|| {
                    context
                        .intrinsics()
                        .constructors()
                        .date_time_format()
                        .constructor()
                })
                .into()
        } else {
            new_target.clone()
        };
        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 2. Let dateTimeFormat be ? CreateDateTimeFormat(newTarget, locales, options, any, date).
        let date_time_format = create_date_time_format(&new_target, locales, options, context)?;


        // TODO: Should we support the ChainDateTimeFormat
        // 3. If the implementation supports the normative optional constructor mode of 4.3 Note 1, then
            // a. Let this be the this value.
            // b. Return ? ChainDateTimeFormat(dateTimeFormat, NewTarget, this).
        // 4. Return dateTimeFormat.
        Ok(date_time_format.into())
    }
}

// ==== Abstract Operations ====

fn create_date_time_format(
    new_target: &JsValue,
    locales: &JsValue,
    options: &JsValue,
    context: &mut Context,
) -> JsResult<JsObject> {
    // 1. Let dateTimeFormat be ? OrdinaryCreateFromConstructor(newTarget, "%Intl.DateTimeFormat.prototype%",
    // « [[InitializedDateTimeFormat]], [[Locale]], [[Calendar]], [[NumberingSystem]], [[TimeZone]],
    // [[HourCycle]], [[DateStyle]], [[TimeStyle]], [[DateTimeFormat]], [[BoundFormat]] »).
    let date_time_format = get_prototype_from_constructor(new_target, StandardConstructors::date_time_format, context)?;

    // 2. Let hour12 be undefined. <- TODO
    // 3. Let modifyResolutionOptions be a new Abstract Closure with parameters (options) that captures hour12 and performs the following steps when called:
    //        a. Set hour12 to options.[[hour12]].
    //        b. Remove field [[hour12]] from options.
    //        c. If hour12 is not undefined, set options.[[hc]] to null.
    // 4. Let optionsResolution be ? ResolveOptions(%Intl.DateTimeFormat%, %Intl.DateTimeFormat%.[[LocaleData]],
    // locales, options, « coerce-options », modifyResolutionOptions).
    // NOTE: We inline ResolveOptions here. (Could be worked into an abstract operation util function)
    // ResolveOptions 1. Let requestedLocales be ? CanonicalizeLocaleList(locales).
    let requested_locales = canonicalize_locale_list(locales, context)?;
    // NOTE: skip ResolveOptions 2, which is based on `REQUIRE-OPTIONS` vs `COERCE-OPTIONS`
    // ResolveOptions 3. If specialBehaviours is present and contains coerce-options,
    // set options to ? CoerceOptionsToObject(options). Otherwise, set options to ? GetOptionsObject(options).
    let options = coerce_options_to_object(options, context)?;
    // ResolveOptions 4. Let matcher be ? GetOption(options, "localeMatcher", string, « "lookup", "best fit" », "best fit").
    let matcher =
        get_option(&options, js_string!("localeMatcher"), context)?.unwrap_or_default();

    // NOTE: We unroll the below const loop in step 6 using the
    // ResolutionOptionDescriptors from the internal slots
    // https://tc39.es/ecma402/#sec-intl.datetimeformat-internal-slots
    let mut service_options = DateTimeFormatterPreferences::default();

    // 6. For each Resolution Option Descriptor desc of constructor.[[ResolutionOptionDescriptors]], do
    // a. If desc has a [[Type]] field, let type be desc.[[Type]]. Otherwise, let type be string.
    // b. If desc has a [[Values]] field, let values be desc.[[Values]]. Otherwise, let values be empty.
    // c. Let value be ? GetOption(options, desc.[[Property]], type, values, undefined).
    // d. If value is not undefined, then
        // i. Set value to ! ToString(value).
        // ii. If value cannot be matched by the type Unicode locale nonterminal, throw a RangeError exception.
    // e. Let key be desc.[[Key]].
    // f. Set opt.[[<key>]] to value.

    // Handle { [[Key]]: "ca", [[Property]]: "calendar" }
    service_options.calendar_algorithm = get_option::<Value>(&options, js_string!("calendar"), context)?
        .map(|ca| CalendarAlgorithm::try_from(&ca))
        .transpose()
        .map_err(|_icu4x_error| js_error!(RangeError: "unknown calendar algorithm"))?;

    // { [[Key]]: "nu", [[Property]]: "numberingSystem" }
    service_options.numbering_system = get_option::<Value>(&options, js_string!("numberingSystem"), context)?
        .map(|nu| NumberingSystem::try_from(nu))
        .transpose()
        .map_err(|_icu4x_error| js_error!(RangeError: "unknown numbering system"))?;

    // { [[Key]]: "hour12", [[Property]]: "hour12", [[Type]]: boolean }
    let hour_12 = get_option::<bool>(&options, js_string!("hour12"), context)?;

    // { [[Key]]: "hc", [[Property]]: "hourCycle", [[Values]]: « "h11", "h12", "h23", "h24" » }
    service_options.hour_cycle = get_option::<options::HourCycle>(&options, js_string!("hourCycle"), context)?
        .map(|hc| {
            let hc = Value::try_from_utf8(hc.as_utf8()).expect("As utf8 returns a valid subtag");
            // Handle steps 3.a-c here
            // c. If hour12 is not undefined, set options.[[hc]] to null.
            if hour_12.is_some() {
                Ok(None)
            } else {
                HourCycle::try_from(&hc).map(Some)
            }
        })
        .transpose()
        .map_err(|_icu4x_error| js_error!(RangeError: "unknown hour cycle"))?
        .flatten();

    let mut opt = IntlOptions {
        matcher,
        service_options,
    };

    // ResolveOptions 8. Let resolution be ResolveLocale(constructor.[[AvailableLocales]], requestedLocales,
    // opt, constructor.[[RelevantExtensionKeys]], localeData).
    let resolved_locale = resolve_locale::<DateTimeFormat>(
        requested_locales,
        &mut opt,
        context.intl_provider(),
    )?;

    // 5. Set options to optionsResolution.[[Options]].
    // 6. Let r be optionsResolution.[[ResolvedLocale]].
    // 7. Set (deferred) dateTimeFormat.[[Locale]] to r.[[Locale]].
    // 8. Let (deferred) resolvedCalendar be r.[[ca]].
    // 9. Set (deferred) dateTimeFormat.[[Calendar]] to resolvedCalendar.
    // 10. Set (deferred) dateTimeFormat.[[NumberingSystem]] to r.[[nu]].
    // 11. Let (deferred) resolvedLocaleData be r.[[LocaleData]].

    // TODO: Handle hour12 and hc
    // 12. If hour12 is true, then
    // a. Let hc be resolvedLocaleData.[[hourCycle12]].
    // 13. Else if hour12 is false, then
    // a. Let hc be resolvedLocaleData.[[hourCycle24]].
    // 14. Else,
    // a. Assert: hour12 is undefined.
    // b. Let hc be r.[[hc]].
    // c. If hc is null, set hc to resolvedLocaleData.[[hourCycle]].

    // 15. Let timeZone be ? Get(options, "timeZone").
    let time_zone = options.get(js_string!("timeZone"), context)?;

    // 16. If timeZone is undefined, then
    let time_zone = if time_zone.is_undefined() {
        // a. Set timeZone to SystemTimeZoneIdentifier().
        JsString::from(system_time_zone_id()?)
    // 17. Else,
    } else {
        // a. Set timeZone to ? ToString(timeZone).
        time_zone.to_string(context)?
    };
    // 18. If IsTimeZoneOffsetString(timeZone) is true, then
    let time_zone_string = time_zone.to_std_string_escaped();
    // Note: Should a timezone enum be part of temporal_rs, icu_time, or an ECMA402 wrapper lib
    let time_zone = if let Ok(utc_offset) = UtcOffset::try_from_str(&time_zone_string) {
        //  a. Let parseResult be ParseText(StringToCodePoints(timeZone), UTCOffset).
        //  b. Assert: parseResult is a Parse Node.
        //  c. If parseResult contains more than one MinuteSecond Parse Node, throw a RangeError exception.
        //  d. Let offsetNanoseconds be ParseTimeZoneOffsetString(timeZone).
        //  e. Let offsetMinutes be offsetNanoseconds / (6 × 10**10).
        //  f. Assert: offsetMinutes is an integer.
        //  g. Set timeZone to FormatOffsetTimeZoneIdentifier(offsetMinutes).
        FormatTimeZone::UtcOffset(utc_offset)
    } else {
        // 19. Else,
        //  a. Let timeZoneIdentifierRecord be GetAvailableNamedTimeZoneIdentifier(timeZone).
        //  b. If timeZoneIdentifierRecord is empty, throw a RangeError exception.
        //  c. Set timeZone to timeZoneIdentifierRecord.[[PrimaryIdentifier]].
        let parser = IanaParser::try_new_with_buffer_provider(context.intl_provider().erased_provider())
            .map_err(|_err| JsNativeError::error().with_message("Failed to init time zone data provider"))?;
        FormatTimeZone::Identifier(parser.as_borrowed().parse(&time_zone_string))
    };
    // 20. (deferred) Set dateTimeFormat.[[TimeZone]] to timeZone.

    // 21. Let formatOptions be a new Record.
    // 22. Set formatOptions.[[hourCycle]] to hc.
    // 23. Let hasExplicitFormatComponents be false.

    // 24. For each row of Table 16, except the header row, in table order, do
    //         a. Let prop be the name given in the Property column of the current row.
    //         b. If prop is "fractionalSecondDigits", then
    //                i. Let value be ? GetNumberOption(options, "fractionalSecondDigits", 1, 3, undefined).
    //         c. Else,
    //                i. Let values be a List whose elements are the strings given in the Values column of the current row.
    //                ii. Let value be ? GetOption(options, prop, string, values, undefined).
    //         d. Set formatOptions.[[<prop>]] to value.
    //         e. If value is not undefined, then
    //                i. Set hasExplicitFormatComponents to true.
    let format_options = FormatOptions::try_init(&options, service_options.hour_cycle, context)?;

    // 25. Let formatMatcher be ? GetOption(options, "formatMatcher", string, « "basic", "best fit" », "best fit").
    let format_matcher = get_option::<FormatMatcher>(&options, js_string!("formatMatcher"), context)?
        .unwrap_or(FormatMatcher::BestFit);
    // 26. Let dateStyle be ? GetOption(options, "dateStyle", string, « "full", "long", "medium", "short" », undefined).
    let date_style = get_option::<DateStyle>(&options, js_string!("dateStyle"), context)?;
    // 27. Set dateTimeFormat.[[DateStyle]] to dateStyle.
    // 28. Let timeStyle be ? GetOption(options, "timeStyle", string, « "full", "long", "medium", "short" », undefined).
    let time_style = get_option::<TimeStyle>(&options, js_string!("timeStyle"), context)?;
    // 29. (deferred) Set dateTimeFormat.[[TimeStyle]] to timeStyle.
    // 30. If dateStyle is not undefined or timeStyle is not undefined, then
    //         a. If hasExplicitFormatComponents is true, then
    //                i. Throw a TypeError exception.
    //         b. If required is date and timeStyle is not undefined, then
    //                i. Throw a TypeError exception.
    //         c. If required is time and dateStyle is not undefined, then
    //                i. Throw a TypeError exception.
    //         d. Let styles be resolvedLocaleData.[[styles]].[[<resolvedCalendar>]].
    //         e. Let bestFormat be DateTimeStyleFormat(dateStyle, timeStyle, styles).
    // 31. Else,
    //         a. Let needDefaults be true.
    //         b. If required is date or any, then
    //                i. For each property name prop of « "weekday", "year", "month", "day" », do
    //                       1. Let value be formatOptions.[[<prop>]].
    //                       2. If value is not undefined, set needDefaults to false.
    //         c. If required is time or any, then
    //                i. For each property name prop of « "dayPeriod", "hour", "minute", "second", "fractionalSecondDigits" », do
    //                       1. Let value be formatOptions.[[<prop>]].
    //                       2. If value is not undefined, set needDefaults to false.
    //         d. If needDefaults is true and defaults is either date or all, then
    //                i. For each property name prop of « "year", "month", "day" », do
    //                       1. Set formatOptions.[[<prop>]] to "numeric".
    //         e. If needDefaults is true and defaults is either time or all, then
    //                i. For each property name prop of « "hour", "minute", "second" », do
    //                       1. Set formatOptions.[[<prop>]] to "numeric".
    //         f. Let formats be resolvedLocaleData.[[formats]].[[<resolvedCalendar>]].
    //         g. If formatMatcher is "basic", then
    //                i. Let bestFormat be BasicFormatMatcher(formatOptions, formats).
    //         h. Else,
    //                i. Let bestFormat be BestFitFormatMatcher(formatOptions, formats).
    // 32. Set dateTimeFormat.[[DateTimeFormat]] to bestFormat.
    // 33. If bestFormat has a field [[hour]], then
    //         a. Set dateTimeFormat.[[HourCycle]] to hc.
    // 34. Return dateTimeFormat.
    todo!()
}

/// Represents the `required` and `defaults` arguments in the abstract operation
/// `toDateTimeOptions`.
///
/// Since `required` and `defaults` differ only in the `any` and `all` variants,
/// we combine both in a single variant `AnyAll`.
#[allow(unused)]
#[derive(Debug, PartialEq)]
pub(crate) enum DateTimeReqs {
    Date,
    Time,
    AnyAll,
}

/// The abstract operation `toDateTimeOptions` is called with arguments `options`, `required` and
/// `defaults`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-todatetimeoptions
#[allow(unused)]
pub(crate) fn to_date_time_options(
    options: &JsValue,
    required: &DateTimeReqs,
    defaults: &DateTimeReqs,
    context: &mut Context,
) -> JsResult<JsObject> {
    // 1. If options is undefined, let options be null;
    // otherwise let options be ? ToObject(options).
    // 2. Let options be ! OrdinaryObjectCreate(options).
    let options = if options.is_undefined() {
        None
    } else {
        Some(options.to_object(context)?)
    };
    let options = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        options,
        OrdinaryObject,
    );

    // 3. Let needDefaults be true.
    let mut need_defaults = true;

    // 4. If required is "date" or "any", then
    if [DateTimeReqs::Date, DateTimeReqs::AnyAll].contains(required) {
        // a. For each property name prop of « "weekday", "year", "month", "day" », do
        for property in [
            js_string!("weekday"),
            js_string!("year"),
            js_string!("month"),
            js_string!("day"),
        ] {
            // i. Let value be ? Get(options, prop).
            let value = options.get(property, context)?;

            // ii. If value is not undefined, let needDefaults be false.
            if !value.is_undefined() {
                need_defaults = false;
            }
        }
    }

    // 5. If required is "time" or "any", then
    if [DateTimeReqs::Time, DateTimeReqs::AnyAll].contains(required) {
        // a. For each property name prop of « "dayPeriod", "hour", "minute", "second",
        // "fractionalSecondDigits" », do
        for property in [
            js_string!("dayPeriod"),
            js_string!("hour"),
            js_string!("minute"),
            js_string!("second"),
            js_string!("fractionalSecondDigits"),
        ] {
            // i. Let value be ? Get(options, prop).
            let value = options.get(property, context)?;

            // ii. If value is not undefined, let needDefaults be false.
            if !value.is_undefined() {
                need_defaults = false;
            }
        }
    }

    // 6. Let dateStyle be ? Get(options, "dateStyle").
    let date_style = options.get(js_string!("dateStyle"), context)?;

    // 7. Let timeStyle be ? Get(options, "timeStyle").
    let time_style = options.get(js_string!("timeStyle"), context)?;

    // 8. If dateStyle is not undefined or timeStyle is not undefined, let needDefaults be false.
    if !date_style.is_undefined() || !time_style.is_undefined() {
        need_defaults = false;
    }

    // 9. If required is "date" and timeStyle is not undefined, then
    if required == &DateTimeReqs::Date && !time_style.is_undefined() {
        // a. Throw a TypeError exception.
        return Err(JsNativeError::typ()
            .with_message("'date' is required, but timeStyle was defined")
            .into());
    }

    // 10. If required is "time" and dateStyle is not undefined, then
    if required == &DateTimeReqs::Time && !date_style.is_undefined() {
        // a. Throw a TypeError exception.
        return Err(JsNativeError::typ()
            .with_message("'time' is required, but dateStyle was defined")
            .into());
    }

    // 11. If needDefaults is true and defaults is either "date" or "all", then
    if need_defaults && [DateTimeReqs::Date, DateTimeReqs::AnyAll].contains(defaults) {
        // a. For each property name prop of « "year", "month", "day" », do
        for property in [js_string!("year"), js_string!("month"), js_string!("day")] {
            // i. Perform ? CreateDataPropertyOrThrow(options, prop, "numeric").
            options.create_data_property_or_throw(property, js_string!("numeric"), context)?;
        }
    }

    // 12. If needDefaults is true and defaults is either "time" or "all", then
    if need_defaults && [DateTimeReqs::Time, DateTimeReqs::AnyAll].contains(defaults) {
        // a. For each property name prop of « "hour", "minute", "second" », do
        for property in [
            js_string!("hour"),
            js_string!("minute"),
            js_string!("second"),
        ] {
            // i. Perform ? CreateDataPropertyOrThrow(options, prop, "numeric").
            options.create_data_property_or_throw(property, js_string!("numeric"), context)?;
        }
    }

    // 13. Return options.
    Ok(options)
}

impl OptionType for CalendarAlgorithm {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        let s = value.to_string(context)?.to_std_string_escaped();
        Value::try_from_str(&s)
            .ok()
            .and_then(|v| CalendarAlgorithm::try_from(&v).ok())
            .ok_or_else(|| {
                JsNativeError::range()
                    .with_message(format!("provided calendar `{s}` is invalid"))
                    .into()
            })
    }
}

// TODO: track https://github.com/unicode-org/icu4x/issues/6597 and
// https://github.com/tc39/ecma402/issues/1002 for resolution on
// `HourCycle::H24`.
impl OptionType for HourCycle {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_str() {
            "h11" => Ok(HourCycle::H11),
            "h12" => Ok(HourCycle::H12),
            "h23" => Ok(HourCycle::H23),
            _ => Err(JsNativeError::range()
                .with_message("provided hour cycle was not `h11`, `h12` or `h23`")
                .into()),
        }
    }
}
