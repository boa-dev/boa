//! The ECMAScript `Temporal` stage 3 built-in implementation.
//!
//! More information:
//!
//! [spec]: https://tc39.es/proposal-temporal/

mod calendar;
mod duration;
mod error;
mod instant;
mod now;
mod options;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod time_zone;
mod zoned_date_time;

#[cfg(test)]
mod tests;

pub use self::{
    duration::*, instant::*, now::*, plain_date::*, plain_date_time::*, plain_month_day::*,
    plain_time::*, plain_year_month::*, zoned_date_time::*,
};

use crate::{
    builtins::{iterable::IteratorRecord, BuiltInBuilder, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::Type,
    Context, JsBigInt, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;
use temporal_rs::{PlainDate as TemporalDate, ZonedDateTime as TemporalZonedDateTime, NS_PER_DAY};

// TODO: Remove in favor of `temporal_rs`
pub(crate) fn ns_max_instant() -> JsBigInt {
    JsBigInt::from(i128::from(NS_PER_DAY) * 100_000_000_i128)
}

// TODO: Remove in favor of `temporal_rs`
pub(crate) fn ns_min_instant() -> JsBigInt {
    JsBigInt::from(i128::from(NS_PER_DAY) * -100_000_000_i128)
}

// An enum representing common fields across `Temporal` objects.
#[allow(unused)]
pub(crate) enum DateTimeValues {
    Year,
    Month,
    MonthCode,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

/// The [`Temporal`][spec] builtin object.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Temporal;

impl BuiltInObject for Temporal {
    const NAME: JsString = StaticJsStrings::TEMPORAL;
}

impl IntrinsicObject for Temporal {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("Now"),
                realm.intrinsics().objects().now(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("Calendar"),
                realm.intrinsics().constructors().calendar().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("Duration"),
                realm.intrinsics().constructors().duration().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("Instant"),
                realm.intrinsics().constructors().instant().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("PlainDate"),
                realm.intrinsics().constructors().plain_date().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("PlainDateTime"),
                realm
                    .intrinsics()
                    .constructors()
                    .plain_date_time()
                    .constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("PlainMonthDay"),
                realm
                    .intrinsics()
                    .constructors()
                    .plain_month_day()
                    .constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("PlainTime"),
                realm.intrinsics().constructors().plain_time().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("PlainYearMonth"),
                realm
                    .intrinsics()
                    .constructors()
                    .plain_year_month()
                    .constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("TimeZone"),
                realm.intrinsics().constructors().time_zone().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("ZonedDateTime"),
                realm
                    .intrinsics()
                    .constructors()
                    .zoned_date_time()
                    .constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().temporal()
    }
}

// -- Temporal Abstract Operations --

/// Abstract operation `ToZeroPaddedDecimalString ( n, minLength )`
///
/// The abstract operation `ToZeroPaddedDecimalString` takes arguments `n` (a non-negative integer)
/// and `minLength` (a non-negative integer) and returns a String.
fn to_zero_padded_decimal_string(n: u64, min_length: usize) -> String {
    format!("{n:0min_length$}")
}

/// Abstract Operation 13.1 [`IteratorToListOfType`][proposal]
///
/// [proposal]: https://tc39.es/proposal-temporal/#sec-iteratortolistoftype
pub(crate) fn _iterator_to_list_of_types(
    iterator: &mut IteratorRecord,
    element_types: &[Type],
    context: &mut Context,
) -> JsResult<Vec<JsValue>> {
    // 1. Let values be a new empty List.
    let mut values = Vec::new();

    // 2. Repeat,
    //     a. Let next be ? IteratorStepValue(iteratorRecord).
    while let Some(next) = iterator.step_value(context)? {
        // c. If Type(next) is not an element of elementTypes, then

        if element_types.contains(&next.get_type()) {
            //     i. Let completion be ThrowCompletion(a newly created TypeError object).
            let completion = JsNativeError::typ()
                .with_message("IteratorNext is not within allowed type values.");

            //     ii. Return ? IteratorClose(iteratorRecord, completion).
            let _never = iterator.close(Err(completion.into()), context)?;
        }

        // d. Append next to the end of the List values.
        values.push(next);
    }

    // b. If next is done, then
    //     i. Return values.
    Ok(values)
}

// Abstract Operation 13.3 `EpochDaysToEpochMs`
// Migrated to `temporal_rs`

// 13.4 Date Equations
// implemented in temporal/date_equations.rs

// Abstract Operation 13.5 `GetOptionsObject ( options )`
// Implemented in builtin/options.rs

// 13.6 `GetOption ( options, property, type, values, default )`
// Implemented in builtin/options.rs

// 13.7 `ToTemporalOverflow (options)`
// Now implemented in temporal/options.rs

// 13.10 `ToTemporalRoundingMode ( normalizedOptions, fallback )`
// Now implemented in builtin/options.rs

// 13.11 `NegateTemporalRoundingMode ( roundingMode )`
// Now implemented in builtin/options.rs

// 13.16 `ToTemporalRoundingIncrement ( normalizedOptions )`
// Now implemented in temporal/options.rs

// 13.17 `ValidateTemporalRoundingIncrement ( increment, dividend, inclusive )`
// Moved to temporal_rs

type RelativeTemporalObjectResult = JsResult<(Option<TemporalDate>, Option<TemporalZonedDateTime>)>;

/// 13.21 `ToRelativeTemporalObject ( options )`
pub(crate) fn to_relative_temporal_object(
    options: &JsObject,
    context: &mut Context,
) -> RelativeTemporalObjectResult {
    let relative_to = options.get(js_string!("relativeTo"), context)?;
    let plain_date = match relative_to {
        JsValue::String(relative_to_str) => JsValue::from(relative_to_str),
        JsValue::Object(relative_to_obj) => JsValue::from(relative_to_obj),
        JsValue::Undefined => return Ok((None, None)),
        _ => {
            return Err(JsNativeError::typ()
                .with_message("Invalid type for converting to relativeTo object")
                .into())
        }
    };
    let plain_date = to_temporal_date(&plain_date, None, context)?;

    // TODO: Implement TemporalZonedDateTime conversion when ZonedDateTime is implemented
    Ok((Some(plain_date), None))
}

// 13.22 `LargerOfTwoTemporalUnits ( u1, u2 )`
// use core::cmp::max

// 13.23 `MaximumTemporalDurationRoundingIncrement ( unit )`
// Implemented on TemporalUnit in temporal/options.rs

// 13.26 `GetUnsignedRoundingMode ( roundingMode, isNegative )`
// Implemented on RoundingMode in builtins/options.rs

// 13.26 IsPartialTemporalObject ( object )
pub(crate) fn is_partial_temporal_object<'value>(
    value: &'value JsValue,
    context: &mut Context,
) -> JsResult<Option<&'value JsObject>> {
    // 1. If value is not an Object, return false.
    let Some(obj) = value.as_object() else {
        return Ok(None);
    };

    // 2. If value has an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]],
    // [[InitializedTemporalMonthDay]], [[InitializedTemporalTime]],
    // [[InitializedTemporalYearMonth]], or
    // [[InitializedTemporalZonedDateTime]] internal slot, return false.
    if obj.is::<PlainDate>()
        || obj.is::<PlainDateTime>()
        || obj.is::<PlainMonthDay>()
        || obj.is::<PlainYearMonth>()
        || obj.is::<PlainTime>()
        || obj.is::<ZonedDateTime>()
    {
        return Ok(None);
    }

    // 3. Let calendarProperty be ? Get(value, "calendar").
    let calendar_property = obj.get(js_string!("calendar"), context)?;
    // 4. If calendarProperty is not undefined, return false.
    if !calendar_property.is_undefined() {
        return Ok(None);
    }
    // 5. Let timeZoneProperty be ? Get(value, "timeZone").
    let time_zone_property = obj.get(js_string!("timeZone"), context)?;
    // 6. If timeZoneProperty is not undefined, return false.
    if !time_zone_property.is_undefined() {
        return Ok(None);
    }
    // 7. Return true.
    Ok(Some(obj))
}

// 13.27 `ApplyUnsignedRoundingMode ( x, r1, r2, unsignedRoundingMode )`
// Migrated to `temporal_rs`

// 13.28 `RoundNumberToIncrement ( x, increment, roundingMode )`
// Migrated to `temporal_rs`

// 13.29 `RoundNumberToIncrementAsIfPositive ( x, increment, roundingMode )`
// Migrated to `temporal_rs`

/// 13.43 `ToPositiveIntegerWithTruncation ( argument )`
#[inline]
#[allow(unused)]
pub(crate) fn to_positive_integer_with_trunc(
    value: &JsValue,
    context: &mut Context,
) -> JsResult<i32> {
    // 1. Let integer be ? ToIntegerWithTruncation(argument).
    let int = to_integer_with_truncation(value, context)?;
    // 2. If integer ≤ 0, throw a RangeError exception.
    if int <= 0 {
        return Err(JsNativeError::range()
            .with_message("value is not a positive integer")
            .into());
    }
    // 3. Return integer.
    Ok(int)
}

/// 13.44 `ToIntegerWithTruncation ( argument )`
#[inline]
pub(crate) fn to_integer_with_truncation(value: &JsValue, context: &mut Context) -> JsResult<i32> {
    // 1. Let number be ? ToNumber(argument).
    let number = value.to_number(context)?;
    // 2. If number is NaN, +∞𝔽 or -∞𝔽, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() {
        return Err(JsNativeError::range()
            .with_message("truncation target must be an integer.")
            .into());
    }
    // 3. Return truncate(ℝ(number)).
    Ok(number.trunc() as i32)
}

/// Abstract operation 13.45 `ToIntegerIfIntegral( argument )`
#[inline]
pub(crate) fn to_integer_if_integral(arg: &JsValue, context: &mut Context) -> JsResult<i32> {
    // 1. Let number be ? ToNumber(argument).
    // 2. If IsIntegralNumber(number) is false, throw a RangeError exception.
    // 3. Return ℝ(number).
    if !arg.is_integral_number() {
        return Err(JsNativeError::range()
            .with_message("value to convert is not an integral number.")
            .into());
    }

    arg.to_i32(context)
}

// 13.46 `PrepareTemporalFields ( fields, fieldNames, requiredFields [ , duplicateBehaviour ] )`
// See fields.rs

// NOTE: op -> true == until | false == since
// 13.47 `GetDifferenceSettings ( operation, options, unitGroup, disallowedUnits, fallbackSmallestUnit, smallestLargestDefaultUnit )`
// Migrated to `temporal_rs`

// NOTE: used for MergeFields methods. Potentially can be omitted in favor of `TemporalFields`.
// 14.6 `CopyDataProperties ( target, source, excludedKeys [ , excludedValues ] )`
// Migrated or repurposed to `temporal_rs`/`fields.rs`

// Note: Deviates from Proposal spec -> proto appears to be always null across the specification.
// 14.7 `SnapshotOwnProperties ( source, proto [ , excludedKeys [ , excludedValues ] ] )`
// Migrated or repurposed to `temporal_rs`/`fields.rs`

fn extract_from_temporal_type<DF, DTF, YMF, MDF, ZDTF, Ret>(
    object: &JsObject,
    date_f: DF,
    datetime_f: DTF,
    year_month_f: YMF,
    month_day_f: MDF,
    zoned_datetime_f: ZDTF,
) -> JsResult<Option<Ret>>
where
    DF: FnOnce(JsObject<PlainDate>) -> JsResult<Option<Ret>>,
    DTF: FnOnce(JsObject<PlainDateTime>) -> JsResult<Option<Ret>>,
    YMF: FnOnce(JsObject<PlainYearMonth>) -> JsResult<Option<Ret>>,
    MDF: FnOnce(JsObject<PlainMonthDay>) -> JsResult<Option<Ret>>,
    ZDTF: FnOnce(JsObject<ZonedDateTime>) -> JsResult<Option<Ret>>,
{
    if let Ok(date) = object.clone().downcast::<PlainDate>() {
        return date_f(date);
    } else if let Ok(dt) = object.clone().downcast::<PlainDateTime>() {
        return datetime_f(dt);
    } else if let Ok(ym) = object.clone().downcast::<PlainYearMonth>() {
        return year_month_f(ym);
    } else if let Ok(md) = object.clone().downcast::<PlainMonthDay>() {
        return month_day_f(md);
    } else if let Ok(dt) = object.clone().downcast::<ZonedDateTime>() {
        return zoned_datetime_f(dt);
    }

    Ok(None)
}
