//! This module contains the errors used by the lexer.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard

use boa_ast::Position;
use std::{error, fmt, io};

/// An error that occurred during the lexing.
#[derive(Debug)]
pub enum Error {
    /// An IO error is raised to indicate an issue when the lexer is reading data that isn't
    /// related to the sourcecode itself.
    IO(io::Error),

    /// Indicates a parsing error due to the presence, or lack of, one or more characters.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-syntaxerror
    Syntax(Box<str>, Position),
}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl Error {
    /// Creates a new syntax error.
    #[inline]
    pub(super) fn syntax<M, P>(err: M, pos: P) -> Self
    where
        M: Into<Box<str>>,
        P: Into<Position>,
    {
        Self::Syntax(err.into(), pos.into())
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "{e}"),
            Self::Syntax(e, pos) => write!(f, "{e} at position: {pos}"),
        }
    }
}

impl error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IO(err) => Some(err),
            Self::Syntax(_, _) => None,
        }
    }
}
