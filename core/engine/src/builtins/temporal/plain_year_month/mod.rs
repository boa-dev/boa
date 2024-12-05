//! Boa's implementation of the `Temporal.PlainYearMonth` builtin object.

use std::str::FromStr;

use crate::{
    builtins::{
        options::{get_option, get_options_object},
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

use temporal_rs::{
    options::{ArithmeticOverflow, CalendarName},
    Duration, PlainYearMonth as InnerYearMonth,
};

use super::{calendar::to_temporal_calendar_slot_value, to_temporal_duration, DateTimeValues};

/// The `Temporal.PlainYearMonth` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)] // TODO: Remove this!!! `InnerYearMonth` could contain `Trace` types.
pub struct PlainYearMonth {
    pub(crate) inner: InnerYearMonth,
}

impl PlainYearMonth {
    pub(crate) fn new(inner: InnerYearMonth) -> Self {
        Self { inner }
    }
}

impl BuiltInObject for PlainYearMonth {
    const NAME: JsString = StaticJsStrings::PLAIN_YM_NAME;
}

impl IntrinsicObject for PlainYearMonth {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_calendar_id = BuiltInBuilder::callable(realm, Self::get_calendar_id)
            .name(js_string!("get calendarId"))
            .build();

        let get_year = BuiltInBuilder::callable(realm, Self::get_year)
            .name(js_string!("get year"))
            .build();

        let get_month = BuiltInBuilder::callable(realm, Self::get_month)
            .name(js_string!("get month"))
            .build();

        let get_month_code = BuiltInBuilder::callable(realm, Self::get_month_code)
            .name(js_string!("get monthCode"))
            .build();

        let get_days_in_month = BuiltInBuilder::callable(realm, Self::get_days_in_month)
            .name(js_string!("get daysInMonth"))
            .build();

        let get_days_in_year = BuiltInBuilder::callable(realm, Self::get_days_in_year)
            .name(js_string!("get daysInYear"))
            .build();

        let get_months_in_year = BuiltInBuilder::callable(realm, Self::get_months_in_year)
            .name(js_string!("get monthsInYear"))
            .build();

        let get_in_leap_year = BuiltInBuilder::callable(realm, Self::get_in_leap_year)
            .name(js_string!("get inLeapYear"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                StaticJsStrings::PLAIN_YM_TAG,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("calendarId"),
                Some(get_calendar_id),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("year"),
                Some(get_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("month"),
                Some(get_month),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("monthCode"),
                Some(get_month_code),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("daysInMonth"),
                Some(get_days_in_month),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("daysInYear"),
                Some(get_days_in_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("monthsInYear"),
                Some(get_months_in_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("inLeapYear"),
                Some(get_in_leap_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .static_method(Self::from, js_string!("from"), 2)
            .method(Self::with, js_string!("with"), 2)
            .method(Self::add, js_string!("add"), 2)
            .method(Self::subtract, js_string!("subtract"), 2)
            .method(Self::until, js_string!("until"), 2)
            .method(Self::since, js_string!("since"), 2)
            .method(Self::equals, js_string!("equals"), 1)
            .method(Self::to_string, js_string!("toString"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for PlainYearMonth {
    const LENGTH: usize = 2;
    const P: usize = 16;
    const SP: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_year_month;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
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
            None
        } else {
            // 6. Let ref be ? ToIntegerWithTruncation(referenceISODay).
            Some(super::to_integer_with_truncation(day, context)?)
        };

        // 3. Let y be ? ToIntegerWithTruncation(isoYear).
        let y = super::to_integer_with_truncation(args.get_or_undefined(0), context)?;
        // 4. Let m be ? ToIntegerWithTruncation(isoMonth).
        let m = super::to_integer_with_truncation(args.get_or_undefined(1), context)?;
        // 5. Let calendar be ? ToTemporalCalendarSlotValue(calendarLike, "iso8601").
        let calendar = to_temporal_calendar_slot_value(args.get_or_undefined(2))?;

        // 7. Return ? CreateTemporalYearMonth(y, m, calendar, ref, NewTarget).
        let inner =
            InnerYearMonth::new_with_overflow(y, m, ref_day, calendar, ArithmeticOverflow::Reject)?;

        create_temporal_year_month(inner, Some(new_target), context)
    }
}

// ==== `Temporal.PlainYearMonth` static Methods ====

impl PlainYearMonth {
    // 9.2.2 `Temporal.PlainYearMonth.from ( item [ , options ] )`
    fn from(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let item = args.get_or_undefined(0);
        // 1. If Type(item) is Object or Type(item) is String and item is not null, then

        let inner = if item.is_object() {
            // 9.2.2.2
            if let Some(data) = item
                .as_object()
                .and_then(JsObject::downcast_ref::<PlainYearMonth>)
            {
                // Perform ? [GetTemporalOverflowOption](https://tc39.es/proposal-temporal/#sec-temporal-gettemporaloverflowoption)(options).
                let options = get_options_object(args.get_or_undefined(1))?;
                let _ =
                    get_option::<ArithmeticOverflow>(&options, js_string!("overflow"), context)?;
                data.inner.clone()
            } else {
                let options = get_options_object(args.get_or_undefined(1))?;
                let overflow = get_option(&options, js_string!("overflow"), context)?
                    .unwrap_or(ArithmeticOverflow::Constrain);

                // a. Let calendar be ? ToTemporalCalendar(item).
                let calendar = to_temporal_calendar_slot_value(args.get_or_undefined(1))?;
                InnerYearMonth::new_with_overflow(
                    super::to_integer_with_truncation(
                        &item.get_v(js_string!("year"), context)?,
                        context,
                    )?,
                    super::to_integer_with_truncation(
                        &item.get_v(js_string!("month"), context)?,
                        context,
                    )?,
                    super::to_integer_with_truncation(
                        &item.get_v(js_string!("day"), context)?,
                        context,
                    )
                    .ok(),
                    calendar,
                    overflow,
                )?
            }
        } else if let Some(item_as_string) = item.as_string() {
            InnerYearMonth::from_str(item_as_string.to_std_string_escaped().as_str())?
        } else {
            return Err(JsNativeError::typ()
                .with_message("item must be an object, string, or null.")
                .into());
        };

        // b. Return ? ToTemporalYearMonth(item, calendar).
        create_temporal_year_month(inner, None, context)
    }
}

// ==== `PlainYearMonth` Accessor Implementations ====/

impl PlainYearMonth {
    fn get_internal_field(this: &JsValue, field: &DateTimeValues) -> JsResult<JsValue> {
        let year_month = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainYearMonth object.")
            })?;
        let inner = &year_month.inner;
        match field {
            DateTimeValues::Year => Ok(inner.iso_year().into()),
            DateTimeValues::Month => Ok(inner.iso_month().into()),
            DateTimeValues::MonthCode => {
                Ok(JsString::from(InnerYearMonth::month_code(inner)?.as_str()).into())
            }
            _ => unreachable!(),
        }
    }

    fn get_calendar_id(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(year_month) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainYearMonth object.")
                .into());
        };

        let calendar = year_month.borrow().data().inner.calendar().clone();
        Ok(js_string!(calendar.identifier()).into())
    }

    fn get_year(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Year)
    }

    fn get_month(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Month)
    }

    fn get_month_code(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::MonthCode)
    }

    fn get_days_in_year(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let year_month = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainYearMonth object.")
            })?;
        let inner = &year_month.inner;
        Ok(inner.get_days_in_year()?.into())
    }

    fn get_days_in_month(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let year_month = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainYearMonth object.")
            })?;
        let inner = &year_month.inner;
        Ok(inner.get_days_in_month()?.into())
    }

    fn get_months_in_year(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let year_month = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainYearMonth object.")
            })?;
        let inner = &year_month.inner;
        Ok(inner.get_months_in_year()?.into())
    }

    fn get_in_leap_year(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let year_month = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainYearMonth object.")
            })?;

        Ok(year_month.inner.in_leap_year().into())
    }
}

// ==== `PlainYearMonth` Method Implementations ====

impl PlainYearMonth {
    fn with(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let duration_like = args.get_or_undefined(0);
        let options = get_options_object(args.get_or_undefined(1))?;

        add_or_subtract_duration(true, this, duration_like, &options, context)
    }

    fn subtract(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let duration_like = args.get_or_undefined(0);
        let options = get_options_object(args.get_or_undefined(1))?;

        add_or_subtract_duration(false, this, duration_like, &options, context)
    }

    fn until(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn since(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn equals(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn to_string(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let YearMonth be the this value.
        // 2. Perform ? RequireInternalSlot(yearMonth, [[InitializedTemporalYearMonth]]).
        let year_month = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainYearMonth object.")
            })?;

        let inner = &year_month.inner;
        // 3. Set options to ? NormalizeOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(0))?;
        // 4. Let showCalendar be ? ToShowCalendarOption(options).
        // Get calendarName from the options object
        let show_calendar =
            get_option::<CalendarName>(&options, js_string!("calendarName"), context)?
                .unwrap_or(CalendarName::Auto);

        Ok(year_month_to_string(inner, show_calendar))
    }
}

// ==== Abstract Operations ====

// 9.5.2 `RegulateISOYearMonth ( year, month, overflow )`
// Implemented on `TemporalFields`.

// 9.5.6 `CreateTemporalYearMonth ( isoYear, isoMonth, calendar, referenceISODay [ , newTarget ] )`
pub(crate) fn create_temporal_year_month(
    ym: InnerYearMonth,
    new_target: Option<&JsValue>,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. If IsValidISODate(isoYear, isoMonth, referenceISODay) is false, throw a RangeError exception.
    // 2. If ! ISOYearMonthWithinLimits(isoYear, isoMonth) is false, throw a RangeError exception.

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
    let proto = get_prototype_from_constructor(
        &new_target,
        StandardConstructors::plain_year_month,
        context,
    )?;

    // 5. Set object.[[ISOYear]] to isoYear.
    // 6. Set object.[[ISOMonth]] to isoMonth.
    // 7. Set object.[[Calendar]] to calendar.
    // 8. Set object.[[ISODay]] to referenceISODay.

    let obj = JsObject::from_proto_and_data(proto, PlainYearMonth::new(ym));

    // 9. Return object.
    Ok(obj.into())
}

// 9.5.9 AddDurationToOrSubtractDurationFromPlainYearMonth ( operation, yearMonth, temporalDurationLike, options )
fn add_or_subtract_duration(
    is_addition: bool,
    this: &JsValue,
    duration_like: &JsValue,
    options: &JsObject,
    context: &mut Context,
) -> JsResult<JsValue> {
    let duration: Duration = if duration_like.is_object() {
        to_temporal_duration(duration_like, context)?
    } else if let Some(duration_string) = duration_like.as_string() {
        Duration::from_str(duration_string.to_std_string_escaped().as_str())?
    } else {
        return Err(JsNativeError::typ()
            .with_message("cannot handler string durations yet.")
            .into());
    };

    let overflow = get_option(options, js_string!("overflow"), context)?
        .unwrap_or(ArithmeticOverflow::Constrain);

    let year_month = this
        .as_object()
        .and_then(JsObject::downcast_ref::<PlainYearMonth>)
        .ok_or_else(|| {
            JsNativeError::typ().with_message("this value must be a PlainYearMonth object.")
        })?;

    let inner = &year_month.inner;
    let year_month_result = if is_addition {
        inner.add_duration(&duration, overflow)?
    } else {
        inner.subtract_duration(&duration, overflow)?
    };

    create_temporal_year_month(year_month_result, None, context)
}

fn year_month_to_string(inner: &InnerYearMonth, show_calendar: CalendarName) -> JsValue {
    // Let year be PadISOYear(yearMonth.[[ISOYear]]).
    let year = inner.padded_iso_year_string();
    // Let month be ToZeroPaddedDecimalString(yearMonth.[[ISOMonth]], 2).
    let month = inner.iso_month().to_string();

    // Let result be the string-concatenation of year, the code unit 0x002D (HYPHEN-MINUS), and month.
    let mut result = format!("{year}-{month:0>2}");

    // 5. If showCalendar is one of "always" or "critical", or if calendarIdentifier is not "iso8601", then
    // a. Let day be ToZeroPaddedDecimalString(yearMonth.[[ISODay]], 2).
    // b. Set result to the string-concatenation of result, the code unit 0x002D (HYPHEN-MINUS), and day.
    // 6. Let calendarString be FormatCalendarAnnotation(calendarIdentifier, showCalendar).
    // 7. Set result to the string-concatenation of result and calendarString.
    if matches!(
        show_calendar,
        CalendarName::Critical | CalendarName::Always | CalendarName::Auto
    ) && !(matches!(show_calendar, CalendarName::Auto) && inner.calendar_id() == "iso8601")
    {
        let calendar = inner.calendar_id();
        let calendar_string = calendar.to_string();
        let flag = if matches!(show_calendar, CalendarName::Critical) {
            "!"
        } else {
            ""
        };
        result.push_str(&format!("[{flag}c={calendar_string}]",));
    }
    // 8. Return result.
    js_string!(result).into()
}
