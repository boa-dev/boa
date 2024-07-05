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
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_macros::js_str;
use boa_profiler::Profiler;

use temporal_rs::{
    components::{
        calendar::{Calendar, GetTemporalCalendar},
        DateTime, MonthDay as InnerMonthDay,
    },
    iso::IsoDateSlots,
    options::ArithmeticOverflow,
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
        let inner = if item.is_object() {
            let overflow = get_option(&options, js_str!("overflow"), context)?
                .unwrap_or(ArithmeticOverflow::Constrain);

            let calendar = to_temporal_calendar_slot_value(args.get_or_undefined(1))?;

            InnerMonthDay::new(
                item.get_v(js_str!("month"), context)
                    .expect("Month not found")
                    .to_i32(context)
                    .expect("Cannot convert month to i32"),
                item.get_v(js_str!("day"), context)
                    .expect("Day not found")
                    .to_i32(context)
                    .expect("Cannot convert day to i32"),
                calendar,
                overflow,
            )?
        } else if item.is_string() {
            let item_str = &item
                .as_string()
                .expect("item is not a string")
                .to_std_string_escaped();
            InnerMonthDay::from_str(item_str)?
        } else {
            return Err(JsNativeError::typ()
                .with_message("item must be an object or a string")
                .into());
        };

        create_temporal_month_day(inner, None, context)
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
            DateTimeValues::Day => Ok(inner.day().into()),
            DateTimeValues::MonthCode => {
                Ok(JsString::from(InnerMonthDay::month_code(inner)?.as_str()).into())
            }
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
}
impl IsoDateSlots for JsObject<PlainMonthDay> {
    fn iso_date(&self) -> temporal_rs::iso::IsoDate {
        self.borrow().data().inner.iso_date()
    }
}

impl GetTemporalCalendar for JsObject<PlainMonthDay> {
    fn get_calendar(&self) -> Calendar {
        self.borrow().data().inner.get_calendar()
    }
}

impl BuiltInObject for PlainMonthDay {
    const NAME: JsString = StaticJsStrings::PLAIN_MD_NAME;
}

impl IntrinsicObject for PlainMonthDay {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");
        let get_day = BuiltInBuilder::callable(realm, Self::get_day)
            .name(js_string!("get month"))
            .build();

        let get_month_code = BuiltInBuilder::callable(realm, Self::get_month_code)
            .name(js_string!("get monthCode"))
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
            .static_method(Self::from, js_string!("from"), 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for PlainMonthDay {
    const LENGTH: usize = 2;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_month_day;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::range()
            .with_message("Not yet implemented.")
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
    if !DateTime::validate(&inner) {
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
