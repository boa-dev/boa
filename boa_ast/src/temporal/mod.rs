//! AST nodes for Temporal's implementation of ISO8601 grammar.

/// An `ISOParseRecord` is the full record of a node that is returned via the parse records.
///
/// This node comes complete with the parsed date, time, time zone, and calendar data.
#[derive(Default, Debug)]
pub struct IsoParseRecord {
    /// Parsed Date Record
    pub date: DateRecord,
    /// Parsed Time
    pub time: Option<TimeSpec>,
    /// Parsed Offset
    /// Parsed `TimeZoneAnnotation`
    pub tz: Option<TimeZone>,
    /// Parsed Annotations
    pub calendar: Option<String>,
}

#[derive(Default, Debug, Clone, Copy)]
/// The record of a parsed date.
pub struct DateRecord {
    /// Date Year
    pub year: i32,
    /// Date Month
    pub month: i32,
    /// Date Day
    pub day: i32,
}

/// Parsed Time info
#[derive(Debug, Default, Clone, Copy)]
#[allow(dead_code)]
pub struct TimeSpec {
    /// An hour
    pub hour: i8,
    /// A minute value
    pub minute: i8,
    /// A floating point second value.
    pub second: f64,
}

/// `TimeZone` UTC Offset info.
#[derive(Debug, Clone, Copy)]
pub struct DateTimeUtcOffset;

#[derive(Debug, Default, Clone)]
/// A `DateTime` Parse Node that contains the date, time, and offset info.
pub struct DateTimeRecord {
    /// Date
    pub date: DateRecord,
    /// Time
    pub time: Option<TimeSpec>,
    /// Tz Offset
    pub time_zone: Option<TimeZone>,
}

/// A `TimeZoneAnnotation`.
#[derive(Debug, Clone)]
pub struct TimeZoneAnnotation {
    /// Critical Flag for the annotation.
    pub critical: bool,
    /// TimeZone Data
    pub tz: TimeZone,
}

/// `TimeZone` data
#[derive(Default, Debug, Clone)]
pub struct TimeZone {
    /// TimeZoneIANAName
    pub name: Option<String>,
    /// TimeZoneOffset
    pub offset: Option<UtcOffset>,
}

/// A valid `TimeZoneIdentifier` that is defined by
/// the specification as either a UTC Offset to minute
/// precision or a `TimeZoneIANAName`
#[derive(Debug, Clone)]
pub enum TzIdentifier {
    /// A valid UTC `TimeZoneIdentifier` value
    UtcOffset(UtcOffset),
    /// A valid IANA name `TimeZoneIdentifier` value
    TzIANAName(String),
}

/// A full precision `UtcOffset`
#[derive(Debug, Clone, Copy)]
pub struct UtcOffset {
    /// The `+`/`-` sign of this `UtcOffset`
    pub sign: i8,
    /// The hour value of the `UtcOffset`
    pub hour: i8,
    /// The minute value of the `UtcOffset`.
    pub minute: i8,
    /// A float representing the second value of the `UtcOffset`.
    pub second: f64,
}

/// A `KeyValueAnnotation` Parse Node.
#[derive(Debug, Clone)]
pub struct KeyValueAnnotation {
    /// An `Annotation`'s Key.
    pub key: String,
    /// An `Annotation`'s value.
    pub value: String,
    /// Whether the annotation was flagged as critical.
    pub critical: bool,
}

/// A ISO8601 `DurationRecord` Parse Node.
#[derive(Debug, Clone, Copy)]
pub struct DurationParseRecord {
    /// Duration Sign
    pub sign: bool,
    /// A `DateDuration` record.
    pub date: DateDuration,
    /// A `TimeDuration` record.
    pub time: TimeDuration,
}

/// A `DateDuration` Parse Node.
#[derive(Default, Debug, Clone, Copy)]
pub struct DateDuration {
    /// Years value.
    pub years: i32,
    /// Months value.
    pub months: i32,
    /// Weeks value.
    pub weeks: i32,
    /// Days value.
    pub days: i32,
}

/// A `TimeDuration` Parse Node
#[derive(Default, Debug, Clone, Copy)]
pub struct TimeDuration {
    /// Hours value with fraction.
    pub hours: f64,
    /// Minutes value with fraction.
    pub minutes: f64,
    /// Seconds value with fraction.
    pub seconds: f64,
}
