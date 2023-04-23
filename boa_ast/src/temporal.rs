//! AST nodes for Temporal.

#[derive(Debug, Clone, Copy)]
#[allow(dead_code, missing_docs)]
pub enum OffsetSign {
    Positive,
    Negative,
}

/// UTC offset information.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-UTCOffset
#[derive(Debug, Clone)]
#[allow(dead_code, missing_docs)]
pub struct UtcOffset {
    pub sign: OffsetSign,
    pub hour: String,
    pub minute: Option<String>,
    pub second: Option<String>,
    pub fraction: Option<String>,
}
