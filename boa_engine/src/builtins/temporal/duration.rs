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
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

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
            0_i32
        };

        // 3. If months is undefined, let mo be 0; else let mo be ? ToIntegerIfIntegral(months).
        let months = args.get(1);
        let months = if let Some(mo) = months {
            to_integer_if_integral(mo, context)?
        } else {
            0_i32
        };

        // 4. If weeks is undefined, let w be 0; else let w be ? ToIntegerIfIntegral(weeks).
        let weeks = args.get(2);
        let weeks = if let Some(w) = weeks {
            to_integer_if_integral(w, context)?
        } else {
            0_i32
        };

        // 5. If days is undefined, let d be 0; else let d be ? ToIntegerIfIntegral(days).
        let days = args.get(3);
        let days = if let Some(d) = days {
            to_integer_if_integral(d, context)?
        } else {
            0_i32
        };

        // 6. If hours is undefined, let h be 0; else let h be ? ToIntegerIfIntegral(hours).
        let hours = args.get(4);
        let hours = if let Some(h) = hours {
            to_integer_if_integral(h, context)?
        } else {
            0_i32
        };

        // 7. If minutes is undefined, let m be 0; else let m be ? ToIntegerIfIntegral(minutes).
        let minutes = args.get(5);
        let minutes = if let Some(m) = minutes {
            to_integer_if_integral(m, context)?
        } else {
            0_i32
        };

        // 8. If seconds is undefined, let s be 0; else let s be ? ToIntegerIfIntegral(seconds).
        let seconds = args.get(6);
        let seconds = if let Some(s) = seconds {
            to_integer_if_integral(s, context)?
        } else {
            0_i32
        };

        // 9. If milliseconds is undefined, let ms be 0; else let ms be ? ToIntegerIfIntegral(milliseconds).
        let milliseconds = args.get(7);
        let milliseconds = if let Some(ms) = milliseconds {
            to_integer_if_integral(ms, context)?
        } else {
            0_i32
        };

        // 10. If microseconds is undefined, let mis be 0; else let mis be ? ToIntegerIfIntegral(microseconds).
        let microseconds = args.get(8);
        let microseconds = if let Some(mis) = microseconds {
            to_integer_if_integral(mis, context)?
        } else {
            0_i32
        };

        // 11. If nanoseconds is undefined, let ns be 0; else let ns be ? ToIntegerIfIntegral(nanoseconds).
        let nanoseconds = args.get(9);
        let nanoseconds = if let Some(ns) = nanoseconds {
            to_integer_if_integral(ns, context)?
        } else {
            0_i32
        };

        // 12. Return ? CreateTemporalDuration(y, mo, w, d, h, m, s, ms, mis, ns, NewTarget).
        create_temporal_duration(
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
        )
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
            duration.years as i32,
            duration.months as i32,
            duration.weeks as i32,
            duration.days as i32,
            duration.hours as i32,
            duration.minutes as i32,
            duration.seconds as i32,
            duration.milliseconds as i32,
            duration.microseconds as i32,
            duration.nanoseconds as i32,
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
            duration.years as i32,
            duration.months as i32,
            duration.weeks as i32,
            duration.days as i32,
            duration.hours as i32,
            duration.minutes as i32,
            duration.seconds as i32,
            duration.milliseconds as i32,
            duration.microseconds as i32,
            duration.nanoseconds as i32,
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
    create_temporal_duration(
        result.years as i32,
        result.months as i32,
        result.weeks as i32,
        result.days as i32,
        result.hours as i32,
        result.minutes as i32,
        result.seconds as i32,
        result.milliseconds as i32,
        result.microseconds as i32,
        result.nanoseconds as i32,
        &context
            .realm()
            .intrinsics()
            .constructors()
            .duration()
            .constructor()
            .into(),
        context,
    )
}

/// 7.5.9 ToTemporalDurationRecord ( temporalDurationLike )
pub(crate) fn to_temporal_duration_record(_temporal_duration_like: &JsValue) -> JsResult<Duration> {
    todo!()
}

/// 7.5.10 DurationSign ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )
pub(crate) fn duration_sign(values: &[i32]) -> i32 {
    for v in values {
        if *v < 0 {
            return -1;
        } else if *v > 0 {
            return 1;
        }
    }
    return 0;
}

/// 7.5.11 IsValidDuration ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )
pub(crate) fn is_valid_duration(values: &[i32]) -> bool {
    // 1. Let sign be ! DurationSign(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
    let sign = duration_sign(values);
    // 2. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
    for v in values {
        // a. If 𝔽(v) is not finite, return false.
        if f64::from(*v).is_finite() {
            return false;
        }
        // b. If v < 0 and sign > 0, return false.
        if *v < 0 && sign > 0 {
            return false;
        }
        // c. If v > 0 and sign < 0, return false.
        if *v > 0 && sign < 0 {
            return false;
        }
    }
    // 3. Return true.
    return true;
}

/// 7.5.14 CreateTemporalDuration ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds [ , newTarget ] )
pub(crate) fn create_temporal_duration(
    years: i32,
    months: i32,
    weeks: i32,
    days: i32,
    hours: i32,
    minutes: i32,
    seconds: i32,
    milliseconds: i32,
    microseconds: i32,
    nanoseconds: i32,
    new_target: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
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
            .with_message("newTarget must be present whne constructing a Temporal.Duration.")
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
    duration.years = years as f64;
    // 5. Set object.[[Months]] to ℝ(𝔽(months)).
    duration.months = months as f64;
    // 6. Set object.[[Weeks]] to ℝ(𝔽(weeks)).
    duration.weeks = weeks as f64;
    // 7. Set object.[[Days]] to ℝ(𝔽(days)).
    duration.days = days as f64;
    // 8. Set object.[[Hours]] to ℝ(𝔽(hours)).
    duration.hours = hours as f64;
    // 9. Set object.[[Minutes]] to ℝ(𝔽(minutes)).
    duration.minutes = minutes as f64;
    // 10. Set object.[[Seconds]] to ℝ(𝔽(seconds)).
    duration.seconds = seconds as f64;
    // 11. Set object.[[Milliseconds]] to ℝ(𝔽(milliseconds)).
    duration.milliseconds = milliseconds as f64;
    // 12. Set object.[[Microseconds]] to ℝ(𝔽(microseconds)).
    duration.microseconds = microseconds as f64;
    // 13. Set object.[[Nanoseconds]] to ℝ(𝔽(nanoseconds)).
    duration.nanoseconds = nanoseconds as f64;

    drop(object);

    // 14. Return object.
    return Ok(prototype.into());
}
