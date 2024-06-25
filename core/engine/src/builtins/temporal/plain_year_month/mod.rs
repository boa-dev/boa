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
use boa_macros::js_str;
use boa_profiler::Profiler;

use temporal_rs::{
    iso::IsoDateSlots,
    iso::IsoDateSlots,
    options::ArithmeticOverflow,
    {
        components::{
            calendar::{Calendar as InnerCalendar, GetTemporalCalendar},
            YearMonth as InnerYearMonth,
        },
        options::ArithmeticOverflow,
    },
};

use super::calendar;

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

impl IsoDateSlots for JsObject<PlainYearMonth> {
    fn iso_date(&self) -> temporal_rs::iso::IsoDate {
        self.borrow().data().inner.iso_date()
    }
}

impl GetTemporalCalendar for JsObject<PlainYearMonth> {
    fn get_calendar(&self) -> InnerCalendar {
        self.borrow().data().inner.get_calendar()
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
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for PlainYearMonth {
    const LENGTH: usize = 2;

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
        let calendar = calendar::to_temporal_calendar_slot_value(args.get_or_undefined(2))?;

        // 7. Return ? CreateTemporalYearMonth(y, m, calendar, ref, NewTarget).
        let inner = InnerYearMonth::new(y, m, ref_day, calendar, ArithmeticOverflow::Reject)?;

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
            let options = get_options_object(args.get_or_undefined(1))?;
            let overflow = get_option(&options, js_str!("overflow"), context)?
                .unwrap_or(ArithmeticOverflow::Constrain);

            // a. Let calendar be ? ToTemporalCalendar(item).
            let calendar = to_temporal_calendar_slot_value(args.get_or_undefined(1), context)?;
            InnerYearMonth::new(
                item.get_v(js_str!("year"), context)
                    .expect("Year not found")
                    .to_i32(context)
                    .expect("Cannot convert year to i32"),
                item.get_v(js_str!("month"), context)
                    .expect("Month not found")
                    .to_i32(context)
                    .expect("Cannot convert month to i32"),
                item.get_v(js_str!("day"), context)
                    .map_or(Some(1), |x| x.to_i32(context).ok()),
                calendar,
                overflow,
            )?
        } else if item.is_string() {
            let item_str = &item
                .as_string()
                .expect("Value passed not a string")
                .to_std_string_escaped();

            InnerYearMonth::from_str(item_str)?
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
            DateTimeValues::Year => Ok(inner.year().into()),
            DateTimeValues::Month => Ok(inner.month().into()),
            _ => unreachable!(),
        }
    }

    fn get_calendar_id(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(year_month) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainYearMonth object.")
                .into());
        };

        Ok(year_month
            .get_calendar()
            .identifier(context)
            .map(JsString::from)?
            .into())
    }

    fn get_year(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Year)
    }

    fn get_month(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Month)
    }

    fn get_month_code(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(year_month) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainYearMonth object.")
                .into());
        };

        Ok(JsString::from(
            InnerYearMonth::<JsObject>::contextual_month_code(&year_month, context)?.as_str(),
        )
        .into())
    }

    fn get_days_in_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(year_month) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainYearMonth object.")
                .into());
        };

        Ok(InnerYearMonth::<JsObject>::contextual_get_days_in_year(&year_month, context)?.into())
    }

    fn get_days_in_month(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(year_month) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainYearMonth object.")
                .into());
        };

        Ok(InnerYearMonth::<JsObject>::contextual_get_days_in_month(&year_month, context)?.into())
    }

    fn get_months_in_year(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(year_month) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainYearMonth object.")
                .into());
        };

        Ok(InnerYearMonth::<JsObject>::contextual_get_months_in_year(&year_month, context)?.into())
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

        return add_or_subtract_duration(true, this, duration_like, &options, context);
    }

    fn subtract(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let duration_like = args.get_or_undefined(0);
        let options = get_options_object(args.get_or_undefined(1))?;

        return add_or_subtract_duration(false, this, duration_like, &options, context);
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
    } else if duration_like.is_string() {
        Duration::from_str(
            duration_like
                .as_string()
                .expect("Value passed not a string")
                .to_std_string_escaped()
                .as_str(),
        )
        .expect("Unable to parse Duration from string")
    } else {
        return Err(JsNativeError::typ()
            .with_message("cannot handler string durations yet.")
            .into());
    };

    let overflow = get_option(&options, js_str!("overflow"), context)?
        .unwrap_or(ArithmeticOverflow::Constrain);

    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

    let Ok(year_month) = obj.clone().downcast::<PlainYearMonth>() else {
        return Err(JsNativeError::typ()
            .with_message("the this object must be a PlainYearMonth object.")
            .into());
    };

    let year_month_result = if is_addition {
        InnerYearMonth::<JsObject>::add_duration(&year_month, duration, overflow, context)
            .expect("Error adding duration to year month")
    } else {
        InnerYearMonth::<JsObject>::subtract_duration(&year_month, duration, overflow, context)
            .expect("Error subtracting duration from year month")
    };

    create_temporal_year_month(year_month_result, None, context)
}
