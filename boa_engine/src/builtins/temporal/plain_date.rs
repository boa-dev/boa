#![allow(dead_code, unused_variables)]

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::utf16,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

#[derive(Debug, Clone)]
pub(crate) struct IsoDateRecord {
    y: i32,
    m: i32,
    d: i32,
}

impl IsoDateRecord {
    pub(crate) const fn new(y: i32, m: i32, d: i32) -> Self {
        Self { y, m, d }
    }

    pub(crate) fn is_valid_iso_date(&self) -> bool {
        if self.m < 1 || self.m > 12 {
            return false;
        }

        let days_in_month = super::calendar::iso_days_in_month(self.y, self.m);

        if self.d < 1 || self.d > days_in_month {
            return false;
        }
        true
    }
}

/// The `Temporal.PlainDate` object.
#[derive(Debug, Clone)]
pub struct PlainDate {
    pub(crate) iso_year: i32,
    pub(crate) iso_month: i32,
    pub(crate) iso_day: i32,
    pub(crate) calendar: JsObject, // Calendar can probably be stored as a JsObject.
}

impl BuiltInObject for PlainDate {
    const NAME: &'static str = "Temporal.PlainDate";
}

impl IntrinsicObject for PlainDate {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

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
                utf16!("calendarId"),
                Some(get_calendar_id),
                None,
                Attribute::default(),
            )
            .accessor(utf16!("year"), Some(get_year), None, Attribute::default())
            .accessor(utf16!("month"), Some(get_month), None, Attribute::default())
            .accessor(
                utf16!("monthCode"),
                Some(get_month_code),
                None,
                Attribute::default(),
            )
            .accessor(utf16!("day"), Some(get_day), None, Attribute::default())
            .accessor(
                utf16!("dayOfWeek"),
                Some(get_day_of_week),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("dayOfYear"),
                Some(get_day_of_year),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("weekOfYear"),
                Some(get_week_of_year),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("yearOfWeek"),
                Some(get_year_of_week),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("daysInWeek"),
                Some(get_days_in_week),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("daysInMonth"),
                Some(get_days_in_month),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("daysInYear"),
                Some(get_days_in_year),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("monthsInYear"),
                Some(get_months_in_year),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("inLeapYear"),
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
        let calendar_like = JsObject::with_null_proto(); // args.get_or_undefined(3);

        create_temporal_date(
            iso_year,
            iso_month,
            iso_day,
            calendar_like,
            Some(new_target),
            context,
        )
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

// -- `PlainDate` method implementation --
impl PlainDate {
    fn to_plain_year_month(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn to_plain_month_day(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_iso_fields(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_calendar(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn add(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn subtract(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn with(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn with_calendar(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn until(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn since(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn equals(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _plain_date = o.as_plain_date().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }
}

// -- `PlainDate` Abstract Operations --

/// 3.5.3 `CreateTemporalDate ( isoYear, isoMonth, isoDay, calendar [ , newTarget ] )`
fn create_temporal_date(
    y: i32,
    mo: i32,
    d: i32,
    calendar: JsObject,
    new_target: Option<&JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. If IsValidISODate(isoYear, isoMonth, isoDay) is false, throw a RangeError exception.
    if IsoDateRecord::new(y, mo, d).is_valid_iso_date() {
        return Err(JsNativeError::range()
            .with_message("Date is not a valid ISO date.")
            .into());
    };

    // 2. If ISODateTimeWithinLimits(isoYear, isoMonth, isoDay, 12, 0, 0, 0, 0, 0) is false, throw a RangeError exception.
    if super::plain_date_time::iso_datetime_within_limits(y, mo, d, 12, 0, 0, 0, 0, 0) {
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
    let new_date =
        get_prototype_from_constructor(&new_target, StandardConstructors::plain_date, context)?;

    let mut obj = new_date.borrow_mut();
    let date = obj.as_plain_date_mut().expect("this value must be a date");

    // 5. Set object.[[ISOYear]] to isoYear.
    date.iso_year = y;
    // 6. Set object.[[ISOMonth]] to isoMonth.
    date.iso_month = mo;
    // 7. Set object.[[ISODay]] to isoDay.
    date.iso_day = d;
    // 8. Set object.[[Calendar]] to calendar.
    date.calendar = calendar;

    drop(obj);
    // 9. Return object.
    Ok(new_date.into())
}

pub(crate) fn to_temporal_date(item: &JsValue, options: Option<JsObject>) -> JsResult<PlainDate> {
    Err(JsNativeError::error()
        .with_message("not yet implemented.")
        .into())
}
