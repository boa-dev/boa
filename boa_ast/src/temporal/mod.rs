//! AST nodes for Temporal's implementation of ISO8601 grammar.

/// TBD...
#[derive(Default, Debug)]
pub struct IsoParseRecord {
    /// Parsed Date Record
    pub date: DateRecord,
    /// Parsed Time
    pub time: Option<TimeSpec>,
    /// Parsed Offset
    pub offset: Option<UtcOffset>,
    /// Parsed `TimeZoneAnnotation`
    pub tz_annotation: Option<TimeZoneAnnotation>,
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

#[derive(Debug, Default, Clone, Copy)]
/// A `DateTime` Parse Node that contains the date, time, and offset info.
pub struct DateTimeRecord {
    /// Date
    pub date: DateRecord,
    /// Time
    pub time: Option<TimeSpec>,
    /// Tz Offset
    pub offset: Option<UtcOffset>,
}

/// A `TimeZoneAnnotation`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TimeZoneAnnotation {
    /// Critical Flag for the annotation.
    pub critical: bool,
    /// TimeZone Data
    pub tz: TzIdentifier,
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

// NOTE: is it worth consolidating MinutePrecision vs. Offset
/// A UTC Offset that maintains only minute precision.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct UtcOffsetMinutePrecision {
    sign: i8,
    hour: i8,
    minute: i8,
}

/// A full precision `UtcOffset`
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct UtcOffset {
    /// The UTC flag
    pub utc: bool,
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
#[allow(dead_code)]
pub struct KeyValueAnnotation {
    /// An `Annotation`'s Key.
    pub key: String,
    /// An `Annotation`'s value.
    pub value: String,
    /// Whether the annotation was flagged as critical.
    pub critical: bool,
}
