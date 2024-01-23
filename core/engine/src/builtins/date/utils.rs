use crate::{context::HostHooks, js_string, value::IntegerOrInfinity, JsString};
use boa_macros::utf16;
use std::{iter::Peekable, str::Chars};
use time::{macros::format_description, OffsetDateTime, PrimitiveDateTime};

// Time-related Constants
//
// More info:
// - [ECMAScript reference][spec]
//
// https://tc39.es/ecma262/#sec-time-related-constants

// HoursPerDay = 24
const HOURS_PER_DAY: f64 = 24.0;

// MinutesPerHour = 60
const MINUTES_PER_HOUR: f64 = 60.0;

// SecondsPerMinute = 60
const SECONDS_PER_MINUTE: f64 = 60.0;

// msPerSecond = 1000𝔽
const MS_PER_SECOND: f64 = 1000.0;

// msPerMinute = 60000𝔽 = msPerSecond × 𝔽(SecondsPerMinute)
pub(super) const MS_PER_MINUTE: f64 = MS_PER_SECOND * SECONDS_PER_MINUTE;

// msPerHour = 3600000𝔽 = msPerMinute × 𝔽(MinutesPerHour)
const MS_PER_HOUR: f64 = MS_PER_MINUTE * MINUTES_PER_HOUR;

// msPerDay = 86400000𝔽 = msPerHour × 𝔽(HoursPerDay)
const MS_PER_DAY: f64 = MS_PER_HOUR * HOURS_PER_DAY;

/// Abstract operation `Day ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-day
pub(super) fn day(t: f64) -> f64 {
    // 1. Return 𝔽(floor(ℝ(t / msPerDay))).
    (t / MS_PER_DAY).floor()
}

/// Abstract operation `TimeWithinDay ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-timewithinday
pub(super) fn time_within_day(t: f64) -> f64 {
    // 1. Return 𝔽(ℝ(t) modulo ℝ(msPerDay)).
    t.rem_euclid(MS_PER_DAY)
}

/// Abstract operation `DaysInYear ( y )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-daysinyear
fn days_in_year(y: f64) -> u16 {
    // 1. Let ry be ℝ(y).
    let ry = y;

    // 2. If (ry modulo 400) = 0, return 366𝔽.
    if ry.rem_euclid(400.0) == 0.0 {
        return 366;
    }

    // 3. If (ry modulo 100) = 0, return 365𝔽.
    if ry.rem_euclid(100.0) == 0.0 {
        return 365;
    }

    // 4. If (ry modulo 4) = 0, return 366𝔽.
    if ry.rem_euclid(4.0) == 0.0 {
        return 366;
    }

    // 5. Return 365𝔽.
    365
}

/// Abstract operation `DayFromYear ( y )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-dayfromyear
fn day_from_year(y: f64) -> f64 {
    // 1. Let ry be ℝ(y).
    // 2. NOTE: In the following steps, each _numYearsN_ is the number of years divisible by N
    //          that occur between the epoch and the start of year y.
    //          (The number is negative if y is before the epoch.)

    // 3. Let numYears1 be (ry - 1970).
    let num_years_1 = y - 1970.0;

    // 4. Let numYears4 be floor((ry - 1969) / 4).
    let num_years_4 = ((y - 1969.0) / 4.0).floor();

    // 5. Let numYears100 be floor((ry - 1901) / 100).
    let num_years_100 = ((y - 1901.0) / 100.0).floor();

    // 6. Let numYears400 be floor((ry - 1601) / 400).
    let num_years_400 = ((y - 1601.0) / 400.0).floor();

    // 7. Return 𝔽(365 × numYears1 + numYears4 - numYears100 + numYears400).
    365.0 * num_years_1 + num_years_4 - num_years_100 + num_years_400
}

/// Abstract operation `TimeFromYear ( y )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-timefromyear
fn time_from_year(y: f64) -> f64 {
    // 1. Return msPerDay × DayFromYear(y).
    MS_PER_DAY * day_from_year(y)
}

/// Abstract operation `YearFromTime ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-yearfromtime
pub(super) fn year_from_time(t: f64) -> i32 {
    const MS_PER_AVERAGE_YEAR: f64 = 12.0 * 30.436_875 * MS_PER_DAY;

    // 1. Return the largest integral Number y (closest to +∞) such that TimeFromYear(y) ≤ t.
    let mut year = (((t + MS_PER_AVERAGE_YEAR / 2.0) / MS_PER_AVERAGE_YEAR).floor()) as i32 + 1970;
    if time_from_year(year.into()) > t {
        year -= 1;
    }
    year
}

/// Abstract operation `DayWithinYear ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-daywithinyear
fn day_within_year(t: f64) -> u16 {
    // 1. Return Day(t) - DayFromYear(YearFromTime(t)).
    (day(t) - day_from_year(year_from_time(t).into())) as u16
}

/// Abstract operation `InLeapYear ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-inleapyear
fn in_leap_year(t: f64) -> u16 {
    // 1. If DaysInYear(YearFromTime(t)) is 366𝔽, return 1𝔽; else return +0𝔽.
    (days_in_year(year_from_time(t).into()) == 366).into()
}

/// Abstract operation `MonthFromTime ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-monthfromtime
pub(super) fn month_from_time(t: f64) -> u8 {
    // 1. Let inLeapYear be InLeapYear(t).
    let in_leap_year = in_leap_year(t);

    // 2. Let dayWithinYear be DayWithinYear(t).
    let day_within_year = day_within_year(t);

    match day_within_year {
        // 3. If dayWithinYear < 31𝔽, return +0𝔽.
        t if t < 31 => 0,
        // 4. If dayWithinYear < 59𝔽 + inLeapYear, return 1𝔽.
        t if t < 59 + in_leap_year => 1,
        // 5. If dayWithinYear < 90𝔽 + inLeapYear, return 2𝔽.
        t if t < 90 + in_leap_year => 2,
        // 6. If dayWithinYear < 120𝔽 + inLeapYear, return 3𝔽.
        t if t < 120 + in_leap_year => 3,
        // 7. If dayWithinYear < 151𝔽 + inLeapYear, return 4𝔽.
        t if t < 151 + in_leap_year => 4,
        // 8. If dayWithinYear < 181𝔽 + inLeapYear, return 5𝔽.
        t if t < 181 + in_leap_year => 5,
        // 9. If dayWithinYear < 212𝔽 + inLeapYear, return 6𝔽.
        t if t < 212 + in_leap_year => 6,
        // 10. If dayWithinYear < 243𝔽 + inLeapYear, return 7𝔽.
        t if t < 243 + in_leap_year => 7,
        // 11. If dayWithinYear < 273𝔽 + inLeapYear, return 8𝔽.
        t if t < 273 + in_leap_year => 8,
        // 12. If dayWithinYear < 304𝔽 + inLeapYear, return 9𝔽.
        t if t < 304 + in_leap_year => 9,
        // 13. If dayWithinYear < 334𝔽 + inLeapYear, return 10𝔽.
        t if t < 334 + in_leap_year => 10,
        // 14. Assert: dayWithinYear < 365𝔽 + inLeapYear.
        // 15. Return 11𝔽.
        _ => 11,
    }
}

/// Abstract operation `DateFromTime ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-datefromtime
pub(super) fn date_from_time(t: f64) -> u8 {
    // 1. Let inLeapYear be InLeapYear(t).
    let in_leap_year = in_leap_year(t);

    // 2. Let dayWithinYear be DayWithinYear(t).
    let day_within_year = day_within_year(t);

    // 3. Let month be MonthFromTime(t).
    let month = month_from_time(t);

    let date = match month {
        // 4. If month is +0𝔽, return dayWithinYear + 1𝔽.
        0 => day_within_year + 1,
        // 5. If month is 1𝔽, return dayWithinYear - 30𝔽.
        1 => day_within_year - 30,
        // 6. If month is 2𝔽, return dayWithinYear - 58𝔽 - inLeapYear.
        2 => day_within_year - 58 - in_leap_year,
        // 7. If month is 3𝔽, return dayWithinYear - 89𝔽 - inLeapYear.
        3 => day_within_year - 89 - in_leap_year,
        // 8. If month is 4𝔽, return dayWithinYear - 119𝔽 - inLeapYear.
        4 => day_within_year - 119 - in_leap_year,
        // 9. If month is 5𝔽, return dayWithinYear - 150𝔽 - inLeapYear.
        5 => day_within_year - 150 - in_leap_year,
        // 10. If month is 6𝔽, return dayWithinYear - 180𝔽 - inLeapYear.
        6 => day_within_year - 180 - in_leap_year,
        // 11. If month is 7𝔽, return dayWithinYear - 211𝔽 - inLeapYear.
        7 => day_within_year - 211 - in_leap_year,
        // 12. If month is 8𝔽, return dayWithinYear - 242𝔽 - inLeapYear.
        8 => day_within_year - 242 - in_leap_year,
        // 13. If month is 9𝔽, return dayWithinYear - 272𝔽 - inLeapYear.
        9 => day_within_year - 272 - in_leap_year,
        // 14. If month is 10𝔽, return dayWithinYear - 303𝔽 - inLeapYear.
        10 => day_within_year - 303 - in_leap_year,
        // 15. Assert: month is 11𝔽.
        // 16. Return dayWithinYear - 333𝔽 - inLeapYear.
        _ => day_within_year - 333 - in_leap_year,
    };
    date as u8
}

/// Abstract operation `WeekDay ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-weekday
pub(super) fn week_day(t: f64) -> u8 {
    // 1. Return 𝔽(ℝ(Day(t) + 4𝔽) modulo 7).
    (day(t) + 4.0).rem_euclid(7.0) as u8
}

/// Abstract operation `HourFromTime ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-hourfromtime
pub(super) fn hour_from_time(t: f64) -> u8 {
    // 1. Return 𝔽(floor(ℝ(t / msPerHour)) modulo HoursPerDay).
    ((t / MS_PER_HOUR).floor()).rem_euclid(HOURS_PER_DAY) as u8
}

/// Abstract operation `MinFromTime ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-minfromtime
pub(super) fn min_from_time(t: f64) -> u8 {
    // 1. Return 𝔽(floor(ℝ(t / msPerMinute)) modulo MinutesPerHour).
    ((t / MS_PER_MINUTE).floor()).rem_euclid(MINUTES_PER_HOUR) as u8
}

/// Abstract operation `SecFromTime ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-secfromtime
pub(super) fn sec_from_time(t: f64) -> u8 {
    // 1. Return 𝔽(floor(ℝ(t / msPerSecond)) modulo SecondsPerMinute).
    ((t / MS_PER_SECOND).floor()).rem_euclid(SECONDS_PER_MINUTE) as u8
}

/// Abstract operation `msFromTime ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-msfromtime
pub(super) fn ms_from_time(t: f64) -> u16 {
    // 1. Return 𝔽(ℝ(t) modulo ℝ(msPerSecond)).
    t.rem_euclid(MS_PER_SECOND) as u16
}

/// Abstract operation `LocalTime ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-localtime
pub(super) fn local_time(t: f64, hooks: &dyn HostHooks) -> f64 {
    t + f64::from(local_timezone_offset_seconds(t, hooks)) * MS_PER_SECOND
}

/// Abstract operation `UTC ( t )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-utc-t
pub(super) fn utc_t(t: f64, hooks: &dyn HostHooks) -> f64 {
    // 1. If t is not finite, return NaN.
    if !t.is_finite() {
        return f64::NAN;
    }

    t - f64::from(local_timezone_offset_seconds(t, hooks)) * MS_PER_SECOND
}

/// Abstract operation `MakeTime ( hour, min, sec, ms )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-maketime
pub(super) fn make_time(hour: f64, min: f64, sec: f64, ms: f64) -> f64 {
    // 1. If hour is not finite, min is not finite, sec is not finite, or ms is not finite, return NaN.
    if !hour.is_finite() || !min.is_finite() || !sec.is_finite() || !ms.is_finite() {
        return f64::NAN;
    }

    // 2. Let h be 𝔽(! ToIntegerOrInfinity(hour)).
    let h = hour.abs().floor().copysign(hour);

    // 3. Let m be 𝔽(! ToIntegerOrInfinity(min)).
    let m = min.abs().floor().copysign(min);

    // 4. Let s be 𝔽(! ToIntegerOrInfinity(sec)).
    let s = sec.abs().floor().copysign(sec);

    // 5. Let milli be 𝔽(! ToIntegerOrInfinity(ms)).
    let milli = ms.abs().floor().copysign(ms);

    // 6. Return ((h × msPerHour + m × msPerMinute) + s × msPerSecond) + milli.
    ((h * MS_PER_HOUR + m * MS_PER_MINUTE) + s * MS_PER_SECOND) + milli
}

/// Abstract operation `MakeDay ( year, month, date )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-makeday
pub(super) fn make_day(year: f64, month: f64, date: f64) -> f64 {
    // 1. If year is not finite, month is not finite, or date is not finite, return NaN.
    if !year.is_finite() || !month.is_finite() || !date.is_finite() {
        return f64::NAN;
    }

    // 2. Let y be 𝔽(! ToIntegerOrInfinity(year)).
    let y = year.abs().floor().copysign(year);

    // 3. Let m be 𝔽(! ToIntegerOrInfinity(month)).
    let m = month.abs().floor().copysign(month);

    // 4. Let dt be 𝔽(! ToIntegerOrInfinity(date)).
    let dt = date.abs().floor().copysign(date);

    // 5. Let ym be y + 𝔽(floor(ℝ(m) / 12)).
    let ym = y + (m / 12.0).floor();

    // 6. If ym is not finite, return NaN.
    if !ym.is_finite() {
        return f64::NAN;
    }

    // 7. Let mn be 𝔽(ℝ(m) modulo 12).
    let mn = m.rem_euclid(12.0) as u8;

    // 8. Find a finite time value t such that YearFromTime(t) is ym, MonthFromTime(t) is mn,
    //    and DateFromTime(t) is 1𝔽;
    //    but if this is not possible (because some argument is out of range), return NaN.
    let rest = if mn > 1 { 1.0 } else { 0.0 };
    let days_within_year_to_end_of_month = match mn {
        0 => 0.0,
        1 => 31.0,
        2 => 59.0,
        3 => 90.0,
        4 => 120.0,
        5 => 151.0,
        6 => 181.0,
        7 => 212.0,
        8 => 243.0,
        9 => 273.0,
        10 => 304.0,
        11 => 334.0,
        12 => 365.0,
        _ => unreachable!(),
    };
    let t =
        (day_from_year(ym + rest) - 365.0 * rest + days_within_year_to_end_of_month) * MS_PER_DAY;

    // 9. Return Day(t) + dt - 1𝔽.
    day(t) + dt - 1.0
}

/// Abstract operation `MakeDate ( day, time )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-makedate
pub(super) fn make_date(day: f64, time: f64) -> f64 {
    // 1. If day is not finite or time is not finite, return NaN.
    if !day.is_finite() || !time.is_finite() {
        return f64::NAN;
    }

    // 2. Let tv be day × msPerDay + time.
    let tv = day * MS_PER_DAY + time;

    // 3. If tv is not finite, return NaN.
    if !tv.is_finite() {
        return f64::NAN;
    }

    // 4. Return tv.
    tv
}

/// Abstract operation `MakeFullYear ( year )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-makefullyear
pub(super) fn make_full_year(year: f64) -> f64 {
    // 1. If year is NaN, return NaN.
    if year.is_nan() {
        return f64::NAN;
    }

    // 2. Let truncated be ! ToIntegerOrInfinity(year).
    let truncated = IntegerOrInfinity::from(year);

    // 3. If truncated is in the inclusive interval from 0 to 99, return 1900𝔽 + 𝔽(truncated).
    // 4. Return 𝔽(truncated).
    match truncated {
        IntegerOrInfinity::Integer(i) if (0..=99).contains(&i) => 1900.0 + i as f64,
        IntegerOrInfinity::Integer(i) => i as f64,
        IntegerOrInfinity::PositiveInfinity => f64::INFINITY,
        IntegerOrInfinity::NegativeInfinity => f64::NEG_INFINITY,
    }
}

/// Abstract operation `TimeClip ( time )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-timeclip
pub(crate) fn time_clip(time: f64) -> f64 {
    // 1. If time is not finite, return NaN.
    if !time.is_finite() {
        return f64::NAN;
    }

    // 2. If abs(ℝ(time)) > 8.64 × 10**15, return NaN.
    if time.abs() > 8.64e15 {
        return f64::NAN;
    }

    // 3. Return 𝔽(! ToIntegerOrInfinity(time)).
    let time = time.trunc();
    if time.abs() == 0.0 {
        return 0.0;
    }

    time
}

/// Abstract operation `TimeString ( tv )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-timestring
pub(super) fn time_string(tv: f64) -> JsString {
    // 1. Let hour be ToZeroPaddedDecimalString(ℝ(HourFromTime(tv)), 2).
    let hour = pad_two(hour_from_time(tv));

    // 2. Let minute be ToZeroPaddedDecimalString(ℝ(MinFromTime(tv)), 2).
    let minute = pad_two(min_from_time(tv));

    // 3. Let second be ToZeroPaddedDecimalString(ℝ(SecFromTime(tv)), 2).
    let second = pad_two(sec_from_time(tv));

    // 4. Return the string-concatenation of
    //  hour,
    //  ":",
    //  minute,
    //  ":",
    //  second,
    //  the code unit 0x0020 (SPACE),
    //  and "GMT".
    js_string!(
        &hour,
        utf16!(":"),
        &minute,
        utf16!(":"),
        &second,
        utf16!(" GMT")
    )
}

/// Abstract operation `DateString ( tv )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-datestring
pub(super) fn date_string(tv: f64) -> JsString {
    // 1. Let weekday be the Name of the entry in Table 63 with the Number WeekDay(tv).
    let weekday = match week_day(tv) {
        0 => utf16!("Sun"),
        1 => utf16!("Mon"),
        2 => utf16!("Tue"),
        3 => utf16!("Wed"),
        4 => utf16!("Thu"),
        5 => utf16!("Fri"),
        6 => utf16!("Sat"),
        _ => unreachable!(),
    };

    // 2. Let month be the Name of the entry in Table 64 with the Number MonthFromTime(tv).
    let month = match month_from_time(tv) {
        0 => utf16!("Jan"),
        1 => utf16!("Feb"),
        2 => utf16!("Mar"),
        3 => utf16!("Apr"),
        4 => utf16!("May"),
        5 => utf16!("Jun"),
        6 => utf16!("Jul"),
        7 => utf16!("Aug"),
        8 => utf16!("Sep"),
        9 => utf16!("Oct"),
        10 => utf16!("Nov"),
        11 => utf16!("Dec"),
        _ => unreachable!(),
    };

    // 3. Let day be ToZeroPaddedDecimalString(ℝ(DateFromTime(tv)), 2).
    let day = pad_two(date_from_time(tv));

    // 4. Let yv be YearFromTime(tv).
    let yv = year_from_time(tv);

    // 5. If yv is +0𝔽 or yv > +0𝔽, let yearSign be the empty String; otherwise, let yearSign be "-".
    let year_sign = if yv >= 0 { utf16!("") } else { utf16!("-") };

    // 6. Let paddedYear be ToZeroPaddedDecimalString(abs(ℝ(yv)), 4).
    let yv = yv.unsigned_abs();
    let padded_year = if yv >= 100_000 {
        js_string!(&pad_six(yv))
    } else if yv >= 10000 {
        js_string!(&pad_five(yv))
    } else {
        js_string!(&pad_four(yv))
    };

    // 7. Return the string-concatenation of
    // weekday,
    // the code unit 0x0020 (SPACE),
    // month,
    // the code unit 0x0020 (SPACE),
    // day,
    // the code unit 0x0020 (SPACE),
    // yearSign,
    // and paddedYear.
    js_string!(
        weekday,
        utf16!(" "),
        month,
        utf16!(" "),
        &day,
        utf16!(" "),
        year_sign,
        &padded_year
    )
}

/// Abstract operation `TimeZoneString ( tv )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-timezoneestring
pub(super) fn time_zone_string(t: f64, hooks: &dyn HostHooks) -> JsString {
    // 1. Let systemTimeZoneIdentifier be SystemTimeZoneIdentifier().
    // 2. If IsTimeZoneOffsetString(systemTimeZoneIdentifier) is true, then
    //     a. Let offsetNs be ParseTimeZoneOffsetString(systemTimeZoneIdentifier).
    // 3. Else,
    //     a. Let offsetNs be GetNamedTimeZoneOffsetNanoseconds(systemTimeZoneIdentifier, ℤ(ℝ(tv) × 10**6)).
    // 4. Let offset be 𝔽(truncate(offsetNs / 10**6)).
    let offset = f64::from(local_timezone_offset_seconds(t, hooks)) * MS_PER_SECOND;
    //let offset = hooks.local_timezone_offset_seconds((t / MS_PER_SECOND).floor() as i64);

    // 5. If offset is +0𝔽 or offset > +0𝔽, then
    let (offset_sign, abs_offset) = if offset >= 0.0 {
        // a. Let offsetSign be "+".
        // b. Let absOffset be offset.
        (utf16!("+"), offset)
    }
    // 6. Else,
    else {
        // a. Let offsetSign be "-".
        // b. Let absOffset be -offset.
        (utf16!("-"), -offset)
    };

    // 7. Let offsetMin be ToZeroPaddedDecimalString(ℝ(MinFromTime(absOffset)), 2).
    let offset_min = pad_two(min_from_time(abs_offset));

    // 8. Let offsetHour be ToZeroPaddedDecimalString(ℝ(HourFromTime(absOffset)), 2).
    let offset_hour = pad_two(hour_from_time(abs_offset));

    // 9. Let tzName be an implementation-defined string that is either the empty String or the
    // string-concatenation of the code unit 0x0020 (SPACE), the code unit 0x0028 (LEFT PARENTHESIS),
    // an implementation-defined timezone name, and the code unit 0x0029 (RIGHT PARENTHESIS).
    // 10. Return the string-concatenation of offsetSign, offsetHour, offsetMin, and tzName.
    js_string!(offset_sign, &offset_hour, &offset_min)
}

/// Abstract operation `ToDateString ( tv )`
///
/// More info:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-todatestring
pub(super) fn to_date_string_t(tv: f64, hooks: &dyn HostHooks) -> JsString {
    // 1. If tv is NaN, return "Invalid Date".
    if tv.is_nan() {
        return JsString::from("Invalid Date");
    }

    // 2. Let t be LocalTime(tv).
    let t = local_time(tv, hooks);

    // 3. Return the string-concatenation of
    // DateString(t),
    // the code unit 0x0020 (SPACE),
    // TimeString(t),
    // and TimeZoneString(tv).
    js_string!(
        &date_string(t),
        utf16!(" "),
        &time_string(t),
        &time_zone_string(t, hooks)
    )
}

fn local_timezone_offset_seconds(t: f64, hooks: &dyn HostHooks) -> i32 {
    let millis = t.rem_euclid(MS_PER_SECOND);
    let seconds = ((t - millis) / MS_PER_SECOND) as i64;
    hooks.local_timezone_offset_seconds(seconds)
}

pub(super) fn pad_two(t: u8) -> [u16; 2] {
    if t < 10 {
        [0x30, 0x30 + u16::from(t)]
    } else {
        [0x30 + (u16::from(t) / 10), 0x30 + (u16::from(t) % 10)]
    }
}

pub(super) fn pad_three(t: u16) -> [u16; 3] {
    [0x30 + t / 100, 0x30 + ((t / 10) % 10), 0x30 + (t % 10)]
}

pub(super) fn pad_four(t: u32) -> [u16; 4] {
    [
        0x30 + (t / 1000) as u16,
        0x30 + ((t / 100) % 10) as u16,
        0x30 + ((t / 10) % 10) as u16,
        0x30 + (t % 10) as u16,
    ]
}

pub(super) fn pad_five(t: u32) -> [u16; 5] {
    [
        0x30 + (t / 10_000) as u16,
        0x30 + ((t / 1000) % 10) as u16,
        0x30 + ((t / 100) % 10) as u16,
        0x30 + ((t / 10) % 10) as u16,
        0x30 + (t % 10) as u16,
    ]
}

pub(super) fn pad_six(t: u32) -> [u16; 6] {
    [
        0x30 + (t / 100_000) as u16,
        0x30 + ((t / 10_000) % 10) as u16,
        0x30 + ((t / 1000) % 10) as u16,
        0x30 + ((t / 100) % 10) as u16,
        0x30 + ((t / 10) % 10) as u16,
        0x30 + (t % 10) as u16,
    ]
}

/// Parse a date string according to the steps specified in [`Date.parse`][spec].
///
/// We parse three different formats:
/// - The [`Date Time String Format`][spec-format] specified in the spec: `YYYY-MM-DDTHH:mm:ss.sssZ`
/// - The `toString` format: `Thu Jan 01 1970 00:00:00 GMT+0000`
/// - The `toUTCString` format: `Thu, 01 Jan 1970 00:00:00 GMT`
///
/// [spec]: https://tc39.es/ecma262/#sec-date.parse
/// [spec-format]: https://tc39.es/ecma262/#sec-date-time-string-format
pub(super) fn parse_date(date: &JsString, hooks: &dyn HostHooks) -> Option<i64> {
    // All characters must be ASCII so we can return early if we find a non-ASCII character.
    let Ok(date) = date.to_std_string() else {
        return None;
    };

    // Date Time String Format: 'YYYY-MM-DDTHH:mm:ss.sssZ'
    if let Some(dt) = DateParser::new(&date, hooks).parse() {
        return Some(dt);
    }

    // `toString` format: `Thu Jan 01 1970 00:00:00 GMT+0000`
    if let Ok(t) = OffsetDateTime::parse(&date, &format_description!("[weekday repr:short] [month repr:short] [day] [year] [hour]:[minute]:[second] GMT[offset_hour sign:mandatory][offset_minute][end]")) {
        return Some(t.unix_timestamp() * 1000 + i64::from(t.millisecond()));
    }

    // `toUTCString` format: `Thu, 01 Jan 1970 00:00:00 GMT`
    if let Ok(t) = PrimitiveDateTime::parse(&date, &format_description!("[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT[end]")) {
        let t = t.assume_utc();
        return Some(t.unix_timestamp() * 1000 + i64::from(t.millisecond()));
    }

    None
}

/// Parses a date string according to the [`Date Time String Format`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-date-time-string-format
struct DateParser<'a> {
    hooks: &'a dyn HostHooks,
    input: Peekable<Chars<'a>>,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
    offset: i64,
}

impl<'a> DateParser<'a> {
    fn new(s: &'a str, hooks: &'a dyn HostHooks) -> Self {
        Self {
            hooks,
            input: s.chars().peekable(),
            year: 0,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
            offset: 0,
        }
    }

    fn next_expect(&mut self, expect: char) -> Option<()> {
        self.input
            .next()
            .and_then(|c| if c == expect { Some(()) } else { None })
    }

    fn next_digit(&mut self) -> Option<u8> {
        self.input.next().and_then(|c| {
            if c.is_ascii_digit() {
                Some((u32::from(c) - u32::from('0')) as u8)
            } else {
                None
            }
        })
    }

    fn finish(&mut self) -> Option<i64> {
        if self.input.peek().is_some() {
            return None;
        }

        let date = make_date(
            make_day(self.year.into(), (self.month - 1).into(), self.day.into()),
            make_time(
                self.hour.into(),
                self.minute.into(),
                self.second.into(),
                self.millisecond.into(),
            ),
        );

        let date = date + (self.offset as f64) * MS_PER_MINUTE;

        let t = time_clip(date);
        if t.is_finite() {
            Some(t as i64)
        } else {
            None
        }
    }

    fn finish_local(&mut self) -> Option<i64> {
        if self.input.peek().is_some() {
            return None;
        }

        let date = make_date(
            make_day(self.year.into(), (self.month - 1).into(), self.day.into()),
            make_time(
                self.hour.into(),
                self.minute.into(),
                self.second.into(),
                self.millisecond.into(),
            ),
        );

        let t = time_clip(utc_t(date, self.hooks));
        if t.is_finite() {
            Some(t as i64)
        } else {
            None
        }
    }

    fn parse(&mut self) -> Option<i64> {
        self.parse_year()?;
        match self.input.peek() {
            Some('T') => return self.parse_time(),
            None => return self.finish(),
            _ => {}
        }
        self.next_expect('-')?;
        self.month = u32::from(self.next_digit()?) * 10 + u32::from(self.next_digit()?);
        if self.month < 1 || self.month > 12 {
            return None;
        }
        match self.input.peek() {
            Some('T') => return self.parse_time(),
            None => return self.finish(),
            _ => {}
        }
        self.next_expect('-')?;
        self.day = u32::from(self.next_digit()?) * 10 + u32::from(self.next_digit()?);
        if self.day < 1 || self.day > 31 {
            return None;
        }
        match self.input.peek() {
            Some('T') => self.parse_time(),
            _ => self.finish(),
        }
    }

    fn parse_year(&mut self) -> Option<()> {
        match self.input.next()? {
            '+' => {
                self.year = i32::from(self.next_digit()?) * 100_000
                    + i32::from(self.next_digit()?) * 10000
                    + i32::from(self.next_digit()?) * 1000
                    + i32::from(self.next_digit()?) * 100
                    + i32::from(self.next_digit()?) * 10
                    + i32::from(self.next_digit()?);
                Some(())
            }
            '-' => {
                let year = i32::from(self.next_digit()?) * 100_000
                    + i32::from(self.next_digit()?) * 10000
                    + i32::from(self.next_digit()?) * 1000
                    + i32::from(self.next_digit()?) * 100
                    + i32::from(self.next_digit()?) * 10
                    + i32::from(self.next_digit()?);
                if year == 0 {
                    return None;
                }
                self.year = -year;
                Some(())
            }
            c if c.is_ascii_digit() => {
                self.year = i32::from((u32::from(c) - u32::from('0')) as u8) * 1000
                    + i32::from(self.next_digit()?) * 100
                    + i32::from(self.next_digit()?) * 10
                    + i32::from(self.next_digit()?);
                Some(())
            }
            _ => None,
        }
    }

    fn parse_time(&mut self) -> Option<i64> {
        self.next_expect('T')?;
        self.hour = u32::from(self.next_digit()?) * 10 + u32::from(self.next_digit()?);
        if self.hour > 24 {
            return None;
        }
        self.next_expect(':')?;
        self.minute = u32::from(self.next_digit()?) * 10 + u32::from(self.next_digit()?);
        if self.minute > 59 {
            return None;
        }
        match self.input.peek() {
            Some(':') => {}
            None => return self.finish_local(),
            _ => {
                self.parse_timezone()?;
                return self.finish();
            }
        }
        self.next_expect(':')?;
        self.second = u32::from(self.next_digit()?) * 10 + u32::from(self.next_digit()?);
        if self.second > 59 {
            return None;
        }
        match self.input.peek() {
            Some('.') => {}
            None => return self.finish_local(),
            _ => {
                self.parse_timezone()?;
                return self.finish();
            }
        }
        self.next_expect('.')?;
        self.millisecond = u32::from(self.next_digit()?) * 100
            + u32::from(self.next_digit()?) * 10
            + u32::from(self.next_digit()?);
        if self.input.peek().is_some() {
            self.parse_timezone()?;
            self.finish()
        } else {
            self.finish_local()
        }
    }

    fn parse_timezone(&mut self) -> Option<()> {
        match self.input.next() {
            Some('Z') => return Some(()),
            Some('+') => {
                let offset_hour =
                    i64::from(self.next_digit()?) * 10 + i64::from(self.next_digit()?);
                if offset_hour > 23 {
                    return None;
                }
                self.offset = -offset_hour * 60;
                if self.input.peek().is_none() {
                    return Some(());
                }
                self.next_expect(':')?;
                let offset_minute =
                    i64::from(self.next_digit()?) * 10 + i64::from(self.next_digit()?);
                if offset_minute > 59 {
                    return None;
                }
                self.offset += -offset_minute;
            }
            Some('-') => {
                let offset_hour =
                    i64::from(self.next_digit()?) * 10 + i64::from(self.next_digit()?);
                if offset_hour > 23 {
                    return None;
                }
                self.offset = offset_hour * 60;
                if self.input.peek().is_none() {
                    return Some(());
                }
                self.next_expect(':')?;
                let offset_minute =
                    i64::from(self.next_digit()?) * 10 + i64::from(self.next_digit()?);
                if offset_minute > 59 {
                    return None;
                }
                self.offset += offset_minute;
            }
            _ => return None,
        }
        Some(())
    }
}
