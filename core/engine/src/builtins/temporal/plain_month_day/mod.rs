//! Boa's implementation of the ECMAScript `Temporal.PlainMonthDay` builtin object.
#![allow(dead_code, unused_variables)]
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

use temporal_rs::{
    components::{
        calendar::{Calendar, GetTemporalCalendar},
        DateTime, MonthDay as InnerMonthDay,
    },
    iso::IsoDateSlots,
};

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

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                StaticJsStrings::PLAIN_MD_TAG,
                Attribute::CONFIGURABLE,
            )
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
    if DateTime::validate(&inner) {
        return Err(JsNativeError::range()
            .with_message("PlainMonthDay is not a valid ISO date time.")
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
