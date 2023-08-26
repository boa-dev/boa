#![allow(dead_code, unused_variables)]
use self::iso::IsoCalendar;

use super::PlainDate;
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, ObjectData},
    property::{Attribute, PropertyKey},
    realm::Realm,
    string::utf16,
    Context, JsArgs, JsBigInt, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

mod iso;
pub(crate) mod utils;

/// A trait for implementing a Builtin Calendar
pub trait BuiltinCalendar {
    /// Return the calendar identifier.
    fn identifier(&self) -> &str;
    /// Creates a `Temporal.PlainDate` object from provided fields.
    fn date_from_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Creates a `Temporal.PlainYearMonth` object from the provided fields.
    fn year_month_from_fields(
        &self,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Creates a `Temporal.PlainMonthDay` object from the provided fields.
    fn month_day_from_fields(
        &self,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns a `Temporal.PlainDate` based off an added date.
    fn date_add(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns a `Temporal.Duration` representing the duration between two dates.
    fn date_until(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the era for a given `temporaldatelike`.
    fn era(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the era year for a given `temporaldatelike`
    fn era_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `year` for a given `temporaldatelike`
    fn year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `month` for a given `temporaldatelike`
    fn month(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `monthCode` for a given `temporaldatelike`
    fn month_code(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `day` for a given `temporaldatelike`
    fn day(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns a value representing the day of the week for a date.
    fn day_of_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns a value representing the day of the year for a given calendar.
    fn day_of_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns a value representing the week of the year for a given calendar.
    fn week_of_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the year of a given week.
    fn year_of_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the days in a week for a given calendar.
    fn days_in_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the days in a month for a given calendar.
    fn days_in_month(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the days in a year for a given calendar.
    fn days_in_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the months in a year for a given calendar.
    fn months_in_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn in_leap_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn merge_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
}

impl core::fmt::Debug for dyn BuiltinCalendar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

fn is_builtin_calendar(identifer: String) -> Option<Box<impl BuiltinCalendar>> {
    match identifer.to_ascii_lowercase().as_str() {
        "iso8601" => Some(Box::new(IsoCalendar)),
        _ => None,
    }
}

/// The `Temporal.Calendar` object.
#[derive(Debug)]
pub struct Calendar {
    inner: Box<dyn BuiltinCalendar>,
}

impl BuiltInObject for Calendar {
    const NAME: &'static str = "Temporal.Calendar";
}

impl IntrinsicObject for Calendar {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let get_id = BuiltInBuilder::callable(realm, Self::get_id)
            .name("get Id")
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .accessor(utf16!("id"), Some(get_id), None, Attribute::default())
            .method(Self::date_from_fields, "dateFromFields", 2)
            .method(Self::year_month_from_fields, "yearMonthFromFields", 2)
            .method(Self::month_day_from_fields, "monthDayFromFields", 2)
            .method(Self::date_add, "dateAdd", 3)
            .method(Self::date_until, "dateUntil", 3)
            .method(Self::year, "year", 1)
            .method(Self::month, "month", 1)
            .method(Self::month_code, "monthCode", 1)
            .method(Self::day, "day", 1)
            .method(Self::day_of_week, "dayOfWeek", 1)
            .method(Self::day_of_year, "dayOfYear", 1)
            .method(Self::week_of_year, "weekOfYear", 1)
            .method(Self::year_of_week, "yearOfWeek", 1)
            .method(Self::days_in_week, "daysInWeek", 1)
            .method(Self::days_in_month, "daysInMonth", 1)
            .method(Self::days_in_year, "daysInYear", 1)
            .method(Self::months_in_year, "monthsInYear", 1)
            .method(Self::in_leap_year, "inLeapYear", 1)
            .method(Self::fields, "fields", 1)
            .method(Self::merge_fields, "mergeFields", 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for Calendar {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::calendar;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message(
                    "newTarget cannot be undefined when constructing a Temporal.Calendar object.",
                )
                .into());
        }

        let identifier = args.get_or_undefined(0);

        if let Some(id) = identifier.as_string() {
            create_temporal_calendar(id, Some(new_target.clone()), context)
        } else {
            Err(JsNativeError::typ()
                .with_message("Calendar id must be a string.")
                .into())
        }
    }
}

impl Calendar {
    fn get_id(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        Ok(calendar.inner.identifier().into())
    }

    /// 15.8.2.1 Temporal.Calendar.prototype.dateFromFields ( fields [ , options ] ) - Supercedes 12.5.4
    fn date_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.date_from_fields(args, context)
    }

    /// 15.8.2.2 Temporal.Calendar.prototype.yearMonthFromFields ( fields [ , options ] ) - Supercedes 12.5.5
    fn year_month_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.year_month_from_fields(args, context)
    }

    /// 15.8.2.3 Temporal.Calendar.prototype.monthDayFromFields ( fields [ , options ] ) - Supercedes 12.5.6
    fn month_day_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.month_day_from_fields(args, context)
    }

    /// 15.8.2.4 Temporal.Calendar.prototype.dateAdd ( date, duration [ , options ] ) - supercedes 12.5.7
    fn date_add(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.date_add(args, context)
    }

    ///15.8.2.5 Temporal.Calendar.prototype.dateUntil ( one, two [ , options ] ) - Supercedes 12.5.8
    fn date_until(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.date_until(args, context)
    }

    /// 15.8.2.6 Temporal.Calendar.prototype.era ( temporalDateLike )
    fn era(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.era(args, context)
    }

    /// 15.8.2.7 Temporal.Calendar.prototype.eraYear ( temporalDateLike )
    fn era_year(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.era_year(args, context)
    }

    fn year(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.year(args, context)
    }

    fn month(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.month(args, context)
    }

    fn month_code(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.month_code(args, context)
    }

    fn day(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.day(args, context)
    }

    fn day_of_week(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.day_of_week(args, context)
    }

    fn day_of_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.day_of_year(args, context)
    }

    fn week_of_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.week_of_year(args, context)
    }

    fn year_of_week(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.year_of_week(args, context)
    }

    fn days_in_week(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.days_in_week(args, context)
    }

    fn days_in_month(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.days_in_month(args, context)
    }

    fn days_in_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.days_in_year(args, context)
    }

    fn months_in_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.months_in_year(args, context)
    }

    fn in_leap_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.in_leap_year(args, context)
    }

    fn fields(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.fields(args, context)
    }

    fn merge_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.merge_fields(args, context)
    }
}

// -- `Calendar` Abstract Operations --

/// 12.2.1 `CreateTemporalCalendar ( identifier [ , newTarget ] )`
pub(crate) fn create_temporal_calendar(
    identifier: &JsString,
    new_target: Option<JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    let inner = is_builtin_calendar(identifier.to_std_string_escaped());

    // 1. Assert: IsBuiltinCalendar(identifier) is true.
    if let Some(inner) = inner {
        let calendar = Calendar { inner };
        // 2. If newTarget is not provided, set newTarget to %Temporal.Calendar%.
        let new_target = new_target.unwrap_or(
            context
                .realm()
                .intrinsics()
                .constructors()
                .calendar()
                .constructor()
                .into(),
        );

        // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Calendar.prototype%", « [[InitializedTemporalCalendar]], [[Identifier]] »).
        let proto =
            get_prototype_from_constructor(&new_target, StandardConstructors::calendar, context)?;

        let obj = JsObject::from_proto_and_data(proto, ObjectData::calendar(calendar));

        // 4. Set object.[[Identifier]] to the ASCII-lowercase of identifier.
        // 5. Return object.
        Ok(obj.into())
    } else {
        // Note: Due to storing a trait object over a string. Move the RangeError from Contructor to `CreateTemporalCalendar`
        Err(JsNativeError::range()
            .with_message("Calendar id is not a supported Builtin Calendar.")
            .into())
    }
}

/// 12.2.4 `CalendarDateAdd ( calendar, date, duration [ , options [ , dateAdd ] ] )`
pub(crate) fn calendar_date_add(
    calendar: &JsValue,
    date: &JsObject,
    duration: &JsObject,
    options: &JsValue,
    date_add: Option<&JsValue>,
) -> JsResult<JsObject> {
    todo!()
}

/// 12.2.5 `CalendarDateUntil ( calendar, one, two, options [ , dateUntil ] )`
pub(crate) fn calendar_date_until(
    calendar: &JsValue,
    one: &JsObject,
    two: &JsObject,
    options: &JsValue,
    date_until: Option<&JsValue>,
) -> JsResult<super::duration::DurationRecord> {
    todo!()
}

/// 12.2.24 CalendarDateFromFields ( calendar, fields [ , options [ , dateFromFields ] ] )
pub(crate) fn calendar_date_from_fields(
    calendar: &JsValue,
    fields: &JsObject,
    options: Option<&JsValue>,
    date_from_fields: Option<&JsObject>,
) -> JsResult<PlainDate> {
    let options = match options {
        Some(o) => o.clone(),
        _ => JsValue::undefined(),
    };

    todo!()
}


/// 12.2.21 GetTemporalCalendarSlotValueWithISODefault ( item )
pub(crate) fn get_temporal_calendar_slot_value_with_default(item: &JsObject, context: &mut Context<'_>) -> JsResult<JsValue> {
    // 1. If item has an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]], [[InitializedTemporalMonthDay]], [[InitializedTemporalYearMonth]], or [[InitializedTemporalZonedDateTime]] internal slot, then
        // a. Return item.[[Calendar]].
    // 2. Let calendarLike be ? Get(item, "calendar").
    // 3. Return ? ToTemporalCalendarSlotValue(calendarLike, "iso8601").
    if item.is_plain_date() {
        let obj = item.borrow();
        let date = obj.as_plain_date();
        if let Some(date) = date {
            let calendar = date.calendar.clone();
            drop(obj);
            return Ok(calendar)
        }
    } else if item.is_plain_date_time() {
        let obj = item.borrow();
        let date_time = obj.as_plain_date_time();
        if let Some(dt) = date_time {
            let calendar = dt.calendar.clone();
            drop(obj);
            return Ok(calendar)
        }
    } else if item.is_plain_year_month() {
        let obj = item.borrow();
        let year_month = obj.as_plain_year_month();
        if let Some(ym) = year_month {
            let calendar = ym.calendar.clone();
            drop(obj);
            return Ok(calendar)
        }
    } else if item.is_plain_month_day() {
        let obj = item.borrow();
        let month_day = obj.as_plain_month_day();
        if let Some(md) = month_day {
            let calendar = md.calendar.clone();
            drop(obj);
            return Ok(calendar)
        }
    } else if item.is_zoned_date_time() {
        return Err(JsNativeError::range().with_message("Not yet implemented.").into())
    }

    let calendar_like = item.get(PropertyKey::from("calendar"), context)?;
    to_temporal_calendar_slot_value(&calendar_like, Some(JsString::from("iso8601")))
}

fn to_temporal_calendar_slot_value(calendar_like: &JsValue, default: Option<JsString>) -> JsResult<JsValue> {
    todo!()
}