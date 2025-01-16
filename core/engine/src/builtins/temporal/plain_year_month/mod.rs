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
    Context, JsArgs, JsData, JsError, JsNativeError, JsObject, JsResult, JsString, JsSymbol,
    JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

use temporal_rs::{
    options::{ArithmeticOverflow, DisplayCalendar},
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
            .method(Self::with, js_string!("with"), 1)
            .method(Self::add, js_string!("add"), 1)
            .method(Self::subtract, js_string!("subtract"), 1)
            .method(Self::until, js_string!("until"), 1)
            .method(Self::since, js_string!("since"), 1)
            .method(Self::equals, js_string!("equals"), 1)
            .method(Self::to_string, js_string!("toString"), 0)
            .method(Self::to_json, js_string!("toJSON"), 0)
            .method(Self::value_of, js_string!("valueOf"), 0)
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

        // 2. If referenceISODay is undefined, then
        // a. Set referenceISODay to 1𝔽.
        // 3. Let y be ? ToIntegerWithTruncation(isoYear).
        let y = args
            .get_or_undefined(0)
            .to_finitef64(context)?
            .as_integer_with_truncation::<i32>();

        // 4. Let m be ? ToIntegerWithTruncation(isoMonth).
        let m = args
            .get_or_undefined(1)
            .to_finitef64(context)?
            .as_integer_with_truncation::<u8>();

        // 5. Let calendar be ? ToTemporalCalendarSlotValue(calendarLike, "iso8601").
        let calendar = to_temporal_calendar_slot_value(args.get_or_undefined(2))?;

        // 6. Let ref be ? ToIntegerWithTruncation(referenceISODay).
        let ref_day = args
            .get_or_undefined(3)
            .map(|v| {
                let finite = v.to_finitef64(context)?;
                Ok::<u8, JsError>(finite.as_integer_with_truncation::<u8>())
            })
            .transpose()?;

        // 7. Return ? CreateTemporalYearMonth(y, m, calendar, ref, NewTarget).
        let inner = InnerYearMonth::new_with_overflow(
            y,
            m,
            ref_day.map(Into::into),
            calendar,
            ArithmeticOverflow::Reject,
        )?;

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

                let year = item
                    .get_v(js_string!("year"), context)?
                    .map(|v| {
                        let finite = v.to_finitef64(context)?;
                        Ok::<i32, JsError>(finite.as_integer_with_truncation::<i32>())
                    })
                    .transpose()?
                    .unwrap_or_default();

                let month = item
                    .get_v(js_string!("month"), context)?
                    .map(|v| {
                        let finite = v.to_finitef64(context)?;
                        Ok::<u8, JsError>(finite.as_integer_with_truncation::<u8>())
                    })
                    .transpose()?
                    .unwrap_or_default();

                let ref_day = item
                    .get_v(js_string!("day"), context)?
                    .map(|v| {
                        let finite = v.to_finitef64(context)?;
                        Ok::<u8, JsError>(finite.as_integer_with_truncation::<u8>())
                    })
                    .transpose()?;

                // a. Let calendar be ? ToTemporalCalendar(item).
                let calendar = to_temporal_calendar_slot_value(args.get_or_undefined(1))?;
                InnerYearMonth::new_with_overflow(
                    year,
                    month,
                    ref_day.map(Into::into),
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
        Err(JsNativeError::error()
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

    /// `9.3.19 Temporal.PlainYearMonth.prototype.toString ( [ options ] )`
    fn to_string(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let YearMonth be the this value.
        // 2. Perform ? RequireInternalSlot(yearMonth, [[InitializedTemporalYearMonth]]).
        let year_month = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainYearMonth object.")
            })?;

        // 3. Set options to ? NormalizeOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(0))?;
        // 4. Let showCalendar be ? ToShowCalendarOption(options).
        // Get calendarName from the options object
        let show_calendar =
            get_option::<DisplayCalendar>(&options, js_string!("calendarName"), context)?
                .unwrap_or(DisplayCalendar::Auto);

        let ixdtf = year_month.inner.to_ixdtf_string(show_calendar);
        Ok(JsString::from(ixdtf).into())
    }

    /// `9.3.21 Temporal.PlainYearMonth.prototype.toJSON ( )`
    pub(crate) fn to_json(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let year_month = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainYearMonth object.")
            })?;

        Ok(JsString::from(year_month.inner.to_string()).into())
    }

    /// `9.3.22 Temporal.PlainYearMonth.prototype.valueOf ( )`
    pub(crate) fn value_of(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("`valueOf` not supported by Temporal built-ins. See 'compare', 'equals', or `toString`")
            .into())
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
