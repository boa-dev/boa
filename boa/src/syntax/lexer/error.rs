//! This module contains the errors used by the lexer.

use std::{error::Error as StdError, fmt, io};

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Syntax(Box<str>),
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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "I/O error: {}", e),
            Self::Syntax(e) => write!(f, "Syntax Error: {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::IO(err) => Some(err),
            Self::Syntax(_) => None,
        }
    }
}
