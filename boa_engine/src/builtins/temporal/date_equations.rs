//! This file represents all equations listed under section 13.4 of the [Temporal Specification][spec]
//!
//! [spec]: https://tc39.es/proposal-temporal/#sec-date-equations

use std::ops::Mul;

pub(crate) fn epoch_time_to_day_number(t: f64) -> i32 {
    (t / f64::from(super::MS_PER_DAY)).floor() as i32
}

pub(crate) fn mathematical_days_in_year(y: i32) -> i32 {
    if y % 4 != 0 {
        365
    } else if y % 4 == 0 && y % 100 != 0 {
        366
    } else if y % 100 == 0 && y % 400 != 0 {
        365
    } else {
        // Assert that y is divisble by 400 to ensure we are returning the correct result.
        assert_eq!(y % 400, 0);
        366
    }
}

pub(crate) fn epoch_day_number_for_year(y: f64) -> f64 {
    365.0f64.mul_add(y - 1970.0, ((y - 1969.0) / 4.0).floor()) - ((y - 1901.0) / 100.0).floor()
        + ((y - 1601.0) / 400.0).floor()
}

pub(crate) fn epoch_time_for_year(y: i32) -> f64 {
    f64::from(super::MS_PER_DAY) * epoch_day_number_for_year(f64::from(y))
}

// NOTE: The below returns the epoch years (years since 1970). The spec
// appears to assume the below returns with the epoch applied.
pub(crate) fn epoch_time_to_epoch_year(t: f64) -> i32 {
    // roughly calculate the largest possible year given the time t,
    // then check and refine the year.
    let day_count = epoch_time_to_day_number(t);
    let mut year = day_count / 365;
    loop {
        if epoch_time_for_year(year) <= t {
            break;
        }
        year -= 1;
    }

    year + 1970
}

/// Returns either 1 (true) or 0 (false)
pub(crate) fn mathematical_in_leap_year(t: f64) -> i32 {
    mathematical_days_in_year(epoch_time_to_epoch_year(t)) - 365
}

pub(crate) fn epoch_time_to_month_in_year(t: f64) -> i32 {
    const DAYS: [i32; 11] = [30, 58, 89, 120, 150, 181, 212, 242, 272, 303, 333];
    const LEAP_DAYS: [i32; 11] = [30, 59, 90, 121, 151, 182, 213, 242, 272, 303, 334];

    let in_leap_year = mathematical_in_leap_year(t) == 1;
    let day = epoch_time_to_day_in_year(t);

    let result = if in_leap_year {
        LEAP_DAYS.binary_search(&day)
    } else {
        DAYS.binary_search(&day)
    };

    match result {
        Ok(i) | Err(i) => i as i32,
    }
}

pub(crate) fn epoch_time_for_month_given_year(m: i32, y: i32) -> f64 {
    let leap_day = mathematical_days_in_year(y) - 365;

    let days = match m {
        0 => 1,
        1 => 31,
        2 => 59 + leap_day,
        3 => 90 + leap_day,
        4 => 121 + leap_day,
        5 => 151 + leap_day,
        6 => 182 + leap_day,
        7 => 213 + leap_day,
        8 => 243 + leap_day,
        9 => 273 + leap_day,
        10 => 304 + leap_day,
        11 => 334 + leap_day,
        _ => unreachable!(),
    };

    (super::NS_PER_DAY as f64).mul(f64::from(days))
}

pub(crate) fn epoch_time_to_date(t: f64) -> i32 {
    const OFFSETS: [i16; 12] = [
        1, -30, -58, -89, -119, -150, -180, -211, -242, -272, -303, -333,
    ];
    let day_in_year = epoch_time_to_day_in_year(t);
    let in_leap_year = mathematical_in_leap_year(t);
    let month = epoch_time_to_month_in_year(t);

    // Cast from i32 to usize should be safe as the return must be 0-11
    let mut date = day_in_year + i32::from(OFFSETS[month as usize]);

    if month >= 2 {
        date -= in_leap_year;
    }

    date
}

pub(crate) fn epoch_time_to_day_in_year(t: f64) -> i32 {
    epoch_time_to_day_number(t)
        - (epoch_day_number_for_year(f64::from(epoch_time_to_epoch_year(t))) as i32)
}

pub(crate) fn epoch_time_to_week_day(t: f64) -> i32 {
    (epoch_time_to_day_number(t) + 4) % 7
}
