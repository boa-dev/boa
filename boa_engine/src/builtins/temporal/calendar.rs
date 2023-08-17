#![allow(dead_code, unused_variables)]
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::Attribute,
    realm::Realm,
    Context, JsBigInt, JsObject, JsResult, JsString, JsSymbol, JsValue, JsNativeError
};
use boa_profiler::Profiler;
use super::PlainDate;

mod iso;

/// A trait for implementing a Builtin Calendar
pub trait BuiltinCalendar {
    /// TODO: Docs
    fn identifier(&self) -> &str;
    /// TODO: Docs
    fn date_from_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn year_month_from_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn month_day_from_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn date_add(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn date_until(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn era(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn era_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn month_code(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn day(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn day_of_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn day_of_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn week_of_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn year_of_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
    fn days_in_month(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// TODO: Docs
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

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
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
        todo!()
    }
}

impl Calendar {
    /// 15.8.2.1 Temporal.Calendar.prototype.dateFromFields ( fields [ , options ] ) - Supercedes 12.5.4
    fn date_from_fields(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.date_from_fields(args, context)
    }

    /// 15.8.2.2 Temporal.Calendar.prototype.yearMonthFromFields ( fields [ , options ] ) - Supercedes 12.5.5
    fn year_month_from_fields(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.year_month_from_fields(args, context)
    }

    /// 15.8.2.3 Temporal.Calendar.prototype.monthDayFromFields ( fields [ , options ] ) - Supercedes 12.5.6
    fn month_day_from_fields(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.month_day_from_fields(args, context)
    }

    /// 15.8.2.4 Temporal.Calendar.prototype.dateAdd ( date, duration [ , options ] ) - supercedes 12.5.7
    fn date_add(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.date_add(args, context)
    }

    ///15.8.2.5 Temporal.Calendar.prototype.dateUntil ( one, two [ , options ] ) - Supercedes 12.5.8
    fn date_until(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.date_until(args, context)
    }

    /// 15.8.2.6 Temporal.Calendar.prototype.era ( temporalDateLike )
    fn era(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.era(args, context)
    }

    /// 15.8.2.7 Temporal.Calendar.prototype.eraYear ( temporalDateLike )
    fn era_year(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.era_year(args, context)
    }

    fn year(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.year(args, context)
    }

    fn month_code(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.month_code(args, context)
    }

    fn day(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.day(args, context)
    }

    fn day_of_week(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.day_of_week(args, context)
    }

    fn day_of_year(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.day_of_year(args, context)
    }

    fn week_of_year(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.week_of_year(args, context)
    }

    fn year_of_week(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.year_of_week(args, context)
    }

    fn days_in_month(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.days_in_month(args, context)
    }

    fn months_in_year(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.months_in_year(args, context)
    }

    fn in_leap_year(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.in_leap_year(args, context)
    }

    fn fields(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.fields(args, context)
    }

    fn merge_fields(this:&JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ().with_message("the this value of Calendar must be a Calendar object.")
        })?;

        calendar.inner.merge_fields(args, context)
    }
}

// -- `Calendar` Abstract Operations --

/// 12.2.1 `CreateTemporalCalendar ( identifier [ , newTarget ] )`
pub(crate) fn create_temporal_calendar(
    identifier: &JsString,
    new_target: Option<JsValue>,
) -> JsResult<JsValue> {
    // 1. Assert: IsBuiltinCalendar(identifier) is true.
    // 2. If newTarget is not provided, set newTarget to %Temporal.Calendar%.
    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Calendar.prototype%", « [[InitializedTemporalCalendar]], [[Identifier]] »).
    // 4. Set object.[[Identifier]] to the ASCII-lowercase of identifier.
    // 5. Return object.
    todo!()
}

/// 12.2.4 `CalendarDateAdd ( calendar, date, duration [ , options [ , dateAdd ] ] )`
pub(crate) fn calendar_date_add(
    calendar: &JsObject,
    date: &JsObject,
    duration: &JsObject,
    options: &JsValue,
    date_add: Option<&JsValue>,
) -> JsResult<JsObject> {
    todo!()
}

/// 12.2.5 `CalendarDateUntil ( calendar, one, two, options [ , dateUntil ] )`
pub(crate) fn calendar_date_until(
    calendar: &JsObject,
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
        _=> JsValue::undefined(),
    };

    todo!()
}

/// 12.2.31 `ISODaysInMonth ( year, month )`
pub(crate) fn iso_days_in_month(year: i32, month: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            28 + super::date_equations::mathematical_in_leap_year(
                super::date_equations::epoch_time_for_year(f64::from(year)),
            )
        }
        _ => unreachable!("an invalid month value is an implementation error."),
    }
}
