use std::fmt::{Debug, Display};

/// TODO: doc
#[derive(Debug)]
pub enum SnapshotError {
    /// Input/output error.
    ///
    /// See: [`std::io::Error`].
    Io(std::io::Error),
}

impl Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: implement better formatting
        <Self as Debug>::fmt(self, f)
    }
}

impl std::error::Error for SnapshotError {}

impl From<std::io::Error> for SnapshotError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
