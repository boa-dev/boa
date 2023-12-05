//! AST nodes for Temporal's implementation of ISO8601 grammar.

// TODO: Slowly remove the below nodes in favor of Temporal components.

/// An ISO Date Node consisting of non-validated date fields and calendar value.
#[derive(Default, Debug)]
pub struct IsoDate {
    /// Date Year
    pub year: i32,
    /// Date Month
    pub month: i32,
    /// Date Day
    pub day: i32,
    /// The calendar value.
    pub calendar: Option<String>,
}

/// The `IsoTime` node consists of non-validated time fields.
#[derive(Default, Debug, Clone, Copy)]
pub struct IsoTime {
    /// An hour value between 0-23
    pub hour: u8,
    /// A minute value between 0-59
    pub minute: u8,
    /// A second value between 0-60
    pub second: u8,
    /// A millisecond value between 0-999
    pub millisecond: u16,
    /// A microsecond value between 0-999
    pub microsecond: u16,
    /// A nanosecond value between 0-999
    pub nanosecond: u16,
}

impl IsoTime {
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    /// A utility initialization function to create `ISOTime` from the `TimeSpec` components.
    pub fn from_components(hour: u8, minute: u8, second: u8, fraction: f64) -> Self {
        // Note: Precision on nanoseconds drifts, so opting for round over floor or ceil for now.
        // e.g. 0.329402834 becomes 329.402833.999
        let millisecond = fraction * 1000f64;
        let micros = millisecond.rem_euclid(1f64) * 1000f64;
        let nanos = micros.rem_euclid(1f64) * 1000f64;

        Self {
            hour,
            minute,
            second,
            millisecond: millisecond.floor() as u16,
            microsecond: micros.floor() as u16,
            nanosecond: nanos.round() as u16,
        }
    }
}

/// The `IsoDateTime` node output by the ISO parser
#[derive(Default, Debug)]
pub struct IsoDateTime {
    /// The `ISODate` record
    pub date: IsoDate,
    /// The `ISOTime` record
    pub time: IsoTime,
    /// The `TimeZone` value for this `ISODateTime`
    pub tz: Option<TimeZone>,
}

/// `TimeZone` data
#[derive(Default, Debug, Clone)]
pub struct TimeZone {
    /// TimeZoneIANAName
    pub name: Option<String>,
    /// TimeZoneOffset
    pub offset: Option<UTCOffset>,
}

/// A full precision `UtcOffset`
#[derive(Debug, Clone, Copy)]
pub struct UTCOffset {
    /// The `+`/`-` sign of this `UtcOffset`
    pub sign: i8,
    /// The hour value of the `UtcOffset`
    pub hour: u8,
    /// The minute value of the `UtcOffset`.
    pub minute: u8,
    /// The second value of the `UtcOffset`.
    pub second: u8,
    /// Any sub second components of the `UTCOffset`
    pub fraction: f64,
}
