//! This module implements the `Pos` structure, which represents a position in the source code.

#[cfg(feature = "serde-ast")]
use serde::{Deserialize, Serialize};

/// A position in the Javascript source code.
///
/// Stores both the column number and the line number
///
/// ## Similar Implementations
/// [V8: Location](https://cs.chromium.org/chromium/src/v8/src/parsing/scanner.h?type=cs&q=isValid+Location&g=0&l=216)
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Position {
    // Column number
    pub column_number: u64,
    // Line number
    pub line_number: u64,
}

impl Position {
    /// Creates a new `Position`.
    ///
    /// Positions are usually created by a [`Token`](struct.token/Token.html).
    ///
    /// # Arguments
    ///
    /// * `line_number` - The line number the token starts at
    /// * `column_number` - The column number the token starts at
    pub fn new(line_number: u64, column_number: u64) -> Self {
        Self {
            line_number,
            column_number,
        }
    }
}
