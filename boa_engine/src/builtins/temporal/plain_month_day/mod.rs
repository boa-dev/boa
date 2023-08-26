#![allow(dead_code, unused_variables)]
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    Context, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;
use icu_datetime::input::DateInput;

use super::{plain_date::iso::IsoDateRecord, plain_date_time::iso::IsoDateTimeRecord};

/// The `Temporal.PlainMonthDay` object.
#[derive(Debug, Clone)]
pub struct PlainMonthDay {
    iso_record: IsoDateRecord,
    pub(crate) calendar: JsValue,
}

impl BuiltInObject for PlainMonthDay {
    const NAME: &'static str = "Temporal.PlainMonthDay";
}

impl IntrinsicObject for PlainMonthDay {
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

impl BuiltInConstructor for PlainMonthDay {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_month_day;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }
}

// ==== `PlainMonthDay` Abstract Operations ====

pub(crate) fn create_temporal_month_day(
    iso: IsoDateRecord,
    calendar: JsValue,
    new_target: Option<&JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. If IsValidISODate(referenceISOYear, isoMonth, isoDay) is false, throw a RangeError exception.
    if iso.is_valid() {
        return Err(JsNativeError::range()
            .with_message("PlainMonthDay is not a valid ISO date.")
            .into());
    }

    // 2. If ISODateTimeWithinLimits(referenceISOYear, isoMonth, isoDay, 12, 0, 0, 0, 0, 0) is false, throw a RangeError exception.
    let iso_date_time = IsoDateTimeRecord::default()
        .with_date(iso.year(), iso.month(), iso.day())
        .with_time(12, 0, 0, 0, 0, 0);

    if !iso_date_time.is_valid() {
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
    let new_month_day = get_prototype_from_constructor(
        &new_target,
        StandardConstructors::plain_month_day,
        context,
    )?;

    let mut obj = new_month_day.borrow_mut();
    let month_day = obj
        .as_plain_month_day_mut()
        .expect("this value must be a date");

    // 5. Set object.[[ISOMonth]] to isoMonth.
    // 6. Set object.[[ISODay]] to isoDay.
    // 7. Set object.[[Calendar]] to calendar.
    // 8. Set object.[[ISOYear]] to referenceISOYear.
    month_day.iso_record = iso;
    month_day.calendar = calendar;

    drop(obj);

    // 9. Return object.
    Ok(new_month_day.into())
}
