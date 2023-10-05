// Boa's implementation of the `Temporal.Duration` Builtin Object.

use crate::{
    builtins::{
        options::{get_option, get_options_object, RoundingMode},
        temporal::validate_temporal_rounding_increment,
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, ObjectData},
    property::Attribute,
    realm::Realm,
    string::{common::StaticJsStrings, utf16},
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

use super::{
    calendar,
    options::{
        get_temporal_rounding_increment, get_temporal_unit, TemporalUnit, TemporalUnitGroup,
    },
    to_integer_if_integral, DateTimeValues,
};

mod record;

#[cfg(feature = "experimental")]
#[cfg(test)]
mod tests;

pub(crate) use record::{DateDuration, DurationRecord, TimeDuration};

/// The `Temporal.Duration` object.
///
/// Per [spec], `Duration` records are float64-representable integers
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[derive(Debug, Clone, Copy)]
pub struct Duration {
    pub(crate) inner: DurationRecord,
}

impl BuiltInObject for Duration {
    const NAME: JsString = StaticJsStrings::DURATION;
}

impl IntrinsicObject for Duration {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_years = BuiltInBuilder::callable(realm, Self::get_years)
            .name(js_string!("get Years"))
            .build();

        let get_months = BuiltInBuilder::callable(realm, Self::get_months)
            .name(js_string!("get Months"))
            .build();

        let get_weeks = BuiltInBuilder::callable(realm, Self::get_weeks)
            .name(js_string!("get Weeks"))
            .build();

        let get_days = BuiltInBuilder::callable(realm, Self::get_days)
            .name(js_string!("get Days"))
            .build();

        let get_hours = BuiltInBuilder::callable(realm, Self::get_hours)
            .name(js_string!("get Hours"))
            .build();

        let get_minutes = BuiltInBuilder::callable(realm, Self::get_minutes)
            .name(js_string!("get Minutes"))
            .build();

        let get_seconds = BuiltInBuilder::callable(realm, Self::get_seconds)
            .name(js_string!("get Seconds"))
            .build();

        let get_milliseconds = BuiltInBuilder::callable(realm, Self::get_milliseconds)
            .name(js_string!("get Milliseconds"))
            .build();

        let get_microseconds = BuiltInBuilder::callable(realm, Self::get_microseconds)
            .name(js_string!("get Microseconds"))
            .build();

        let get_nanoseconds = BuiltInBuilder::callable(realm, Self::get_nanoseconds)
            .name(js_string!("get Nanoseconds"))
            .build();

        let get_sign = BuiltInBuilder::callable(realm, Self::get_sign)
            .name(js_string!("get Sign"))
            .build();

        let is_blank = BuiltInBuilder::callable(realm, Self::get_blank)
            .name(js_string!("get blank"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(utf16!("years"), Some(get_years), None, Attribute::default())
            .accessor(
                utf16!("months"),
                Some(get_months),
                None,
                Attribute::default(),
            )
            .accessor(utf16!("weeks"), Some(get_weeks), None, Attribute::default())
            .accessor(utf16!("days"), Some(get_days), None, Attribute::default())
            .accessor(utf16!("hours"), Some(get_hours), None, Attribute::default())
            .accessor(
                utf16!("minutes"),
                Some(get_minutes),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("seconds"),
                Some(get_seconds),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("milliseconds"),
                Some(get_milliseconds),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("microseconds"),
                Some(get_microseconds),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("nanoseconds"),
                Some(get_nanoseconds),
                None,
                Attribute::default(),
            )
            .accessor(utf16!("sign"), Some(get_sign), None, Attribute::default())
            .accessor(utf16!("blank"), Some(is_blank), None, Attribute::default())
            .method(Self::with, js_string!("with"), 1)
            .method(Self::negated, js_string!("negated"), 0)
            .method(Self::abs, js_string!("abs"), 0)
            .method(Self::add, js_string!("add"), 2)
            .method(Self::subtract, js_string!("subtract"), 2)
            .method(Self::round, js_string!("round"), 1)
            .method(Self::total, js_string!("total"), 1)
            .method(Self::to_string, js_string!("toString"), 1)
            .method(Self::to_json, js_string!("toJSON"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for Duration {
    const LENGTH: usize = 10;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::duration;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, then
        if new_target.is_undefined() {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message("NewTarget cannot be undefined for Temporal.Duration constructor.")
                .into());
        }

        // 2. If years is undefined, let y be 0; else let y be ? ToIntegerIfIntegral(years).
        let years = f64::from(
            args.get(0)
                .map_or(Ok(0), |y| to_integer_if_integral(y, context))?,
        );

        // 3. If months is undefined, let mo be 0; else let mo be ? ToIntegerIfIntegral(months).
        let months = f64::from(
            args.get(1)
                .map_or(Ok(0), |mo| to_integer_if_integral(mo, context))?,
        );

        // 4. If weeks is undefined, let w be 0; else let w be ? ToIntegerIfIntegral(weeks).
        let weeks = f64::from(
            args.get(2)
                .map_or(Ok(0), |wk| to_integer_if_integral(wk, context))?,
        );

        // 5. If days is undefined, let d be 0; else let d be ? ToIntegerIfIntegral(days).
        let days = f64::from(
            args.get(3)
                .map_or(Ok(0), |d| to_integer_if_integral(d, context))?,
        );

        // 6. If hours is undefined, let h be 0; else let h be ? ToIntegerIfIntegral(hours).
        let hours = f64::from(
            args.get(4)
                .map_or(Ok(0), |h| to_integer_if_integral(h, context))?,
        );

        // 7. If minutes is undefined, let m be 0; else let m be ? ToIntegerIfIntegral(minutes).
        let minutes = f64::from(
            args.get(5)
                .map_or(Ok(0), |m| to_integer_if_integral(m, context))?,
        );

        // 8. If seconds is undefined, let s be 0; else let s be ? ToIntegerIfIntegral(seconds).
        let seconds = f64::from(
            args.get(6)
                .map_or(Ok(0), |s| to_integer_if_integral(s, context))?,
        );

        // 9. If milliseconds is undefined, let ms be 0; else let ms be ? ToIntegerIfIntegral(milliseconds).
        let milliseconds = f64::from(
            args.get(7)
                .map_or(Ok(0), |ms| to_integer_if_integral(ms, context))?,
        );

        // 10. If microseconds is undefined, let mis be 0; else let mis be ? ToIntegerIfIntegral(microseconds).
        let microseconds = f64::from(
            args.get(8)
                .map_or(Ok(0), |mis| to_integer_if_integral(mis, context))?,
        );

        // 11. If nanoseconds is undefined, let ns be 0; else let ns be ? ToIntegerIfIntegral(nanoseconds).
        let nanoseconds = f64::from(
            args.get(8)
                .map_or(Ok(0), |ns| to_integer_if_integral(ns, context))?,
        );

        let record = DurationRecord::new(
            DateDuration::new(years, months, weeks, days),
            TimeDuration::new(
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanoseconds,
            ),
        );

        // 12. Return ? CreateTemporalDuration(y, mo, w, d, h, m, s, ms, mis, ns, NewTarget).
        create_temporal_duration(record, Some(new_target), context).map(Into::into)
    }
}

// -- Duration accessor property implementations --

impl Duration {
    // Internal utility function for getting `Duration` field values.
    fn get_internal_field(this: &JsValue, field: &DateTimeValues) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        match field {
            DateTimeValues::Year => Ok(JsValue::Rational(duration.inner.years())),
            DateTimeValues::Month => Ok(JsValue::Rational(duration.inner.months())),
            DateTimeValues::Week => Ok(JsValue::Rational(duration.inner.weeks())),
            DateTimeValues::Day => Ok(JsValue::Rational(duration.inner.days())),
            DateTimeValues::Hour => Ok(JsValue::Rational(duration.inner.hours())),
            DateTimeValues::Minute => Ok(JsValue::Rational(duration.inner.minutes())),
            DateTimeValues::Second => Ok(JsValue::Rational(duration.inner.seconds())),
            DateTimeValues::Millisecond => Ok(JsValue::Rational(duration.inner.milliseconds())),
            DateTimeValues::Microsecond => Ok(JsValue::Rational(duration.inner.microseconds())),
            DateTimeValues::Nanosecond => Ok(JsValue::Rational(duration.inner.nanoseconds())),
            DateTimeValues::MonthCode => unreachable!(
                "Any other DateTimeValue fields on Duration would be an implementation error."
            ),
        }
    }

    /// 7.3.3 get Temporal.Duration.prototype.years
    fn get_years(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Year)
    }

    // 7.3.4 get Temporal.Duration.prototype.months
    fn get_months(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Month)
    }

    /// 7.3.5 get Temporal.Duration.prototype.weeks
    fn get_weeks(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Week)
    }

    /// 7.3.6 get Temporal.Duration.prototype.days
    fn get_days(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Day)
    }

    /// 7.3.7 get Temporal.Duration.prototype.hours
    fn get_hours(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Hour)
    }

    /// 7.3.8 get Temporal.Duration.prototype.minutes
    fn get_minutes(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Minute)
    }

    /// 7.3.9 get Temporal.Duration.prototype.seconds
    fn get_seconds(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Second)
    }

    /// 7.3.10 get Temporal.Duration.prototype.milliseconds
    fn get_milliseconds(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Millisecond)
    }

    /// 7.3.11 get Temporal.Duration.prototype.microseconds
    fn get_microseconds(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Microsecond)
    }

    /// 7.3.12 get Temporal.Duration.prototype.nanoseconds
    fn get_nanoseconds(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Nanosecond)
    }

    /// 7.3.13 get Temporal.Duration.prototype.sign
    fn get_sign(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        // 3. Return 𝔽(! DurationSign(duration.[[Years]], duration.[[Months]], duration.[[Weeks]],
        // duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]],
        // duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]])).
        Ok(duration.inner.duration_sign().into())
    }

    /// 7.3.14 get Temporal.Duration.prototype.blank
    fn get_blank(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        // 3. Let sign be ! DurationSign(duration.[[Years]], duration.[[Months]], duration.[[Weeks]],
        // duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]],
        // duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]]).
        let sign = duration.inner.duration_sign();

        // 4. If sign = 0, return true.
        // 5. Return false.
        match sign {
            0 => Ok(true.into()),
            _ => Ok(false.into()),
        }
    }
}

// -- Duration Method implementations --

impl Duration {
    /// 7.3.15 `Temporal.Duration.prototype.with ( temporalDurationLike )`
    pub(crate) fn with(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        // 3. Let temporalDurationLike be ? ToTemporalPartialDurationRecord(temporalDurationLike).
        let temporal_duration_like =
            DurationRecord::from_partial_js_object(args.get_or_undefined(0), context)?;

        // 4. If temporalDurationLike.[[Years]] is not undefined, then
        // a. Let years be temporalDurationLike.[[Years]].
        // 5. Else,
        // a. Let years be duration.[[Years]].
        let years = if temporal_duration_like.years().is_nan() {
            duration.inner.years()
        } else {
            temporal_duration_like.years()
        };

        // 6. If temporalDurationLike.[[Months]] is not undefined, then
        // a. Let months be temporalDurationLike.[[Months]].
        // 7. Else,
        // a. Let months be duration.[[Months]].
        let months = if temporal_duration_like.months().is_nan() {
            duration.inner.months()
        } else {
            temporal_duration_like.months()
        };

        // 8. If temporalDurationLike.[[Weeks]] is not undefined, then
        // a. Let weeks be temporalDurationLike.[[Weeks]].
        // 9. Else,
        // a. Let weeks be duration.[[Weeks]].
        let weeks = if temporal_duration_like.weeks().is_nan() {
            duration.inner.weeks()
        } else {
            temporal_duration_like.weeks()
        };

        // 10. If temporalDurationLike.[[Days]] is not undefined, then
        // a. Let days be temporalDurationLike.[[Days]].
        // 11. Else,
        // a. Let days be duration.[[Days]].
        let days = if temporal_duration_like.days().is_nan() {
            duration.inner.days()
        } else {
            temporal_duration_like.days()
        };

        // 12. If temporalDurationLike.[[Hours]] is not undefined, then
        // a. Let hours be temporalDurationLike.[[Hours]].
        // 13. Else,
        // a. Let hours be duration.[[Hours]].
        let hours = if temporal_duration_like.hours().is_nan() {
            duration.inner.hours()
        } else {
            temporal_duration_like.hours()
        };

        // 14. If temporalDurationLike.[[Minutes]] is not undefined, then
        // a. Let minutes be temporalDurationLike.[[Minutes]].
        // 15. Else,
        // a. Let minutes be duration.[[Minutes]].
        let minutes = if temporal_duration_like.minutes().is_nan() {
            duration.inner.minutes()
        } else {
            temporal_duration_like.minutes()
        };

        // 16. If temporalDurationLike.[[Seconds]] is not undefined, then
        // a. Let seconds be temporalDurationLike.[[Seconds]].
        // 17. Else,
        // a. Let seconds be duration.[[Seconds]].
        let seconds = if temporal_duration_like.seconds().is_nan() {
            duration.inner.seconds()
        } else {
            temporal_duration_like.seconds()
        };

        // 18. If temporalDurationLike.[[Milliseconds]] is not undefined, then
        // a. Let milliseconds be temporalDurationLike.[[Milliseconds]].
        // 19. Else,
        // a. Let milliseconds be duration.[[Milliseconds]].
        let milliseconds = if temporal_duration_like.milliseconds().is_nan() {
            duration.inner.milliseconds()
        } else {
            temporal_duration_like.milliseconds()
        };

        // 20. If temporalDurationLike.[[Microseconds]] is not undefined, then
        // a. Let microseconds be temporalDurationLike.[[Microseconds]].
        // 21. Else,
        // a. Let microseconds be duration.[[Microseconds]].
        let microseconds = if temporal_duration_like.microseconds().is_nan() {
            duration.inner.microseconds()
        } else {
            temporal_duration_like.microseconds()
        };

        // 22. If temporalDurationLike.[[Nanoseconds]] is not undefined, then
        // a. Let nanoseconds be temporalDurationLike.[[Nanoseconds]].
        // 23. Else,
        // a. Let nanoseconds be duration.[[Nanoseconds]].
        let nanoseconds = if temporal_duration_like.nanoseconds().is_nan() {
            duration.inner.nanoseconds()
        } else {
            temporal_duration_like.nanoseconds()
        };

        // 24. Return ? CreateTemporalDuration(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
        let new_duration = DurationRecord::new(
            DateDuration::new(years, months, weeks, days),
            TimeDuration::new(
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanoseconds,
            ),
        );

        new_duration.as_object(context).map(Into::into)
    }

    /// 7.3.16 `Temporal.Duration.prototype.negated ( )`
    pub(crate) fn negated(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        // 3. Return ! CreateNegatedTemporalDuration(duration).

        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 7.3.17 `Temporal.Duration.prototype.abs ( )`
    pub(crate) fn abs(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        // 3. Return ! CreateTemporalDuration(abs(duration.[[Years]]), abs(duration.[[Months]]),
        //    abs(duration.[[Weeks]]), abs(duration.[[Days]]), abs(duration.[[Hours]]), abs(duration.[[Minutes]]),
        //    abs(duration.[[Seconds]]), abs(duration.[[Milliseconds]]), abs(duration.[[Microseconds]]), abs(duration.[[Nanoseconds]])).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        let abs = duration.inner.abs();

        abs.as_object(context).map(Into::into)
    }

    /// 7.3.18 `Temporal.Duration.prototype.add ( other [ , options ] )`
    pub(crate) fn add(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 7.3.19 `Temporal.Duration.prototype.subtract ( other [ , options ] )`
    pub(crate) fn subtract(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 7.3.20 `Temporal.Duration.prototype.round ( roundTo )`
    pub(crate) fn round(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        let round_to = args.get_or_undefined(0);
        let round_to = match round_to {
            // 3. If roundTo is undefined, then
            JsValue::Undefined => {
                return Err(JsNativeError::typ()
                    .with_message("roundTo cannot be undefined.")
                    .into())
            }
            // 4. If Type(roundTo) is String, then
            JsValue::String(rt) => {
                // a. Let paramString be roundTo.
                let param_string = rt.clone();
                // b. Set roundTo to OrdinaryObjectCreate(null).
                let new_round_to = JsObject::with_null_proto();
                // c. Perform ! CreateDataPropertyOrThrow(roundTo, "smallestUnit", paramString).
                new_round_to.create_data_property_or_throw(
                    utf16!("smallestUnit"),
                    param_string,
                    context,
                )?;
                new_round_to
            }
            // 5. Else,
            _ => {
                // a. Set roundTo to ? GetOptionsObject(roundTo).
                get_options_object(round_to)?
            }
        };

        // 6. Let smallestUnitPresent be true.
        let mut smallest_unit_present = true;
        // 7. Let largestUnitPresent be true.
        let mut largest_unit_present = true;
        // 8. NOTE: The following steps read options and perform independent validation in alphabetical order
        //   (ToRelativeTemporalObject reads "relativeTo", ToTemporalRoundingIncrement reads "roundingIncrement" and ToTemporalRoundingMode reads "roundingMode").

        // 9. Let largestUnit be ? GetTemporalUnit(roundTo, "largestUnit", datetime, undefined, « "auto" »).
        let largest_unit = get_temporal_unit(
            &round_to,
            utf16!("largestUnit"),
            TemporalUnitGroup::DateTime,
            false,
            None,
            Some([TemporalUnit::Auto].into()),
            context,
        )?;

        // 10. Let relativeTo be ? ToRelativeTemporalObject(roundTo).
        let relative_to = super::to_relative_temporal_object(&round_to, context)?;

        // 11. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        let rounding_increment = get_temporal_rounding_increment(&round_to, context)?;

        // 12. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        let rounding_mode =
            get_option::<RoundingMode>(&round_to, utf16!("roundingMode"), false, context)?
                .unwrap_or(RoundingMode::HalfExpand);

        // 13. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit", datetime, undefined).
        let smallest_unit = get_temporal_unit(
            &round_to,
            utf16!("smallestUnit"),
            TemporalUnitGroup::DateTime,
            false,
            None,
            None,
            context,
        )?;

        // 14. If smallestUnit is undefined, then
        let smallest_unit = if let Some(unit) = smallest_unit {
            unit
        } else {
            // a. Set smallestUnitPresent to false.
            smallest_unit_present = false;
            // b. Set smallestUnit to "nanosecond".
            TemporalUnit::Nanosecond
        };

        // 15. Let defaultLargestUnit be ! DefaultTemporalLargestUnit(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]]).
        let mut default_largest_unit = duration.inner.default_temporal_largest_unit();

        // 16. Set defaultLargestUnit to ! LargerOfTwoTemporalUnits(defaultLargestUnit, smallestUnit).
        default_largest_unit = core::cmp::max(default_largest_unit, smallest_unit);

        // 17. If largestUnit is undefined, then
        let largest_unit = match largest_unit {
            Some(u) if u == TemporalUnit::Auto => default_largest_unit,
            Some(u) => u,
            None => {
                // a. Set largestUnitPresent to false.
                largest_unit_present = false;
                // b. Set largestUnit to defaultLargestUnit.
                default_largest_unit
            }
        };

        // 19. If smallestUnitPresent is false and largestUnitPresent is false, then
        if !smallest_unit_present && !largest_unit_present {
            // a. Throw a RangeError exception.
            return Err(JsNativeError::range()
                .with_message("smallestUnit or largestUnit must be present.")
                .into());
        }

        // 20. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        if core::cmp::max(largest_unit, smallest_unit) != largest_unit {
            return Err(JsNativeError::range()
                .with_message("largestUnit must be larger than smallestUnit")
                .into());
        }

        // 21. Let maximum be ! MaximumTemporalDurationRoundingIncrement(smallestUnit).
        let maximum = smallest_unit.to_maximum_rounding_increment();

        // 22. If maximum is not undefined, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
        if let Some(max) = maximum {
            validate_temporal_rounding_increment(rounding_increment, f64::from(max), false)?;
        }

        let mut unbalance_duration = DurationRecord::from_date_duration(duration.inner.date());

        // 23. Let unbalanceResult be ? UnbalanceDateDurationRelative(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], largestUnit, relativeTo).
        unbalance_duration.unbalance_duration_relative(largest_unit, &relative_to, context)?;

        let mut roundable_duration =
            DurationRecord::new(unbalance_duration.date(), duration.inner.time());

        // 24. Let roundResult be (? RoundDuration(unbalanceResult.[[Years]], unbalanceResult.[[Months]], unbalanceResult.[[Weeks]],
        //     unbalanceResult.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]],
        //     duration.[[Microseconds]], duration.[[Nanoseconds]], roundingIncrement, smallestUnit, roundingMode, relativeTo)).[[DurationRecord]].
        let _rem = roundable_duration.round_duration(
            rounding_increment,
            smallest_unit,
            rounding_mode,
            Some(&relative_to),
            context,
        )?;

        // 25. Let roundResult be roundRecord.[[DurationRecord]].
        // 26. If relativeTo is not undefined and relativeTo has an [[InitializedTemporalZonedDateTime]] internal slot, then
        match relative_to {
            JsValue::Object(o) if o.is_zoned_date_time() => {
                // TODO: AdjustRoundedDurationDays requires 6.5.5 AddZonedDateTime.
                // a. Set roundResult to ? AdjustRoundedDurationDays(roundResult.[[Years]], roundResult.[[Months]], roundResult.[[Weeks]], roundResult.[[Days]], roundResult.[[Hours]], roundResult.[[Minutes]], roundResult.[[Seconds]], roundResult.[[Milliseconds]], roundResult.[[Microseconds]], roundResult.[[Nanoseconds]], roundingIncrement, smallestUnit, roundingMode, relativeTo).
                // b. Let balanceResult be ? BalanceTimeDurationRelative(roundResult.[[Days]], roundResult.[[Hours]], roundResult.[[Minutes]], roundResult.[[Seconds]], roundResult.[[Milliseconds]], roundResult.[[Microseconds]], roundResult.[[Nanoseconds]], largestUnit, relativeTo).
                return Err(JsNativeError::range()
                    .with_message("not yet implemented.")
                    .into());
            }
            // 27. Else,
            _ => {
                // a. Let balanceResult be ? BalanceTimeDuration(roundResult.[[Days]], roundResult.[[Hours]], roundResult.[[Minutes]], roundResult.[[Seconds]], roundResult.[[Milliseconds]], roundResult.[[Microseconds]], roundResult.[[Nanoseconds]], largestUnit).
                roundable_duration.balance_time_duration(largest_unit, None)?;
            }
        }
        // 28. Let result be ? BalanceDateDurationRelative(roundResult.[[Years]], roundResult.[[Months]], roundResult.[[Weeks]], balanceResult.[[Days]], largestUnit, relativeTo).
        // 29. Return ! CreateTemporalDuration(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], balanceResult.[[Hours]], balanceResult.[[Minutes]], balanceResult.[[Seconds]], balanceResult.[[Milliseconds]], balanceResult.[[Microseconds]], balanceResult.[[Nanoseconds]]).

        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 7.3.21 `Temporal.Duration.prototype.total ( totalOf )`
    pub(crate) fn total(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        let total_of = args.get_or_undefined(0);

        let total_of = match total_of {
            // 3. If totalOf is undefined, throw a TypeError exception.
            JsValue::Undefined => {
                return Err(JsNativeError::typ()
                    .with_message("totalOf cannot be undefined.")
                    .into());
            }
            // 4. If Type(totalOf) is String, then
            JsValue::String(param_string) => {
                // a. Let paramString be totalOf.
                // b. Set totalOf to OrdinaryObjectCreate(null).
                let total_of = JsObject::with_null_proto();
                // c. Perform ! CreateDataPropertyOrThrow(totalOf, "unit", paramString).
                total_of.create_data_property_or_throw(
                    utf16!("unit"),
                    param_string.clone(),
                    context,
                )?;
                total_of
            }
            // 5. Else,
            _ => {
                // a. Set totalOf to ? GetOptionsObject(totalOf).
                get_options_object(total_of)?
            }
        };

        // 6. NOTE: The following steps read options and perform independent validation in alphabetical order (ToRelativeTemporalObject reads "relativeTo").
        // 7. Let relativeTo be ? ToRelativeTemporalObject(totalOf).
        // NOTE TO SELF: Should relative_to_temporal_object just return a JsValue and we live with the expect?
        let relative_to = super::to_relative_temporal_object(&total_of, context)?;

        // 8. Let unit be ? GetTemporalUnit(totalOf, "unit", datetime, required).
        let unit = get_temporal_unit(
            &total_of,
            utf16!("unit"),
            TemporalUnitGroup::DateTime,
            true,
            None,
            None,
            context,
        )?;

        let Some(unit) = unit else {
            return Err(JsNativeError::range()
                .with_message("TemporalUnit cannot be undefined in this context.")
                .into());
        };

        let mut unbalance_duration = DurationRecord::from_date_duration(duration.inner.date());

        // 9. Let unbalanceResult be ? UnbalanceDurationRelative(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], unit, relativeTo).
        unbalance_duration.unbalance_duration_relative(unit, &relative_to, context)?;

        // 10. Let intermediate be undefined.
        let mut _intermediate = JsValue::undefined();

        // 11. If Type(relativeTo) is Object and relativeTo has an [[InitializedTemporalZonedDateTime]] internal slot, then
        if relative_to.is_object()
            && relative_to
                .as_object()
                .expect("relative_to must be an object")
                .is_zoned_date_time()
        {
            // a. Set intermediate to ? MoveRelativeZonedDateTime(relativeTo, unbalanceResult.[[Years]], unbalanceResult.[[Months]], unbalanceResult.[[Weeks]], 0).
            return Err(JsNativeError::error()
                .with_message("not yet implemented.")
                .into());
        }

        let mut balance_duration = DurationRecord::new(
            DateDuration::new(0.0, 0.0, 0.0, unbalance_duration.days()),
            duration.inner.time(),
        );
        // 12. Let balanceResult be ? BalancePossiblyInfiniteDuration(unbalanceResult.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]], unit, intermediate).
        balance_duration.balance_possibly_infinite_duration(unit, Some(&relative_to))?;

        // 13. If balanceResult is positive overflow, return +∞𝔽.
        if balance_duration.is_positive_overflow() {
            return Ok(f64::INFINITY.into());
        };

        // 14. If balanceResult is negative overflow, return -∞𝔽.
        if balance_duration.is_negative_overflow() {
            return Ok(f64::NEG_INFINITY.into());
        }

        // TODO: determine whether and how to assert 15.
        // 15. Assert: balanceResult is a Time Duration Record.

        // 16. Let roundRecord be ? RoundDuration(unbalanceResult.[[Years]], unbalanceResult.[[Months]], unbalanceResult.[[Weeks]], balanceResult.[[Days]],
        //   balanceResult.[[Hours]], balanceResult.[[Minutes]], balanceResult.[[Seconds]], balanceResult.[[Milliseconds]], balanceResult.[[Microseconds]],
        //   balanceResult.[[Nanoseconds]], 1, unit, "trunc", relativeTo).
        // 17. Let roundResult be roundRecord.[[DurationRecord]].
        let mut round_record = DurationRecord::new(
            DateDuration::new(
                unbalance_duration.years(),
                unbalance_duration.months(),
                unbalance_duration.weeks(),
                balance_duration.days(),
            ),
            balance_duration.time(),
        );

        let remainder = round_record.round_duration(
            1_f64,
            unit,
            RoundingMode::Trunc,
            Some(&relative_to),
            context,
        )?;

        let whole = match unit {
            // 18. If unit is "year", then
            // a. Let whole be roundResult.[[Years]].
            TemporalUnit::Year => round_record.years(),
            // 19. Else if unit is "month", then
            // a. Let whole be roundResult.[[Months]].
            TemporalUnit::Month => round_record.months(),
            // 20. Else if unit is "week", then
            // a. Let whole be roundResult.[[Weeks]].
            TemporalUnit::Week => round_record.weeks(),
            // 21. Else if unit is "day", then
            // a. Let whole be roundResult.[[Days]].
            TemporalUnit::Day => round_record.days(),
            // 22. Else if unit is "hour", then
            // a. Let whole be roundResult.[[Hours]].
            TemporalUnit::Hour => round_record.hours(),
            // 23. Else if unit is "minute", then
            // a. Let whole be roundResult.[[Minutes]].
            TemporalUnit::Minute => round_record.minutes(),
            // 24. Else if unit is "second", then
            // a. Let whole be roundResult.[[Seconds]].
            TemporalUnit::Second => round_record.seconds(),
            // 25. Else if unit is "millisecond", then
            // a. Let whole be roundResult.[[Milliseconds]].
            TemporalUnit::Millisecond => round_record.milliseconds(),
            // 26. Else if unit is "microsecond", then
            // a. Let whole be roundResult.[[Microseconds]].
            TemporalUnit::Microsecond => round_record.microseconds(),
            // 27. Else,
            // b. Let whole be roundResult.[[Nanoseconds]].
            TemporalUnit::Nanosecond => round_record.nanoseconds(),
            // a. Assert: unit is "nanosecond".
            TemporalUnit::Auto=> unreachable!("Unit must be a valid temporal unit. Any other value would be an implementation error."),
        };

        // 28. Return 𝔽(whole + roundRecord.[[Remainder]]).
        Ok((whole + remainder).into())
    }

    /// 7.3.22 `Temporal.Duration.prototype.toString ( [ options ] )`
    pub(crate) fn to_string(
        _this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 7.3.23 `Temporal.Duration.prototype.toJSON ( )`
    pub(crate) fn to_json(
        _this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }
}

// -- Duration Abstract Operations --

/// 7.5.8 `ToTemporalDuration ( item )`
pub(crate) fn to_temporal_duration(item: &JsValue) -> JsResult<DurationRecord> {
    // 1a. If Type(item) is Object
    if item.is_object() {
        // 1b. and item has an [[InitializedTemporalDuration]] internal slot, then
        let o = item
            .as_object()
            .expect("Value must be an object in this instance.");
        if o.is_duration() {
            // a. Return item.
            let obj = o.borrow();
            let duration = obj.as_duration().expect("must be a duration.");
            return Ok(duration.inner);
        }
    }

    // 2. Let result be ? ToTemporalDurationRecord(item).
    let result = to_temporal_duration_record(item)?;
    // 3. Return ! CreateTemporalDuration(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], result.[[Hours]], result.[[Minutes]], result.[[Seconds]], result.[[Milliseconds]], result.[[Microseconds]], result.[[Nanoseconds]]).
    Ok(result)
}

/// 7.5.9 `ToTemporalDurationRecord ( temporalDurationLike )`
pub(crate) fn to_temporal_duration_record(
    _temporal_duration_like: &JsValue,
) -> JsResult<DurationRecord> {
    Err(JsNativeError::range()
        .with_message("Duration Parsing is not yet complete.")
        .into())
}

/// 7.5.14 `CreateTemporalDuration ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds [ , newTarget ] )`
pub(crate) fn create_temporal_duration(
    record: DurationRecord,
    new_target: Option<&JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsObject> {
    // 1. If ! IsValidDuration(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds) is false, throw a RangeError exception.
    if !record.is_valid_duration() {
        return Err(JsNativeError::range()
            .with_message("Duration values are not valid.")
            .into());
    }

    // 2. If newTarget is not present, set newTarget to %Temporal.Duration%.
    let new_target = if let Some(target) = new_target {
        target.clone()
    } else {
        context
            .realm()
            .intrinsics()
            .constructors()
            .duration()
            .constructor()
            .into()
    };

    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Duration.prototype%", « [[InitializedTemporalDuration]], [[Years]], [[Months]], [[Weeks]], [[Days]], [[Hours]], [[Minutes]], [[Seconds]], [[Milliseconds]], [[Microseconds]], [[Nanoseconds]] »).
    let prototype =
        get_prototype_from_constructor(&new_target, StandardConstructors::duration, context)?;

    // 4. Set object.[[Years]] to ℝ(𝔽(years)).
    // 5. Set object.[[Months]] to ℝ(𝔽(months)).
    // 6. Set object.[[Weeks]] to ℝ(𝔽(weeks)).
    // 7. Set object.[[Days]] to ℝ(𝔽(days)).
    // 8. Set object.[[Hours]] to ℝ(𝔽(hours)).
    // 9. Set object.[[Minutes]] to ℝ(𝔽(minutes)).
    // 10. Set object.[[Seconds]] to ℝ(𝔽(seconds)).
    // 11. Set object.[[Milliseconds]] to ℝ(𝔽(milliseconds)).
    // 12. Set object.[[Microseconds]] to ℝ(𝔽(microseconds)).
    // 13. Set object.[[Nanoseconds]] to ℝ(𝔽(nanoseconds)).

    let obj =
        JsObject::from_proto_and_data(prototype, ObjectData::duration(Duration { inner: record }));
    // 14. Return object.
    Ok(obj)
}

/// 7.5.23 `DaysUntil ( earlier, later )`
fn days_until(earlier: &JsObject, later: &JsObject) -> i32 {
    // 1. Let epochDays1 be ISODateToEpochDays(earlier.[[ISOYear]], earlier.[[ISOMonth]] - 1, earlier.[[ISODay]]).
    let obj = earlier.borrow();
    let date_one = obj
        .as_plain_date()
        .expect("earlier must be a PlainDate obj.");

    let epoch_days_one = date_one.inner.as_epoch_days();

    drop(obj);

    // 2. Let epochDays2 be ISODateToEpochDays(later.[[ISOYear]], later.[[ISOMonth]] - 1, later.[[ISODay]]).
    let obj = later.borrow();
    let date_two = obj
        .as_plain_date()
        .expect("earlier must be a PlainDate obj.");

    let epoch_days_two = date_two.inner.as_epoch_days();

    // 3. Return epochDays2 - epochDays1.
    epoch_days_two - epoch_days_one
}

/// Abstract Operation 7.5.24 `MoveRelativeDate ( calendar, relativeTo, duration, dateAdd )`
fn move_relative_date(
    calendar: &JsValue,
    relative_to: &JsObject,
    duration: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<(JsObject, f64)> {
    let new_date = calendar::calendar_date_add(calendar, relative_to, duration, None, context)?;
    let days = f64::from(days_until(relative_to, &new_date));
    Ok((new_date, days))
}
