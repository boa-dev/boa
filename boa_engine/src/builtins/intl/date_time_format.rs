//! This module implements the global `Intl.DateTimeFormat` object.
//!
//! `Intl.DateTimeFormat` is a built-in object that has properties and methods for date and time i18n.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma402/#datetimeformat-objects

use crate::{
    builtins::intl::{
        canonicalize_locale_list, default_locale, get_number_option, get_option, resolve_locale,
        DateTimeFormatRecord, GetOptionType, LocaleDataRecord,
    },
    builtins::JsArgs,
    context::intrinsics::StandardConstructors,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, JsFunction, JsObject,
        ObjectData,
    },
    Context, JsResult, JsString, JsValue,
};

use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use chrono_tz::Tz;
use icu::{
    calendar::{buddhist::Buddhist, japanese::Japanese, Gregorian},
    datetime,
    datetime::{
        options::{components, length, preferences},
        DateTimeFormatOptions,
    },
    locid::Locale,
};
use icu_provider::inv::InvariantDataProvider;
use rustc_hash::FxHashMap;
use std::cmp::{max, min};

/// JavaScript `Intl.DateTimeFormat` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct DateTimeFormat {
    initialized_date_time_format: bool,
    locale: JsString,
    calendar: JsValue,
    numbering_system: JsValue,
    time_zone: JsString,
    weekday: JsValue,
    era: JsValue,
    year: JsValue,
    month: JsValue,
    day: JsValue,
    day_period: JsString,
    hour: JsValue,
    minute: JsValue,
    second: JsValue,
    fractional_second_digits: JsString,
    time_zone_name: JsValue,
    hour_cycle: JsValue,
    pattern: JsString,
    bound_format: JsString,
    date_style: JsValue,
    time_style: JsValue,
}

impl DateTimeFormat {
    const NAME: &'static str = "DateTimeFormat";

    pub(super) fn init(context: &mut Context) -> JsFunction {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        ConstructorBuilder::new(context, Self::constructor)
            .name(Self::NAME)
            .length(0)
            .build()
    }
}

impl DateTimeFormat {
    /// The `Intl.DateTimeFormat` constructor is the `%DateTimeFormat%` intrinsic object and a standard built-in property of the `Intl` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#datetimeformat-objects
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::date_time_format,
            context,
        )?;
        // 2. Let dateTimeFormat be ? OrdinaryCreateFromConstructor(newTarget, "%DateTimeFormat.prototype%",
        // « [[InitializedDateTimeFormat]], [[Locale]], [[Calendar]], [[NumberingSystem]], [[TimeZone]], [[Weekday]],
        // [[Era]], [[Year]], [[Month]], [[Day]], [[DayPeriod]], [[Hour]], [[Minute]], [[Second]],
        // [[FractionalSecondDigits]], [[TimeZoneName]], [[HourCycle]], [[Pattern]], [[BoundFormat]] »).
        let date_time_format = JsObject::from_proto_and_data(
            prototype,
            ObjectData::date_time_format(Box::new(Self {
                initialized_date_time_format: true,
                locale: JsString::from("en-US"),
                calendar: JsValue::String(JsString::from("gregory")),
                numbering_system: JsValue::String(JsString::from("arab")),
                time_zone: JsString::from("UTC"),
                weekday: JsValue::String(JsString::from("narrow")),
                era: JsValue::String(JsString::from("narrow")),
                year: JsValue::String(JsString::from("numeric")),
                month: JsValue::String(JsString::from("narrow")),
                day: JsValue::String(JsString::from("numeric")),
                day_period: JsString::from("narrow"),
                hour: JsValue::String(JsString::from("numeric")),
                minute: JsValue::String(JsString::from("numeric")),
                second: JsValue::String(JsString::from("numeric")),
                fractional_second_digits: JsString::from(""),
                time_zone_name: JsValue::String(JsString::from("")),
                hour_cycle: JsValue::String(JsString::from("h24")),
                pattern: JsString::from("{hour}:{minute}"),
                bound_format: JsString::from("undefined"),
                date_style: JsValue::String(JsString::from("full")),
                time_style: JsValue::String(JsString::from("full")),
            })),
        );

        // 3. Perform ? InitializeDateTimeFormat(dateTimeFormat, locales, options).
        let maybe_locales = args.get_or_undefined(0);
        let maybe_options = args.get_or_undefined(1);
        let date_time_format =
            initialize_date_time_format(&date_time_format, maybe_locales, maybe_options, context)?;

        // TODO 4. If the implementation supports the normative optional constructor mode of 4.3 Note 1, then
        // TODO a. Let this be the this value.
        // TODO b. Return ? ChainDateTimeFormat(dateTimeFormat, NewTarget, this).

        // 5. Return dateTimeFormat.
        Ok(date_time_format.into())
    }
}

/// Represents the `required` and `defaults` arguments in the abstract operation
/// `toDateTimeOptions`.
///
/// Since `required` and `defaults` differ only in the `any` and `all` variants,
/// we combine both in a single variant `AnyAll`.
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
    let options = JsObject::from_proto_and_data(options, ObjectData::ordinary());

    // 3. Let needDefaults be true.
    let mut need_defaults = true;

    // 4. If required is "date" or "any", then
    if [DateTimeReqs::Date, DateTimeReqs::AnyAll].contains(required) {
        // a. For each property name prop of « "weekday", "year", "month", "day" », do
        for property in ["weekday", "year", "month", "day"] {
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
            "dayPeriod",
            "hour",
            "minute",
            "second",
            "fractionalSecondDigits",
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
    let date_style = options.get("dateStyle", context)?;

    // 7. Let timeStyle be ? Get(options, "timeStyle").
    let time_style = options.get("timeStyle", context)?;

    // 8. If dateStyle is not undefined or timeStyle is not undefined, let needDefaults be false.
    if !date_style.is_undefined() || !time_style.is_undefined() {
        need_defaults = false;
    }

    // 9. If required is "date" and timeStyle is not undefined, then
    if required == &DateTimeReqs::Date && !time_style.is_undefined() {
        // a. Throw a TypeError exception.
        return context.throw_type_error("'date' is required, but timeStyle was defined");
    }

    // 10. If required is "time" and dateStyle is not undefined, then
    if required == &DateTimeReqs::Time && !date_style.is_undefined() {
        // a. Throw a TypeError exception.
        return context.throw_type_error("'time' is required, but dateStyle was defined");
    }

    // 11. If needDefaults is true and defaults is either "date" or "all", then
    if need_defaults && [DateTimeReqs::Date, DateTimeReqs::AnyAll].contains(defaults) {
        // a. For each property name prop of « "year", "month", "day" », do
        for property in ["year", "month", "day"] {
            // i. Perform ? CreateDataPropertyOrThrow(options, prop, "numeric").
            options.create_data_property_or_throw(property, "numeric", context)?;
        }
    }

    // 12. If needDefaults is true and defaults is either "time" or "all", then
    if need_defaults && [DateTimeReqs::Time, DateTimeReqs::AnyAll].contains(defaults) {
        // a. For each property name prop of « "hour", "minute", "second" », do
        for property in ["hour", "minute", "second"] {
            // i. Perform ? CreateDataPropertyOrThrow(options, prop, "numeric").
            options.create_data_property_or_throw(property, "numeric", context)?;
        }
    }

    // 13. Return options.
    Ok(options)
}

/// The abstract operation `is_terminal` determines whether `opt` `JsValue` contains a
/// nonterminal symbol.
///
/// More information:
///  - [Unicode LDML reference][spec]
///
/// [spec]: https://www.unicode.org/reports/tr35/#Unicode_locale_identifier
pub(crate) fn is_terminal(opt_str: &str) -> bool {
    if opt_str.is_empty() {
        return true;
    }

    // nonterminal = alphanum{3,8} (sep alphanum{3,8})*
    // Any number of alphanumeric characters between 3 and 8,
    // separated by dash,
    // followed by any number of alphanumeric characters between 3 and 8 (repeated)

    // First, replace all underscores (legacy format) with dashes.
    let opt_str = opt_str.replace('_', "-");

    // Next, split the string by dashes.
    let options_vec: Vec<&str> = opt_str.split('-').collect();

    // If the vector contains less than 1 element, that cannot be a nonterminal.
    if options_vec.is_empty() {
        return true;
    }

    // Check that each slice is has length between 3 and 8 and all characters are alphanumeric.
    for option in options_vec {
        if option.len() < 3 || option.len() > 8 {
            return true;
        }

        if !option
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
        {
            return true;
        }
    }

    false
}

/// The value of the `LocaleData` internal slot is implementation-defined
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-intl.datetimeformat-internal-slots
fn build_locale_data(available_locales: &[JsString]) -> LocaleDataRecord {
    let mut locale_data_entry = FxHashMap::default();
    let nu_values = Vec::from([JsString::new("arab")]);
    locale_data_entry.insert(JsString::new("nu"), nu_values);

    let hc_values = Vec::from([
        JsString::new("h11"),
        JsString::new("h12"),
        JsString::new("h23"),
        JsString::new("h24"),
    ]);
    locale_data_entry.insert(JsString::new("hc"), hc_values);

    let ca_values = Vec::from([JsString::new("gregory")]);
    locale_data_entry.insert(JsString::new("ca"), ca_values);

    let hour_cycle_values = Vec::from([JsString::new("h24")]);
    locale_data_entry.insert(JsString::new("hourCycle"), hour_cycle_values);

    let mut locale_data = FxHashMap::default();

    for avail_locale in available_locales {
        locale_data.insert(avail_locale.clone(), locale_data_entry.clone());
    }

    locale_data
}

/// The value of the `RelevantExtensionKeys` internal slot is « "ca", "hc", "nu" ».
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-intl.datetimeformat-internal-slots
fn build_relevant_ext_keys() -> Vec<JsString> {
    vec![
        JsString::new("ca"),
        JsString::new("hc"),
        JsString::new("nu"),
    ]
}

/// `AvailableLocales` is a `List` that contains structurally valid and canonicalized Unicode
/// BCP 47 locale identifiers identifying the locales for which the implementation provides the
/// functionality of the constructed objects. Language tags on the list must not have a Unicode
/// locale extension sequence. The list must include the value returned by the `DefaultLocale`
/// abstract operation, and must not include duplicates. Implementations must include in
/// `AvailableLocales` locales that can serve as fallbacks in the algorithm used to resolve locales.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-internal-slots
fn build_available_locales(context: &Context) -> Vec<JsString> {
    let default_locale_str = default_locale(context.icu().locale_canonicalizer()).to_string();
    let canonicalized_locale = default_locale_str.replace('_', "-");
    let splitted_locale: Vec<&str> = canonicalized_locale.split('-').collect();
    let default_locale_fallback = splitted_locale
        .get(0)
        .expect("Failed to split default locale");
    let available_locales = vec![
        JsString::new(default_locale_str),
        JsString::new(default_locale_fallback),
    ];

    available_locales
}

/// The `DefaultTimeZone` abstract operation returns a String value representing the valid and
/// canonicalized time zone name for the host environment's current time zone.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-defaulttimezone
fn default_time_zone() -> JsString {
    // FIXME fetch default time zone from the environment
    JsString::new("UTC")
}

/// The abstract operation `IsValidTimeZoneName` takes argument `timeZone`, a String value, and
/// verifies that it represents a valid Zone or Link name of the IANA Time Zone Database.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-isvalidtimezonename
pub(crate) fn is_valid_time_zone_name(time_zone: &JsString) -> bool {
    let time_zone_str = time_zone.to_string();
    let maybe_time_zone: Result<Tz, _> = time_zone_str.parse();
    maybe_time_zone.is_ok()
}

/// The abstract operation `CanonicalizeTimeZoneName` takes argument `timeZone` (a String value
/// that is a valid time zone name as verified by `IsValidTimeZoneName`). It returns the canonical
/// and case-regularized form of timeZone.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-canonicalizetimezonename
pub(crate) fn canonicalize_time_zone_name(time_zone: &JsString) -> JsString {
    let time_zone_str = time_zone.to_string();
    let canonical_tz: Tz = time_zone_str
        .parse()
        .expect("CanonicalizeTimeZoneName: time zone name is not supported");
    JsString::from(canonical_tz.name())
}

/// Converts `hour_cycle_str` to `preferences::HourCycle`
pub(crate) fn string_to_hour_cycle(hour_cycle_str: &JsString) -> preferences::HourCycle {
    match hour_cycle_str.as_str() {
        "h11" => preferences::HourCycle::H11,
        "h12" => preferences::HourCycle::H12,
        "h23" => preferences::HourCycle::H23,
        "h24" => preferences::HourCycle::H24,
        _ => panic!("Invalid hour cycle"),
    }
}

/// Converts `JsValue` to `length::Date`
pub(crate) fn value_to_date_style(
    date_style_val: &JsValue,
    context: &mut Context,
) -> Option<length::Date> {
    if date_style_val.is_undefined() {
        return None;
    }
    let date_style_str = date_style_val
        .to_string(context)
        .unwrap_or_else(|_| JsString::empty());
    match date_style_str.as_str() {
        "full" => Some(length::Date::Full),
        "long" => Some(length::Date::Long),
        "medium" => Some(length::Date::Medium),
        "short" => Some(length::Date::Short),
        _ => None,
    }
}

/// Converts `JsValue` to `length::Time`
pub(crate) fn value_to_time_style(
    time_style_val: &JsValue,
    context: &mut Context,
) -> Option<length::Time> {
    if time_style_val.is_undefined() {
        return None;
    }
    let time_style_str = time_style_val
        .to_string(context)
        .unwrap_or_else(|_| JsString::empty());
    match time_style_str.as_str() {
        "full" => Some(length::Time::Full),
        "long" => Some(length::Time::Long),
        "medium" => Some(length::Time::Medium),
        "short" => Some(length::Time::Short),
        _ => None,
    }
}

/// Converts `components::Text` to corresponding `JsString`
pub(crate) fn text_to_value(maybe_txt: Option<components::Text>) -> JsValue {
    match maybe_txt {
        None => JsValue::undefined(),
        Some(txt) => match txt {
            components::Text::Long => JsValue::String(JsString::from("long")),
            components::Text::Short => JsValue::String(JsString::from("short")),
            components::Text::Narrow => JsValue::String(JsString::from("narrow")),
            _ => JsValue::undefined(),
        },
    }
}
/// Converts `components::Year` to corresponding `JsString`
pub(crate) fn year_to_value(maybe_year: Option<components::Year>) -> JsValue {
    match maybe_year {
        None => JsValue::undefined(),
        Some(year) => match year {
            components::Year::Numeric => JsValue::String(JsString::from("numeric")),
            components::Year::TwoDigit => JsValue::String(JsString::from("2-digit")),
            components::Year::NumericWeekOf => JsValue::String(JsString::from("numericWeek")),
            components::Year::TwoDigitWeekOf => JsValue::String(JsString::from("2-digitWeek")),
            _ => JsValue::undefined(),
        },
    }
}

/// Converts `components::Month` to corresponding `JsString`
pub(crate) fn month_to_value(maybe_month: Option<components::Month>) -> JsValue {
    match maybe_month {
        None => JsValue::undefined(),
        Some(month_val) => match month_val {
            components::Month::Numeric => JsValue::String(JsString::from("numeric")),
            components::Month::TwoDigit => JsValue::String(JsString::from("2-digit")),
            components::Month::Long => JsValue::String(JsString::from("long")),
            components::Month::Short => JsValue::String(JsString::from("short")),
            components::Month::Narrow => JsValue::String(JsString::from("narrow")),
            _ => JsValue::undefined(),
        },
    }
}

/// Converts `components::Day` to corresponding `JsString`
pub(crate) fn day_to_value(maybe_day: Option<components::Day>) -> JsValue {
    match maybe_day {
        None => JsValue::undefined(),
        Some(day_val) => match day_val {
            components::Day::NumericDayOfMonth => JsValue::String(JsString::from("numeric")),
            components::Day::TwoDigitDayOfMonth => JsValue::String(JsString::from("2-digit")),
            components::Day::DayOfWeekInMonth => JsValue::String(JsString::from("dayOfWeek")),
            _ => JsValue::undefined(),
        },
    }
}

/// Converts `components::Numeric` to corresponding `JsString`
pub(crate) fn numeric_to_value(maybe_num: Option<components::Numeric>) -> JsValue {
    match maybe_num {
        None => JsValue::undefined(),
        Some(num_val) => match num_val {
            components::Numeric::Numeric => JsValue::String(JsString::from("numeric")),
            components::Numeric::TwoDigit => JsValue::String(JsString::from("2-digit")),
            _ => JsValue::undefined(),
        },
    }
}

/// Converts `components::TimeZoneName` to corresponding `JsString`
pub(crate) fn time_zone_to_value(maybe_tz: Option<components::TimeZoneName>) -> JsValue {
    match maybe_tz {
        None => JsValue::undefined(),
        Some(tz_val) => match tz_val {
            components::TimeZoneName::ShortSpecific => JsValue::String(JsString::from("short")),
            components::TimeZoneName::LongSpecific => JsValue::String(JsString::from("long")),
            components::TimeZoneName::GmtOffset => JsValue::String(JsString::from("gmt")),
            components::TimeZoneName::ShortGeneric => {
                JsValue::String(JsString::from("shortGeneric"))
            }
            components::TimeZoneName::LongGeneric => JsValue::String(JsString::from("longGeneric")),
            _ => JsValue::undefined(),
        },
    }
}

/// Fetches field with name `property` from `format` bag
fn get_format_field(format: &components::Bag, property: &str) -> JsValue {
    match property {
        "weekday" => text_to_value(format.weekday),
        "era" => text_to_value(format.era),
        "year" => year_to_value(format.year),
        "month" => month_to_value(format.month),
        "day" => day_to_value(format.day),
        "hour" => numeric_to_value(format.hour),
        "minute" => numeric_to_value(format.minute),
        "second" => numeric_to_value(format.second),
        "timeZoneName" => time_zone_to_value(format.time_zone_name),
        _ => JsValue::undefined(),
    }
}

/// `FormatOptionsRecord` type aggregates `DateTimeFormatOptions` string and `components` map.
/// At this point, `DateTimeFormatOptions` does not support `components::Bag`.
#[derive(Debug)]
pub(crate) struct FormatOptionsRecord {
    pub(crate) date_time_format_opts: DateTimeFormatOptions,
    pub(crate) components: FxHashMap<JsString, JsValue>,
}

/// `DateTimeComponents` type contains `property` map and list of `values`
#[derive(Debug)]
struct DateTimeComponents {
    property: JsString,
    values: Vec<JsString>,
}

/// Builds a list of `DateTimeComponents` which is commonly referred to as "Table 6"
fn build_date_time_components() -> Vec<DateTimeComponents> {
    Vec::from([
        DateTimeComponents {
            property: JsString::new("weekday"),
            values: Vec::from([
                JsString::new("narrow"),
                JsString::new("short"),
                JsString::new("long"),
            ]),
        },
        DateTimeComponents {
            property: JsString::new("era"),
            values: Vec::from([
                JsString::new("narrow"),
                JsString::new("short"),
                JsString::new("long"),
            ]),
        },
        DateTimeComponents {
            property: JsString::new("year"),
            values: Vec::from([JsString::new("2-digit"), JsString::new("numeric")]),
        },
        DateTimeComponents {
            property: JsString::new("month"),
            values: Vec::from([
                JsString::new("2-digit"),
                JsString::new("numeric"),
                JsString::new("narrow"),
                JsString::new("short"),
                JsString::new("long"),
            ]),
        },
        DateTimeComponents {
            property: JsString::new("day"),
            values: Vec::from([JsString::new("2-digit"), JsString::new("numeric")]),
        },
        DateTimeComponents {
            property: JsString::new("dayPeriod"),
            values: Vec::from([
                JsString::new("narrow"),
                JsString::new("short"),
                JsString::new("long"),
            ]),
        },
        DateTimeComponents {
            property: JsString::new("hour"),
            values: Vec::from([JsString::new("2-digit"), JsString::new("numeric")]),
        },
        DateTimeComponents {
            property: JsString::new("minute"),
            values: Vec::from([JsString::new("2-digit"), JsString::new("numeric")]),
        },
        DateTimeComponents {
            property: JsString::new("second"),
            values: Vec::from([JsString::new("2-digit"), JsString::new("numeric")]),
        },
        DateTimeComponents {
            property: JsString::new("fractionalSecondDigits"),
            values: Vec::from([
                JsString::new("1.0"),
                JsString::new("2.0"),
                JsString::new("3.0"),
            ]),
        },
        DateTimeComponents {
            property: JsString::new("timeZoneName"),
            values: Vec::from([
                JsString::new("short"),
                JsString::new("long"),
                JsString::new("shortOffset"),
                JsString::new("longOffset"),
                JsString::new("shortGeneric"),
                JsString::new("longGeneric"),
            ]),
        },
    ])
}

fn build_dtf_options(
    maybe_date: Option<length::Date>,
    maybe_time: Option<length::Time>,
) -> DateTimeFormatOptions {
    let dtf_bag = match maybe_date {
        Some(date_style) => match maybe_time {
            Some(time_style) => length::Bag::from_date_time_style(date_style, time_style),
            None => length::Bag::from_date_style(date_style),
        },
        None => match maybe_time {
            Some(time_style) => length::Bag::from_time_style(time_style),
            None => length::Bag::empty(),
        },
    };

    DateTimeFormatOptions::Length(dtf_bag)
}

/// Builds a list of `components::Bag` for all possible combinations of dateStyle and timeStyle
/// ("full", "medium", "short", "long", undefined) for specified `locale` and `calendar`
pub(crate) fn build_formats(locale: &JsString, calendar: &JsString) -> Vec<components::Bag> {
    let locale_str = locale.to_string();
    let locale = Locale::from_bytes(locale_str.as_bytes()).expect("Locale parsing failed");
    let provider = InvariantDataProvider;
    let mut formats_vec = Vec::<components::Bag>::new();
    for date_style in [
        Some(length::Date::Full),
        Some(length::Date::Long),
        Some(length::Date::Medium),
        Some(length::Date::Short),
        None,
    ] {
        for time_style in [
            Some(length::Time::Full),
            Some(length::Time::Long),
            Some(length::Time::Medium),
            Some(length::Time::Short),
            None,
        ] {
            let options = build_dtf_options(date_style, time_style);

            if calendar.eq(&JsString::from("buddhist")) {
                let maybe_dtf = datetime::DateTimeFormat::<Buddhist>::try_new(
                    locale.clone(),
                    &provider,
                    &options,
                );
                match maybe_dtf {
                    Ok(dtf) => formats_vec.push(dtf.resolve_components()),
                    Err(_) => continue,
                };
            } else if calendar.eq(&JsString::from("gregory")) {
                let maybe_dtf = datetime::DateTimeFormat::<Gregorian>::try_new(
                    locale.clone(),
                    &provider,
                    &options,
                );
                match maybe_dtf {
                    Ok(dtf) => formats_vec.push(dtf.resolve_components()),
                    Err(_) => continue,
                };
            } else if calendar.eq(&JsString::from("japanese")) {
                let maybe_dtf = datetime::DateTimeFormat::<Japanese>::try_new(
                    locale.clone(),
                    &provider,
                    &options,
                );
                match maybe_dtf {
                    Ok(dtf) => formats_vec.push(dtf.resolve_components()),
                    Err(_) => continue,
                };
            } else {
                continue;
            }
        }
    }

    formats_vec
}

#[derive(Debug)]
pub(crate) struct StylesRecord {
    pub(crate) locale: JsString,
    pub(crate) calendar: JsString,
}

/// The `DateTimeStyleFormat` abstract operation accepts arguments `dateStyle` and `timeStyle`,
/// which are each either undefined, "full", "long", "medium", or "short", at least one of which
/// is not undefined, and styles, which is a record from
/// `%DateTimeFormat%.[[LocaleData]].[[<locale>]].[[styles]].[[<calendar>]]` for some locale and
/// calendar. It returns the appropriate format record for date time formatting based on the
/// parameters.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-date-time-style-format
pub(crate) fn date_time_style_format(
    date_style: &JsValue,
    time_style: &JsValue,
    styles: &StylesRecord,
    context: &mut Context,
) -> Option<components::Bag> {
    if date_style.is_undefined() && time_style.is_undefined() {
        return None;
    }

    let date_style = value_to_date_style(date_style, context);
    let time_style = value_to_time_style(time_style, context);

    let options = build_dtf_options(date_style, time_style);

    let locale_str = styles.locale.to_string();
    let locale = Locale::from_bytes(locale_str.as_bytes()).expect("Locale parsing failed");
    let provider = InvariantDataProvider;

    if styles.calendar.eq(&JsString::from("buddhist")) {
        let maybe_dtf = datetime::DateTimeFormat::<Buddhist>::try_new(locale, &provider, &options);
        match maybe_dtf {
            Ok(dtf) => Some(dtf.resolve_components()),
            Err(_) => None,
        }
    } else if styles.calendar.eq(&JsString::from("gregory")) {
        let maybe_dtf = datetime::DateTimeFormat::<Gregorian>::try_new(locale, &provider, &options);
        match maybe_dtf {
            Ok(dtf) => Some(dtf.resolve_components()),
            Err(_) => None,
        }
    } else if styles.calendar.eq(&JsString::from("japanese")) {
        let maybe_dtf = datetime::DateTimeFormat::<Japanese>::try_new(locale, &provider, &options);
        match maybe_dtf {
            Ok(dtf) => Some(dtf.resolve_components()),
            Err(_) => None,
        }
    } else {
        None
    }
}

/// `BasicFormatMatcher` abstract operation is called with two arguments `options` and `formats`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-basicformatmatcher
pub(crate) fn basic_format_matcher(
    options: &FormatOptionsRecord,
    formats: &[components::Bag],
) -> Option<components::Bag> {
    // 1. Let removalPenalty be 120.
    let removal_penalty = 120;

    // 2. Let additionPenalty be 20.
    let addition_penalty = 20;

    // 3. Let longLessPenalty be 8.
    let long_less_penalty = 8;

    // 4. Let longMorePenalty be 6.
    let long_more_penalty = 6;

    // 5. Let shortLessPenalty be 6.
    let short_less_penalty = 6;

    // 6. Let shortMorePenalty be 3.
    let short_more_penalty = 3;

    // 7. Let offsetPenalty be 1.
    let offset_penalty = 1;

    // 8. Let bestScore be -Infinity.
    let mut best_score = i32::MIN;

    // 9. Let bestFormat be undefined.
    let mut best_format = None;

    // 10. Assert: Type(formats) is List.
    // 11. For each element format of formats, do
    for format in formats {
        // a. Let score be 0.
        let mut score = 0;

        // b. For each property name property shown in Table 6, do
        let date_time_components = build_date_time_components();
        for table_row in date_time_components {
            let property = table_row.property.to_string();
            // i. If options has a field [[<property>]],
            // let optionsProp be options.[[<property>]];
            // else let optionsProp be undefined.
            let options_prop = match options.components.get(&table_row.property) {
                Some(opt) => opt.clone(),
                None => JsValue::undefined(),
            };

            // ii. If format has a field [[<property>]],
            // let formatProp be format.[[<property>]];
            // else let formatProp be undefined.
            let format_prop = get_format_field(format, &property);

            // iii. If optionsProp is undefined and formatProp is not undefined,
            // decrease score by additionPenalty.
            if options_prop.is_undefined() && !format_prop.is_undefined() {
                score -= addition_penalty;

            // iv. Else if optionsProp is not undefined and formatProp is undefined,
            // decrease score by removalPenalty.
            } else if !options_prop.is_undefined() && format_prop.is_undefined() {
                score -= removal_penalty;
            // v. Else if property is "timeZoneName", then
            } else if property.eq("timeZoneName") {
                // 1. If optionsProp is "short" or "shortGeneric", then
                if options_prop.eq(&JsValue::String(JsString::new("short")))
                    || options_prop.eq(&JsValue::String(JsString::new("shortGeneric")))
                {
                    // a. If formatProp is "shortOffset", decrease score by offsetPenalty.
                    // b. Else if formatProp is "longOffset",
                    // decrease score by (offsetPenalty + shortMorePenalty).
                    // c. Else if optionsProp is "short" and formatProp is "long",
                    // decrease score by shortMorePenalty.
                    // d. Else if optionsProp is "shortGeneric" and formatProp is "longGeneric",
                    // decrease score by shortMorePenalty.
                    // e. Else if optionsProp ≠ formatProp, decrease score by removalPenalty.
                    if format_prop.eq(&JsValue::String(JsString::new("shortOffset"))) {
                        // a.
                        score -= offset_penalty;
                    } else if format_prop.eq(&JsValue::String(JsString::new("longOffset"))) {
                        // b.
                        score -= offset_penalty + short_more_penalty;
                    } else if (options_prop.eq(&JsValue::String(JsString::new("short")))
                        && format_prop.eq(&JsValue::String(JsString::new("long"))))
                        || (options_prop.eq(&JsValue::String(JsString::new("shortGeneric")))
                            && format_prop.eq(&JsValue::String(JsString::new("longGeneric"))))
                    {
                        // c & d.
                        score -= short_more_penalty;
                    } else if options_prop.ne(&format_prop) {
                        // e.
                        score -= removal_penalty;
                    }

                // 2. Else if optionsProp is "shortOffset" and formatProp is "longOffset",
                // decrease score by shortMorePenalty.
                } else if options_prop.eq(&JsValue::String(JsString::new("shortOffset")))
                    || format_prop.eq(&JsValue::String(JsString::new("longOffset")))
                {
                    score -= short_more_penalty;

                // 3. Else if optionsProp is "long" or "longGeneric", then
                } else if options_prop.eq(&JsValue::String(JsString::new("long")))
                    || options_prop.eq(&JsValue::String(JsString::new("longGeneric")))
                {
                    // a. If formatProp is "longOffset", decrease score by offsetPenalty.
                    // b. Else if formatProp is "shortOffset",
                    // decrease score by (offsetPenalty + longLessPenalty).
                    // c. Else if optionsProp is "long" and formatProp is "short",
                    // decrease score by longLessPenalty.
                    // d. Else if optionsProp is "longGeneric" and formatProp is "shortGeneric",
                    // decrease score by longLessPenalty.
                    // e. Else if optionsProp ≠ formatProp, decrease score by removalPenalty.
                    if format_prop.eq(&JsValue::String(JsString::new("longOffset"))) {
                        // a.
                        score -= offset_penalty;
                    } else if format_prop.eq(&JsValue::String(JsString::new("shortOffset"))) {
                        // b.
                        score -= offset_penalty + long_less_penalty;
                    } else if (options_prop.eq(&JsValue::String(JsString::new("long")))
                        && format_prop.eq(&JsValue::String(JsString::new("short"))))
                        || (options_prop.eq(&JsValue::String(JsString::new("longGeneric")))
                            && format_prop.eq(&JsValue::String(JsString::new("shortGeneric"))))
                    {
                        // c & d.
                        score -= long_less_penalty;
                    } else if options_prop.ne(&format_prop) {
                        // e.
                        score -= removal_penalty;
                    }

                // 4. Else if optionsProp is "longOffset" and formatProp is "shortOffset",
                // decrease score by longLessPenalty.
                } else if options_prop.eq(&JsValue::String(JsString::new("longOffset")))
                    || format_prop.eq(&JsValue::String(JsString::new("shortOffset")))
                {
                    score -= long_less_penalty;

                // 5. Else if optionsProp ≠ formatProp, decrease score by removalPenalty.
                } else if options_prop.ne(&format_prop) {
                    score -= removal_penalty;
                }

            // vi. Else if optionsProp ≠ formatProp, then
            } else if options_prop.ne(&format_prop) {
                // 1. If property is "fractionalSecondDigits", then
                //     a. Let values be « 1𝔽, 2𝔽, 3𝔽 ».
                // 2. Else,
                //     a. Let values be « "2-digit", "numeric", "narrow", "short", "long" ».
                let values = if property.eq("fractionalSecondDigits") {
                    vec![JsValue::new(1.0), JsValue::new(2.0), JsValue::new(3.0)]
                } else {
                    vec![
                        JsValue::String(JsString::new("2-digit")),
                        JsValue::String(JsString::new("numeric")),
                        JsValue::String(JsString::new("narrow")),
                        JsValue::String(JsString::new("short")),
                        JsValue::String(JsString::new("long")),
                    ]
                };

                // 3. Let optionsPropIndex be the index of optionsProp within values.
                let options_prop_index = values
                    .iter()
                    .position(|val| val.eq(&options_prop))
                    .expect("Option not found") as i32;

                // 4. Let formatPropIndex be the index of formatProp within values.
                let format_prop_index = values
                    .iter()
                    .position(|val| val.eq(&format_prop))
                    .expect("Format not found") as i32;

                // 5. Let delta be max(min(formatPropIndex - optionsPropIndex, 2), -2).
                let delta = max(min(format_prop_index - options_prop_index, 2), -2);

                // 6. If delta = 2, decrease score by longMorePenalty.
                // 7. Else if delta = 1, decrease score by shortMorePenalty.
                // 8. Else if delta = -1, decrease score by shortLessPenalty.
                // 9. Else if delta = -2, decrease score by longLessPenalty.
                if delta == 2 {
                    score -= long_more_penalty;
                } else if delta == 1 {
                    score -= short_more_penalty;
                } else if delta == -1 {
                    score -= short_less_penalty;
                } else if delta == -2 {
                    score -= long_less_penalty;
                }
            }
        }

        // c. If score > bestScore, then
        if score > best_score {
            // i. Let bestScore be score.
            best_score = score;

            // ii. Let bestFormat be format.
            best_format = Some(*format);
        }
    }

    // 12. Return bestFormat.
    best_format
}

/// When the `BestFitFormatMatcher` abstract operation is called with two arguments `options` and
/// `formats`, it performs implementation dependent steps, which should return a set of component
/// representations that a typical user of the selected locale would perceive as at least as good
/// as the one returned by `BasicFormatMatcher`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-bestfitformatmatcher
fn best_fit_format_matcher(
    options: &FormatOptionsRecord,
    formats: &[components::Bag],
) -> Option<components::Bag> {
    basic_format_matcher(options, formats)
}

/// The abstract operation `InitializeDateTimeFormat` accepts the arguments `dateTimeFormat` (which
/// must be an object), `locales`, and `options`. It initializes `dateTimeFormat` as a
/// `DateTimeFormat` object.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-initializedatetimeformat
fn initialize_date_time_format(
    date_time_format: &JsObject,
    locales: &JsValue,
    options: &JsValue,
    context: &mut Context,
) -> JsResult<JsObject> {
    // 1. Let requestedLocales be ? CanonicalizeLocaleList(locales).
    let locales_arr = if locales.is_undefined() {
        vec![JsValue::undefined()]
    } else if locales.is_string() {
        vec![locales.clone()]
    } else {
        let locales_obj = locales
            .to_object(context)
            .unwrap_or_else(|_| JsObject::empty());
        let locales_len = locales_obj.length_of_array_like(context).unwrap_or(0);
        let mut locales_acc = Vec::<JsValue>::new();
        for index in 0..locales_len as u32 {
            let maybe_locale = locales_obj
                .get(index, context)
                .unwrap_or_else(|_| JsValue::undefined());
            locales_acc.push(maybe_locale);
        }
        locales_acc
    };
    let requested_locales = canonicalize_locale_list(&locales_arr, context)?;
    let requested_locales = requested_locales
        .iter()
        .map(|locale| JsString::new(locale.to_string()))
        .collect::<Vec<JsString>>();

    // 2. Set options to ? ToDateTimeOptions(options, "any", "date").
    let options =
        to_date_time_options(options, &DateTimeReqs::AnyAll, &DateTimeReqs::Date, context)?;

    // 3. Let opt be a new Record.
    let mut opt = DateTimeFormatRecord {
        locale_matcher: JsString::empty(),
        properties: FxHashMap::default(),
    };

    // 4. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
    let matcher_values = vec![JsString::new("lookup"), JsString::new("best fit")];
    let matcher = get_option(
        &options,
        "localeMatcher",
        &GetOptionType::String,
        &matcher_values,
        &JsValue::String(JsString::new("best fit")),
        context,
    )?;

    // 5. Set opt.[[localeMatcher]] to matcher.
    opt.locale_matcher = matcher
        .to_string(context)
        .unwrap_or_else(|_| JsString::empty());

    // 6. Let calendar be ? GetOption(options, "calendar", "string", undefined, undefined).
    let calendar = get_option(
        &options,
        "calendar",
        &GetOptionType::String,
        &Vec::<JsString>::new(),
        &JsValue::undefined(),
        context,
    )?;

    // 7. If calendar is not undefined, then
    if !calendar.is_undefined() {
        // a. If calendar does not match the Unicode Locale Identifier type nonterminal,
        // throw a RangeError exception.
        let calendar_str = calendar
            .to_string(context)
            .unwrap_or_else(|_| JsString::empty());
        if is_terminal(&calendar_str) {
            return context.throw_range_error("calendar must be nonterminal");
        }
    }

    // 8. Set opt.[[ca]] to calendar.
    opt.properties.insert(JsString::new("ca"), calendar);

    // 9. Let numberingSystem be ? GetOption(options, "numberingSystem", "string", undefined, undefined).
    let numbering_system = get_option(
        &options,
        "numberingSystem",
        &GetOptionType::String,
        &Vec::<JsString>::new(),
        &JsValue::undefined(),
        context,
    )?;

    // 10. If numberingSystem is not undefined, then
    if !numbering_system.is_undefined() {
        // a. If numberingSystem does not match the Unicode Locale Identifier type nonterminal,
        // throw a RangeError exception.
        let numbering_system_str = numbering_system
            .to_string(context)
            .unwrap_or_else(|_| JsString::empty());
        if is_terminal(&numbering_system_str) {
            return context.throw_range_error("numberingSystem must be nonterminal");
        }
    }

    // 11. Set opt.[[nu]] to numberingSystem.
    opt.properties.insert(JsString::new("nu"), numbering_system);

    // 12. Let hour12 be ? GetOption(options, "hour12", "boolean", undefined, undefined).
    let hour_12 = get_option(
        &options,
        "hour12",
        &GetOptionType::Boolean,
        &Vec::<JsString>::new(),
        &JsValue::undefined(),
        context,
    )?;

    // 13. Let hourCycle be ? GetOption(options, "hourCycle", "string", « "h11", "h12", "h23", "h24" », undefined).
    let hour_cycle_values = vec![
        JsString::new("h11"),
        JsString::new("h12"),
        JsString::new("h23"),
        JsString::new("h24"),
    ];
    let mut hour_cycle = get_option(
        &options,
        "hourCycle",
        &GetOptionType::String,
        &hour_cycle_values,
        &JsValue::undefined(),
        context,
    )?;

    // 14. If hour12 is not undefined, then
    if !hour_12.is_undefined() {
        // a. Set hourCycle to null.
        hour_cycle = JsValue::null();
    }

    // 15. Set opt.[[hc]] to hourCycle.
    opt.properties.insert(JsString::new("hc"), hour_cycle);

    // 16. Let localeData be %DateTimeFormat%.[[LocaleData]].
    let available_locales = build_available_locales(context);
    let locale_data = build_locale_data(&available_locales);
    let relevant_ext_keys = build_relevant_ext_keys();

    // 17. Let r be ResolveLocale(%DateTimeFormat%.[[AvailableLocales]], requestedLocales, opt,
    // %DateTimeFormat%.[[RelevantExtensionKeys]], localeData).
    let r = resolve_locale(
        &available_locales,
        &requested_locales,
        &opt,
        &relevant_ext_keys,
        &locale_data,
        context,
    );

    let mut date_time_fmt_borrow = date_time_format.borrow_mut();
    let date_time_fmt = date_time_fmt_borrow
        .as_date_time_format_mut()
        .expect("Cast to DateTimeFormat failed");

    // 18. Set dateTimeFormat.[[Locale]] to r.[[locale]].
    date_time_fmt.locale = r.locale.clone();

    // 19. Let resolvedCalendar be r.[[ca]].
    let resolved_calendar = r
        .properties
        .get(&JsString::new("ca"))
        .expect("Failed to resolve calendar");

    // 20. Set dateTimeFormat.[[Calendar]] to resolvedCalendar.
    date_time_fmt.calendar = resolved_calendar.clone();

    // 21. Set dateTimeFormat.[[NumberingSystem]] to r.[[nu]].
    let resolved_nu = r
        .properties
        .get(&JsString::new("nu"))
        .expect("Failed to resolve numbering system");
    date_time_fmt.numbering_system = resolved_nu.clone();

    // 22. Let dataLocale be r.[[dataLocale]].
    let data_locale = r.data_locale;

    // 23. Let dataLocaleData be localeData.[[<dataLocale>]].
    let data_locale_data = locale_data
        .get(&data_locale)
        .expect("Failed to resolve data locale");

    // 24. Let hcDefault be dataLocaleData.[[hourCycle]].
    let hc_default = data_locale_data
        .get(&JsString::new("hourCycle"))
        .expect("Failed to resolve hour cycle");

    // 25. If hour12 is true, then
    //      a. If hcDefault is "h11" or "h23", let hc be "h11". Otherwise, let hc be "h12".
    // 26. Else if hour12 is false, then
    //      a. If hcDefault is "h11" or "h23", let hc be "h23". Otherwise, let hc be "h24".
    // 27. Else,
    //      a. Assert: hour12 is undefined.
    //      b. Let hc be r.[[hc]].
    //      c. If hc is null, set hc to hcDefault.
    let hc = if hour_12.is_boolean() {
        if hour_12.to_boolean() {
            if hc_default[0].eq(&JsString::new("h11")) || hc_default[0].eq(&JsString::new("h23")) {
                JsString::new("h11")
            } else {
                JsString::new("h12")
            }
        } else if hc_default[0].eq(&JsString::new("h11")) || hc_default[0].eq(&JsString::new("h23"))
        {
            JsString::new("h23")
        } else {
            JsString::new("h24")
        }
    } else {
        let hc_prop = r
            .properties
            .get(&JsString::new("hc"))
            .expect("Failed to resolve hc");
        if hc_prop.is_null() {
            hc_default[0].clone()
        } else {
            hc_prop.to_string(context)?
        }
    };

    // 28. Set dateTimeFormat.[[HourCycle]] to hc.
    date_time_fmt.hour_cycle = JsValue::String(hc.clone());

    // 29. Let timeZone be ? Get(options, "timeZone").
    let time_zone = options.get("timeZone", context)?;

    // 30. If timeZone is undefined, then
    //     a. Set timeZone to ! DefaultTimeZone().
    // 31. Else,
    //     a. Set timeZone to ? ToString(timeZone).
    //     b. If the result of ! IsValidTimeZoneName(timeZone) is false, then
    //         i. Throw a RangeError exception.
    //     c. Set timeZone to ! CanonicalizeTimeZoneName(timeZone).
    let time_zone_str = if time_zone.is_undefined() {
        default_time_zone()
    } else {
        let time_zone = time_zone.to_string(context)?;
        if !is_valid_time_zone_name(&time_zone) {
            return context.throw_range_error("Invalid time zone name");
        }

        canonicalize_time_zone_name(&time_zone)
    };

    // 32. Set dateTimeFormat.[[TimeZone]] to timeZone.
    date_time_fmt.time_zone = time_zone_str;

    // 33. Let formatOptions be a new Record.
    let mut format_options = FormatOptionsRecord {
        date_time_format_opts: DateTimeFormatOptions::default(),
        components: FxHashMap::default(),
    };

    // 34. Set formatOptions.[[hourCycle]] to hc.
    // TODO is it actually used anywhere?
    let prefs = preferences::Bag::from_hour_cycle(string_to_hour_cycle(&hc));
    let mut hc_len_bag = length::Bag::empty();
    hc_len_bag.preferences = Some(prefs);
    format_options.date_time_format_opts = DateTimeFormatOptions::Length(hc_len_bag);

    // 35. Let hasExplicitFormatComponents be false.
    let mut has_explicit_format_components = false;

    // 36. For each row of Table 6, except the header row, in table order, do
    let date_time_components = build_date_time_components();

    for table_row in date_time_components {
        // a. Let prop be the name given in the Property column of the row.
        let prop = table_row.property;

        // b. If prop is "fractionalSecondDigits", then
        //      i. Let value be ? GetNumberOption(options, "fractionalSecondDigits", 1, 3,
        //      undefined).
        // c. Else,
        //      i. Let values be a List whose elements are the strings given in the Values
        //      column of the row.
        //      ii. Let value be ? GetOption(options, prop, "string", values, undefined).
        let value = if prop.eq("fractionalSecondDigits") {
            let number_opt =
                get_number_option(&options, "fractionalSecondDigits", 1.0, 3.0, None, context)?;
            match number_opt {
                Some(num) => JsValue::new(num),
                None => JsValue::undefined(),
            }
        } else {
            let values = table_row.values;
            get_option(
                &options,
                &prop,
                &GetOptionType::String,
                &values,
                &JsValue::undefined(),
                context,
            )?
        };

        // d. Set formatOptions.[[<prop>]] to value.
        // e. If value is not undefined, then
        if !value.is_undefined() {
            // i. Set hasExplicitFormatComponents to true.
            has_explicit_format_components = true;
        }
        format_options.components.insert(prop, value);
    }

    // 37. Let matcher be ? GetOption(options, "formatMatcher", "string", « "basic", "best fit" »,
    // "best fit").
    let matcher_values = vec![JsString::new("basic"), JsString::new("best fit")];
    let matcher = get_option(
        &options,
        "formatMatcher",
        &GetOptionType::String,
        &matcher_values,
        &JsValue::String(JsString::new("best fit")),
        context,
    )?;

    // 38. Let dateStyle be ? GetOption(options, "dateStyle", "string",
    // « "full", "long", "medium", "short" », undefined).
    let date_style_values = vec![
        JsString::new("full"),
        JsString::new("long"),
        JsString::new("medium"),
        JsString::new("short"),
    ];
    let date_style = get_option(
        &options,
        "dateStyle",
        &GetOptionType::String,
        &date_style_values,
        &JsValue::undefined(),
        context,
    )?;

    // 39. Set dateTimeFormat.[[DateStyle]] to dateStyle.
    date_time_fmt.date_style = date_style.clone();

    // 40. Let timeStyle be ? GetOption(options, "timeStyle", "string",
    // « "full", "long", "medium", "short" », undefined).
    let time_style_values = vec![
        JsString::new("full"),
        JsString::new("long"),
        JsString::new("medium"),
        JsString::new("short"),
    ];
    let time_style = get_option(
        &options,
        "timeStyle",
        &GetOptionType::String,
        &time_style_values,
        &JsValue::undefined(),
        context,
    )?;

    // 41. Set dateTimeFormat.[[TimeStyle]] to timeStyle.
    date_time_fmt.time_style = time_style.clone();

    // 42. If dateStyle is not undefined or timeStyle is not undefined, then
    let best_format = if !date_style.is_undefined() || !time_style.is_undefined() {
        // a. If hasExplicitFormatComponents is true, then
        if has_explicit_format_components {
            // i. Throw a TypeError exception.
            return context.throw_type_error(
                "dateStyle or timeStyle is defined, while components have explicit format",
            );
        }

        // b. Let styles be dataLocaleData.[[styles]].[[<resolvedCalendar>]].
        let styles = StylesRecord {
            locale: r.locale.clone(),
            calendar: resolved_calendar
                .to_string(context)
                .unwrap_or_else(|_| JsString::empty()),
        };
        // c. Let bestFormat be DateTimeStyleFormat(dateStyle, timeStyle, styles).
        date_time_style_format(&date_style, &time_style, &styles, context)
            .expect("DateTimeStyleFormat failed")
    // 43. Else,
    } else {
        // a. Let formats be dataLocaleData.[[formats]].[[<resolvedCalendar>]].
        let formats = build_formats(
            &r.locale,
            &resolved_calendar
                .to_string(context)
                .unwrap_or_else(|_| JsString::empty()),
        );

        // b. If matcher is "basic", then
        if matcher.eq(&JsValue::String(JsString::new("basic"))) {
            // i. Let bestFormat be BasicFormatMatcher(formatOptions, formats).
            basic_format_matcher(&format_options, &formats).expect("Failed to get basic format")
        // c. Else,
        } else {
            // i. Let bestFormat be BestFitFormatMatcher(formatOptions, formats).
            best_fit_format_matcher(&format_options, &formats)
                .expect("Failed to get best fit format")
        }
    };

    // 44. For each row in Table 6, except the header row, in table order, do
    //      a. Let prop be the name given in the Property column of the row.
    //      b. If bestFormat has a field [[<prop>]], then
    date_time_fmt.weekday = get_format_field(&best_format, "weekday");
    date_time_fmt.era = get_format_field(&best_format, "era");
    date_time_fmt.year = get_format_field(&best_format, "year");
    date_time_fmt.month = get_format_field(&best_format, "month");
    date_time_fmt.day = get_format_field(&best_format, "day");
    date_time_fmt.hour = get_format_field(&best_format, "hour");
    date_time_fmt.minute = get_format_field(&best_format, "minute");
    date_time_fmt.second = get_format_field(&best_format, "second");
    date_time_fmt.time_zone_name = get_format_field(&best_format, "timeZoneName");

    // 45. If dateTimeFormat.[[Hour]] is undefined, then
    if date_time_fmt.hour.is_undefined() {
        // a. Set dateTimeFormat.[[HourCycle]] to undefined.
        date_time_fmt.hour_cycle = JsValue::undefined();
    }

    // 46. If dateTimeformat.[[HourCycle]] is "h11" or "h12", then
    //      a. Let pattern be bestFormat.[[pattern12]].
    //      b. Let rangePatterns be bestFormat.[[rangePatterns12]].
    // 47. Else,
    //      a. Let pattern be bestFormat.[[pattern]].
    //      b. Let rangePatterns be bestFormat.[[rangePatterns]].
    // 48. Set dateTimeFormat.[[Pattern]] to pattern.
    // 49. Set dateTimeFormat.[[RangePatterns]] to rangePatterns.

    // FIXME icu::options::components::Bag does not provide patterns

    // 50. Return dateTimeFormat.
    Ok(date_time_format.clone())
}
