use std::{error::Error as StdError, fmt, io};

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Syntax(Box<str>),
    StrictMode(Box<str>), // Not 100% decided on this name.
    // Reverted(),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl Error {
    /// Creates a new syntax error.
    pub(super) fn syntax<M>(err: M) -> Self
    where
        M: Into<Box<str>>,
    {
        Self::Syntax(err.into())
    }

    /// Creates a new StrictMode error.
    ///
    /// This error is used to represent the case where a piece of javascript
    /// cannot be lexed/parsed because it is in invalid when strict mdoe is
    /// enabled.
    pub(super) fn strict<M>(err: M) -> Self
    where
        M: Into<Box<str>>,
    {
        Self::StrictMode(err.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "I/O error: {}", e),
            Self::Syntax(e) => write!(f, "Syntax Error: {}", e),
            Self::StrictMode(e) => write!(f, "Strict Mode Error: {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::IO(err) => Some(err),
            Self::Syntax(_) => None,
            Self::StrictMode(_) => None,
        }
    }
}
