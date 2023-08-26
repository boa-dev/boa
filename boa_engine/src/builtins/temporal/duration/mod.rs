#![allow(dead_code, unused_variables)]

use std::arch::x86_64::_mm_undefined_pd;

use crate::{
    builtins::{
        temporal::validate_temporal_rounding_increment, BuiltInBuilder, BuiltInConstructor,
        BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::internal_methods::get_prototype_from_constructor,
    property::{Attribute, PropertyKey},
    realm::Realm,
    string::utf16,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

use super::{
    calendar, plain_date::iso::IsoDateRecord, to_integer_if_integral, zoned_date_time,
    DateTimeValues,
};

mod record;

pub(crate) use record::DurationRecord;

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
    const NAME: &'static str = "Temporal.Duration";
}

impl IntrinsicObject for Duration {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let get_years = BuiltInBuilder::callable(realm, Self::get_years)
            .name("get Years")
            .build();

        let get_months = BuiltInBuilder::callable(realm, Self::get_months)
            .name("get Months")
            .build();

        let get_weeks = BuiltInBuilder::callable(realm, Self::get_weeks)
            .name("get Weeks")
            .build();

        let get_days = BuiltInBuilder::callable(realm, Self::get_days)
            .name("get Days")
            .build();

        let get_hours = BuiltInBuilder::callable(realm, Self::get_hours)
            .name("get Hours")
            .build();

        let get_minutes = BuiltInBuilder::callable(realm, Self::get_minutes)
            .name("get Minutes")
            .build();

        let get_seconds = BuiltInBuilder::callable(realm, Self::get_seconds)
            .name("get Seconds")
            .build();

        let get_milliseconds = BuiltInBuilder::callable(realm, Self::get_milliseconds)
            .name("get Milliseconds")
            .build();

        let get_microseconds = BuiltInBuilder::callable(realm, Self::get_microseconds)
            .name("get Microseconds")
            .build();

        let get_nanoseconds = BuiltInBuilder::callable(realm, Self::get_nanoseconds)
            .name("get Nanoseconds")
            .build();

        let get_sign = BuiltInBuilder::callable(realm, Self::get_sign)
            .name("get Sign")
            .build();

        let is_blank = BuiltInBuilder::callable(realm, Self::get_blank)
            .name("get blank")
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
            .method(Self::with, "with", 1)
            .method(Self::negated, "negated", 0)
            .method(Self::abs, "abs", 0)
            .method(Self::add, "add", 2)
            .method(Self::subtract, "subtract", 2)
            .method(Self::round, "round", 1)
            .method(Self::total, "total", 1)
            .method(Self::to_string, "toString", 1)
            .method(Self::to_json, "toJSON", 0)
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

        let mut record = DurationRecord::default();

        // 2. If years is undefined, let y be 0; else let y be ? ToIntegerIfIntegral(years).
        let years = args.get(0);
        if let Some(y) = years {
            record.set_years(f64::from(to_integer_if_integral(y, context)?));
        };

        // 3. If months is undefined, let mo be 0; else let mo be ? ToIntegerIfIntegral(months).
        let months = args.get(1);
        if let Some(mo) = months {
            record.set_months(f64::from(to_integer_if_integral(mo, context)?));
        };

        // 4. If weeks is undefined, let w be 0; else let w be ? ToIntegerIfIntegral(weeks).
        let weeks = args.get(2);
        if let Some(w) = weeks {
            record.set_weeks(f64::from(to_integer_if_integral(w, context)?));
        };

        // 5. If days is undefined, let d be 0; else let d be ? ToIntegerIfIntegral(days).
        let days = args.get(3);
        if let Some(d) = days {
            record.set_days(f64::from(to_integer_if_integral(d, context)?));
        };

        // 6. If hours is undefined, let h be 0; else let h be ? ToIntegerIfIntegral(hours).
        let hours = args.get(4);
        if let Some(h) = hours {
            record.set_days(f64::from(to_integer_if_integral(h, context)?));
        };

        // 7. If minutes is undefined, let m be 0; else let m be ? ToIntegerIfIntegral(minutes).
        let minutes = args.get(5);
        if let Some(m) = minutes {
            record.set_minutes(f64::from(to_integer_if_integral(m, context)?));
        };

        // 8. If seconds is undefined, let s be 0; else let s be ? ToIntegerIfIntegral(seconds).
        let seconds = args.get(6);
        if let Some(s) = seconds {
            record.set_seconds(f64::from(to_integer_if_integral(s, context)?));
        };

        // 9. If milliseconds is undefined, let ms be 0; else let ms be ? ToIntegerIfIntegral(milliseconds).
        let milliseconds = args.get(7);
        if let Some(ms) = milliseconds {
            record.set_milliseconds(f64::from(to_integer_if_integral(ms, context)?));
        };

        // 10. If microseconds is undefined, let mis be 0; else let mis be ? ToIntegerIfIntegral(microseconds).
        let microseconds = args.get(8);
        if let Some(mis) = microseconds {
            record.set_microseconds(f64::from(to_integer_if_integral(mis, context)?));
        };

        // 11. If nanoseconds is undefined, let ns be 0; else let ns be ? ToIntegerIfIntegral(nanoseconds).
        let nanoseconds = args.get(9);
        if let Some(ns) = nanoseconds {
            record.set_nanoseconds(f64::from(to_integer_if_integral(ns, context)?));
        };

        // 12. Return ? CreateTemporalDuration(y, mo, w, d, h, m, s, ms, mis, ns, NewTarget).
        Ok(create_temporal_duration(record, Some(new_target), context)?.into())
    }
}

// -- Duration accessor property implementations --

impl Duration {
    // Internal utility function for getting `Duration` field values.
    fn get_internal_field(this: &JsValue, field: &DateTimeValues) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
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
            _ => unreachable!(
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
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
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
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
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
    pub(crate) fn with(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        // 3. Let temporalDurationLike be ? ToTemporalPartialDurationRecord(temporalDurationLike).
        // 4. If temporalDurationLike.[[Years]] is not undefined, then
        // a. Let years be temporalDurationLike.[[Years]].
        // 5. Else,
        // a. Let years be duration.[[Years]].
        // 6. If temporalDurationLike.[[Months]] is not undefined, then
        // a. Let months be temporalDurationLike.[[Months]].
        // 7. Else,
        // a. Let months be duration.[[Months]].
        // 8. If temporalDurationLike.[[Weeks]] is not undefined, then
        // a. Let weeks be temporalDurationLike.[[Weeks]].
        // 9. Else,
        // a. Let weeks be duration.[[Weeks]].
        // 10. If temporalDurationLike.[[Days]] is not undefined, then
        // a. Let days be temporalDurationLike.[[Days]].
        // 11. Else,
        // a. Let days be duration.[[Days]].
        // 12. If temporalDurationLike.[[Hours]] is not undefined, then
        // a. Let hours be temporalDurationLike.[[Hours]].
        // 13. Else,
        // a. Let hours be duration.[[Hours]].
        // 14. If temporalDurationLike.[[Minutes]] is not undefined, then
        // a. Let minutes be temporalDurationLike.[[Minutes]].
        // 15. Else,
        // a. Let minutes be duration.[[Minutes]].
        // 16. If temporalDurationLike.[[Seconds]] is not undefined, then
        // a. Let seconds be temporalDurationLike.[[Seconds]].
        // 17. Else,
        // a. Let seconds be duration.[[Seconds]].
        // 18. If temporalDurationLike.[[Milliseconds]] is not undefined, then
        // a. Let milliseconds be temporalDurationLike.[[Milliseconds]].
        // 19. Else,
        // a. Let milliseconds be duration.[[Milliseconds]].
        // 20. If temporalDurationLike.[[Microseconds]] is not undefined, then
        // a. Let microseconds be temporalDurationLike.[[Microseconds]].
        // 21. Else,
        // a. Let microseconds be duration.[[Microseconds]].
        // 22. If temporalDurationLike.[[Nanoseconds]] is not undefined, then
        // a. Let nanoseconds be temporalDurationLike.[[Nanoseconds]].
        // 23. Else,
        // a. Let nanoseconds be duration.[[Nanoseconds]].
        // 24. Return ? CreateTemporalDuration(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).

        todo!()
    }

    /// 7.3.16 `Temporal.Duration.prototype.negated ( )`
    pub(crate) fn negated(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        // 3. Return ! CreateNegatedTemporalDuration(duration).

        todo!()
    }

    /// 7.3.17 `Temporal.Duration.prototype.abs ( )`
    pub(crate) fn abs(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        // 3. Return ! CreateTemporalDuration(abs(duration.[[Years]]), abs(duration.[[Months]]),
        //    abs(duration.[[Weeks]]), abs(duration.[[Days]]), abs(duration.[[Hours]]), abs(duration.[[Minutes]]),
        //    abs(duration.[[Seconds]]), abs(duration.[[Milliseconds]]), abs(duration.[[Microseconds]]), abs(duration.[[Nanoseconds]])).

        todo!()
    }

    /// 7.3.18 `Temporal.Duration.prototype.add ( other [ , options ] )`
    pub(crate) fn add(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        // 3. Return ? AddDurationToOrSubtractDurationFromDuration(add, duration, other, options).

        todo!()
    }

    /// 7.3.19 `Temporal.Duration.prototype.subtract ( other [ , options ] )`
    pub(crate) fn subtract(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        // 3. Return ? AddDurationToOrSubtractDurationFromDuration(subtract, duration, other, options).

        todo!()
    }

    /// 7.3.20 `Temporal.Duration.prototype.round ( roundTo )`
    pub(crate) fn round(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
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
                    "smallestUnit",
                    param_string,
                    context,
                )?;
                new_round_to
            }
            // 5. Else,
            _ => {
                // a. Set roundTo to ? GetOptionsObject(roundTo).
                super::get_options_object(round_to)?
            }
        };

        // 6. Let smallestUnitPresent be true.
        let mut smallest_unit_present = true;
        // 7. Let largestUnitPresent be true.
        let mut largest_unit_present = true;
        // 8. NOTE: The following steps read options and perform independent validation in alphabetical order
        //   (ToRelativeTemporalObject reads "relativeTo", ToTemporalRoundingIncrement reads "roundingIncrement" and ToTemporalRoundingMode reads "roundingMode").

        // 9. Let largestUnit be ? GetTemporalUnit(roundTo, "largestUnit", datetime, undefined, « "auto" »).
        let largest_unit = super::get_temporal_unit(
            &round_to,
            PropertyKey::from("largestUnit"),
            &JsString::from("datetime"),
            Some(&JsValue::undefined()),
            Some(vec![JsString::from("auto")]),
            context,
        )?;

        // 10. Let relativeTo be ? ToRelativeTemporalObject(roundTo).
        let relative_to = super::to_relative_temporal_object(&round_to, context)?;

        // 11. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        let rounding_increment = super::to_temporal_rounding_increment(&round_to, context)?;

        // 12. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        let rounding_mode = super::to_temporal_rounding_mode(
            &round_to,
            &JsString::from("halfExpand").into(),
            context,
        )?;

        // 13. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit", datetime, undefined).
        let smallest_unit = super::get_temporal_unit(
            &round_to,
            PropertyKey::from("smallestUnit"),
            &JsString::from("datetime"),
            Some(&JsValue::undefined()),
            None,
            context,
        )?;

        // 14. If smallestUnit is undefined, then
        let smallest_unit = if smallest_unit.is_undefined() {
            // a. Set smallestUnitPresent to false.
            smallest_unit_present = false;
            // b. Set smallestUnit to "nanosecond".
            JsString::from("nanosecond")
        } else {
            smallest_unit
                .as_string()
                .expect("smallestUnit must be a string if it is not undefined.")
                .clone()
        };

        // 15. Let defaultLargestUnit be ! DefaultTemporalLargestUnit(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]]).
        let mut default_largest_unit = duration.inner.default_temporal_largest_unit();
        // 16. Set defaultLargestUnit to ! LargerOfTwoTemporalUnits(defaultLargestUnit, smallestUnit).
        default_largest_unit =
            super::larger_of_two_temporal_units(&default_largest_unit, &smallest_unit);

        let auto = JsString::from("auto");
        // 17. If largestUnit is undefined, then
        let largest_unit = match largest_unit {
            JsValue::Undefined => {
                // a. Set largestUnitPresent to false.
                largest_unit_present = false;
                // b. Set largestUnit to defaultLargestUnit.
                default_largest_unit
            }
            // 18. Else if largestUnit is "auto", then
            JsValue::String(s) if s == auto => {
                // a. Set largestUnit to defaultLargestUnit.
                default_largest_unit
            }
            JsValue::String(s) => s,
            _ => unreachable!("largestUnit must be a string or undefined."),
        };

        // 19. If smallestUnitPresent is false and largestUnitPresent is false, then
        if !smallest_unit_present && !largest_unit_present {
            // a. Throw a RangeError exception.
            return Err(JsNativeError::range()
                .with_message("smallestUnit or largestUnit must be present.")
                .into());
        }

        // 20. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        if super::larger_of_two_temporal_units(&largest_unit, &smallest_unit) != largest_unit {
            return Err(JsNativeError::range()
                .with_message("largestUnit must be larger than smallestUnit")
                .into());
        }

        // 21. Let maximum be ! MaximumTemporalDurationRoundingIncrement(smallestUnit).
        let maximum = super::maximum_temporal_duration_rounding_increment(&smallest_unit);

        // 22. If maximum is not undefined, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
        if !maximum.is_undefined() {
            validate_temporal_rounding_increment(
                rounding_increment,
                maximum.to_number(context)?,
                false,
            )?;
        }

        let mut unbalance_duration = DurationRecord::from_date_duration(&duration.inner);
        // 23. Let unbalanceResult be ? UnbalanceDurationRelative(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], largestUnit, relativeTo).
        unbalance_duration.unbalance_duration_relative(&largest_unit, &relative_to, context)?;

        let mut roundable_duration =
            DurationRecord::from_date_and_time_duration(&unbalance_duration, &duration.inner);
        // 24. Let roundResult be (? RoundDuration(unbalanceResult.[[Years]], unbalanceResult.[[Months]], unbalanceResult.[[Weeks]],
        //     unbalanceResult.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]],
        //     duration.[[Microseconds]], duration.[[Nanoseconds]], roundingIncrement, smallestUnit, roundingMode, relativeTo)).[[DurationRecord]].
        let _rem = roundable_duration.round_duration(
            rounding_increment,
            &smallest_unit,
            &rounding_mode,
            Some(&relative_to),
            context,
        )?;

        // 25. Let adjustResult be ? AdjustRoundedDurationDays(roundResult.[[Years]], roundResult.[[Months]], roundResult.[[Weeks]], roundResult.[[Days]], roundResult.[[Hours]], roundResult.[[Minutes]], roundResult.[[Seconds]], roundResult.[[Milliseconds]], roundResult.[[Microseconds]], roundResult.[[Nanoseconds]], roundingIncrement, smallestUnit, roundingMode, relativeTo).
        // 26. Let balanceResult be ? BalanceDuration(adjustResult.[[Days]], adjustResult.[[Hours]], adjustResult.[[Minutes]], adjustResult.[[Seconds]], adjustResult.[[Milliseconds]], adjustResult.[[Microseconds]], adjustResult.[[Nanoseconds]], largestUnit, relativeTo).
        // 27. Let result be ? BalanceDurationRelative(adjustResult.[[Years]], adjustResult.[[Months]], adjustResult.[[Weeks]], balanceResult.[[Days]], largestUnit, relativeTo).
        // 28. Return ! CreateTemporalDuration(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], balanceResult.[[Hours]], balanceResult.[[Minutes]], balanceResult.[[Seconds]], balanceResult.[[Milliseconds]], balanceResult.[[Microseconds]], balanceResult.[[Nanoseconds]]).

        todo!()
    }

    /// 7.3.21 `Temporal.Duration.prototype.total ( totalOf )`
    pub(crate) fn total(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let duration be the this value.
        // 2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
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
                total_of.create_data_property_or_throw("unit", param_string.clone(), context)?;
                total_of
            }
            // 5. Else,
            JsValue::Object(options_obj) => {
                // a. Set totalOf to ? GetOptionsObject(totalOf).
                super::get_options_object(&options_obj.clone().into())?
            }
            _ => unreachable!("total_of must be a String, Object, or undefined. Any other value is an implementation error."),
        };

        // 6. NOTE: The following steps read options and perform independent validation in alphabetical order (ToRelativeTemporalObject reads "relativeTo").
        // 7. Let relativeTo be ? ToRelativeTemporalObject(totalOf).
        // NOTE TO SELF: Should relative_to_temporal_object just return a JsValue and we live with the expect?
        let relative_to = super::to_relative_temporal_object(&total_of, context)?;

        // 8. Let unit be ? GetTemporalUnit(totalOf, "unit", datetime, required).
        let unit = super::get_temporal_unit(
            &total_of,
            PropertyKey::from("unit"),
            &JsString::from("datetime"),
            None,
            None,
            context,
        )?
        .as_string()
        .expect("GetTemporalUnit must return a string if default is required.")
        .clone();

        let mut unbalance_duration = DurationRecord::default()
            .with_years(duration.inner.years())
            .with_months(duration.inner.months())
            .with_weeks(duration.inner.weeks())
            .with_days(duration.inner.days());
        // 9. Let unbalanceResult be ? UnbalanceDurationRelative(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], unit, relativeTo).
        unbalance_duration.unbalance_duration_relative(&unit, &relative_to, context)?;

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
            todo!()
        }

        let mut balance_duration = DurationRecord::default()
            .with_days(unbalance_duration.days())
            .with_hours(duration.inner.hours())
            .with_minutes(duration.inner.minutes())
            .with_seconds(duration.inner.seconds())
            .with_milliseconds(duration.inner.milliseconds())
            .with_microseconds(duration.inner.microseconds())
            .with_milliseconds(duration.inner.nanoseconds());
        // 12. Let balanceResult be ? BalancePossiblyInfiniteDuration(unbalanceResult.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]], unit, intermediate).
        balance_duration.balance_possibly_infinite_duration(&unit, Some(&relative_to))?;

        // 13. If balanceResult is positive overflow, return +∞𝔽.
        if balance_duration.is_positive_overflow() {
            return Ok(f64::INFINITY.into());
        };

        // 14. If balanceResult is negative overflow, return -∞𝔽.
        if balance_duration.is_negative_overflow() {
            return Ok(f64::NEG_INFINITY.into());
        }
        // 15. Assert: balanceResult is a Time Duration Record.
        assert!(balance_duration.is_time_duration());

        // 16. Let roundRecord be ? RoundDuration(unbalanceResult.[[Years]], unbalanceResult.[[Months]], unbalanceResult.[[Weeks]], balanceResult.[[Days]],
        //   balanceResult.[[Hours]], balanceResult.[[Minutes]], balanceResult.[[Seconds]], balanceResult.[[Milliseconds]], balanceResult.[[Microseconds]],
        //   balanceResult.[[Nanoseconds]], 1, unit, "trunc", relativeTo).
        // 17. Let roundResult be roundRecord.[[DurationRecord]].
        let mut round_record = DurationRecord::new(
            unbalance_duration.years(),
            unbalance_duration.months(),
            unbalance_duration.weeks(),
            balance_duration.days(),
            balance_duration.hours(),
            balance_duration.minutes(),
            balance_duration.seconds(),
            balance_duration.milliseconds(),
            balance_duration.microseconds(),
            balance_duration.nanoseconds(),
        );
        let remainder = round_record.round_duration(
            1_f64,
            &unit,
            &JsString::from("trunc"),
            Some(&relative_to),
            context,
        )?;

        let whole = match unit.to_std_string_escaped().as_str() {
            // 18. If unit is "year", then
            // a. Let whole be roundResult.[[Years]].
            "year" => round_record.years(),
            // 19. Else if unit is "month", then
            // a. Let whole be roundResult.[[Months]].
            "month" => round_record.months(),
            // 20. Else if unit is "week", then
            // a. Let whole be roundResult.[[Weeks]].
            "week" => round_record.weeks(),
            // 21. Else if unit is "day", then
            // a. Let whole be roundResult.[[Days]].
            "day" => round_record.days(),
            // 22. Else if unit is "hour", then
            // a. Let whole be roundResult.[[Hours]].
            "hour" => round_record.hours(),
            // 23. Else if unit is "minute", then
            // a. Let whole be roundResult.[[Minutes]].
            "minute" => round_record.minutes(),
            // 24. Else if unit is "second", then
            // a. Let whole be roundResult.[[Seconds]].
            "second" => round_record.seconds(),
            // 25. Else if unit is "millisecond", then
            // a. Let whole be roundResult.[[Milliseconds]].
            "millisecond"=> round_record.milliseconds(),
            // 26. Else if unit is "microsecond", then
            // a. Let whole be roundResult.[[Microseconds]].
            "microsecond" => round_record.microseconds(),
            // 27. Else,
            // b. Let whole be roundResult.[[Nanoseconds]].
            "nanosecond" => round_record.nanoseconds(),
            // a. Assert: unit is "nanosecond".
            _=> unreachable!("Unit must be a valid temporal unit. Any other value would be an implementation error."),
        };

        // 28. Return 𝔽(whole + roundRecord.[[Remainder]]).
        Ok((whole + remainder).into())
    }

    /// 7.3.22 Temporal.Duration.prototype.toString ( [ options ] )
    pub(crate) fn to_string(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        todo!()
    }

    /// 7.3.23 Temporal.Duration.prototype.toJSON ( )
    pub(crate) fn to_json(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        todo!()
    }
}

// -- Duration Abstract Operations --

/// 7.5.8 `ToTemporalDuration ( item )`
pub(crate) fn to_temporal_duration(item: &JsValue, context: &mut Context<'_>) -> JsResult<JsValue> {
    // 1a. If Type(item) is Object
    if item.is_object() {
        // 1b. and item has an [[InitializedTemporalDuration]] internal slot, then
        let o = item
            .as_object()
            .expect("Value must be an object in this instance.");
        if o.is_duration() {
            // a. Return item.
            return Ok(item.clone());
        }
    }

    // 2. Let result be ? ToTemporalDurationRecord(item).
    let result = to_temporal_duration_record(item)?;
    // 3. Return ! CreateTemporalDuration(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], result.[[Hours]], result.[[Minutes]], result.[[Seconds]], result.[[Milliseconds]], result.[[Microseconds]], result.[[Nanoseconds]]).
    Ok(create_temporal_duration(result, None, context)?.into())
}

/// 7.5.9 `ToTemporalDurationRecord ( temporalDurationLike )`
pub(crate) fn to_temporal_duration_record(
    _temporal_duration_like: &JsValue,
) -> JsResult<DurationRecord> {
    todo!()
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
    let mut object = prototype.borrow_mut();
    let duration = object
        .as_duration_mut()
        .expect("prototype must be a Temporal.Duration.");
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
    duration.inner = record;
    drop(object);

    // 14. Return object.
    Ok(prototype)
}

// 7.5.17 `TotalDurationNanoseconds ( days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, offsetShift )`
fn total_duration_nanoseconds(
    days: f64,
    hours: f64,
    minutes: f64,
    seconds: f64,
    milliseconds: f64,
    microseconds: f64,
    nanoseconds: f64,
    offset_shift: f64,
) -> f64 {
    let nanoseconds = if days == 0_f64 {
        nanoseconds
    } else {
        nanoseconds - offset_shift
    };

    days.mul_add(24_f64, hours)
        .mul_add(60_f64, minutes)
        .mul_add(60_f64, seconds)
        .mul_add(1_000_f64, milliseconds)
        .mul_add(1_000_f64, microseconds)
        .mul_add(1_000_f64, nanoseconds)
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
    duration: JsObject,
    date_add: Option<&JsValue>,
) -> JsResult<(JsObject, f64)> {
    let new_date = calendar::calendar_date_add(
        calendar,
        relative_to,
        &duration,
        &JsValue::undefined(),
        date_add,
    )?;
    let days = f64::from(days_until(relative_to, &new_date));
    Ok((new_date, days))
}
