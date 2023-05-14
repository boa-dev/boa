#![allow(dead_code)]

use crate::{
    builtins::{
        temporal::to_integer_if_integral, BuiltInBuilder, BuiltInConstructor, BuiltInObject,
        IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::utf16,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

use super::{DAY, MICROSECOND, MILLISECOND, MONTH, NANOSECOND, WEEK, YEAR};

/// The `Temporal.Duration` object.
#[derive(Debug, Clone, Copy)]
pub struct Duration {
    pub(crate) years: f64,
    pub(crate) months: f64,
    pub(crate) weeks: f64,
    pub(crate) days: f64,
    pub(crate) hours: f64,
    pub(crate) minutes: f64,
    pub(crate) seconds: f64,
    pub(crate) milliseconds: f64,
    pub(crate) microseconds: f64,
    pub(crate) nanoseconds: f64,
}

impl BuiltInObject for Duration {
    const NAME: &'static str = "Temporal.Duration";
}

impl IntrinsicObject for Duration {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let get_years = BuiltInBuilder::new(realm)
            .callable(Self::get_years)
            .name("get Years")
            .build();

        let get_months = BuiltInBuilder::new(realm)
            .callable(Self::get_months)
            .name("get Months")
            .build();

        let get_weeks = BuiltInBuilder::new(realm)
            .callable(Self::get_weeks)
            .name("get Weeks")
            .build();

        let get_days = BuiltInBuilder::new(realm)
            .callable(Self::get_days)
            .name("get Days")
            .build();

        let get_hours = BuiltInBuilder::new(realm)
            .callable(Self::get_hours)
            .name("get Hours")
            .build();

        let get_minutes = BuiltInBuilder::new(realm)
            .callable(Self::get_minutes)
            .name("get Minutes")
            .build();

        let get_seconds = BuiltInBuilder::new(realm)
            .callable(Self::get_seconds)
            .name("get Seconds")
            .build();

        let get_milliseconds = BuiltInBuilder::new(realm)
            .callable(Self::get_milliseconds)
            .name("get Milliseconds")
            .build();

        let get_microseconds = BuiltInBuilder::new(realm)
            .callable(Self::get_microseconds)
            .name("get Microseconds")
            .build();

        let get_nanoseconds = BuiltInBuilder::new(realm)
            .callable(Self::get_nanoseconds)
            .name("get Nanoseconds")
            .build();

        let get_sign = BuiltInBuilder::new(realm)
            .callable(Self::get_sign)
            .name("get Sign")
            .build();

        let is_blank = BuiltInBuilder::new(realm)
            .callable(Self::get_blank)
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

        // 2. If years is undefined, let y be 0; else let y be ? ToIntegerIfIntegral(years).
        let years = args.get(0);
        let years = if let Some(y) = years {
            to_integer_if_integral(y, context)?
        } else {
            0_f64
        };

        // 3. If months is undefined, let mo be 0; else let mo be ? ToIntegerIfIntegral(months).
        let months = args.get(1);
        let months = if let Some(mo) = months {
            to_integer_if_integral(mo, context)?
        } else {
            0_f64
        };

        // 4. If weeks is undefined, let w be 0; else let w be ? ToIntegerIfIntegral(weeks).
        let weeks = args.get(2);
        let weeks = if let Some(w) = weeks {
            to_integer_if_integral(w, context)?
        } else {
            0_f64
        };

        // 5. If days is undefined, let d be 0; else let d be ? ToIntegerIfIntegral(days).
        let days = args.get(3);
        let days = if let Some(d) = days {
            to_integer_if_integral(d, context)?
        } else {
            0_f64
        };

        // 6. If hours is undefined, let h be 0; else let h be ? ToIntegerIfIntegral(hours).
        let hours = args.get(4);
        let hours = if let Some(h) = hours {
            to_integer_if_integral(h, context)?
        } else {
            0_f64
        };

        // 7. If minutes is undefined, let m be 0; else let m be ? ToIntegerIfIntegral(minutes).
        let minutes = args.get(5);
        let minutes = if let Some(m) = minutes {
            to_integer_if_integral(m, context)?
        } else {
            0_f64
        };

        // 8. If seconds is undefined, let s be 0; else let s be ? ToIntegerIfIntegral(seconds).
        let seconds = args.get(6);
        let seconds = if let Some(s) = seconds {
            to_integer_if_integral(s, context)?
        } else {
            0_f64
        };

        // 9. If milliseconds is undefined, let ms be 0; else let ms be ? ToIntegerIfIntegral(milliseconds).
        let milliseconds = args.get(7);
        let milliseconds = if let Some(ms) = milliseconds {
            to_integer_if_integral(ms, context)?
        } else {
            0_f64
        };

        // 10. If microseconds is undefined, let mis be 0; else let mis be ? ToIntegerIfIntegral(microseconds).
        let microseconds = args.get(8);
        let microseconds = if let Some(mis) = microseconds {
            to_integer_if_integral(mis, context)?
        } else {
            0_f64
        };

        // 11. If nanoseconds is undefined, let ns be 0; else let ns be ? ToIntegerIfIntegral(nanoseconds).
        let nanoseconds = args.get(9);
        let nanoseconds = if let Some(ns) = nanoseconds {
            to_integer_if_integral(ns, context)?
        } else {
            0_f64
        };

        // 12. Return ? CreateTemporalDuration(y, mo, w, d, h, m, s, ms, mis, ns, NewTarget).
        Ok(create_temporal_duration(
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
            new_target,
            context,
        )?.into())
    }
}

// -- Duration accessor property implementations --

enum DurationField {
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

impl Duration {
    // Internal utility function for getting `Duration` field values.
    fn get_internal_field(this: &JsValue, field: DurationField) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        match field {
            DurationField::Years => Ok(JsValue::Rational(duration.years)),
            DurationField::Months => Ok(JsValue::Rational(duration.months)),
            DurationField::Weeks => Ok(JsValue::Rational(duration.weeks)),
            DurationField::Days => Ok(JsValue::Rational(duration.days)),
            DurationField::Hours => Ok(JsValue::Rational(duration.hours)),
            DurationField::Minutes => Ok(JsValue::Rational(duration.minutes)),
            DurationField::Seconds => Ok(JsValue::Rational(duration.seconds)),
            DurationField::Milliseconds => Ok(JsValue::Rational(duration.milliseconds)),
            DurationField::Microseconds => Ok(JsValue::Rational(duration.microseconds)),
            DurationField::Nanoseconds => Ok(JsValue::Rational(duration.nanoseconds)),
        }
    }

    /// 7.3.3 get Temporal.Duration.prototype.years
    fn get_years(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, DurationField::Years)
    }

    // 7.3.4 get Temporal.Duration.prototype.months
    fn get_months(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, DurationField::Months)
    }

    /// 7.3.5 get Temporal.Duration.prototype.weeks
    fn get_weeks(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, DurationField::Weeks)
    }

    /// 7.3.6 get Temporal.Duration.prototype.days
    fn get_days(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, DurationField::Days)
    }

    /// 7.3.7 get Temporal.Duration.prototype.hours
    fn get_hours(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, DurationField::Hours)
    }

    /// 7.3.8 get Temporal.Duration.prototype.minutes
    fn get_minutes(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, DurationField::Minutes)
    }

    /// 7.3.9 get Temporal.Duration.prototype.seconds
    fn get_seconds(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, DurationField::Seconds)
    }

    /// 7.3.10 get Temporal.Duration.prototype.milliseconds
    fn get_milliseconds(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, DurationField::Milliseconds)
    }

    /// 7.3.11 get Temporal.Duration.prototype.microseconds
    fn get_microseconds(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, DurationField::Microseconds)
    }

    /// 7.3.12 get Temporal.Duration.prototype.nanoseconds
    fn get_nanoseconds(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Self::get_internal_field(this, DurationField::Nanoseconds)
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
        Ok(duration_sign(&[
            duration.years,
            duration.months,
            duration.weeks,
            duration.days,
            duration.hours,
            duration.minutes,
            duration.seconds,
            duration.milliseconds,
            duration.microseconds,
            duration.nanoseconds,
        ])
        .into())
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
        let sign = duration_sign(&[
            duration.years,
            duration.months,
            duration.weeks,
            duration.days,
            duration.hours,
            duration.minutes,
            duration.seconds,
            duration.milliseconds,
            duration.microseconds,
            duration.nanoseconds,
        ]);

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
    /// 7.3.15 Temporal.Duration.prototype.with ( temporalDurationLike )
    pub(crate) fn with(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        todo!()
    }

    /// 7.3.16 Temporal.Duration.prototype.negated ( )
    pub(crate) fn negated(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        todo!()
    }

    /// 7.3.17 Temporal.Duration.prototype.abs ( )
    pub(crate) fn abs(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        todo!()
    }

    /// 7.3.18 Temporal.Duration.prototype.add ( other [ , options ] )
    pub(crate) fn add(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        todo!()
    }

    /// 7.3.19 Temporal.Duration.prototype.subtract ( other [ , options ] )
    pub(crate) fn subtract(
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

    /// 7.3.20 Temporal.Duration.prototype.round ( roundTo )
    pub(crate) fn round(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        todo!()
    }

    /// 7.3.21 Temporal.Duration.prototype.total ( totalOf )
    pub(crate) fn total(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Duration must be an object.")
        })?;
        let o = o.borrow();
        let _duration = o.as_duration().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be a Duration object.")
        })?;

        todo!()
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

/// 7.5.8 ToTemporalDuration ( item )
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
    Ok(create_temporal_duration(
        result.years,
        result.months,
        result.weeks,
        result.days,
        result.hours,
        result.minutes,
        result.seconds,
        result.milliseconds,
        result.microseconds,
        result.nanoseconds,
        &context
            .realm()
            .intrinsics()
            .constructors()
            .duration()
            .constructor()
            .into(),
        context,
    )?.into())
}

/// 7.5.9 ToTemporalDurationRecord ( temporalDurationLike )
pub(crate) fn to_temporal_duration_record(_temporal_duration_like: &JsValue) -> JsResult<Duration> {
    todo!()
}

/// 7.5.10 DurationSign ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )
pub(crate) fn duration_sign(values: &[f64]) -> i32 {
    for v in values {
        if *v < 0_f64 {
            return -1;
        } else if *v > 0_f64 {
            return 1;
        }
    }
    return 0;
}

/// 7.5.11 IsValidDuration ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )
pub(crate) fn is_valid_duration(values: &[f64]) -> bool {
    // 1. Let sign be ! DurationSign(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
    let sign = duration_sign(values);
    // 2. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
    for v in values {
        // a. If 𝔽(v) is not finite, return false.
        if f64::from(*v).is_finite() {
            return false;
        }
        // b. If v < 0 and sign > 0, return false.
        if *v < 0_f64 && sign > 0 {
            return false;
        }
        // c. If v > 0 and sign < 0, return false.
        if *v > 0_f64 && sign < 0 {
            return false;
        }
    }
    // 3. Return true.
    return true;
}

/// 7.5.14 CreateTemporalDuration ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds [ , newTarget ] )
pub(crate) fn create_temporal_duration(
    years: f64,
    months: f64,
    weeks: f64,
    days: f64,
    hours: f64,
    minutes: f64,
    seconds: f64,
    milliseconds: f64,
    microseconds: f64,
    nanoseconds: f64,
    new_target: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsObject> {
    // 1. If ! IsValidDuration(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds) is false, throw a RangeError exception.
    if !is_valid_duration(&[
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
    ]) {
        return Err(JsNativeError::range()
            .with_message("Duration values are not valid.")
            .into());
    }
    // 2. If newTarget is not present, set newTarget to %Temporal.Duration%.
    if new_target.is_undefined() {
        return Err(JsNativeError::typ()
            .with_message("newTarget must be present when constructing a Temporal.Duration.")
            .into());
    }
    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Duration.prototype%", « [[InitializedTemporalDuration]], [[Years]], [[Months]], [[Weeks]], [[Days]], [[Hours]], [[Minutes]], [[Seconds]], [[Milliseconds]], [[Microseconds]], [[Nanoseconds]] »).
    let prototype =
        get_prototype_from_constructor(new_target, StandardConstructors::duration, context)?;
    let mut object = prototype.borrow_mut();
    let mut duration = object
        .as_duration_mut()
        .expect("prototype must be a Temporal.Duration.");
    // 4. Set object.[[Years]] to ℝ(𝔽(years)).
    duration.years = years;
    // 5. Set object.[[Months]] to ℝ(𝔽(months)).
    duration.months = months;
    // 6. Set object.[[Weeks]] to ℝ(𝔽(weeks)).
    duration.weeks = weeks;
    // 7. Set object.[[Days]] to ℝ(𝔽(days)).
    duration.days = days;
    // 8. Set object.[[Hours]] to ℝ(𝔽(hours)).
    duration.hours = hours;
    // 9. Set object.[[Minutes]] to ℝ(𝔽(minutes)).
    duration.minutes = minutes;
    // 10. Set object.[[Seconds]] to ℝ(𝔽(seconds)).
    duration.seconds = seconds;
    // 11. Set object.[[Milliseconds]] to ℝ(𝔽(milliseconds)).
    duration.milliseconds = milliseconds;
    // 12. Set object.[[Microseconds]] to ℝ(𝔽(microseconds)).
    duration.microseconds = microseconds;
    // 13. Set object.[[Nanoseconds]] to ℝ(𝔽(nanoseconds)).
    duration.nanoseconds = nanoseconds;

    drop(object);

    // 14. Return object.
    return Ok(prototype);
}

/// Abstract Operation 7.5.26 RoundDuration ( years, months, weeks, days, hours, minutes,
///   seconds, milliseconds, microseconds, nanoseconds, increment, unit,
///   roundingMode [ , relativeTo ] )
///
pub(crate) fn round_duration(
    years: f64,
    months: f64,
    weeks: f64,
    days: f64,
    hours: f64,
    minutes: f64,
    seconds: f64,
    milliseconds: f64,
    microseconds: f64,
    nanoseconds: f64,
    increment: f64,
    unit: &JsString,
    rounding_mode: &JsString,
    relative_to: Option<&JsValue>,
    context: &mut Context<'_>,
) -> JsResult<(Duration, f64)> {
    // 1. If relativeTo is not present, set relativeTo to undefined.
    let relative_to = if let Some(val) = relative_to {
        val.clone()
    } else {
        JsValue::undefined()
    };

    // 2. If unit is "year", "month", or "week", and relativeTo is undefined, then
    if relative_to.is_undefined()
        && (unit.as_slice() == YEAR || unit.as_slice() == MONTH || unit.as_slice() == WEEK)
    {
        // a. Throw a RangeError exception.
        return Err(JsNativeError::range()
            .with_message("relativeTo was out of range while rounding duration.")
            .into());
    }

    // 3. Let zonedRelativeTo be undefined.
    let zoned_relative_to = JsValue::undefined();

    // 4. If relativeTo is not undefined, then
    if !relative_to.is_undefined() {
        // TODO implement ZonedDateTime and TemporalDate Handling
        todo!()

        // a. If relativeTo has an [[InitializedTemporalZonedDateTime]] internal slot, then
        // i. Set zonedRelativeTo to relativeTo.
        // ii. Set relativeTo to ? ToTemporalDate(relativeTo).
        // b. Else,
        // i. Assert: relativeTo has an [[InitializedTemporalDate]] internal slot.
        // c. Let calendar be relativeTo.[[Calendar]].
    }
    // 5. Else,
    // a. NOTE: calendar will not be used below.

    // 6. If unit is one of "year", "month", "week", or "day", then
    if unit.as_slice() == YEAR
        || unit.as_slice() == MONTH
        || unit.as_slice() == WEEK
        || unit.as_slice() == DAY
    {
        // a. Let nanoseconds be ! TotalDurationNanoseconds(0, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, 0).
        // b. Let intermediate be undefined.
        // c. If zonedRelativeTo is not undefined, then
        // i. Let intermediate be ? MoveRelativeZonedDateTime(zonedRelativeTo, years, months, weeks, days).
        // d. Let result be ? NanosecondsToDays(nanoseconds, intermediate).
        // e. Set days to days + result.[[Days]] + result.[[Nanoseconds]] / result.[[DayLength]].
        // f. Set hours, minutes, seconds, milliseconds, microseconds, and nanoseconds to 0.
        // 7. Else,
    } else {
        // a. Let fractionalSeconds be nanoseconds × 10-9 + microseconds × 10-6 + milliseconds × 10-3 + seconds.
    }

    // 8. Let remainder be undefined.
    let mut remainder = JsValue::undefined();
    match unit.as_slice() {
        // 9. If unit is "year", then
        YEAR => {
            // a. Let yearsDuration be ! CreateTemporalDuration(years, 0, 0, 0, 0, 0, 0, 0, 0, 0).
            let years_duration = create_temporal_duration(
                years,
                0_f64,
                0_f64,
                0_f64,
                0_f64,
                0_f64,
                0_f64,
                0_f64,
                0_f64,
                0_f64,
                &context
                    .realm()
                    .intrinsics()
                    .constructors()
                    .duration()
                    .constructor()
                    .into(),
                context,
            )?;
            // TODO: deal with calendar object.
            // b. If calendar is an Object, then
            // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
            // c. Else,
            // i. Let dateAdd be unused.
            // d. Let yearsLater be ? CalendarDateAdd(calendar, relativeTo, yearsDuration, undefined, dateAdd).
            // e. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
            // f. Let yearsMonthsWeeksLater be ? CalendarDateAdd(calendar, relativeTo, yearsMonthsWeeks, undefined, dateAdd).
            // g. Let monthsWeeksInDays be DaysUntil(yearsLater, yearsMonthsWeeksLater).
            // h. Set relativeTo to yearsLater.
            // i. Let days be days + monthsWeeksInDays.
            // j. Let wholeDaysDuration be ? CreateTemporalDuration(0, 0, 0, truncate(days), 0, 0, 0, 0, 0, 0).
            // k. Let wholeDaysLater be ? CalendarDateAdd(calendar, relativeTo, wholeDaysDuration, undefined, dateAdd).
            // l. Let untilOptions be OrdinaryObjectCreate(null).
            // m. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "year").
            // n. Let timePassed be ? CalendarDateUntil(calendar, relativeTo, wholeDaysLater, untilOptions).
            // o. Let yearsPassed be timePassed.[[Years]].
            // p. Set years to years + yearsPassed.
            // q. Let oldRelativeTo be relativeTo.
            // r. Let yearsDuration be ! CreateTemporalDuration(yearsPassed, 0, 0, 0, 0, 0, 0, 0, 0, 0).
            // s. Set relativeTo to ? CalendarDateAdd(calendar, relativeTo, yearsDuration, undefined, dateAdd).
            // t. Let daysPassed be DaysUntil(oldRelativeTo, relativeTo).
            // u. Set days to days - daysPassed.
            // v. If days < 0, let sign be -1; else, let sign be 1.
            // w. Let oneYear be ! CreateTemporalDuration(sign, 0, 0, 0, 0, 0, 0, 0, 0, 0).
            // x. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneYear, dateAdd).
            // y. Let oneYearDays be moveResult.[[Days]].
            // z. Let fractionalYears be years + days / abs(oneYearDays).
            // ?. Set years to RoundNumberToIncrement(fractionalYears, increment, roundingMode).
            // ?. Set remainder to fractionalYears - years.
            // ?. Set months, weeks, and days to 0.
        }
        MONTH => {
            // 10. Else if unit is "month", then
            // a. Let yearsMonths be ! CreateTemporalDuration(years, months, 0, 0, 0, 0, 0, 0, 0, 0).
            // b. If calendar is an Object, then
            // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
            // c. Else,
            // i. Let dateAdd be unused.
            // d. Let yearsMonthsLater be ? CalendarDateAdd(calendar, relativeTo, yearsMonths, undefined, dateAdd).
            // e. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
            // f. Let yearsMonthsWeeksLater be ? CalendarDateAdd(calendar, relativeTo, yearsMonthsWeeks, undefined, dateAdd).
            // g. Let weeksInDays be DaysUntil(yearsMonthsLater, yearsMonthsWeeksLater).
            // h. Set relativeTo to yearsMonthsLater.
            // i. Let days be days + weeksInDays.
            // j. If days < 0, let sign be -1; else, let sign be 1.
            // k. Let oneMonth be ! CreateTemporalDuration(0, sign, 0, 0, 0, 0, 0, 0, 0, 0).
            // l. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
            // m. Set relativeTo to moveResult.[[RelativeTo]].
            // n. Let oneMonthDays be moveResult.[[Days]].
            // o. Repeat, while abs(days) ≥ abs(oneMonthDays),
            // i. Set months to months + sign.
            // ii. Set days to days - oneMonthDays.
            // iii. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
            // iv. Set relativeTo to moveResult.[[RelativeTo]].
            // v. Set oneMonthDays to moveResult.[[Days]].
            // p. Let fractionalMonths be months + days / abs(oneMonthDays).
            // q. Set months to RoundNumberToIncrement(fractionalMonths, increment, roundingMode).
            // r. Set remainder to fractionalMonths - months.
            // s. Set weeks and days to 0.
        }
        WEEK => {
            // 11. Else if unit is "week", then
            // a. If days < 0, let sign be -1; else, let sign be 1.
            // b. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
            // c. If calendar is an Object, then
            // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
            // d. Else,
            // i. Let dateAdd be unused.
            // e. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneWeek, dateAdd).
            // f. Set relativeTo to moveResult.[[RelativeTo]].
            // g. Let oneWeekDays be moveResult.[[Days]].
            // h. Repeat, while abs(days) ≥ abs(oneWeekDays),
            // i. Set weeks to weeks + sign.
            // ii. Set days to days - oneWeekDays.
            // iii. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneWeek, dateAdd).
            // iv. Set relativeTo to moveResult.[[RelativeTo]].
            // v. Set oneWeekDays to moveResult.[[Days]].
            // i. Let fractionalWeeks be weeks + days / abs(oneWeekDays).
            // j. Set weeks to RoundNumberToIncrement(fractionalWeeks, increment, roundingMode).
            // k. Set remainder to fractionalWeeks - weeks.
            // l. Set days to 0.
        }
        DAY => {
            // 12. Else if unit is "day", then
            // a. Let fractionalDays be days.
            // b. Set days to RoundNumberToIncrement(days, increment, roundingMode).
            // c. Set remainder to fractionalDays - days.
        }
        HOUR => {
            // 13. Else if unit is "hour", then
            // a. Let fractionalHours be (fractionalSeconds / 60 + minutes) / 60 + hours.
            // b. Set hours to RoundNumberToIncrement(fractionalHours, increment, roundingMode).
            // c. Set remainder to fractionalHours - hours.
            // d. Set minutes, seconds, milliseconds, microseconds, and nanoseconds to 0.
        }
        MINUTE => {
            // 14. Else if unit is "minute", then
            // a. Let fractionalMinutes be fractionalSeconds / 60 + minutes.
            // b. Set minutes to RoundNumberToIncrement(fractionalMinutes, increment, roundingMode).
            // c. Set remainder to fractionalMinutes - minutes.
            // d. Set seconds, milliseconds, microseconds, and nanoseconds to 0.
        }
        SECOND => {
            // 15. Else if unit is "second", then
            // a. Set seconds to RoundNumberToIncrement(fractionalSeconds, increment, roundingMode).
            // b. Set remainder to fractionalSeconds - seconds.
            // c. Set milliseconds, microseconds, and nanoseconds to 0.
        }
        MILLISECOND => {
            // 16. Else if unit is "millisecond", then
            // a. Let fractionalMilliseconds be nanoseconds × 10-6 + microseconds × 10-3 + milliseconds.
            // b. Set milliseconds to RoundNumberToIncrement(fractionalMilliseconds, increment, roundingMode).
            // c. Set remainder to fractionalMilliseconds - milliseconds.
            // d. Set microseconds and nanoseconds to 0.
        }
        MICROSECOND => {
            // 17. Else if unit is "microsecond", then
            // a. Let fractionalMicroseconds be nanoseconds × 10-3 + microseconds.
            // b. Set microseconds to RoundNumberToIncrement(fractionalMicroseconds, increment, roundingMode).
            // c. Set remainder to fractionalMicroseconds - microseconds.
            // d. Set nanoseconds to 0.
        }
        NANOSECOND => {
            // 18. Else,
            // a. Assert: unit is "nanosecond".
            // b. Set remainder to nanoseconds.
            // c. Set nanoseconds to RoundNumberToIncrement(nanoseconds, increment, roundingMode).
            // d. Set remainder to remainder - nanoseconds.
        }
    }

    // 19. Assert: days is an integer.
    // 20. Let duration be ? CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
    // 21. Return the Record { [[DurationRecord]]: duration, [[Remainder]]: remainder }.
    todo!()
}
