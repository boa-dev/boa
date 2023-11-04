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
use boa_temporal::{duration::Duration as InnerDuration, options::TemporalUnit};

use super::{
    options::{get_temporal_rounding_increment, get_temporal_unit, TemporalUnitGroup},
    plain_date::PlainDate,
    to_integer_if_integral, DateTimeValues, PlainDateTime,
};

#[cfg(test)]
mod tests;

/// The `Temporal.Duration` object.
///
/// Per [spec], `Duration` records are float64-representable integers
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[derive(Debug, Clone, Copy)]
pub struct Duration {
    pub(crate) inner: InnerDuration,
}

impl Duration {
    pub(crate) fn new(inner: InnerDuration) -> Self {
        Self { inner }
    }
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
        context: &mut Context,
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

        let record = InnerDuration::new(
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        )?;

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

        let inner = &duration.inner;

        match field {
            DateTimeValues::Year => Ok(JsValue::Rational(inner.date().years())),
            DateTimeValues::Month => Ok(JsValue::Rational(inner.date().months())),
            DateTimeValues::Week => Ok(JsValue::Rational(inner.date().weeks())),
            DateTimeValues::Day => Ok(JsValue::Rational(inner.date().days())),
            DateTimeValues::Hour => Ok(JsValue::Rational(inner.time().hours())),
            DateTimeValues::Minute => Ok(JsValue::Rational(inner.time().minutes())),
            DateTimeValues::Second => Ok(JsValue::Rational(inner.time().seconds())),
            DateTimeValues::Millisecond => Ok(JsValue::Rational(inner.time().milliseconds())),
            DateTimeValues::Microsecond => Ok(JsValue::Rational(inner.time().microseconds())),
            DateTimeValues::Nanosecond => Ok(JsValue::Rational(inner.time().nanoseconds())),
            DateTimeValues::MonthCode => unreachable!(
                "Any other DateTimeValue fields on Duration would be an implementation error."
            ),
        }
    }

    /// 7.3.3 get Temporal.Duration.prototype.years
    fn get_years(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Year)
    }

    // 7.3.4 get Temporal.Duration.prototype.months
    fn get_months(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Month)
    }

    /// 7.3.5 get Temporal.Duration.prototype.weeks
    fn get_weeks(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Week)
    }

    /// 7.3.6 get Temporal.Duration.prototype.days
    fn get_days(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Day)
    }

    /// 7.3.7 get Temporal.Duration.prototype.hours
    fn get_hours(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Hour)
    }

    /// 7.3.8 get Temporal.Duration.prototype.minutes
    fn get_minutes(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Minute)
    }

    /// 7.3.9 get Temporal.Duration.prototype.seconds
    fn get_seconds(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Second)
    }

    /// 7.3.10 get Temporal.Duration.prototype.milliseconds
    fn get_milliseconds(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Millisecond)
    }

    /// 7.3.11 get Temporal.Duration.prototype.microseconds
    fn get_microseconds(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Microsecond)
    }

    /// 7.3.12 get Temporal.Duration.prototype.nanoseconds
    fn get_nanoseconds(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_internal_field(this, &DateTimeValues::Nanosecond)
    }

    /// 7.3.13 get Temporal.Duration.prototype.sign
    fn get_sign(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
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
    fn get_blank(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
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
        context: &mut Context,
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
            to_temporal_partial_duration(args.get_or_undefined(0), context)?;

        // 4. If temporalDurationLike.[[Years]] is not undefined, then
        // a. Let years be temporalDurationLike.[[Years]].
        // 5. Else,
        // a. Let years be duration.[[Years]].
        let years = if temporal_duration_like.date().years().is_nan() {
            duration.inner.date().years()
        } else {
            temporal_duration_like.date().years()
        };

        // 6. If temporalDurationLike.[[Months]] is not undefined, then
        // a. Let months be temporalDurationLike.[[Months]].
        // 7. Else,
        // a. Let months be duration.[[Months]].
        let months = if temporal_duration_like.date().months().is_nan() {
            duration.inner.date().months()
        } else {
            temporal_duration_like.date().months()
        };

        // 8. If temporalDurationLike.[[Weeks]] is not undefined, then
        // a. Let weeks be temporalDurationLike.[[Weeks]].
        // 9. Else,
        // a. Let weeks be duration.[[Weeks]].
        let weeks = if temporal_duration_like.date().weeks().is_nan() {
            duration.inner.date().weeks()
        } else {
            temporal_duration_like.date().weeks()
        };

        // 10. If temporalDurationLike.[[Days]] is not undefined, then
        // a. Let days be temporalDurationLike.[[Days]].
        // 11. Else,
        // a. Let days be duration.[[Days]].
        let days = if temporal_duration_like.date().days().is_nan() {
            duration.inner.date().days()
        } else {
            temporal_duration_like.date().days()
        };

        // 12. If temporalDurationLike.[[Hours]] is not undefined, then
        // a. Let hours be temporalDurationLike.[[Hours]].
        // 13. Else,
        // a. Let hours be duration.[[Hours]].
        let hours = if temporal_duration_like.time().hours().is_nan() {
            duration.inner.time().hours()
        } else {
            temporal_duration_like.time().hours()
        };

        // 14. If temporalDurationLike.[[Minutes]] is not undefined, then
        // a. Let minutes be temporalDurationLike.[[Minutes]].
        // 15. Else,
        // a. Let minutes be duration.[[Minutes]].
        let minutes = if temporal_duration_like.time().minutes().is_nan() {
            duration.inner.time().minutes()
        } else {
            temporal_duration_like.time().minutes()
        };

        // 16. If temporalDurationLike.[[Seconds]] is not undefined, then
        // a. Let seconds be temporalDurationLike.[[Seconds]].
        // 17. Else,
        // a. Let seconds be duration.[[Seconds]].
        let seconds = if temporal_duration_like.time().seconds().is_nan() {
            duration.inner.time().seconds()
        } else {
            temporal_duration_like.time().seconds()
        };

        // 18. If temporalDurationLike.[[Milliseconds]] is not undefined, then
        // a. Let milliseconds be temporalDurationLike.[[Milliseconds]].
        // 19. Else,
        // a. Let milliseconds be duration.[[Milliseconds]].
        let milliseconds = if temporal_duration_like.time().milliseconds().is_nan() {
            duration.inner.time().milliseconds()
        } else {
            temporal_duration_like.time().milliseconds()
        };

        // 20. If temporalDurationLike.[[Microseconds]] is not undefined, then
        // a. Let microseconds be temporalDurationLike.[[Microseconds]].
        // 21. Else,
        // a. Let microseconds be duration.[[Microseconds]].
        let microseconds = if temporal_duration_like.time().microseconds().is_nan() {
            duration.inner.time().microseconds()
        } else {
            temporal_duration_like.time().microseconds()
        };

        // 22. If temporalDurationLike.[[Nanoseconds]] is not undefined, then
        // a. Let nanoseconds be temporalDurationLike.[[Nanoseconds]].
        // 23. Else,
        // a. Let nanoseconds be duration.[[Nanoseconds]].
        let nanoseconds = if temporal_duration_like.time().nanoseconds().is_nan() {
            duration.inner.time().nanoseconds()
        } else {
            temporal_duration_like.time().nanoseconds()
        };

        // 24. Return ? CreateTemporalDuration(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
        let new_duration = InnerDuration::new(
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        )?;

        create_temporal_duration(new_duration, None, context).map(Into::into)
    }

    /// 7.3.16 `Temporal.Duration.prototype.negated ( )`
    pub(crate) fn negated(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        // 3. Return ! CreateNegatedTemporalDuration(duration).

        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 7.3.17 `Temporal.Duration.prototype.abs ( )`
    pub(crate) fn abs(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
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

        create_temporal_duration(abs, None, context).map(Into::into)
    }

    /// 7.3.18 `Temporal.Duration.prototype.add ( other [ , options ] )`
    pub(crate) fn add(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 7.3.19 `Temporal.Duration.prototype.subtract ( other [ , options ] )`
    pub(crate) fn subtract(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    // TODO: Update needed.
    /// 7.3.20 `Temporal.Duration.prototype.round ( roundTo )`
    pub(crate) fn round(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
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

        // NOTE: 6 & 7 unused in favor of `is_none()`.
        // 6. Let smallestUnitPresent be true.
        // 7. Let largestUnitPresent be true.

        // 8. NOTE: The following steps read options and perform independent validation in alphabetical order (ToRelativeTemporalObject reads "relativeTo", ToTemporalRoundingIncrement reads "roundingIncrement" and ToTemporalRoundingMode reads "roundingMode").
        // 9. Let largestUnit be ? GetTemporalUnit(roundTo, "largestUnit", datetime, undefined, « "auto" »).
        let largest_unit = get_temporal_unit(
            &round_to,
            utf16!("largestUnit"),
            TemporalUnitGroup::DateTime,
            Some([TemporalUnit::Auto].into()),
            context,
        )?;

        // 10. Let relativeToRecord be ? ToRelativeTemporalObject(roundTo).
        // 11. Let zonedRelativeTo be relativeToRecord.[[ZonedRelativeTo]].
        // 12. Let plainRelativeTo be relativeToRecord.[[PlainRelativeTo]].
        let (plain_relative_to, zoned_relative_to) =
            super::to_relative_temporal_object(&round_to, context)?;

        // 13. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        let rounding_increment = get_temporal_rounding_increment(&round_to, context)?;

        // 14. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        let _rounding_mode = get_option(&round_to, utf16!("roundingMode"), context)?
            .unwrap_or(RoundingMode::HalfExpand);

        // 15. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit", datetime, undefined).
        let smallest_unit = get_temporal_unit(
            &round_to,
            utf16!("smallestUnit"),
            TemporalUnitGroup::DateTime,
            None,
            context,
        )?;

        // NOTE: execute step 21 earlier before initial values are shadowed.
        // 21. If smallestUnitPresent is false and largestUnitPresent is false, then
        if smallest_unit.is_none() && largest_unit.is_none() {
            // a. Throw a RangeError exception.
            return Err(JsNativeError::range()
                .with_message("smallestUnit or largestUnit must be present.")
                .into());
        }

        // 16. If smallestUnit is undefined, then
        let smallest_unit = if let Some(unit) = smallest_unit {
            unit
        } else {
            // a. Set smallestUnitPresent to false.
            // b. Set smallestUnit to "nanosecond".
            TemporalUnit::Nanosecond
        };

        // 17. Let existingLargestUnit be ! DefaultTemporalLargestUnit(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]]).
        let existing_largest_unit = duration.inner.default_temporal_largest_unit();

        // 18. Set defaultLargestUnit to ! LargerOfTwoTemporalUnits(defaultLargestUnit, smallestUnit).
        let default_largest_unit = core::cmp::max(existing_largest_unit, smallest_unit);

        // 19. If largestUnit is undefined, then
        let largest_unit = match largest_unit {
            // 20. Else if largestUnit is "auto", then
            // a. Set largestUnit to defaultLargestUnit.
            Some(TemporalUnit::Auto) => default_largest_unit,
            Some(u) => u,
            None => {
                // a. Set largestUnitPresent to false.
                // b. Set largestUnit to defaultLargestUnit.
                default_largest_unit
            }
        };

        // 22. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        if core::cmp::max(largest_unit, smallest_unit) != largest_unit {
            return Err(JsNativeError::range()
                .with_message("largestUnit must be larger than smallestUnit")
                .into());
        }

        // 23. Let maximum be ! MaximumTemporalDurationRoundingIncrement(smallestUnit).
        let maximum = smallest_unit.to_maximum_rounding_increment();

        // 24. If maximum is not undefined, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
        if let Some(max) = maximum {
            validate_temporal_rounding_increment(rounding_increment.into(), max.into(), false)?;
        }

        // 25. Let hoursToDaysConversionMayOccur be false.
        // 26. If duration.[[Days]] ≠ 0 and zonedRelativeTo is not undefined, set hoursToDaysConversionMayOccur to true.
        // 27. Else if abs(duration.[[Hours]]) ≥ 24, set hoursToDaysConversionMayOccur to true.
        let conversion_may_occur =
            if duration.inner.date().days() != 0.0 && zoned_relative_to.is_some() {
                true
            } else {
                24f64 <= duration.inner.time().hours().abs()
            };

        // 28. If smallestUnit is "nanosecond" and roundingIncrement = 1, let roundingGranularityIsNoop be true; else let roundingGranularityIsNoop be false.
        let is_noop = smallest_unit == TemporalUnit::Nanosecond && rounding_increment == 1;

        // 29. If duration.[[Years]] = 0 and duration.[[Months]] = 0 and duration.[[Weeks]] = 0, let calendarUnitsPresent be false; else let calendarUnitsPresent be true.
        let calendar_units_present = !(duration.inner.date().years() == 0f64
            || duration.inner.date().months() == 0f64
            || duration.inner.date().weeks() == 0f64);

        // 30. If roundingGranularityIsNoop is true, and largestUnit is existingLargestUnit,
        // and calendarUnitsPresent is false, and hoursToDaysConversionMayOccur is false,
        // and abs(duration.[[Minutes]]) < 60, and abs(duration.[[Seconds]]) < 60,
        // and abs(duration.[[Milliseconds]]) < 1000, and abs(duration.[[Microseconds]]) < 1000,
        // and abs(duration.[[Nanoseconds]]) < 1000, then
        if is_noop
            && largest_unit == existing_largest_unit
            && !calendar_units_present
            && !conversion_may_occur
            && duration.inner.is_time_within_range()
        {
            // a. NOTE: The above conditions mean that the operation will have no effect: the smallest unit and
            // rounding increment will leave the total duration unchanged, and it can be determined without
            // calling a calendar or time zone method that no balancing will take place.
            // b. Return ! CreateTemporalDuration(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]]).
        }

        // 31. Let precalculatedPlainDateTime be undefined.
        // let mut precalc_datetime = None;

        // 32. If roundingGranularityIsNoop is false, or largestUnit is "year", or largestUnit is "month",
        // or largestUnit is "week", or largestUnit is "day", or calendarUnitsPresent is true, or duration.[[Days]] ≠ 0,
        // let plainDateTimeOrRelativeToWillBeUsed be true; else let plainDateTimeOrRelativeToWillBeUsed be false.
        let pdt_or_rel_will_be_used = !is_noop
            || largest_unit == TemporalUnit::Year
            || largest_unit == TemporalUnit::Month
            || largest_unit == TemporalUnit::Week
            || largest_unit == TemporalUnit::Day
            || calendar_units_present
            || duration.inner.date().days() != 0f64;

        // 33. If zonedRelativeTo is not undefined and plainDateTimeOrRelativeToWillBeUsed is true, then
        let (_plain_relative_to, _precalc_pdt) = if zoned_relative_to.is_some()
            && pdt_or_rel_will_be_used
        {
            // TODO(TimeZone): Implement GetPlainDateTimeFor
            // TODO(ZonedDateTime): Implement ZonedDateTime related methods.
            return Err(JsNativeError::range()
                .with_message("not yet implemented.")
                .into());

            // a. NOTE: The above conditions mean that the corresponding Temporal.PlainDateTime or Temporal.PlainDate for zonedRelativeTo will be used in one of the operations below.
            // b. Let instant be ! CreateTemporalInstant(zonedRelativeTo.[[Nanoseconds]]).
            // c. Set precalculatedPlainDateTime to ? GetPlainDateTimeFor(zonedRelativeTo.[[TimeZone]], instant, zonedRelativeTo.[[Calendar]]).
            // d. Set plainRelativeTo to ! CreateTemporalDate(precalculatedPlainDateTime.[[ISOYear]], precalculatedPlainDateTime.[[ISOMonth]], precalculatedPlainDateTime.[[ISODay]], zonedRelativeTo.[[Calendar]]).
        } else {
            // TODO: remove after ZonedDateTime is implemented
            let non_zoned: (Option<PlainDate>, Option<PlainDateTime>) = (plain_relative_to, None);
            non_zoned
        };

        // 34. Let unbalanceResult be ? UnbalanceDateDurationRelative(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], largestUnit, plainRelativeTo).
        // 35. Let roundRecord be ? RoundDuration(unbalanceResult.[[Years]], unbalanceResult.[[Months]],
        // unbalanceResult.[[Weeks]], unbalanceResult.[[Days]], duration.[[Hours]], duration.[[Minutes]],
        // duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]],
        // roundingIncrement, smallestUnit, roundingMode, plainRelativeTo, zonedRelativeTo, precalculatedPlainDateTime).
        // 36. Let roundResult be roundRecord.[[DurationRecord]].
        // 37. If zonedRelativeTo is not undefined, then
        // a. Set roundResult to ? AdjustRoundedDurationDays(roundResult.[[Years]], roundResult.[[Months]], roundResult.[[Weeks]], roundResult.[[Days]], roundResult.[[Hours]], roundResult.[[Minutes]], roundResult.[[Seconds]], roundResult.[[Milliseconds]], roundResult.[[Microseconds]], roundResult.[[Nanoseconds]], roundingIncrement, smallestUnit, roundingMode, zonedRelativeTo, precalculatedPlainDateTime).
        // b. Let balanceResult be ? BalanceTimeDurationRelative(roundResult.[[Days]], roundResult.[[Hours]], roundResult.[[Minutes]], roundResult.[[Seconds]], roundResult.[[Milliseconds]], roundResult.[[Microseconds]], roundResult.[[Nanoseconds]], largestUnit, zonedRelativeTo, precalculatedPlainDateTime).
        // 38. Else,
        // a. Let balanceResult be ? BalanceTimeDuration(roundResult.[[Days]], roundResult.[[Hours]], roundResult.[[Minutes]], roundResult.[[Seconds]], roundResult.[[Milliseconds]], roundResult.[[Microseconds]], roundResult.[[Nanoseconds]], largestUnit).
        // 39. Let result be ? BalanceDateDurationRelative(roundResult.[[Years]], roundResult.[[Months]], roundResult.[[Weeks]], balanceResult.[[Days]], largestUnit, plainRelativeTo).
        // 40. Return ! CreateTemporalDuration(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], balanceResult.[[Hours]], balanceResult.[[Minutes]], balanceResult.[[Seconds]], balanceResult.[[Milliseconds]], balanceResult.[[Microseconds]], balanceResult.[[Nanoseconds]]).

        // NOTE: Below is currently incorrect: Handling of zonedRelativeTo and precalculatedPlainDateTime is needed.
        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 7.3.21 `Temporal.Duration.prototype.total ( totalOf )`
    pub(crate) fn total(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let _duration = o.as_duration().ok_or_else(|| {
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
        // 7. Let relativeToRecord be ? ToRelativeTemporalObject(totalOf).
        // 8. Let zonedRelativeTo be relativeToRecord.[[ZonedRelativeTo]].
        // 9. Let plainRelativeTo be relativeToRecord.[[PlainRelativeTo]].
        let (_plain_relative_to, _zoned_relative_to) =
            super::to_relative_temporal_object(&total_of, context)?;

        // 10. Let unit be ? GetTemporalUnit(totalOf, "unit", datetime, required).
        let _unit = get_temporal_unit(
            &total_of,
            utf16!("unit"),
            TemporalUnitGroup::DateTime,
            None,
            context,
        )?
        .ok_or_else(|| JsNativeError::range().with_message("unit cannot be undefined."))?;

        // TODO: Implement the rest of the new `Temporal.Duration.prototype.total`

        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 7.3.22 `Temporal.Duration.prototype.toString ( [ options ] )`
    pub(crate) fn to_string(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 7.3.23 `Temporal.Duration.prototype.toJSON ( )`
    pub(crate) fn to_json(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }
}

// -- Duration Abstract Operations --

/// 7.5.8 `ToTemporalDuration ( item )`
pub(crate) fn to_temporal_duration(item: &JsValue) -> JsResult<InnerDuration> {
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
) -> JsResult<InnerDuration> {
    Err(JsNativeError::range()
        .with_message("Duration Parsing is not yet complete.")
        .into())
}

/// 7.5.14 `CreateTemporalDuration ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds [ , newTarget ] )`
pub(crate) fn create_temporal_duration(
    inner: InnerDuration,
    new_target: Option<&JsValue>,
    context: &mut Context,
) -> JsResult<JsObject> {
    // 1. If ! IsValidDuration(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds) is false, throw a RangeError exception.

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

    let obj = JsObject::from_proto_and_data(prototype, ObjectData::duration(Duration::new(inner)));
    // 14. Return object.
    Ok(obj)
}

/// Equivalent to 7.5.13 `ToTemporalPartialDurationRecord ( temporalDurationLike )`
///
pub(crate) fn to_temporal_partial_duration(
    duration_like: &JsValue,
    context: &mut Context,
) -> JsResult<InnerDuration> {
    // 1. If Type(temporalDurationLike) is not Object, then
    let JsValue::Object(unknown_object) = duration_like else {
        // a. Throw a TypeError exception.
        return Err(JsNativeError::typ()
            .with_message("temporalDurationLike must be an object.")
            .into());
    };

    // 2. Let result be a new partial Duration Record with each field set to undefined.
    let mut result = InnerDuration::partial();

    // 3. NOTE: The following steps read properties and perform independent validation in alphabetical order.
    // 4. Let days be ? Get(temporalDurationLike, "days").
    let days = unknown_object.get(utf16!("days"), context)?;
    if !days.is_undefined() {
        // 5. If days is not undefined, set result.[[Days]] to ? ToIntegerIfIntegral(days).
        result.set_days(f64::from(to_integer_if_integral(&days, context)?));
    }

    // 6. Let hours be ? Get(temporalDurationLike, "hours").
    let hours = unknown_object.get(utf16!("hours"), context)?;
    // 7. If hours is not undefined, set result.[[Hours]] to ? ToIntegerIfIntegral(hours).
    if !hours.is_undefined() {
        result.set_days(f64::from(to_integer_if_integral(&hours, context)?));
    }

    // 8. Let microseconds be ? Get(temporalDurationLike, "microseconds").
    let microseconds = unknown_object.get(utf16!("microseconds"), context)?;
    // 9. If microseconds is not undefined, set result.[[Microseconds]] to ? ToIntegerIfIntegral(microseconds).
    if !microseconds.is_undefined() {
        result.set_days(f64::from(to_integer_if_integral(&microseconds, context)?));
    }

    // 10. Let milliseconds be ? Get(temporalDurationLike, "milliseconds").
    let milliseconds = unknown_object.get(utf16!("milliseconds"), context)?;
    // 11. If milliseconds is not undefined, set result.[[Milliseconds]] to ? ToIntegerIfIntegral(milliseconds).
    if !milliseconds.is_undefined() {
        result.set_days(f64::from(to_integer_if_integral(&milliseconds, context)?));
    }

    // 12. Let minutes be ? Get(temporalDurationLike, "minutes").
    let minutes = unknown_object.get(utf16!("minutes"), context)?;
    // 13. If minutes is not undefined, set result.[[Minutes]] to ? ToIntegerIfIntegral(minutes).
    if !minutes.is_undefined() {
        result.set_days(f64::from(to_integer_if_integral(&minutes, context)?));
    }

    // 14. Let months be ? Get(temporalDurationLike, "months").
    let months = unknown_object.get(utf16!("months"), context)?;
    // 15. If months is not undefined, set result.[[Months]] to ? ToIntegerIfIntegral(months).
    if !months.is_undefined() {
        result.set_days(f64::from(to_integer_if_integral(&months, context)?));
    }

    // 16. Let nanoseconds be ? Get(temporalDurationLike, "nanoseconds").
    let nanoseconds = unknown_object.get(utf16!("nanoseconds"), context)?;
    // 17. If nanoseconds is not undefined, set result.[[Nanoseconds]] to ? ToIntegerIfIntegral(nanoseconds).
    if !nanoseconds.is_undefined() {
        result.set_days(f64::from(to_integer_if_integral(&nanoseconds, context)?));
    }

    // 18. Let seconds be ? Get(temporalDurationLike, "seconds").
    let seconds = unknown_object.get(utf16!("seconds"), context)?;
    // 19. If seconds is not undefined, set result.[[Seconds]] to ? ToIntegerIfIntegral(seconds).
    if !seconds.is_undefined() {
        result.set_days(f64::from(to_integer_if_integral(&seconds, context)?));
    }

    // 20. Let weeks be ? Get(temporalDurationLike, "weeks").
    let weeks = unknown_object.get(utf16!("weeks"), context)?;
    // 21. If weeks is not undefined, set result.[[Weeks]] to ? ToIntegerIfIntegral(weeks).
    if !weeks.is_undefined() {
        result.set_days(f64::from(to_integer_if_integral(&weeks, context)?));
    }

    // 22. Let years be ? Get(temporalDurationLike, "years").
    let years = unknown_object.get(utf16!("years"), context)?;
    // 23. If years is not undefined, set result.[[Years]] to ? ToIntegerIfIntegral(years).
    if !years.is_undefined() {
        result.set_days(f64::from(to_integer_if_integral(&years, context)?));
    }

    // 24. If years is undefined, and months is undefined, and weeks is undefined, and days is undefined, and hours is undefined, and minutes is undefined, and seconds is undefined, and milliseconds is undefined, and microseconds is undefined, and nanoseconds is undefined, throw a TypeError exception.
    if result.into_iter().all(f64::is_nan) {
        return Err(JsNativeError::typ()
            .with_message("no valid Duration fields on temporalDurationLike.")
            .into());
    }

    // 25. Return result.
    Ok(result)
}
