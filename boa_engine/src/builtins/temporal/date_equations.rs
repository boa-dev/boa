//! This file represents all equations listed under section 13.4 of the [Temporal Specification][spec]
//!
//! [spec]: https://tc39.es/proposal-temporal/#sec-date-equations

use std::ops::Mul;

pub(crate) fn epoch_time_to_day_number(t: f64) -> f64 {
    (t / super::NS_PER_DAY as f64).floor()
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
    365_f64.mul_add(y - 1970_f64, ((y - 1969_f64) / 4_f64).floor())
        - ((y - 1901_f64) / 100_f64).floor()
        + ((y - 1601_f64) / 400_f64).floor()
}

pub(crate) fn epoch_time_for_year(y: f64) -> f64 {
    super::NS_PER_DAY as f64 * epoch_day_number_for_year(y)
}

pub(crate) fn epoch_time_to_epoch_year(t: f64) -> f64 {
    // roughly calculate the largest possible year given the time t,
    // then check and refine the year.
    let day_count = epoch_time_to_day_number(t);
    let mut year = (day_count / 365_f64).floor();
    loop {
        if epoch_time_for_year(year) <= t {
            break;
        }
        year -= 1_f64;
    }

    year
}

pub(crate) fn mathematical_in_leap_year(t: f64) -> i32 {
    mathematical_days_in_year(epoch_time_to_epoch_year(t) as i32)
}

pub(crate) fn epoch_time_to_month_in_year(t: f64) -> i32 {
    let days = epoch_time_to_day_in_year(t);
    let in_leap_year = mathematical_in_leap_year(t) == 1;

    match days {
        0..=30 => 0,
        31..=59 if in_leap_year => 1,
        31..=58 => 1,
        60..=90 if in_leap_year => 2,
        59..=89 => 2,
        91..=121 if in_leap_year => 3,
        90..=120 => 3,
        122..=151 if in_leap_year => 4,
        121..=150 => 4,
        152..=182 if in_leap_year => 5,
        151..=181 => 5,
        183..=213 if in_leap_year => 6,
        182..=212 => 6,
        214..=243 if in_leap_year => 7,
        213..=242 => 7,
        244..=273 if in_leap_year => 8,
        243..=272 => 8,
        274..=304 if in_leap_year => 9,
        273..=303 => 9,
        305..=334 if in_leap_year => 10,
        304..=333 => 10,
        335..=366 if in_leap_year => 11,
        334..=365 => 11,
        _ => unreachable!(),
    }
}

pub(crate) fn epoch_time_for_month_given_year(m: i32, y: i32) -> f64 {
    let leap_day = i32::from(mathematical_days_in_year(y) == 366);

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

pub(crate) fn epoch_time_to_day_in_year(t: f64) -> i32 {
    (epoch_time_to_day_number(t) - epoch_day_number_for_year(epoch_time_to_epoch_year(t))) as i32
}
