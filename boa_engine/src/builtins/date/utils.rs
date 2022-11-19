use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Timelike};

use crate::value::IntegerOrNan;

/// The absolute maximum value of a timestamp
pub(super) const MAX_TIMESTAMP: i64 = 864 * 10i64.pow(13);
/// The number of milliseconds in a second.
pub(super) const MILLIS_PER_SECOND: i64 = 1000;
/// The number of milliseconds in a minute.
pub(super) const MILLIS_PER_MINUTE: i64 = MILLIS_PER_SECOND * 60;
/// The number of milliseconds in an hour.
pub(super) const MILLIS_PER_HOUR: i64 = MILLIS_PER_MINUTE * 60;
/// The number of milliseconds in a day.
pub(super) const MILLIS_PER_DAY: i64 = MILLIS_PER_HOUR * 24;

// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-time-values-and-time-range
//
// The smaller range supported by a time value as specified in this section is approximately -273,790 to 273,790
// years relative to 1970.
pub(super) const MIN_YEAR: i64 = -300_000;
pub(super) const MAX_YEAR: i64 = -MIN_YEAR;
pub(super) const MIN_MONTH: i64 = MIN_YEAR * 12;
pub(super) const MAX_MONTH: i64 = MAX_YEAR * 12;

/// Calculates the absolute day number from the year number.
pub(super) const fn day_from_year(year: i64) -> i64 {
    // Taken from https://chromium.googlesource.com/v8/v8/+/refs/heads/main/src/date/date.cc#496
    // Useful to avoid negative divisions and overflows on 32-bit platforms (if we plan to support them).
    const YEAR_DELTA: i64 = 399_999;
    const fn day(year: i64) -> i64 {
        let year = year + YEAR_DELTA;
        365 * year + year / 4 - year / 100 + year / 400
    }

    assert!(MIN_YEAR <= year && year <= MAX_YEAR);
    day(year) - day(1970)
}

/// Abstract operation [`MakeTime`][spec].
///
/// [spec]: https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-maketime
pub(super) fn make_time(hour: i64, min: i64, sec: i64, ms: i64) -> Option<i64> {
    // 1. If hour is not finite or min is not finite or sec is not finite or ms is not finite, return NaN.
    // 2. Let h be 𝔽(! ToIntegerOrInfinity(hour)).
    // 3. Let m be 𝔽(! ToIntegerOrInfinity(min)).
    // 4. Let s be 𝔽(! ToIntegerOrInfinity(sec)).
    // 5. Let milli be 𝔽(! ToIntegerOrInfinity(ms)).

    // 6. Let t be ((h * msPerHour + m * msPerMinute) + s * msPerSecond) + milli, performing the arithmetic according to IEEE 754-2019 rules (that is, as if using the ECMAScript operators * and +).
    // 7. Return t.

    let h_ms = hour.checked_mul(MILLIS_PER_HOUR)?;
    let m_ms = min.checked_mul(MILLIS_PER_MINUTE)?;
    let s_ms = sec.checked_mul(MILLIS_PER_SECOND)?;

    h_ms.checked_add(m_ms)?.checked_add(s_ms)?.checked_add(ms)
}

/// Abstract operation [`MakeDay`][spec].
///
/// [spec]: https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-makeday
pub(super) fn make_day(mut year: i64, mut month: i64, date: i64) -> Option<i64> {
    // 1. If year is not finite or month is not finite or date is not finite, return NaN.
    // 2. Let y be 𝔽(! ToIntegerOrInfinity(year)).
    // 3. Let m be 𝔽(! ToIntegerOrInfinity(month)).
    // 4. Let dt be 𝔽(! ToIntegerOrInfinity(date)).
    if !(MIN_YEAR..=MAX_YEAR).contains(&year) || !(MIN_MONTH..=MAX_MONTH).contains(&month) {
        return None;
    }

    // At this point, we've already asserted that year and month are much less than its theoretical
    // maximum and minimum values (i64::MAX/MIN), so we don't need to do checked operations.

    // 5. Let ym be y + 𝔽(floor(ℝ(m) / 12)).
    // 6. If ym is not finite, return NaN.
    year += month / 12;
    // 7. Let mn be 𝔽(ℝ(m) modulo 12).
    month %= 12;
    if month < 0 {
        month += 12;
        year -= 1;
    }

    // 8. Find a finite time value t such that YearFromTime(t) is ym and MonthFromTime(t) is mn and DateFromTime(t) is
    // 1𝔽; but if this is not possible (because some argument is out of range), return NaN.
    let month = usize::try_from(month).expect("month must be between 0 and 11 at this point");

    let mut day = day_from_year(year);

    // Consider leap years when calculating the cumulative days added to the year from the input month
    if (year % 4 != 0) || (year % 100 == 0 && year % 400 != 0) {
        day += [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334][month];
    } else {
        day += [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335][month];
    }

    // 9. Return Day(t) + dt - 1𝔽.
    (day - 1).checked_add(date)
}

/// Abstract operation [`MakeDate`][spec].
///
/// [spec]: https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-makedate
pub(super) fn make_date(day: i64, time: i64) -> Option<i64> {
    // 1. If day is not finite or time is not finite, return NaN.
    // 2. Let tv be day × msPerDay + time.
    // 3. If tv is not finite, return NaN.
    // 4. Return tv.
    day.checked_mul(MILLIS_PER_DAY)?.checked_add(time)
}

/// Abstract operation [`TimeClip`][spec]
/// Returns the timestamp (number of milliseconds) if it is in the expected range.
/// Otherwise, returns `None`.
///
/// [spec]: https://tc39.es/ecma262/#sec-timeclip
#[inline]
pub(super) fn time_clip(time: i64) -> Option<i64> {
    // 1. If time is not finite, return NaN.
    // 2. If abs(ℝ(time)) > 8.64 × 10^15, return NaN.
    // 3. Return 𝔽(! ToIntegerOrInfinity(time)).
    (time.checked_abs()? <= MAX_TIMESTAMP).then_some(time)
}

#[derive(Default, Debug, Clone, Copy)]
pub(super) struct DateParameters {
    pub(super) year: Option<IntegerOrNan>,
    pub(super) month: Option<IntegerOrNan>,
    pub(super) date: Option<IntegerOrNan>,
    pub(super) hour: Option<IntegerOrNan>,
    pub(super) minute: Option<IntegerOrNan>,
    pub(super) second: Option<IntegerOrNan>,
    pub(super) millisecond: Option<IntegerOrNan>,
}

/// Replaces some (or all) parameters of `date` with the specified parameters
pub(super) fn replace_params(
    datetime: NaiveDateTime,
    params: DateParameters,
    local: bool,
) -> Option<NaiveDateTime> {
    let DateParameters {
        year,
        month,
        date,
        hour,
        minute,
        second,
        millisecond,
    } = params;

    let datetime = if local {
        Local.from_utc_datetime(&datetime).naive_local()
    } else {
        datetime
    };

    let year = match year {
        Some(i) => i.as_integer()?,
        None => i64::from(datetime.year()),
    };
    let month = match month {
        Some(i) => i.as_integer()?,
        None => i64::from(datetime.month() - 1),
    };
    let date = match date {
        Some(i) => i.as_integer()?,
        None => i64::from(datetime.day()),
    };
    let hour = match hour {
        Some(i) => i.as_integer()?,
        None => i64::from(datetime.hour()),
    };
    let minute = match minute {
        Some(i) => i.as_integer()?,
        None => i64::from(datetime.minute()),
    };
    let second = match second {
        Some(i) => i.as_integer()?,
        None => i64::from(datetime.second()),
    };
    let millisecond = match millisecond {
        Some(i) => i.as_integer()?,
        None => i64::from(datetime.timestamp_subsec_millis()),
    };

    let new_day = make_day(year, month, date)?;
    let new_time = make_time(hour, minute, second, millisecond)?;
    let mut ts = make_date(new_day, new_time)?;

    if local {
        ts = Local
            .from_local_datetime(&NaiveDateTime::from_timestamp_millis(ts)?)
            .earliest()?
            .naive_utc()
            .timestamp_millis();
    }

    NaiveDateTime::from_timestamp_millis(time_clip(ts)?)
}
