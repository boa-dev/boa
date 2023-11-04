//! Boa's implementation of the ECMAScript `Temporal.PlainDate` builtin object.
#![allow(dead_code, unused_variables)]

use crate::{
    builtins::{
        options::{get_option, get_options_object},
        temporal::options::TemporalUnit,
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, ObjectData},
    property::Attribute,
    realm::Realm,
    string::common::StaticJsStrings,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_parser::temporal::{IsoCursor, TemporalDateTimeString};
use boa_profiler::Profiler;

use super::{
    calendar, duration::DurationRecord, options::ArithmeticOverflow,
    plain_date::iso::IsoDateRecord, plain_date_time, DateDuration, TimeDuration,
};

pub(crate) mod iso;

/// The `Temporal.PlainDate` object.
#[derive(Debug, Clone)]
pub struct PlainDate {
    pub(crate) inner: IsoDateRecord,
    pub(crate) calendar: JsValue, // Calendar can probably be stored as a JsObject.
}

impl PlainDate {
    pub(crate) fn new(record: IsoDateRecord, calendar: JsValue) -> Self {
        Self {
            inner: record,
            calendar,
        }
    }
}

impl BuiltInObject for PlainDate {
    const NAME: JsString = StaticJsStrings::PLAIN_DATE;
}

impl IntrinsicObject for PlainDate {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_calendar_id = BuiltInBuilder::callable(realm, Self::get_calendar_id)
            .name("get calendarId")
            .build();

        let get_year = BuiltInBuilder::callable(realm, Self::get_year)
            .name("get year")
            .build();

        let get_month = BuiltInBuilder::callable(realm, Self::get_month)
            .name("get month")
            .build();

        let get_month_code = BuiltInBuilder::callable(realm, Self::get_month_code)
            .name("get monthCode")
            .build();

        let get_day = BuiltInBuilder::callable(realm, Self::get_day)
            .name("get day")
            .build();

        let get_day_of_week = BuiltInBuilder::callable(realm, Self::get_day_of_week)
            .name("get dayOfWeek")
            .build();

        let get_day_of_year = BuiltInBuilder::callable(realm, Self::get_day_of_year)
            .name("get dayOfYear")
            .build();

        let get_week_of_year = BuiltInBuilder::callable(realm, Self::get_week_of_year)
            .name("get weekOfYear")
            .build();

        let get_year_of_week = BuiltInBuilder::callable(realm, Self::get_year_of_week)
            .name("get yearOfWeek")
            .build();

        let get_days_in_week = BuiltInBuilder::callable(realm, Self::get_days_in_week)
            .name("get daysInWeek")
            .build();

        let get_days_in_month = BuiltInBuilder::callable(realm, Self::get_days_in_month)
            .name("get daysInMonth")
            .build();

        let get_days_in_year = BuiltInBuilder::callable(realm, Self::get_days_in_year)
            .name("get daysInYear")
            .build();

        let get_months_in_year = BuiltInBuilder::callable(realm, Self::get_months_in_year)
            .name("get monthsInYear")
            .build();

        let get_in_leap_year = BuiltInBuilder::callable(realm, Self::get_in_leap_year)
            .name("get inLeapYear")
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                "calendarId",
                Some(get_calendar_id),
                None,
                Attribute::default(),
            )
            .accessor("year", Some(get_year), None, Attribute::default())
            .accessor("month", Some(get_month), None, Attribute::default())
            .accessor(
                "monthCode",
                Some(get_month_code),
                None,
                Attribute::default(),
            )
            .accessor("day", Some(get_day), None, Attribute::default())
            .accessor(
                "dayOfWeek",
                Some(get_day_of_week),
                None,
                Attribute::default(),
            )
            .accessor(
                "dayOfYear",
                Some(get_day_of_year),
                None,
                Attribute::default(),
            )
            .accessor(
                "weekOfYear",
                Some(get_week_of_year),
                None,
                Attribute::default(),
            )
            .accessor(
                "yearOfWeek",
                Some(get_year_of_week),
                None,
                Attribute::default(),
            )
            .accessor(
                "daysInWeek",
                Some(get_days_in_week),
                None,
                Attribute::default(),
            )
            .accessor(
                "daysInMonth",
                Some(get_days_in_month),
                None,
                Attribute::default(),
            )
            .accessor(
                "daysInYear",
                Some(get_days_in_year),
                None,
                Attribute::default(),
            )
            .accessor(
                "monthsInYear",
                Some(get_months_in_year),
                None,
                Attribute::default(),
            )
            .accessor(
                "inLeapYear",
                Some(get_in_leap_year),
                None,
                Attribute::default(),
            )
            .method(Self::to_plain_year_month, "toPlainYearMonth", 0)
            .method(Self::to_plain_month_day, "toPlainMonthDay", 0)
            .method(Self::get_iso_fields, "getISOFields", 0)
            .method(Self::get_calendar, "getCalendar", 0)
            .method(Self::add, "add", 2)
            .method(Self::subtract, "subtract", 2)
            .method(Self::with, "with", 2)
            .method(Self::with_calendar, "withCalendar", 1)
            .method(Self::until, "until", 2)
            .method(Self::since, "since", 2)
            .method(Self::equals, "equals", 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for PlainDate {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_date;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("NewTarget cannot be undefined.")
                .into());
        };

        let iso_year = super::to_integer_with_truncation(args.get_or_undefined(0), context)?;
        let iso_month = super::to_integer_with_truncation(args.get_or_undefined(1), context)?;
        let iso_day = super::to_integer_with_truncation(args.get_or_undefined(2), context)?;
        let default_calendar = JsValue::from("iso8601");
        let calendar_like = args.get(3).unwrap_or(&default_calendar);

        let iso = IsoDateRecord::new(iso_year, iso_month, iso_day);

        Ok(create_temporal_date(iso, calendar_like.clone(), Some(new_target), context)?.into())
    }
}

// -- `PlainDate` getter methods --
impl PlainDate {
    fn get_calendar_id(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn get_year(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_month(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_month_code(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_day(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_day_of_week(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_day_of_year(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_week_of_year(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_year_of_week(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_days_in_week(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_days_in_month(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_days_in_year(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_months_in_year(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_in_leap_year(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }
}

// ==== `PlainDate.prototype` method implementation ====

impl PlainDate {
    fn to_plain_year_month(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn to_plain_month_day(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_iso_fields(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_calendar(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn add(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn subtract(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn with(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn with_calendar(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn until(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn since(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn equals(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }
}

// -- `PlainDate` Abstract Operations --

impl PlainDate {
    /// Utitily function for translating a `Temporal.PlainDate` into a `JsObject`.
    pub(crate) fn as_object(&self, context: &mut Context<'_>) -> JsResult<JsObject> {
        create_temporal_date(self.inner, self.calendar.clone(), None, context)
    }
}

// 3.5.2 `CreateIsoDateRecord`
// Implemented on `IsoDateRecord`

/// 3.5.3 `CreateTemporalDate ( isoYear, isoMonth, isoDay, calendar [ , newTarget ] )`
pub(crate) fn create_temporal_date(
    iso_date: IsoDateRecord,
    calendar: JsValue,
    new_target: Option<&JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsObject> {
    // 1. If IsValidISODate(isoYear, isoMonth, isoDay) is false, throw a RangeError exception.
    if !iso_date.is_valid() {
        return Err(JsNativeError::range()
            .with_message("Date is not a valid ISO date.")
            .into());
    };

    let iso_date_time = plain_date_time::iso::IsoDateTimeRecord::default()
        .with_date(iso_date.year(), iso_date.month(), iso_date.day())
        .with_time(12, 0, 0, 0, 0, 0);

    // 2. If ISODateTimeWithinLimits(isoYear, isoMonth, isoDay, 12, 0, 0, 0, 0, 0) is false, throw a RangeError exception.
    if iso_date_time.is_valid() {
        return Err(JsNativeError::range()
            .with_message("Date is not within ISO date time limits.")
            .into());
    }

    // 3. If newTarget is not present, set newTarget to %Temporal.PlainDate%.
    let new_target = if let Some(new_target) = new_target {
        new_target.clone()
    } else {
        context
            .realm()
            .intrinsics()
            .constructors()
            .plain_date()
            .constructor()
            .into()
    };

    // 4. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.PlainDate.prototype%", « [[InitializedTemporalDate]], [[ISOYear]], [[ISOMonth]], [[ISODay]], [[Calendar]] »).
    let prototype =
        get_prototype_from_constructor(&new_target, StandardConstructors::plain_date, context)?;

    // 5. Set object.[[ISOYear]] to isoYear.
    // 6. Set object.[[ISOMonth]] to isoMonth.
    // 7. Set object.[[ISODay]] to isoDay.
    // 8. Set object.[[Calendar]] to calendar.
    let obj = JsObject::from_proto_and_data(
        prototype,
        ObjectData::plain_date(PlainDate::new(iso_date, calendar)),
    );

    // 9. Return object.
    Ok(obj)
}

/// 3.5.4 `ToTemporalDate ( item [ , options ] )`
///
/// Converts an ambiguous `JsValue` into a `PlainDate`
pub(crate) fn to_temporal_date(
    item: &JsValue,
    options: Option<JsValue>,
    context: &mut Context<'_>,
) -> JsResult<PlainDate> {
    // 1. If options is not present, set options to undefined.
    let options = options.unwrap_or(JsValue::undefined());

    // 2. Assert: Type(options) is Object or Undefined.
    // 3. If options is not undefined, set options to ? SnapshotOwnProperties(? GetOptionsObject(options), null).
    let options_obj = get_options_object(&options)?;

    // 4. If Type(item) is Object, then
    if let Some(object) = item.as_object() {
        // a. If item has an [[InitializedTemporalDate]] internal slot, then
        if object.is_plain_date() {
            // i. Return item.
            let obj = object.borrow();
            let date = obj.as_plain_date().expect("obj must be a PlainDate.");
            return Ok(PlainDate {
                inner: date.inner,
                calendar: date.calendar.clone(),
            });
        // b. If item has an [[InitializedTemporalZonedDateTime]] internal slot, then
        } else if object.is_zoned_date_time() {
            return Err(JsNativeError::range()
                .with_message("ZonedDateTime not yet implemented.")
                .into());
            // i. Perform ? ToTemporalOverflow(options).
            // ii. Let instant be ! CreateTemporalInstant(item.[[Nanoseconds]]).
            // iii. Let plainDateTime be ? GetPlainDateTimeFor(item.[[TimeZone]], instant, item.[[Calendar]]).
            // iv. Return ! CreateTemporalDate(plainDateTime.[[ISOYear]], plainDateTime.[[ISOMonth]], plainDateTime.[[ISODay]], plainDateTime.[[Calendar]]).

            // c. If item has an [[InitializedTemporalDateTime]] internal slot, then
        } else if object.is_plain_date_time() {
            // i. Perform ? ToTemporalOverflow(options).
            let _o = get_option(&options_obj, "overflow", context)?
                .unwrap_or(ArithmeticOverflow::Constrain);

            let obj = object.borrow();
            let date_time = obj
                .as_plain_date_time()
                .expect("obj must be a PlainDateTime");

            let iso = date_time.inner.iso_date();
            let calendar = date_time.calendar.clone();

            drop(obj);

            // ii. Return ! CreateTemporalDate(item.[[ISOYear]], item.[[ISOMonth]], item.[[ISODay]], item.[[Calendar]]).
            return Ok(PlainDate {
                inner: iso,
                calendar,
            });
        }

        // d. Let calendar be ? GetTemporalCalendarSlotValueWithISODefault(item).
        // e. Let fieldNames be ? CalendarFields(calendar, « "day", "month", "monthCode", "year" »).
        // f. Let fields be ? PrepareTemporalFields(item, fieldNames, «»).
        // g. Return ? CalendarDateFromFields(calendar, fields, options).
        return Err(JsNativeError::error()
            .with_message("CalendarDateFields not yet implemented.")
            .into());
    }

    // 5. If item is not a String, throw a TypeError exception.
    match item {
        JsValue::String(s) => {
            // 6. Let result be ? ParseTemporalDateString(item).
            let result = TemporalDateTimeString::parse(
                false,
                &mut IsoCursor::new(&s.to_std_string_escaped()),
            )
            .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

            // 7. Assert: IsValidISODate(result.[[Year]], result.[[Month]], result.[[Day]]) is true.
            // 8. Let calendar be result.[[Calendar]].
            // 9. If calendar is undefined, set calendar to "iso8601".
            let identifier = result
                .date
                .calendar
                .map_or_else(|| js_string!("iso8601"), JsString::from);

            // 10. If IsBuiltinCalendar(calendar) is false, throw a RangeError exception.
            if !super::calendar::is_builtin_calendar(&identifier) {
                return Err(JsNativeError::range()
                    .with_message("not a valid calendar identifier.")
                    .into());
            }

            // TODO: impl to ASCII-lowercase on JsStirng
            // 11. Set calendar to the ASCII-lowercase of calendar.

            // 12. Perform ? ToTemporalOverflow(options).
            let _result = get_option(&options_obj, "overflow", context)?
                .unwrap_or(ArithmeticOverflow::Constrain);

            // 13. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], calendar).
            Ok(PlainDate {
                inner: IsoDateRecord::new(result.date.year, result.date.month, result.date.day),
                calendar: identifier.into(),
            })
        }
        _ => Err(JsNativeError::typ()
            .with_message("ToTemporalDate item must be an object or string.")
            .into()),
    }
}

// 3.5.5. DifferenceIsoDate
// Implemented on IsoDateRecord.

/// 3.5.6 `DifferenceDate ( calendar, one, two, options )`
pub(crate) fn difference_date(
    calendar: &JsValue,
    one: &PlainDate,
    two: &PlainDate,
    largest_unit: TemporalUnit,
    context: &mut Context<'_>,
) -> JsResult<DurationRecord> {
    // 1. Assert: one.[[Calendar]] and two.[[Calendar]] have been determined to be equivalent as with CalendarEquals.
    // 2. Assert: options is an ordinary Object.
    // 3. Assert: options.[[Prototype]] is null.
    // 4. Assert: options has a "largestUnit" data property.
    // 5. If one.[[ISOYear]] = two.[[ISOYear]] and one.[[ISOMonth]] = two.[[ISOMonth]] and one.[[ISODay]] = two.[[ISODay]], then
    if one.inner.year() == two.inner.year()
        && one.inner.month() == two.inner.month()
        && one.inner.day() == two.inner.day()
    {
        // a. Return ! CreateTemporalDuration(0, 0, 0, 0, 0, 0, 0, 0, 0, 0).
        return Ok(DurationRecord::default());
    }
    // 6. If ! Get(options, "largestUnit") is "day", then
    if largest_unit == TemporalUnit::Day {
        // a. Let days be DaysUntil(one, two).
        let days = super::duration::days_until(one, two);
        // b. Return ! CreateTemporalDuration(0, 0, 0, days, 0, 0, 0, 0, 0, 0).
        return Ok(DurationRecord::new(
            DateDuration::new(0.0, 0.0, 0.0, f64::from(days)),
            TimeDuration::default(),
        ));
    }

    // Create the options object prior to sending it to the calendars.
    let options_obj = JsObject::with_null_proto();

    options_obj.create_data_property_or_throw(
        "largestUnit",
        JsString::from(largest_unit.to_string()),
        context,
    )?;

    // 7. Return ? CalendarDateUntil(calendar, one, two, options).
    calendar::calendar_date_until(calendar, one, two, &options_obj.into(), context)
}

// 3.5.7 RegulateIsoDate
// Implemented on IsoDateRecord.

// 3.5.8 IsValidIsoDate
// Implemented on IsoDateRecord.

// 3.5.9 BalanceIsoDate
// Implemented on IsoDateRecord.

// 3.5.12 AddISODate ( year, month, day, years, months, weeks, days, overflow )
// Implemented on IsoDateRecord

/// 3.5.13 `AddDate ( calendar, plainDate, duration [ , options [ , dateAdd ]] )`
pub(crate) fn add_date(
    calendar: &JsValue,
    plain_date: &PlainDate,
    duration: &DurationRecord,
    options: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<PlainDate> {
    // 1. If options is not present, set options to undefined.
    // 2. If duration.[[Years]] ≠ 0, or duration.[[Months]] ≠ 0, or duration.[[Weeks]] ≠ 0, then
    if duration.years() != 0.0 || duration.months() != 0.0 || duration.weeks() != 0.0 {
        // a. If dateAdd is not present, then
        // i. Set dateAdd to unused.
        // ii. If calendar is an Object, set dateAdd to ? GetMethod(calendar, "dateAdd").
        // b. Return ? CalendarDateAdd(calendar, plainDate, duration, options, dateAdd).
        return calendar::calendar_date_add(calendar, plain_date, duration, options, context);
    }

    // 3. Let overflow be ? ToTemporalOverflow(options).
    let options_obj = get_options_object(options)?;
    let overflow =
        get_option(&options_obj, "overflow", context)?.unwrap_or(ArithmeticOverflow::Constrain);

    let mut intermediate = *duration;
    // 4. Let days be ? BalanceTimeDuration(duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]], "day").[[Days]].
    intermediate.balance_time_duration(TemporalUnit::Day, None)?;

    // 5. Let result be ? AddISODate(plainDate.[[ISOYear]], plainDate.[[ISOMonth]], plainDate.[[ISODay]], 0, 0, 0, days, overflow).
    let result = plain_date
        .inner
        .add_iso_date(intermediate.date(), overflow)?;

    // 6. Return ! CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], calendar).
    Ok(PlainDate::new(result, plain_date.calendar.clone()))
}
