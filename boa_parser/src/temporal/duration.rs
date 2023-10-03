use boa_ast::Position;

use crate::{
    error::{Error, ParseResult},
    temporal::{
        grammar::{
            is_day_designator, is_decimal_separator, is_duration_designator, is_hour_designator,
            is_minute_designator, is_month_designator, is_second_designator, is_sign,
            is_time_designator, is_week_designator, is_year_designator,
        },
        time::parse_fraction,
        IsoCursor,
    },
};

/// A ISO8601 `DurationRecord` Parse Node.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DurationParseRecord {
    /// Duration Sign
    pub(crate) sign: bool,
    /// A `DateDuration` record.
    pub(crate) date: DateDuration,
    /// A `TimeDuration` record.
    pub(crate) time: TimeDuration,
}

/// A `DateDuration` Parse Node.
#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct DateDuration {
    /// Years value.
    pub(crate) years: i32,
    /// Months value.
    pub(crate) months: i32,
    /// Weeks value.
    pub(crate) weeks: i32,
    /// Days value.
    pub(crate) days: i32,
}

/// A `TimeDuration` Parse Node
#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct TimeDuration {
    /// Hours value.
    pub(crate) hours: i32,
    /// Hours fraction value.
    pub(crate) fhours: f64,
    /// Minutes value with fraction.
    pub(crate) minutes: i32,
    /// Minutes fraction value.
    pub(crate) fminutes: f64,
    /// Seconds value with fraction.
    pub(crate) seconds: i32,
    /// Seconds fraction value,
    pub(crate) fseconds: f64,
}

pub(crate) fn parse_duration(cursor: &mut IsoCursor) -> ParseResult<DurationParseRecord> {
    let sign = if cursor.check(is_sign).ok_or_else(|| Error::AbruptEnd)? {
        let sign = cursor.check_or(false, |ch| ch == '+');
        cursor.advance();
        sign
    } else {
        true
    };

    if !cursor
        .check(is_duration_designator)
        .ok_or_else(|| Error::AbruptEnd)?
    {
        return Err(Error::general(
            "DurationString missing DurationDesignator.",
            Position::new(1, cursor.pos() + 1),
        ));
    }

    cursor.advance();

    let date = if cursor.check_or(false, is_time_designator) {
        Some(DateDuration::default())
    } else {
        Some(parse_date_duration(cursor)?)
    };

    let time = if cursor.check_or(false, is_time_designator) {
        cursor.advance();
        Some(parse_time_duration(cursor)?)
    } else {
        None
    };

    if cursor.peek().is_some() {
        return Err(Error::general(
            "Unrecognized value in DurationString.",
            Position::new(1, cursor.pos()),
        ));
    }

    Ok(DurationParseRecord {
        sign,
        date: date.unwrap_or_default(),
        time: time.unwrap_or_default(),
    })
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum DateUnit {
    None = 0,
    Year,
    Month,
    Week,
    Day,
}

pub(crate) fn parse_date_duration(cursor: &mut IsoCursor) -> ParseResult<DateDuration> {
    let mut date = DateDuration::default();

    let mut previous_unit = DateUnit::None;
    while cursor.check_or(false, |ch| ch.is_ascii_digit()) {
        let digit_start = cursor.pos();

        while cursor.check_or(false, |ch| ch.is_ascii_digit()) {
            cursor.advance();
        }

        let value = cursor
            .slice(digit_start, cursor.pos())
            .parse::<i32>()
            .map_err(|err| {
                Error::general(err.to_string(), Position::new(digit_start, cursor.pos()))
            })?;

        match cursor.peek() {
            Some(ch) if is_year_designator(ch) => {
                if previous_unit > DateUnit::Year {
                    return Err(Error::general(
                        "Not a valid DateDuration order",
                        Position::new(1, cursor.pos()),
                    ));
                }
                date.years = value;
                previous_unit = DateUnit::Year;
            }
            Some(ch) if is_month_designator(ch) => {
                if previous_unit > DateUnit::Month {
                    return Err(Error::general(
                        "Not a valid DateDuration order",
                        Position::new(1, cursor.pos()),
                    ));
                }
                date.months = value;
                previous_unit = DateUnit::Month;
            }
            Some(ch) if is_week_designator(ch) => {
                if previous_unit > DateUnit::Week {
                    return Err(Error::general(
                        "Not a valid DateDuration order",
                        Position::new(1, cursor.pos()),
                    ));
                }
                date.weeks = value;
                previous_unit = DateUnit::Week;
            }
            Some(ch) if is_day_designator(ch) => {
                if previous_unit > DateUnit::Day {
                    return Err(Error::general(
                        "Not a valid DateDuration order",
                        Position::new(1, cursor.pos()),
                    ));
                }
                date.days = value;
                previous_unit = DateUnit::Day;
            }
            Some(_) | None => return Err(Error::AbruptEnd),
        }

        cursor.advance();
    }

    Ok(date)
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum TimeUnit {
    None = 0,
    Hour,
    Minute,
    Second,
}

pub(crate) fn parse_time_duration(cursor: &mut IsoCursor) -> ParseResult<TimeDuration> {
    let mut time = TimeDuration::default();

    if !cursor.check_or(false, |ch| ch.is_ascii()) {
        return Err(Error::general(
            "No time values provided after TimeDesignator.",
            Position::new(1, cursor.pos()),
        ));
    }

    let mut previous_unit = TimeUnit::None;
    let mut fraction_present = false;
    while cursor.check_or(false, |ch| ch.is_ascii_digit()) {
        let digit_start = cursor.pos();

        while cursor.check_or(false, |ch| ch.is_ascii_digit()) {
            cursor.advance();
        }

        let value = cursor
            .slice(digit_start, cursor.pos())
            .parse::<i32>()
            .map_err(|err| {
                Error::general(err.to_string(), Position::new(digit_start, cursor.pos()))
            })?;

        let fraction = if cursor.check_or(false, is_decimal_separator) {
            fraction_present = true;
            parse_fraction(cursor)?
        } else {
            0.0
        };

        match cursor.peek() {
            Some(ch) if is_hour_designator(ch) => {
                if previous_unit > TimeUnit::Hour {
                    return Err(Error::general(
                        "Not a valid DateDuration order",
                        Position::new(1, cursor.pos()),
                    ));
                }
                time.hours = value;
                time.fhours = fraction;
                previous_unit = TimeUnit::Hour;
            }
            Some(ch) if is_minute_designator(ch) => {
                if previous_unit > TimeUnit::Minute {
                    return Err(Error::general(
                        "Not a valid DateDuration order",
                        Position::new(1, cursor.pos()),
                    ));
                }
                time.minutes = value;
                time.fminutes = fraction;
                previous_unit = TimeUnit::Minute;
            }
            Some(ch) if is_second_designator(ch) => {
                if previous_unit > TimeUnit::Second {
                    return Err(Error::general(
                        "Not a valid DateDuration order",
                        Position::new(1, cursor.pos()),
                    ));
                }
                time.seconds = value;
                time.fseconds = fraction;
                previous_unit = TimeUnit::Second;
            }
            Some(_) | None => return Err(Error::AbruptEnd),
        }

        cursor.advance();

        if fraction_present {
            if cursor.check_or(false, |ch| ch.is_ascii_digit()) {
                return Err(Error::general(
                    "Invalid TimeDuration continuation after FractionPart.",
                    Position::new(1, cursor.pos()),
                ));
            }

            break;
        }
    }

    Ok(time)
}
