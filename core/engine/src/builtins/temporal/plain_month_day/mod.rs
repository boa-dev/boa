//! Boa's implementation of the ECMAScript `Temporal.PlainMonthDay` builtin object.
#![allow(dead_code, unused_variables)]
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
    options::{ArithmeticOverflow, CalendarName},
    partial::PartialDate,
    PlainDateTime, PlainMonthDay as InnerMonthDay, TinyAsciiStr,
};

use super::{calendar::to_temporal_calendar_slot_value, DateTimeValues};

/// The `Temporal.PlainMonthDay` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)] // TODO: Remove this!!! `InnerMonthDay` could contain `Trace` types.
pub struct PlainMonthDay {
    pub(crate) inner: InnerMonthDay,
}

impl PlainMonthDay {
    fn new(inner: InnerMonthDay) -> Self {
        Self { inner }
    }
}

// ==== `Temporal.PlainMonthDay` static Methods ====
impl PlainMonthDay {
    // 10.2.2 Temporal.PlainMonthDay.from ( item [ , options ] )
    fn from(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let options = get_options_object(args.get_or_undefined(1))?;
        let item = args.get_or_undefined(0);
        to_temporal_month_day(item, &options, context)
    }
}

// === `PlainMonthDay` Accessor Implementations ===== /

impl PlainMonthDay {
    fn get_internal_field(this: &JsValue, field: &DateTimeValues) -> JsResult<JsValue> {
        let month_day = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainMonthDay object.")
            })?;
        let inner = &month_day.inner;
        match field {
            DateTimeValues::Day => Ok(inner.iso_day().into()),
            DateTimeValues::MonthCode => Ok(js_string!(inner.month_code()?.to_string()).into()),
            _ => unreachable!(),
        }
    }

    fn get_day(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Day)
    }

    fn get_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Year)
    }

    fn get_month_code(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::MonthCode)
    }

    fn get_calendar_id(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let month_day = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainMonthDay object.")
            })?;
        let inner = &month_day.inner;
        Ok(js_string!(inner.calendar().identifier()).into())
    }
}

// ==== `Temporal.PlainMonthDay` Methods ====
impl PlainMonthDay {
    // 10.3.7 Temporal.PlainMonthDay.prototype.toString ( [ options ] )
    fn to_string(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let monthDay be the this value.
        // 2. Perform ? RequireInternalSlot(monthDay, [[InitializedTemporalMonthDay]]).
        let month_day = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainMonthDay object.")
            })?;
        let inner = &month_day.inner;
        // 3. Set options to ? NormalizeOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(0))?;
        // 4. Let showCalendar be ? ToShowCalendarOption(options).
        // Get calendarName from the options object
        let show_calendar =
            get_option::<CalendarName>(&options, js_string!("calendarName"), context)?
                .unwrap_or(CalendarName::Auto);

        Ok(month_day_to_string(inner, show_calendar))
    }
}

impl BuiltInObject for PlainMonthDay {
    const NAME: JsString = StaticJsStrings::PLAIN_MD_NAME;
}

impl IntrinsicObject for PlainMonthDay {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");
        let get_day = BuiltInBuilder::callable(realm, Self::get_day)
            .name(js_string!("get day"))
            .build();

        let get_month_code = BuiltInBuilder::callable(realm, Self::get_month_code)
            .name(js_string!("get monthCode"))
            .build();

        let get_calendar_id = BuiltInBuilder::callable(realm, Self::get_calendar_id)
            .name(js_string!("get calendarId"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                StaticJsStrings::PLAIN_MD_TAG,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("day"),
                Some(get_day),
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
                js_string!("calendarId"),
                Some(get_calendar_id),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::to_string, js_string!("toString"), 1)
            .static_method(Self::from, js_string!("from"), 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for PlainMonthDay {
    const LENGTH: usize = 2;
    const P: usize = 5;
    const SP: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_month_day;

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

        let year = args.get_or_undefined(3);
        let ref_year = if year.is_undefined() {
            None
        } else {
            Some(super::to_integer_with_truncation(year, context)?)
        };

        // We can ignore 2 as the underlying temporal library handles the reference year
        let m = super::to_integer_with_truncation(args.get_or_undefined(0), context)?;
        let d = super::to_integer_with_truncation(args.get_or_undefined(1), context)?;
        let calendar = to_temporal_calendar_slot_value(args.get_or_undefined(2))?;
        let inner = InnerMonthDay::new_with_overflow(
            m,
            d,
            calendar,
            ArithmeticOverflow::Constrain,
            ref_year,
        )?;
        create_temporal_month_day(inner, Some(new_target), context)
    }
}

// ==== `PlainMonthDay` Abstract Operations ====

fn month_day_to_string(inner: &InnerMonthDay, show_calendar: CalendarName) -> JsValue {
    // Let month be monthDay.[[ISOMonth]] formatted as a two-digit decimal number, padded to the left with a zero if necessary
    let month = inner.iso_month().to_string();

    // 2. Let day be ! FormatDayOfMonth(monthDay.[[ISODay]]).
    let day = inner.iso_day().to_string();

    // 3. Let result be the string-concatenation of month and the code unit 0x002D (HYPHEN-MINUS).
    let mut result = format!("{month:0>2}-{day:0>2}");

    // 4. Let calendarId be monthDay.[[Calendar]].[[id]].
    let calendar_id = inner.calendar().identifier();

    // 5. Let calendar be monthDay.[[Calendar]].
    // 6. If showCalendar is "auto", then
    //     a. Set showCalendar to "always".
    // 7. If showCalendar is "always", then
    //     a. Let calendarString be ! FormatCalendarAnnotation(calendar).
    //     b. Set result to the string-concatenation of result, the code unit 0x0040 (COMMERCIAL AT), and calendarString.
    if (matches!(
        show_calendar,
        CalendarName::Critical | CalendarName::Always | CalendarName::Auto
    )) && !(matches!(show_calendar, CalendarName::Auto) && calendar_id == "iso8601")
    {
        let year = inner.iso_year().to_string();
        let flag = if matches!(show_calendar, CalendarName::Critical) {
            "!"
        } else {
            ""
        };
        result = format!("{year}-{result}[{flag}u-ca={calendar_id}]");
    }
    // 8. Return result.
    js_string!(result).into()
}

pub(crate) fn create_temporal_month_day(
    inner: InnerMonthDay,
    new_target: Option<&JsValue>,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. If IsValidISODate(referenceISOYear, isoMonth, isoDay) is false, throw a RangeError exception.
    // 2. If ISODateTimeWithinLimits(referenceISOYear, isoMonth, isoDay, 12, 0, 0, 0, 0, 0) is false, throw a RangeError exception.
    if !PlainDateTime::validate(&inner) {
        return Err(JsNativeError::range()
            .with_message("PlainMonthDay does not hold a valid ISO date time.")
            .into());
    }

    // 3. If newTarget is not present, set newTarget to %Temporal.PlainMonthDay%.
    let new_target = if let Some(target) = new_target {
        target.clone()
    } else {
        context
            .realm()
            .intrinsics()
            .constructors()
            .plain_month_day()
            .constructor()
            .into()
    };

    // 4. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.PlainMonthDay.prototype%", « [[InitializedTemporalMonthDay]], [[ISOMonth]], [[ISODay]], [[ISOYear]], [[Calendar]] »).
    let proto = get_prototype_from_constructor(
        &new_target,
        StandardConstructors::plain_month_day,
        context,
    )?;

    // 5. Set object.[[ISOMonth]] to isoMonth.
    // 6. Set object.[[ISODay]] to isoDay.
    // 7. Set object.[[Calendar]] to calendar.
    // 8. Set object.[[ISOYear]] to referenceISOYear.
    let obj = JsObject::from_proto_and_data(proto, PlainMonthDay::new(inner));

    // 9. Return object.
    Ok(obj.into())
}

fn to_temporal_month_day(
    item: &JsValue,
    options: &JsObject,
    context: &mut Context,
) -> JsResult<JsValue> {
    let overflow = get_option::<ArithmeticOverflow>(options, js_string!("overflow"), context)?
        .unwrap_or(ArithmeticOverflow::Constrain);

    // get the calendar property (string) from the item object
    let calender_id = item.get_v(js_string!("calendar"), context)?;
    let calendar = to_temporal_calendar_slot_value(&calender_id)?;

    let inner = if let Some(item_obj) = item
        .as_object()
        .and_then(JsObject::downcast_ref::<PlainMonthDay>)
    {
        item_obj.inner.clone()
    } else if let Some(item_string) = item.as_string() {
        InnerMonthDay::from_str(item_string.to_std_string_escaped().as_str())?
    } else if item.is_object() {
        let day = item
            .get_v(js_string!("day"), context)
            .expect("Day not found")
            .to_i32(context)
            .expect("Cannot convert day to i32");
        let month = item
            .get_v(js_string!("month"), context)
            .expect("Month not found")
            .to_i32(context)
            .expect("Cannot convert month to i32");

        let month_code = item
            .get_v(js_string!("monthCode"), context)
            .expect("monthCode not found");
        let resolved_month_code = if month_code.is_undefined() {
            None
        } else {
            TinyAsciiStr::<4>::from_str(
                &month_code
                    .to_string(context)
                    .expect("Cannot convert monthCode to string")
                    .to_std_string_escaped(),
            )
            .map_err(|e| JsError::from(JsNativeError::range().with_message(e.to_string())))
            .ok()
        };
        let year = item.get_v(js_string!("year"), context).map_or(1972, |val| {
            val.to_i32(context).expect("Cannot convert year to i32")
        });

        let partial_date = &PartialDate {
            month: Some(month),
            day: Some(day),
            year: Some(year),
            month_code: resolved_month_code,
            ..Default::default()
        };

        calendar.month_day_from_partial(partial_date, overflow)?
    } else {
        return Err(JsNativeError::typ()
            .with_message("item must be an object or a string")
            .into());
    };

    create_temporal_month_day(inner, None, context)
}
