//! The primary date-time components provided by Temporal.
//!
//! The below components are the main primitives of the `Temporal` specification:
//!   - `Date` -> `PlainDate`
//!   - `DateTime` -> `PlainDateTime`
//!   - `Time` -> `PlainTime`
//!   - `Duration` -> `Duration`
//!   - `Instant` -> `Instant`
//!   - `MonthDay` -> `PlainMonthDay`
//!   - `YearMonth` -> `PlainYearMonth`
//!   - `ZonedDateTime` -> `ZonedDateTime`
//!
//! The Temporal specification, along with this implementation aims to provide
//! full support for time zones and non-gregorian calendars that are compliant
//! with standards like ISO 8601, RFC 3339, and RFC 5545.

// TODO: Expand upon above introduction.

pub mod calendar;
pub mod tz;

mod date;
mod datetime;
pub(crate) mod duration;
mod instant;
mod month_day;
mod time;
mod year_month;
mod zoneddatetime;

#[doc(inline)]
pub use date::Date;
#[doc(inline)]
pub use datetime::DateTime;
#[doc(inline)]
pub use duration::Duration;
#[doc(inline)]
pub use instant::Instant;
#[doc(inline)]
pub use month_day::MonthDay;
#[doc(inline)]
pub use time::Time;
#[doc(inline)]
pub use year_month::YearMonth;
#[doc(inline)]
pub use zoneddatetime::ZonedDateTime;
