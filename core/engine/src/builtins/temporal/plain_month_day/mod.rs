//! Boa's implementation of the ECMAScript `Temporal.PlainMonthDay` builtin object.
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
    partial::PartialDate,
    Calendar, PlainMonthDay as InnerMonthDay, TinyAsciiStr,
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
            .static_method(Self::from, js_string!("from"), 1)
            .method(Self::equals, js_string!("equals"), 1)
            .method(Self::to_string, js_string!("toString"), 0)
            .method(Self::to_locale_string, js_string!("toLocaleString"), 0)
            .method(Self::to_json, js_string!("toJSON"), 0)
            .method(Self::value_of, js_string!("valueOf"), 0)
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

        let ref_year = args
            .get_or_undefined(3)
            .map(|v| {
                let finite = v.to_finitef64(context)?;
                Ok::<i32, JsError>(finite.as_integer_with_truncation::<i32>())
            })
            .transpose()?;

        // We can ignore 2 as the underlying temporal library handles the reference year
        let m = args
            .get_or_undefined(0)
            .to_finitef64(context)?
            .as_integer_with_truncation::<u8>();

        let d = args
            .get_or_undefined(1)
            .to_finitef64(context)?
            .as_integer_with_truncation::<u8>();

        let calendar = args
            .get_or_undefined(2)
            .map(|s| {
                s.as_string()
                    .map(JsString::to_std_string_lossy)
                    .ok_or_else(|| JsNativeError::typ().with_message("calendar must be a string."))
            })
            .transpose()?
            .map(|s| Calendar::from_utf8(s.as_bytes()))
            .transpose()?
            .unwrap_or_default();

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

// ==== `Temporal.PlainMonthDay` static Methods ====

impl PlainMonthDay {
    // 10.2.2 Temporal.PlainMonthDay.from ( item [ , options ] )
    fn from(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let options = get_options_object(args.get_or_undefined(1))?;
        let item = args.get_or_undefined(0);
        let inner = to_temporal_month_day(item, Some(options), context)?;
        create_temporal_month_day(inner, None, context)
    }
}

// ==== `PlainMonthDay` Accessor Implementations ====

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

    fn get_calendar_id(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let month_day = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainMonthDay object.")
            })?;
        Ok(js_string!(month_day.inner.calendar().identifier()).into())
    }

    fn get_day(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Day)
    }

    fn get_month_code(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::MonthCode)
    }
}

// ==== `Temporal.PlainMonthDay` Methods ====

impl PlainMonthDay {
    fn equals(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let month_day = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainMonthDay object.")
            })?;

        let other = to_temporal_month_day(args.get_or_undefined(0), None, context)?;

        Ok((month_day.inner == other).into())
    }

    /// 10.3.8 `Temporal.PlainMonthDay.prototype.toString ( [ options ] )`
    fn to_string(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let monthDay be the this value.
        // 2. Perform ? RequireInternalSlot(monthDay, [[InitializedTemporalMonthDay]]).
        let month_day = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainMonthDay object.")
            })?;

        // 3. Set options to ? NormalizeOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(0))?;
        // 4. Let showCalendar be ? ToShowCalendarOption(options).
        // Get calendarName from the options object
        let show_calendar =
            get_option::<DisplayCalendar>(&options, js_string!("calendarName"), context)?
                .unwrap_or(DisplayCalendar::Auto);

        let ixdtf = month_day.inner.to_ixdtf_string(show_calendar);
        Ok(JsString::from(ixdtf).into())
    }

    /// 10.3.9 `Temporal.PlainMonthDay.prototype.toLocaleString ( [ locales [ , options ] ] )`
    pub(crate) fn to_locale_string(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // TODO: Update for ECMA-402 compliance
        let month_day = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainMonthDay object.")
            })?;

        Ok(JsString::from(month_day.inner.to_string()).into())
    }

    /// 10.3.10 `Temporal.PlainMonthDay.prototype.toJSON ( )`
    pub(crate) fn to_json(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let month_day = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a PlainMonthDay object.")
            })?;

        Ok(JsString::from(month_day.inner.to_string()).into())
    }

    /// 9.3.11 `Temporal.PlainMonthDay.prototype.valueOf ( )`
    pub(crate) fn value_of(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("`valueOf` not supported by Temporal built-ins. See 'compare', 'equals', or `toString`")
            .into())
    }
}

// ==== `PlainMonthDay` Abstract Operations ====

pub(crate) fn create_temporal_month_day(
    inner: InnerMonthDay,
    new_target: Option<&JsValue>,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. If IsValidISODate(referenceISOYear, isoMonth, isoDay) is false, throw a RangeError exception.
    // 2. If ISODateTimeWithinLimits(referenceISOYear, isoMonth, isoDay, 12, 0, 0, 0, 0, 0) is false, throw a RangeError exception.

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
    options: Option<JsObject>,
    context: &mut Context,
) -> JsResult<InnerMonthDay> {
    let options = options.unwrap_or(JsObject::with_null_proto());
    let overflow = get_option::<ArithmeticOverflow>(&options, js_string!("overflow"), context)?
        .unwrap_or(ArithmeticOverflow::Constrain);

    // get the calendar property (string) from the item object
    let calender_id = item.get_v(js_string!("calendar"), context)?;
    let calendar = to_temporal_calendar_slot_value(&calender_id)?;

    if let Some(obj) = item.as_object() {
        if let Some(md) = obj.downcast_ref::<PlainMonthDay>() {
            return Ok(md.inner.clone());
        }
        let day = obj
            .get(js_string!("day"), context)?
            .map(|v| {
                let finite = v.to_finitef64(context)?;
                finite
                    .as_positive_integer_with_truncation::<u8>()
                    .map_err(JsError::from)
            })
            .transpose()?;

        let month = obj
            .get(js_string!("month"), context)?
            .map(|v| {
                let finite = v.to_finitef64(context)?;
                finite
                    .as_positive_integer_with_truncation::<u8>()
                    .map_err(JsError::from)
            })
            .transpose()?;

        let month_code = obj
            .get(js_string!("monthCode"), context)?
            .map(|v| {
                let primitive = v.to_primitive(context, crate::value::PreferredType::String)?;
                let Some(month_code) = primitive.as_string() else {
                    return Err(JsNativeError::typ()
                        .with_message("The monthCode field value must be a string.")
                        .into());
                };
                TinyAsciiStr::<4>::from_str(&month_code.to_std_string_escaped())
                    .map_err(|e| JsError::from(JsNativeError::typ().with_message(e.to_string())))
            })
            .transpose()?;

        let year = obj
            .get(js_string!("year"), context)?
            .map_or(Ok::<i32, JsError>(1972), |v| {
                let finite = v.to_finitef64(context)?;
                Ok(finite.as_integer_with_truncation::<i32>())
            })?;

        let partial_date = &PartialDate {
            month,
            day,
            year: Some(year),
            month_code,
            ..Default::default()
        };

        return Ok(calendar.month_day_from_partial(partial_date, overflow)?);
    }

    let Some(md_string) = item.as_string() else {
        return Err(JsNativeError::typ()
            .with_message("item must be an object or a string")
            .into());
    };

    Ok(InnerMonthDay::from_str(
        md_string.to_std_string_escaped().as_str(),
    )?)
}
