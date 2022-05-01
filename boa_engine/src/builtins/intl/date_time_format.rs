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
