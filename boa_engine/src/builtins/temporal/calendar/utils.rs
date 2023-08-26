use crate::builtins::temporal::{
    self, date_equations, plain_date::iso::IsoDateRecord, TemporalFields,
};
use crate::JsString;

/// 12.2.31 `ISODaysInMonth ( year, month )`
pub(crate) fn iso_days_in_month(year: i32, month: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            28 + temporal::date_equations::mathematical_in_leap_year(
                temporal::date_equations::epoch_time_for_year(year),
            )
        }
        _ => unreachable!("an invalid month value is an implementation error."),
    }
}

/// 12.2.32 ToISOWeekOfYear ( year, month, day )
fn to_iso_week_of_year(year: i32, month: i32, day: i32) -> (i32, i32) {
    // Function constants
    // 2. Let wednesday be 3.
    // 3. Let thursday be 4.
    // 4. Let friday be 5.
    // 5. Let saturday be 6.
    // 6. Let daysInWeek be 7.
    // 7. Let maxWeekNumber be 53.
    let day_of_year = to_iso_day_of_year(year, month, day);
    let day_of_week = to_iso_day_of_week(year, month, day);
    let week = (day_of_week + 7 - day_of_week + 3) / 7;

    if week < 1 {
        let first_day_of_year = to_iso_day_of_week(year, 1, 1);
        if first_day_of_year == 5 {
            return (53, year - 1);
        } else if first_day_of_year == 6
            && date_equations::mathematical_in_leap_year(date_equations::epoch_time_for_year(
                year - 1,
            )) == 1
        {
            return (52, year - 1);
        }
        return (52, year - 1);
    } else if week == 53 {
        let days_in_year = date_equations::mathematical_days_in_year(year);
        let days_later_in_year = days_in_year - day_of_year;
        let days_after_thursday = 4 - day_of_week;
        if days_later_in_year < days_after_thursday {
            return (1, year - 1);
        }
    }
    (week, year)
}

/// 12.2.33 ISOMonthCode ( month )
fn iso_month_code(month: i32) -> JsString {
    // TODO: optimize
    if month < 10 {
        JsString::from(format!("M0{month}"))
    } else {
        JsString::from(format!("M{month}"))
    }
}

// 12.2.34 ISOResolveMonth ( fields )
// Note: currently implemented on TemporalFields -> implement in this mod?

// 12.2.35 ISODateFromFields ( fields, overflow )
// Note: implemented on IsoDateRecord.

// 12.2.36 ISOYearMonthFromFields ( fields, overflow )
// TODO: implement on a IsoYearMonthRecord

// 12.2.37 ISOMonthDayFromFields ( fields, overflow )
// TODO: implement as method on IsoDateRecord.

// 12.2.38 IsoFieldKeysToIgnore
// TODO: determine usefulness.

/// 12.2.39 ToISODayOfYear ( year, month, day )
fn to_iso_day_of_year(year: i32, month: i32, day: i32) -> i32 {
    // TODO: update fn parameter to take IsoDateRecord.
    let iso = IsoDateRecord::new(year, month - 1, day);
    let epoch_days = iso.as_epoch_days();
    date_equations::epoch_time_to_day_in_year(temporal::epoch_days_to_epoch_ms(epoch_days, 0)) + 1
}

/// 12.2.40 ToISODayOfWeek ( year, month, day )
fn to_iso_day_of_week(year: i32, month: i32, day: i32) -> i32 {
    let iso = IsoDateRecord::new(year, month - 1, day);
    let epoch_days = iso.as_epoch_days();
    let day_of_week =
        date_equations::epoch_time_to_week_day(temporal::epoch_days_to_epoch_ms(epoch_days, 0));
    if day_of_week == 0 {
        return 7;
    }
    day_of_week
}
