//! This module implements the global `Intl.DateTimeFormat` object.
//!
//! `Intl.DateTimeFormat` is a built-in object that has properties and methods for date and time i18n.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma402/#datetimeformat-objects

use crate::{
    context::intrinsics::StandardConstructors,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, JsFunction, JsObject,
        ObjectData,
    },
    Context, JsResult, JsString, JsValue,
};

use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

#[cfg(test)]
use rustc_hash::FxHashMap;

/// JavaScript `Intl.DateTimeFormat` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct DateTimeFormat {
    initialized_date_time_format: bool,
    locale: JsString,
    calendar: JsString,
    numbering_system: JsString,
    time_zone: JsString,
    weekday: JsString,
    era: JsString,
    year: JsString,
    month: JsString,
    day: JsString,
    day_period: JsString,
    hour: JsString,
    minute: JsString,
    second: JsString,
    fractional_second_digits: JsString,
    time_zone_name: JsString,
    hour_cycle: JsString,
    pattern: JsString,
    bound_format: JsString,
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
        _args: &[JsValue],
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
                calendar: JsString::from("gregory"),
                numbering_system: JsString::from("arab"),
                time_zone: JsString::from("UTC"),
                weekday: JsString::from("narrow"),
                era: JsString::from("narrow"),
                year: JsString::from("numeric"),
                month: JsString::from("narrow"),
                day: JsString::from("numeric"),
                day_period: JsString::from("narrow"),
                hour: JsString::from("numeric"),
                minute: JsString::from("numeric"),
                second: JsString::from("numeric"),
                fractional_second_digits: JsString::from(""),
                time_zone_name: JsString::from(""),
                hour_cycle: JsString::from("h24"),
                pattern: JsString::from("{hour}:{minute}"),
                bound_format: JsString::from("undefined"),
            })),
        );

        // TODO 3. Perform ? InitializeDateTimeFormat(dateTimeFormat, locales, options).
        // TODO 4. If the implementation supports the normative optional constructor mode of 4.3 Note 1, then
        // TODO a. Let this be the this value.
        // TODO b. Return ? ChainDateTimeFormat(dateTimeFormat, NewTarget, this).

        // 5. Return dateTimeFormat.
        Ok(date_time_format.into())
    }
}

/// The abstract operation `toDateTimeOptions` is called with arguments `options`, `required` and
/// `defaults`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-todatetimeoptions
#[cfg(test)]
pub(crate) fn to_date_time_options(
    options: &JsValue,
    required: &str,
    defaults: &str,
    context: &mut Context,
) -> JsResult<JsObject> {
    // 1. If options is undefined, let options be null;
    // otherwise let options be ? ToObject(options).
    let maybe_options = if options.is_undefined() {
        Ok(JsObject::empty())
    } else {
        options.to_object(context)
    };
    let options = maybe_options.unwrap_or_else(|_| JsObject::empty());

    // 2. Let options be ! OrdinaryObjectCreate(options).
    let options = JsObject::from_proto_and_data(options, ObjectData::ordinary());

    // 3. Let needDefaults be true.
    let mut need_defaults = true;

    // 4. If required is "date" or "any", then
    if required.eq("date") || required.eq("any") {
        // a. For each property name prop of « "weekday", "year", "month", "day" », do
        let property_names = vec!["weekday", "year", "month", "day"];
        // i. Let value be ? Get(options, prop).
        // ii. If value is not undefined, let needDefaults be false.
        need_defaults = property_names.iter().all(|prop_name| {
            options
                .get(*prop_name, context)
                .unwrap_or_else(|_| JsValue::undefined())
                .is_undefined()
        });
    }

    // 5. If required is "time" or "any", then
    if required.eq("time") || required.eq("any") {
        // a. For each property name prop of « "dayPeriod", "hour", "minute", "second",
        // "fractionalSecondDigits" », do
        let property_names = vec![
            "dayPeriod",
            "hour",
            "minute",
            "second",
            "fractionalSecondDigits",
        ];
        // i. Let value be ? Get(options, prop).
        // ii. If value is not undefined, let needDefaults be false.
        need_defaults = property_names.iter().all(|prop_name| {
            options
                .get(*prop_name, context)
                .unwrap_or_else(|_| JsValue::undefined())
                .is_undefined()
        });
    }

    // 6. Let dateStyle be ? Get(options, "dateStyle").
    let date_style = options
        .get("dateStyle", context)
        .unwrap_or_else(|_| JsValue::undefined());

    // 7. Let timeStyle be ? Get(options, "timeStyle").
    let time_style = options
        .get("timeStyle", context)
        .unwrap_or_else(|_| JsValue::undefined());

    // 8. If dateStyle is not undefined or timeStyle is not undefined, let needDefaults be false.
    if !date_style.is_undefined() || !time_style.is_undefined() {
        need_defaults = false;
    }

    // 9. If required is "date" and timeStyle is not undefined, then
    if required.eq("date") && !time_style.is_undefined() {
        // a. Throw a TypeError exception.
        return context.throw_type_error("'date' is required, but timeStyle was defined");
    }

    // 10. If required is "time" and dateStyle is not undefined, then
    if required.eq("time") && !date_style.is_undefined() {
        // a. Throw a TypeError exception.
        return context.throw_type_error("'time' is required, but dateStyle was defined");
    }

    // 11. If needDefaults is true and defaults is either "date" or "all", then
    if need_defaults && (defaults.eq("date") || defaults.eq("all")) {
        // a. For each property name prop of « "year", "month", "day" », do
        let property_names = vec!["year", "month", "day"];
        // i. Perform ? CreateDataPropertyOrThrow(options, prop, "numeric").
        for prop_name in property_names {
            options
                .create_data_property_or_throw(prop_name, "numeric", context)
                .expect("CreateDataPropertyOrThrow must not fail");
        }
    }

    // 12. If needDefaults is true and defaults is either "time" or "all", then
    if need_defaults && (defaults.eq("time") || defaults.eq("all")) {
        // a. For each property name prop of « "hour", "minute", "second" », do
        let property_names = vec!["hour", "minute", "second"];
        // i. Perform ? CreateDataPropertyOrThrow(options, prop, "numeric").
        for prop_name in property_names {
            options
                .create_data_property_or_throw(prop_name, "numeric", context)
                .expect("CreateDataPropertyOrThrow must not fail");
        }
    }

    // 13. Return options.
    Ok(options)
}

/// `DateTimeRangeFormat` type contains `rangePatterns` record represented as an object and may or
/// may not contain `rangePatterns12`, thus this field is declared optional.
#[cfg(test)]
pub(crate) struct DateTimeRangeFormat {
    pub(crate) range_patterns: JsObject,
    pub(crate) range_patterns12: Option<JsObject>,
}

/// `StylesRecord` type hash maps for `TimeFormat`, `DateFormat`, `DateTimeFormat` and
/// `DateTimeRangeFormat`. The key of these maps should be one of the following:
/// "full", "long", "medium" or "short". Map values contain formatting patterns like
/// "{year}-{month}-{day}"
#[cfg(test)]
pub(crate) struct StylesRecord {
    pub(crate) time_format: FxHashMap<JsString, JsValue>,
    pub(crate) date_format: FxHashMap<JsString, JsValue>,
    pub(crate) date_time_format: FxHashMap<JsString, JsValue>,
    pub(crate) date_time_range_format:
        FxHashMap<JsString, FxHashMap<JsString, DateTimeRangeFormat>>,
}

/// The `DateTimeStyleFormat` abstract operation accepts arguments `dateStyle` and `timeStyle`,
/// which are each either undefined, "full", "long", "medium", or "short", at least one of which
/// is not undefined, and `styles`, which is a record from
/// %`DateTimeFormat`%.[[`LocaleData`]].[[<locale>]].[[styles]].[[<calendar>]] for some locale `locale`
/// and calendar `calendar`. It returns the appropriate format record for date time formatting
/// based on the parameters.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-date-time-style-format
#[cfg(test)]
pub(crate) fn date_time_style_format(
    date_style: &JsValue,
    time_style: &JsValue,
    styles: &StylesRecord,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. If timeStyle is not undefined, then
    let time_format = if time_style.is_undefined() {
        JsValue::undefined()
    } else {
        // a. Assert: timeStyle is one of "full", "long", "medium", or "short".
        let available_time_styles = vec![
            JsValue::String(JsString::new("full")),
            JsValue::String(JsString::new("long")),
            JsValue::String(JsString::new("medium")),
            JsValue::String(JsString::new("short")),
        ];

        if !available_time_styles
            .iter()
            .any(|style| style.eq(time_style))
        {
            return context.throw_type_error("DateTimeStyleFormat: unsupported time style");
        }

        // b. Let timeFormat be styles.[[TimeFormat]].[[<timeStyle>]].
        let time_style_str = time_style
            .to_string(context)
            .unwrap_or_else(|_| JsString::empty());
        let time_fmt = styles
            .time_format
            .get(&time_style_str)
            .expect("Failed to get timeStyle from TimeFormat");
        time_fmt.clone()
    };

    // 2. If dateStyle is not undefined, then
    let date_format = if date_style.is_undefined() {
        JsValue::undefined()
    } else {
        // a. Assert: dateStyle is one of "full", "long", "medium", or "short".
        let available_date_styles = vec![
            JsValue::String(JsString::new("full")),
            JsValue::String(JsString::new("long")),
            JsValue::String(JsString::new("medium")),
            JsValue::String(JsString::new("short")),
        ];

        if !available_date_styles
            .iter()
            .any(|style| style.eq(date_style))
        {
            return context.throw_type_error("DateTimeStyleFormat: unsupported date style");
        }

        // b. Let dateFormat be styles.[[DateFormat]].[[<dateStyle>]].
        let date_style_str = date_style
            .to_string(context)
            .unwrap_or_else(|_| JsString::empty());
        let date_fmt = styles
            .date_format
            .get(&date_style_str)
            .expect("Failed to get dateStyle from DateFormat");
        date_fmt.clone()
    };

    // 3. If dateStyle is not undefined and timeStyle is not undefined, then
    if !date_style.is_undefined() && !time_style.is_undefined() {
        // a. Let format be a new Record.
        let format = JsObject::empty();

        // b. Add to format all fields from dateFormat except [[pattern]] and [[rangePatterns]].
        let date_format_obj = date_format
            .to_object(context)
            .expect("Failed to cast dateFormat to object");
        let entries_list = date_format_obj.enumerable_own_property_names(
            crate::property::PropertyNameKind::KeyAndValue,
            context,
        )?;

        for entry in entries_list {
            let entry_obj = entry.to_object(context)?;
            let entry_key = entry_obj.get(0, context)?;
            let entry_key_str = entry_key.to_string(context)?;
            if entry_key_str.ne(&JsString::new("pattern"))
                && entry_key_str.ne(&JsString::new("rangePatterns"))
            {
                let entry_val = entry_obj.get(1, context)?;
                format.set(entry_key_str, entry_val, true, context)?;
            }
        }

        // c. Add to format all fields from timeFormat except
        // [[pattern]], [[rangePatterns]], [[pattern12]], and [[rangePatterns12]], if present.
        let time_format_obj = time_format
            .to_object(context)
            .expect("Failed to cast timeFormat to object");
        let entries_list = time_format_obj.enumerable_own_property_names(
            crate::property::PropertyNameKind::KeyAndValue,
            context,
        )?;
        for entry in entries_list {
            let entry_obj = entry.to_object(context)?;
            let entry_key = entry_obj.get(0, context)?;
            let entry_key_str = entry_key.to_string(context)?;
            if entry_key_str.ne(&JsString::new("pattern"))
                && entry_key_str.ne(&JsString::new("rangePatterns"))
                && entry_key_str.ne(&JsString::new("pattern12"))
                && entry_key_str.ne(&JsString::new("rangePatterns12"))
            {
                let entry_val = entry_obj.get(1, context)?;
                format.set(entry_key_str, entry_val, true, context)?;
            }
        }

        // d. Let connector be styles.[[DateTimeFormat]].[[<dateStyle>]].
        let date_style_str = date_style
            .to_string(context)
            .unwrap_or_else(|_| JsString::empty());
        let connector = styles
            .date_time_format
            .get(&date_style_str)
            .expect("Failed to get connector");
        let connector_str = connector
            .to_string(context)
            .expect("Failed to cast connector to string");

        // e. Let pattern be the string connector with the substring "{0}" replaced with
        // timeFormat.[[pattern]] and the substring "{1}" replaced with dateFormat.[[pattern]].
        let time_format_pattern = time_format_obj
            .get("pattern", context)
            .expect("Failed to get pattern");
        let time_format_pattern = time_format_pattern
            .to_string(context)
            .expect("Failed to cast pattern to string");
        let time_format_pattern = time_format_pattern.to_string();

        let date_format_pattern = date_format_obj
            .get("pattern", context)
            .expect("Failed to get pattern");
        let date_format_pattern = date_format_pattern
            .to_string(context)
            .expect("Failed to cast pattern to string");
        let date_format_pattern = date_format_pattern.to_string();

        let pattern = connector_str.replace("{0}", &time_format_pattern);
        let pattern = pattern.replace("{1}", &date_format_pattern);

        // f. Set format.[[pattern]] to pattern.
        format.set(
            "pattern",
            JsValue::String(JsString::new(pattern)),
            true,
            context,
        )?;

        // g. If timeFormat has a [[pattern12]] field, then
        let maybe_pattern12 = time_format_obj
            .get("pattern12", context)
            .unwrap_or_else(|_| JsValue::undefined());
        if !maybe_pattern12.is_undefined() {
            // i. Let pattern12 be the string connector with the substring "{0}"
            // replaced with timeFormat.[[pattern12]] and the substring "{1}" replaced with
            // dateFormat.[[pattern]].
            let pattern12_str = maybe_pattern12
                .to_string(context)
                .unwrap_or_else(|_| JsString::empty());
            let pattern12_str = pattern12_str.to_string();

            let date_format_pattern = date_format_obj
                .get("pattern", context)
                .expect("Failed to get pattern");
            let date_format_pattern = date_format_pattern
                .to_string(context)
                .expect("Failed to cast pattern to string");
            let date_format_pattern = date_format_pattern.to_string();

            let pattern12 = connector_str.replace("{0}", &pattern12_str);
            let pattern12 = pattern12.replace("{1}", &date_format_pattern);

            // ii. Set format.[[pattern12]] to pattern12.
            format.set(
                "pattern12",
                JsValue::String(JsString::new(pattern12)),
                true,
                context,
            )?;
        }

        // h. Let dateTimeRangeFormat be styles.[[DateTimeRangeFormat]].[[<dateStyle>]].[[<timeStyle>]].
        let date_style_str = date_style
            .to_string(context)
            .unwrap_or_else(|_| JsString::empty());
        let time_style_str = time_style
            .to_string(context)
            .unwrap_or_else(|_| JsString::empty());
        let dtr_fmt_date_style = styles
            .date_time_range_format
            .get(&date_style_str)
            .expect("Failed to get dateStyle");
        let date_time_range_format = dtr_fmt_date_style
            .get(&time_style_str)
            .expect("Failed to get timeStyle");

        // i. Set format.[[rangePatterns]] to dateTimeRangeFormat.[[rangePatterns]].
        format.set(
            "rangePatterns",
            date_time_range_format.range_patterns.clone(),
            true,
            context,
        )?;

        // j. If dateTimeRangeFormat has a [[rangePatterns12]] field, then
        if let Some(range_patterns12) = &date_time_range_format.range_patterns12 {
            // i. Set format.[[rangePatterns12]] to dateTimeRangeFormat.[[rangePatterns12]].
            format.set("rangePatterns12", range_patterns12.clone(), true, context)?;
        }

        // k. Return format.
        return Ok(JsValue::Object(format));
    }

    // 4. If timeStyle is not undefined, then
    if !time_style.is_undefined() {
        // a. Return timeFormat.
        return Ok(time_format);
    }

    // 5. Assert: dateStyle is not undefined.
    if date_style.is_undefined() {
        return context
            .throw_type_error("DateTimeStyleFormat: date style must be defined at this point.");
    }

    Ok(date_format)
}
