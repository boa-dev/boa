//! This module implements the global `Intl.DateTimeFormat` object.
//!
//! `Intl.DateTimeFormat` is a built-in object that has properties and methods for date and time i18n.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma402/#datetimeformat-objects

use crate::{
    builtins::BuiltIn,
    context::intrinsics::StandardConstructors,
    object::internal_methods::get_prototype_from_constructor,
    object::{JsObject, ObjectData, ObjectInitializer},
    property::Attribute,
    symbol::WellKnownSymbols,
    Context, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use tap::{Conv, Pipe};

/// JavaScript `Intl.DateTimeFormat` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct DateTimeFormat {
    initialized_date_time_format: bool,
    locale: String,
    calendar: String,
    numbering_system: String,
    time_zone: String,
    weekday: String,
    era: String,
    year: String,
    month: String,
    day: String,
    day_period: String,
    hour: String,
    minute: String,
    second: String,
    fractional_second_digits: String,
    time_zone_name: String,
    hour_cycle: String,
    pattern: String,
    bound_format: String,
}

impl BuiltIn for DateTimeFormat {
    const NAME: &'static str = "Intl.DateTimeFormat";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let string_tag = WellKnownSymbols::to_string_tag();
        ObjectInitializer::new(context)
            .property(
                string_tag,
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build()
            .conv::<JsValue>()
            .pipe(Some)
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
        // 2. Let dateTimeFormat be ? OrdinaryCreateFromConstructor(newTarget, "%DateTimeFormat.prototype%", « [[InitializedDateTimeFormat]], [[Locale]], [[Calendar]], [[NumberingSystem]], [[TimeZone]], [[Weekday]], [[Era]], [[Year]], [[Month]], [[Day]], [[DayPeriod]], [[Hour]], [[Minute]], [[Second]], [[FractionalSecondDigits]], [[TimeZoneName]], [[HourCycle]], [[Pattern]], [[BoundFormat]] »).
        let date_time_format = JsObject::from_proto_and_data(
            prototype,
            ObjectData::date_time_format(Box::new(Self {
                initialized_date_time_format: true,
                locale: "en-US".to_string(),
                calendar: "gregory".to_string(),
                numbering_system: "arab".to_string(),
                time_zone: "UTC".to_string(),
                weekday: "narrow".to_string(),
                era: "narrow".to_string(),
                year: "numeric".to_string(),
                month: "narrow".to_string(),
                day: "numeric".to_string(),
                day_period: "narrow".to_string(),
                hour: "numeric".to_string(),
                minute: "numeric".to_string(),
                second: "numeric".to_string(),
                fractional_second_digits: "".to_string(),
                time_zone_name: "".to_string(),
                hour_cycle: "h24".to_string(),
                pattern: "{hour}:{minute}".to_string(),
                bound_format: "undefined".to_string(),
            })),
        );
        Ok(JsValue::Object(date_time_format))
    }
}
