use std::{cmp::Ordering, fmt, num::NonZeroU32};

/// A position in the JavaScript source code.
///
/// Stores both the column number and the line number.
///
/// ## Similar Implementations
/// [V8: Location](https://cs.chromium.org/chromium/src/v8/src/parsing/scanner.h?type=cs&q=isValid+Location&g=0&l=216)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    /// Line number.
    line_number: NonZeroU32,
    /// Column number.
    column_number: NonZeroU32,
}

impl Position {
    /// Creates a new `Position`.
    #[inline]
    #[track_caller]
    #[must_use]
    pub fn new(line_number: u32, column_number: u32) -> Self {
        Self {
            line_number: NonZeroU32::new(line_number).expect("line number cannot be 0"),
            column_number: NonZeroU32::new(column_number).expect("column number cannot be 0"),
        }
    }

    /// Gets the line number of the position.
    #[inline]
    #[must_use]
    pub const fn line_number(self) -> u32 {
        self.line_number.get()
    }

    /// Gets the column number of the position.
    #[inline]
    #[must_use]
    pub const fn column_number(self) -> u32 {
        self.column_number.get()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line_number, self.column_number)
    }
}

/// A span in the JavaScript source code.
///
/// Stores a start position and an end position.
///
/// Note that spans are of the form [start, end) i.e. that the start position is inclusive
/// and the end position is exclusive.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    start: Position,
    end: Position,
}

impl Span {
    /// Creates a new `Span`.
    ///
    /// # Panics
    ///
    /// Panics if the start position is bigger than the end position.
    #[inline]
    #[track_caller]
    #[must_use]
    pub fn new(start: Position, end: Position) -> Self {
        assert!(start <= end, "a span cannot start after its end");

        Self { start, end }
    }

    /// Gets the starting position of the span.
    #[inline]
    #[must_use]
    pub const fn start(self) -> Position {
        self.start
    }

    /// Gets the final position of the span.
    #[inline]
    #[must_use]
    pub const fn end(self) -> Position {
        self.end
    }

    /// Checks if this span inclusively contains another span or position.
    #[inline]
    pub fn contains<S>(self, other: S) -> bool
    where
        S: Into<Self>,
    {
        let other = other.into();
        self.start <= other.start && self.end >= other.end
    }
}

impl From<Position> for Span {
    fn from(pos: Position) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.end < other.start {
            Some(Ordering::Less)
        } else if self.start > other.end {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}..{}]", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::similar_names)]
    #![allow(unused_must_use)]
    use super::{Position, Span};

    /// Checks that we cannot create a position with 0 as the column.
    #[test]
    #[should_panic]
    fn invalid_position_column() {
        Position::new(10, 0);
    }

    /// Checks that we cannot create a position with 0 as the line.
    #[test]
    #[should_panic]
    fn invalid_position_line() {
        Position::new(0, 10);
    }

    /// Checks that the `PartialEq` implementation of `Position` is consistent.
    #[test]
    fn position_equality() {
        assert_eq!(Position::new(10, 50), Position::new(10, 50));
        assert_ne!(Position::new(10, 50), Position::new(10, 51));
        assert_ne!(Position::new(10, 50), Position::new(11, 50));
        assert_ne!(Position::new(10, 50), Position::new(11, 51));
    }

    /// Checks that the `PartialOrd` implementation of `Position` is consistent.
    #[test]
    fn position_order() {
        assert!(Position::new(10, 50) < Position::new(10, 51));
        assert!(Position::new(9, 50) < Position::new(10, 50));
        assert!(Position::new(10, 50) < Position::new(11, 51));
        assert!(Position::new(10, 50) < Position::new(11, 49));

        assert!(Position::new(10, 51) > Position::new(10, 50));
        assert!(Position::new(10, 50) > Position::new(9, 50));
        assert!(Position::new(11, 51) > Position::new(10, 50));
        assert!(Position::new(11, 49) > Position::new(10, 50));
    }

    /// Checks that the position getters actually retrieve correct values.
    #[test]
    fn position_getters() {
        let pos = Position::new(10, 50);
        assert_eq!(pos.line_number(), 10);
        assert_eq!(pos.column_number(), 50);
    }

    /// Checks that the string representation of a position is correct.
    #[test]
    fn position_to_string() {
        let pos = Position::new(10, 50);

        assert_eq!("10:50", pos.to_string());
        assert_eq!("10:50", pos.to_string());
    }

    /// Checks that we cannot create an invalid span.
    #[test]
    #[should_panic]
    fn invalid_span() {
        let a = Position::new(10, 30);
        let b = Position::new(10, 50);
        Span::new(b, a);
    }

    /// Checks that we can create valid spans.
    #[test]
    fn span_creation() {
        let a = Position::new(10, 30);
        let b = Position::new(10, 50);

        let _ = Span::new(a, b);
        let _ = Span::new(a, a);
        let _ = Span::from(a);
    }

    /// Checks that the `PartialEq` implementation of `Span` is consistent.
    #[test]
    fn span_equality() {
        let a = Position::new(10, 50);
        let b = Position::new(10, 52);
        let c = Position::new(11, 20);

        let span_ab = Span::new(a, b);
        let span_ab_2 = Span::new(a, b);
        let span_ac = Span::new(a, c);
        let span_bc = Span::new(b, c);

        assert_eq!(span_ab, span_ab_2);
        assert_ne!(span_ab, span_ac);
        assert_ne!(span_ab, span_bc);
        assert_ne!(span_bc, span_ac);

        let span_a = Span::from(a);
        let span_aa = Span::new(a, a);

        assert_eq!(span_a, span_aa);
    }

    /// Checks that the getters retrieve the correct value.
    #[test]
    fn span_getters() {
        let a = Position::new(10, 50);
        let b = Position::new(10, 52);

        let span = Span::new(a, b);

        assert_eq!(span.start(), a);
        assert_eq!(span.end(), b);
    }

    /// Checks that the `Span::contains()` method works properly.
    #[test]
    fn span_contains() {
        let a = Position::new(10, 50);
        let b = Position::new(10, 52);
        let c = Position::new(11, 20);
        let d = Position::new(12, 5);

        let span_ac = Span::new(a, c);
        assert!(span_ac.contains(b));

        let span_ab = Span::new(a, b);
        let span_cd = Span::new(c, d);

        assert!(!span_ab.contains(span_cd));
        assert!(span_ab.contains(b));

        let span_ad = Span::new(a, d);
        let span_bc = Span::new(b, c);

        assert!(span_ad.contains(span_bc));
        assert!(!span_bc.contains(span_ad));

        let span_ac = Span::new(a, c);
        let span_bd = Span::new(b, d);

        assert!(!span_ac.contains(span_bd));
        assert!(!span_bd.contains(span_ac));
    }

    /// Checks that the string representation of a span is correct.
    #[test]
    fn span_to_string() {
        let a = Position::new(10, 50);
        let b = Position::new(11, 20);
        let span = Span::new(a, b);

        assert_eq!("[10:50..11:20]", span.to_string());
        assert_eq!("[10:50..11:20]", span.to_string());
    }

    /// Checks that the ordering of spans is correct.
    #[test]
    fn span_ordering() {
        let a = Position::new(10, 50);
        let b = Position::new(10, 52);
        let c = Position::new(11, 20);
        let d = Position::new(12, 5);

        let span_ab = Span::new(a, b);
        let span_cd = Span::new(c, d);

        assert!(span_ab < span_cd);
        assert!(span_cd > span_ab);
    }
}
