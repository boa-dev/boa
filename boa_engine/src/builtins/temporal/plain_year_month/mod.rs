//! Boa's implementation of the `Temporal.PlainYearMonth` builtin object.

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, ObjectData},
    property::Attribute,
    realm::Realm,
    string::{common::StaticJsStrings, utf16},
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

use super::plain_date::iso::IsoDateRecord;

/// The `Temporal.PlainYearMonth` object.
#[derive(Debug, Clone)]
pub struct PlainYearMonth {
    pub(crate) inner: IsoDateRecord,
    pub(crate) calendar: JsValue,
}

impl BuiltInObject for PlainYearMonth {
    const NAME: JsString = StaticJsStrings::PLAIN_YM;
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
        create_temporal_year_month(
            record,
            JsValue::from(js_string!("iso8601")),
            Some(new_target),
            context,
        )
    }
}

// ==== `PlainYearMonth` Accessor Implementations ====

impl PlainYearMonth {
    fn get_calendar_id(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_year(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_month(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_month_code(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_days_in_year(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_days_in_month(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_months_in_year(
        _this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    fn get_in_leap_year(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }
}

// ==== `PlainYearMonth` Method Implementations ====

impl PlainYearMonth {
    fn with(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn add(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn subtract(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn until(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn since(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }

    fn equals(_this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("not yet implemented.")
            .into())
    }
}

// ==== Abstract Operations ====

// 9.5.2 `RegulateISOYearMonth ( year, month, overflow )`
// Implemented on `TemporalFields`.

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
    let proto = get_prototype_from_constructor(
        &new_target,
        StandardConstructors::plain_year_month,
        context,
    )?;

    // 5. Set object.[[ISOYear]] to isoYear.
    // 6. Set object.[[ISOMonth]] to isoMonth.
    // 7. Set object.[[Calendar]] to calendar.
    // 8. Set object.[[ISODay]] to referenceISODay.

    let obj = JsObject::from_proto_and_data(
        proto,
        ObjectData::plain_year_month(PlainYearMonth {
            inner: year_month_record,
            calendar,
        }),
    );

    // 9. Return object.
    Ok(obj.into())
}
