//! An error type for Temporal Errors.

use core::fmt;

/// `TemporalError`'s error type.
#[derive(Debug, Default, Clone, Copy)]
pub enum ErrorKind {
    /// Error.
    #[default]
    Generic,
    /// TypeError
    Type,
    /// RangeError
    Range,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic => "Error",
            Self::Type => "TypeError",
            Self::Range => "RangeError",
        }
        .fmt(f)
    }
}

/// The error type for `boa_temporal`.
#[derive(Debug, Clone)]
pub struct TemporalError {
    kind: ErrorKind,
    msg: Box<str>,
}

impl TemporalError {
    fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            msg: Box::default(),
        }
    }

    /// Create a generic error
    #[must_use]
    pub fn general<S>(msg: S) -> Self
    where
        S: Into<Box<str>>,
    {
        Self::new(ErrorKind::Generic).with_message(msg)
    }

    /// Create a range error.
    #[must_use]
    pub fn range() -> Self {
        Self::new(ErrorKind::Range)
    }

    /// Create a type error.
    #[must_use]
    pub fn r#type() -> Self {
        Self::new(ErrorKind::Type)
    }

    /// Add a message to the error.
    #[must_use]
    pub fn with_message<S>(mut self, msg: S) -> Self
    where
        S: Into<Box<str>>,
    {
        self.msg = msg.into();
        self
    }

    /// Returns this error's kind.
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for TemporalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;

        let msg = self.msg.trim();
        if !msg.is_empty() {
            write!(f, ": {msg}")?;
        }

        Ok(())
    }
}
