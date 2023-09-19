#![allow(dead_code, unused_variables)]

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::utf16,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

use super::{plain_date::iso::IsoDateRecord, TemporalFields};

#[derive(Debug, Clone)]
pub(crate) struct IsoYearMonthRecord {
    year: i32,
    month: i32,
    ref_day: i32,
}

impl IsoYearMonthRecord {
    pub(crate) const fn year(&self) -> i32 {
        self.year
    }

    pub(crate) const fn month(&self) -> i32 {
        self.month
    }
}

impl IsoYearMonthRecord {
    pub(crate) const fn new(year: i32, month: i32, ref_day: i32) -> Self {
        Self {
            year,
            month,
            ref_day,
        }
    }

    pub(crate) fn from_temporal_fields(
        fields: &mut TemporalFields,
        overflow: &JsString,
    ) -> JsResult<Self> {
        fields.resolve_month()?;
        Self::from_unregulated(
            fields.year().expect("year must exist for YearMonthRecord."),
            fields
                .month()
                .expect("month must exist for YearMonthRecord"),
            1,
            overflow,
        )
    }

    pub(crate) fn from_unregulated(
        year: i32,
        month: i32,
        day: i32,
        overflow: &JsString,
    ) -> JsResult<Self> {
        match overflow.to_std_string_escaped().as_str() {
            "constrain" => {
                let clamped_month = month.clamp(1, 12);
                Ok(Self {
                    year,
                    month: clamped_month,
                    ref_day: day,
                })
            }
            "reject" => {
                if !(1..=12).contains(&month) {
                    return Err(JsNativeError::range()
                        .with_message("month is not within the valid range.")
                        .into());
                }

                Ok(Self {
                    year,
                    month,
                    ref_day: day,
                })
            }
            _ => unreachable!(),
        }
    }

    fn is_valid_iso_date(&self) -> bool {
        if !(1..=12).contains(&self.month) {
            return false;
        }

        let days_in_month = super::calendar::utils::iso_days_in_month(self.year, self.month);

        if !(1..=days_in_month).contains(&self.ref_day) {
            return false;
        }
        true
    }

    /// 9.5.4 `BalanceISOYearMonth ( year, month )`
    pub(crate) fn balance(&mut self) {
        self.year += (self.month - 1) / 12;
        self.month = ((self.month - 1) % 12) + 1;
    }
}

/// The `Temporal.PlainYearMonth` object.
#[derive(Debug, Clone)]
pub struct PlainYearMonth {
    pub(crate) inner: IsoDateRecord,
    pub(crate) calendar: JsValue,
}

impl BuiltInObject for PlainYearMonth {
    const NAME: &'static str = "Temporal.PlainYearMonth";
}

impl IntrinsicObject for PlainYearMonth {
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
            .method(Self::with, "with", 2)
            .method(Self::add, "add", 2)
            .method(Self::subtract, "subtract", 2)
            .method(Self::until, "until", 2)
            .method(Self::since, "since", 2)
            .method(Self::equals, "equals", 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for PlainYearMonth {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_year_month;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, then
        if new_target.is_undefined() {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message("NewTarget cannot be undefined when constructing a PlainYearMonth.")
                .into());
        }

        let day = args.get_or_undefined(3);
        // 2. If referenceISODay is undefined, then
        let ref_day = if day.is_undefined() {
            // a. Set referenceISODay to 1𝔽.
            1
        } else {
            // 6. Let ref be ? ToIntegerWithTruncation(referenceISODay).
            super::to_integer_with_truncation(day, context)?
        };

        // 3. Let y be ? ToIntegerWithTruncation(isoYear).
        let y = super::to_integer_with_truncation(args.get_or_undefined(0), context)?;
        // 4. Let m be ? ToIntegerWithTruncation(isoMonth).
        let m = super::to_integer_with_truncation(args.get_or_undefined(1), context)?;

        // TODO: calendar handling.
        // 5. Let calendar be ? ToTemporalCalendarSlotValue(calendarLike, "iso8601").

        // 7. Return ? CreateTemporalYearMonth(y, m, calendar, ref, NewTarget).
        let record = IsoDateRecord::new(y, m, ref_day);
        create_temporal_year_month(record, JsValue::from("iso8601"), Some(new_target), context)
    }
}

// ==== `PlainYearMonth` Accessor Implementations ====

impl PlainYearMonth {
    fn get_calendar_id(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
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

    fn get_days_in_year(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_days_in_month(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
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

// ==== `PlainYearMonth` Method Implementations ====

impl PlainYearMonth {
    fn with(this: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn add(this: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn subtract(this: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn until(this: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn since(this: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn equals(this: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }
}

// ==== Abstract Operations ====

// 9.5.2 `RegulateISOYearMonth ( year, month, overflow )`
pub(crate) fn regulate_iso_year_month(
    year: i32,
    month: i32,
    overflow: &JsString,
) -> JsResult<(i32, i32)> {
    // 1. Assert: year and month are integers.
    // 2. Assert: overflow is either "constrain" or "reject".
    // 3. If overflow is "constrain", then
    let month = match overflow.to_std_string_escaped().as_str() {
        "constrain" => {
            // a. Set month to the result of clamping month between 1 and 12.
            // b. Return the Record { [[Year]]: year, [[Month]]: month }.
            month.clamp(1, 12)
        }
        "reject" => {
            // a. Assert: overflow is "reject".
            // b. If month < 1 or month > 12, throw a RangeError exception.
            if !(1..=12).contains(&month) {
                return Err(JsNativeError::range()
                    .with_message("month is not within the valid range.")
                    .into());
            }
            // c. Return the Record { [[Year]]: year, [[Month]]: month }.
            month
        }
        _ => unreachable!(),
    };

    Ok((year, month))
}

// 9.5.5 `CreateTemporalYearMonth ( isoYear, isoMonth, calendar, referenceISODay [ , newTarget ] )`
pub(crate) fn create_temporal_year_month(
    year_month_record: IsoDateRecord,
    calendar: JsValue,
    new_target: Option<&JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. If IsValidISODate(isoYear, isoMonth, referenceISODay) is false, throw a RangeError exception.
    if !year_month_record.is_valid() {
        return Err(JsNativeError::range()
            .with_message("PlainYearMonth values are not a valid ISO date.")
            .into());
    }

    // 2. If ! ISOYearMonthWithinLimits(isoYear, isoMonth) is false, throw a RangeError exception.
    if year_month_record.within_year_month_limits() {
        return Err(JsNativeError::range()
            .with_message("PlainYearMonth values are not a valid ISO date.")
            .into());
    }

    // 3. If newTarget is not present, set newTarget to %Temporal.PlainYearMonth%.
    let new_target = if let Some(target) = new_target {
        target.clone()
    } else {
        context
            .realm()
            .intrinsics()
            .constructors()
            .plain_year_month()
            .constructor()
            .into()
    };

    // 4. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.PlainYearMonth.prototype%", « [[InitializedTemporalYearMonth]], [[ISOYear]], [[ISOMonth]], [[ISODay]], [[Calendar]] »).
    let new_year_month = get_prototype_from_constructor(
        &new_target,
        StandardConstructors::plain_year_month,
        context,
    )?;

    let mut obj = new_year_month.borrow_mut();
    let year_month = obj
        .as_plain_year_month_mut()
        .expect("this value must be a date");

    // 5. Set object.[[ISOYear]] to isoYear.
    // 6. Set object.[[ISOMonth]] to isoMonth.
    // 7. Set object.[[Calendar]] to calendar.
    // 8. Set object.[[ISODay]] to referenceISODay.
    year_month.inner = year_month_record;
    year_month.calendar = calendar;

    drop(obj);

    // 9. Return object.
    Ok(new_year_month.into())
}
